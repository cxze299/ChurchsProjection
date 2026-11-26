"""Build the fixed Windows entry and the same source on GitHub's native Mac runners."""
import argparse
import base64
import concurrent.futures
from datetime import datetime
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile
import time
from release_version import next_version

ROOT = Path(__file__).resolve().parents[1]
BUILD_BRANCH = "codex/build-release"


def command(args, **kwargs):
    result = subprocess.run(args, cwd=ROOT, capture_output=True, **kwargs)
    if result.returncode:
        message = (result.stderr or result.stdout).decode("utf-8", errors="replace")
        raise RuntimeError("\n".join(message.splitlines()[-30:]))
    return result.stdout


def api(repo, endpoint, payload=None, idempotent=False):
    resource = f"repos/{repo}" + (f"/{endpoint}" if endpoint else "")
    args = ["gh", "api", resource]
    options = {}
    if payload is not None:
        args += ["--method", "POST", "--input", "-"]
        options["input"] = json.dumps(payload).encode()
    can_retry = payload is None or idempotent
    for attempt in range(4):
        try:
            data = command(args, **options)
            break
        except RuntimeError as error:
            if not can_retry or attempt == 3 or "HTTP 404" in str(error):
                raise RuntimeError(f"GitHub 请求失败：{repo}/{endpoint}\n{error}") from error
            time.sleep(2 ** (attempt + 1))
    return json.loads(data) if data.strip() else {}


def resolve_base(repo, base_branch=None):
    metadata = api(repo, "")
    branch = base_branch or metadata["default_branch"]
    try:
        sha = api(repo, f"git/ref/heads/{branch}")["object"]["sha"]
    except RuntimeError as error:
        raise RuntimeError(f"无法读取基础分支 {repo}:{branch}，请检查 --base-branch 参数。\n{error}") from error
    return branch, sha


def ensure_build_branch_idle(repo):
    runs = api(repo, f"actions/workflows/build-integrated.yml/runs?branch={BUILD_BRANCH}&per_page=20")["workflow_runs"]
    active = next((run for run in runs if run["status"] != "completed"), None)
    if active:
        raise RuntimeError(f"已有构建正在运行，请完成后再启动：{active['html_url']}")


def publish_build_ref(repo, commit):
    ref = f"refs/heads/{BUILD_BRANCH}"
    refs = api(repo, f"git/matching-refs/heads/{BUILD_BRANCH}")
    if any(item["ref"] == ref for item in refs):
        command(["gh", "api", "--method", "PATCH", f"repos/{repo}/git/{ref}", "--input", "-"],
                input=json.dumps({"sha": commit, "force": False}).encode())
    else:
        api(repo, "git/refs", {"ref": ref, "sha": commit})
    return BUILD_BRANCH


def publish_source(repo, base):
    ensure_build_branch_idle(repo)
    # Publish committed source only: never stage files, include untracked files, or force-push.
    command(["git", "diff", "--quiet", "HEAD", "--", ":/"])
    # Base each snapshot on current main; packaging is independent of source merging.
    base_tree = api(repo, f"git/commits/{base}")["tree"]["sha"]
    remote = api(repo, f"git/trees/{base_tree}?recursive=1")
    if remote.get("truncated"):
        raise RuntimeError("远端文件列表不完整，已停止发布。")
    old = {item["path"]: item for item in remote["tree"] if item["type"] != "tree"}
    local = {}
    for entry in command(["git", "ls-tree", "-rz", "--full-tree", "HEAD"]).split(b"\0"):
        if not entry:
            continue
        header, path = entry.split(b"\t", 1)
        mode, kind, sha = header.decode().split()
        local[path.decode("utf-8")] = {"mode": mode, "type": kind, "sha": sha}
    to_upload = []
    changes = []
    for path, item in local.items():
        previous = old.get(path, {})
        if all(previous.get(key) == item[key] for key in ("mode", "type", "sha")):
            continue
        if item["type"] == "blob":
            to_upload.append((path, item["sha"]))
        changes.append({"path": path, **item})
    for path, item in old.items():
        if path not in local:
            changes.append({"path": path, "mode": item["mode"], "type": item["type"], "sha": None})
    if to_upload:
        unique_blobs = list({sha: path for path, sha in to_upload}.items())
        total = len(unique_blobs)
        print(f"正在并发上传 {total} 个变更文件到云端…", flush=True)

        def upload_one(entry):
            sha, sample_path = entry
            content = command(["git", "cat-file", "blob", sha])
            blob = api(repo, "git/blobs", {
                "encoding": "base64", "content": base64.b64encode(content).decode(),
            }, idempotent=True)
            if blob["sha"] != sha:
                raise RuntimeError(f"文件校验失败：{sample_path}")

        done = 0
        with concurrent.futures.ThreadPoolExecutor(max_workers=8) as pool:
            futures = [pool.submit(upload_one, item) for item in unique_blobs]
            for fut in concurrent.futures.as_completed(futures):
                fut.result()
                done += 1
                if done == total or done % 25 == 0:
                    print(f"  已上传 {done}/{total}", flush=True)
    tree = api(repo, "git/trees", {"base_tree": base_tree, "tree": changes}, idempotent=True)["sha"] if changes else base_tree
    expected = command(["git", "rev-parse", "HEAD^{tree}"]).decode().strip()
    if tree != expected:
        raise RuntimeError("云端源码校验失败，未触发构建。")
    commit = api(repo, "git/commits", {
        "message": "Build integrated desktop packages", "tree": tree, "parents": [base],
    })["sha"]
    # Keep the Actions cache scope stable; updates must fast-forward after the previous merge.
    branch = publish_build_ref(repo, commit)
    print(f"云端源码已校验：{branch}（{tree[:12]}）", flush=True)
    return branch, commit


def build_windows(version):
    if os.name != "nt":
        raise RuntimeError("本机 Windows 构建需要 Windows；其他系统可显式使用 --cloud-only。")
    env = os.environ.copy()
    if not env.get("JAVA_HOME"):
        candidates = sorted((Path(env.get("LOCALAPPDATA", "")) / "ChurchPresenterTools").glob("jdk-21*"))
        if candidates:
            env["JAVA_HOME"] = str(candidates[-1])
    workers = max(4, min(8, (os.cpu_count() or 4) - 1))
    args = [str(ROOT / "gradlew.bat"), "--quiet", "--parallel", "--build-cache", f"--max-workers={workers}"]
    # Optional machine-local repository/proxy configuration; no machine paths are saved in source.
    if env.get("CHURCHPRESENTER_GRADLE_INIT"):
        args += ["--init-script", env["CHURCHPRESENTER_GRADLE_INIT"]]
    args += [f"-PchurchPresenterVersion={version}", ":composeApp:createDistributable", ":composeApp:packageMsi"]
    print("正在构建 Windows 固定入口…", flush=True)
    with tempfile.TemporaryFile() as output:
        result = subprocess.run(args, cwd=ROOT, env=env, stdout=output, stderr=subprocess.STDOUT)
        if result.returncode:
            output.seek(0)
            print("\n".join(output.read().decode("utf-8", errors="replace").splitlines()[-30:]))
            raise RuntimeError("Windows 构建失败；若文件被占用，请先退出软件。")
    entry = ROOT / "composeApp/build/compose/binaries/main/app/ChurchPresenter/ChurchPresenter.exe"
    if not entry.is_file():
        raise RuntimeError("未找到 Windows 固定入口。")
    print(f"Windows 构建成功：{entry}", flush=True)
    packages = list((ROOT / "composeApp/build/compose/binaries/main/msi").glob(f"*{version}*.msi"))
    if not packages:
        raise RuntimeError("未找到本次版本的 Windows MSI 安装包。")
    return max(packages, key=lambda path: path.stat().st_mtime)


def prepare_windows_release(repo, commit, version, package):
    tag = "v" + version
    existing = command(["gh", "release", "list", "--repo", repo, "--limit", "100", "--json", "tagName"])
    if tag not in {item["tagName"] for item in json.loads(existing)}:
        try:
            command(["gh", "release", "create", tag, "--repo", repo, "--target", commit,
                     "--draft", "--title", "ChurchPresenter " + tag,
                     "--notes", "Windows 安装包由本机生成；macOS 安装包将在手动发起的云端构建完成后补充。"])
        except RuntimeError:
            # Cloud release job may have created the release concurrently; upload with --clobber below.
            pass
    for attempt in range(4):
        try:
            command(["gh", "release", "upload", tag, str(package) + f"#ChurchPresenter-{version}-WINDOWS-x64.msi",
                     "--repo", repo, "--clobber"])
            break
        except RuntimeError:
            if attempt == 3:
                raise
            time.sleep(2 ** (attempt + 1))
    subprocess.run(["git", "fetch", "origin", "--tags", "--quiet"], cwd=ROOT)
    print(f"Windows MSI 已上传到 Release：{tag}", flush=True)


def wait_for_packages(repo, branch, commit, no_wait, cloud_windows=True):
    run = None
    for _ in range(24):
        runs = api(repo, f"actions/workflows/build-integrated.yml/runs?branch={branch}&event=workflow_dispatch&per_page=10")["workflow_runs"]
        run = next((item for item in runs if item["head_sha"] == commit), None)
        if run:
            break
        time.sleep(5)
    if not run:
        raise RuntimeError(f"未检测到构建任务，请检查 https://github.com/{repo}/actions")
    print(f"构建任务：{run['html_url']}", flush=True)
    if no_wait:
        print("未等待云端构建完成；本次脚本耗时不包含后续云端打包时间。", flush=True)
        return
    retried = set()
    previous = None
    while True:
        run = api(repo, f"actions/runs/{run['id']}")
        jobs = api(repo, f"actions/runs/{run['id']}/jobs")["jobs"]
        state = [(job["name"], job["status"], job["conclusion"]) for job in jobs]
        if state != previous:
            print(" | ".join(f"{name}: {conclusion or status}" for name, status, conclusion in state
                             if conclusion != "skipped"), flush=True)
            previous = state
        if run["status"] == "completed":
            retry = None
            for job in jobs:
                if job["name"] in retried:
                    continue
                capacity_cancelled = False
                if job["conclusion"] == "cancelled":
                    annotations = api(repo, f"check-runs/{job['id']}/annotations")
                    capacity_cancelled = any("capacity constraints" in item["message"].lower()
                                             or "not acquired by runner" in item["message"].lower()
                                             for item in annotations)
                if job["conclusion"] in ("failure", "timed_out") or capacity_cancelled:
                    retry = job
                    break
            if retry:
                print(f"重试云端任务（每个平台仅一次）：{retry['name']}", flush=True)
                api(repo, f"actions/jobs/{retry['id']}/rerun", {})
                retried.add(retry["name"])
                time.sleep(10)
                continue
            artifacts = api(repo, f"actions/runs/{run['id']}/artifacts")["artifacts"]
            for artifact in artifacts:
                if not artifact["expired"]:
                    print(f"{artifact['name']}: {run['html_url']}/artifacts/{artifact['id']}")
            if run["conclusion"] != "success":
                raise RuntimeError("部分云端构建失败，请查看上述任务链接。")
            names = {item["name"] for item in artifacts if not item["expired"]}
            expected = {"macos-dmg", "macos-dmg-x64"} | ({"windows-msi"} if cloud_windows else set())
            if not expected.issubset(names):
                raise RuntimeError("任务结束，但缺少预期的云端安装包。")
            print("三平台构建成功。未配置 Apple 签名凭据时，Mac 包未签名、未公证。")
            print(f"Release： https://github.com/{repo}/releases")
            return
        time.sleep(30)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--repo", default="cxze299/ChurchsProjection")
    parser.add_argument("--base-branch", help="基础分支；默认使用目标仓库的默认分支")
    parser.add_argument("--cloud-only", action="store_true")
    parser.add_argument("--no-wait", action="store_true")
    args = parser.parse_args()
    for tool in ("git", "gh"):
        if not shutil.which(tool):
            raise RuntimeError(f"请先安装 {tool}。")
    command(["gh", "auth", "status"])
    base_branch, base = resolve_base(args.repo, args.base_branch)
    print(f"发布仓库：{args.repo}，基础分支：{base_branch}", flush=True)
    if subprocess.run(["git", "diff", "--quiet", "HEAD", "--", ":/"], cwd=ROOT).returncode:
        raise RuntimeError("存在未提交的源码修改，请提交后再构建，以保证三个版本使用同一份源码。")
    ensure_build_branch_idle(args.repo)
    tags = api(args.repo, "git/matching-refs/tags/v1.0")
    version = next_version(tag["ref"].rsplit("/", 1)[-1] for tag in tags)
    print(f"本次构建版本：{version}", flush=True)
    package = None if args.cloud_only else build_windows(version)
    branch, commit = publish_source(args.repo, base)
    if package:
        prepare_windows_release(args.repo, commit, version, package)
    api(args.repo, "actions/workflows/build-integrated.yml/dispatches", {
        "ref": branch, "inputs": {"windows": "true" if args.cloud_only else "false", "version": "v" + version},
    })
    print("已手动发起云端打包；该打包结果不作为分支合并条件。", flush=True)
    wait_for_packages(args.repo, branch, commit, args.no_wait, cloud_windows=args.cloud_only)


def run():
    started = time.monotonic()
    print(f"构建脚本开始时间：{datetime.now().astimezone():%Y-%m-%d %H:%M:%S %z}", flush=True)
    try:
        main()
        return 0
    except (RuntimeError, KeyboardInterrupt) as error:
        print(f"构建已停止：{error}", file=sys.stderr)
        return 1
    finally:
        elapsed = max(0, int(time.monotonic() - started))
        hours, remainder = divmod(elapsed, 3600)
        minutes, seconds = divmod(remainder, 60)
        print(f"构建脚本结束时间：{datetime.now().astimezone():%Y-%m-%d %H:%M:%S %z}", flush=True)
        print(f"构建脚本总耗时：{hours:02d}:{minutes:02d}:{seconds:02d}", flush=True)


if __name__ == "__main__":
    sys.exit(run())
