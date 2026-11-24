import json
import unittest
from unittest.mock import patch

import build_all
from release_version import next_version, normalize_version


class ReleaseVersionTest(unittest.TestCase):
    def test_first_release_starts_at_one_zero_zero(self):
        self.assertEqual("1.0.0", next_version(["v26.18.22", "nightly", "v1.0.0-beta"]))

    def test_next_release_increments_highest_matching_tag(self):
        self.assertEqual("1.0.11", next_version(["v1.0.2", "v1.0.10", "v26.18.22", "v1.1.99"]))

    def test_two_segment_versions_are_normalized_for_installers(self):
        self.assertEqual("1.0.0", normalize_version("v1.0"))
        self.assertEqual("1.0.3", normalize_version("v1.0.3"))
        self.assertEqual("1.0.1", next_version(["v1.0"]))

    def test_invalid_versions_are_rejected(self):
        with self.assertRaises(ValueError):
            normalize_version("nightly")


class ApiRetryTest(unittest.TestCase):
    def test_read_recovers_from_temporary_network_failure(self):
        with patch.object(build_all, "command", side_effect=[RuntimeError("EOF"), b'{"status":"completed"}']), \
                patch.object(build_all.time, "sleep"):
            self.assertEqual({"status": "completed"}, build_all.api("o/r", "actions/runs/1"))

    def test_writes_are_not_blindly_replayed_after_network_failure(self):
        with patch.object(build_all, "command", side_effect=RuntimeError("EOF")) as command:
            with self.assertRaises(RuntimeError):
                build_all.api("o/r", "git/commits", {"tree": "source"})
        command.assert_called_once()


class SourcePublishingTest(unittest.TestCase):
    def test_verified_tree_creates_the_fixed_build_branch(self):
        writes = []

        def fake_api(repo, path, payload=None):
            if payload is not None:
                writes.append((path, payload))
            if path.startswith("actions/workflows/"):
                return {"workflow_runs": []}
            if path.startswith("git/matching-refs/heads/"):
                return []
            if path.startswith("git/ref/"):
                return {"object": {"sha": "base"}}
            if path == "git/commits/base":
                return {"tree": {"sha": "tree"}}
            if path.startswith("git/trees/"):
                return {"tree": [{"path": "app.kt", "mode": "100644", "type": "blob", "sha": "blob"}]}
            return {"sha": "commit"}

        def fake_command(args, **kwargs):
            if args[0:2] == ["git", "diff"]:
                self.assertIn(":/", args)
            if args[0:2] == ["git", "ls-tree"]:
                self.assertIn("--full-tree", args)
                return b"100644 blob blob\tapp.kt\0"
            if args[0:2] == ["git", "rev-parse"]:
                return b"tree\n"
            if args[0] == "gh":
                self.assertFalse(json.loads(kwargs["input"])["force"])
            return b""

        with patch.object(build_all, "api", side_effect=fake_api), patch.object(build_all, "command", side_effect=fake_command):
            branch, commit = build_all.publish_source("owner/repo", "base")
        self.assertEqual("codex/build-release", branch)
        self.assertEqual("commit", commit)
        self.assertEqual(["git/commits", "git/refs"], [path for path, _ in writes])

    def test_tree_mismatch_stops_before_creating_a_commit_or_branch(self):
        def fake_api(repo, path, payload=None):
            self.assertIsNone(payload)
            if path.startswith("actions/workflows/"):
                return {"workflow_runs": []}
            if path.startswith("git/ref/"):
                return {"object": {"sha": "base"}}
            if path == "git/commits/base":
                return {"tree": {"sha": "remote-tree"}}
            return {"tree": []}

        def fake_command(args, **kwargs):
            return b"different-tree" if args[1] == "rev-parse" else b""

        with patch.object(build_all, "api", side_effect=fake_api), patch.object(build_all, "command", side_effect=fake_command):
            with self.assertRaisesRegex(RuntimeError, "源码校验失败"):
                build_all.publish_source("owner/repo", "base")


class BuildBranchReuseTest(unittest.TestCase):
    def test_existing_branch_is_updated_without_force_push(self):
        refs = [{"ref": "refs/heads/codex/build-release"}]
        with patch.object(build_all, "api", return_value=refs), patch.object(build_all, "command", return_value=b"") as command:
            self.assertEqual("codex/build-release", build_all.publish_build_ref("o/r", "new-commit"))
        args = command.call_args.args[0]
        self.assertIn("PATCH", args)
        self.assertEqual("repos/o/r/git/refs/heads/codex/build-release", args[-3])
        self.assertEqual({"sha": "new-commit", "force": False}, json.loads(command.call_args.kwargs["input"]))

    def test_active_build_blocks_reusing_the_branch(self):
        runs = {"workflow_runs": [{"status": "in_progress", "html_url": "https://example.com/run"}]}
        with patch.object(build_all, "api", return_value=runs):
            with self.assertRaisesRegex(RuntimeError, "已有构建正在运行"):
                build_all.ensure_build_branch_idle("o/r")

    def test_non_fast_forward_update_failure_is_not_retried_with_force(self):
        refs = [{"ref": "refs/heads/codex/build-release"}]
        with patch.object(build_all, "api", return_value=refs), patch.object(build_all, "command", side_effect=RuntimeError("not fast-forward")) as command:
            with self.assertRaisesRegex(RuntimeError, "not fast-forward"):
                build_all.publish_build_ref("o/r", "new-commit")
        command.assert_called_once()


class PackageRetryTest(unittest.TestCase):
    def test_capacity_cancellation_is_retried_only_once(self):
        retries = []

        def fake_api(repo, path, payload=None):
            run = {"id": 1, "head_sha": "commit", "html_url": "https://github.com/o/r/actions/runs/1",
                   "status": "completed", "conclusion": "cancelled"}
            if path.startswith("actions/workflows/build-integrated.yml/runs?"):
                return {"workflow_runs": [run]}
            if path == "actions/runs/1":
                return run
            if path.endswith("/jobs"):
                return {"jobs": [{"id": 2, "name": "macos-arm64", "status": "completed", "conclusion": "cancelled"}]}
            if path.endswith("/annotations"):
                return [{"message": "Due to capacity constraints, jobs may experience longer queue times."}]
            if path.endswith("/rerun"):
                retries.append(path)
                return {}
            return {"artifacts": []}

        with patch.object(build_all, "api", side_effect=fake_api), patch.object(build_all.time, "sleep"):
            with self.assertRaisesRegex(RuntimeError, "部分云端构建失败"):
                build_all.wait_for_packages("o/r", "branch", "commit", False)
        self.assertEqual(["actions/jobs/2/rerun"], retries)


if __name__ == "__main__":
    unittest.main()
