# 三平台一键构建

完整工程位于仓库的 `project/` 中。在 Windows 本机双击 `project\Build-All.cmd`，或先进入 `project/` 后在 PowerShell 执行：

```powershell
.\scripts\Build-All.ps1
```

- 需要 Python 3、Git、GitHub CLI 和 JDK 21。先执行 `gh auth login`，登录具有仓库写入及 Actions 权限的账号。
- 构建前退出 ChurchPresenter，并提交当前源码修改。脚本不自动暂存、提交或上传未跟踪文件。
- 本机使用 `:composeApp:createDistributable` 更新固定软件入口，并使用 `:composeApp:packageMsi` 生成 Windows MSI。
- 上传已提交源码并校验源码树后，由 CMD 显式手动触发 `build-integrated.yml`，仅在 Actions 构建 macOS MSilicon 和 Intel 两个包。推送源码本身不触发打包。
- 脚本等待构建结束，每个平台遇到失败或 runner 容量不足时自动重试一次，并输出各安装包的下载链接。下载 GitHub Actions 产物需要登录 GitHub。
- Windows MSI 先上传到 Release 草稿；两个 Mac 包成功后加入同一 Release 并正式发布，Mac 失败或取消时保留草稿。三平台使用相同版本号与源码树。
- 构建分支自动合并到 main 不依赖任何打包结果；打包失败不阻止源码合并。
- macOS 使用 GitHub 的 Mac runner，本机无需安装 Xcode。没有配置 Apple 签名和公证凭据时，产物未签名、未公证。

可选的显式全云端构建：`.\scripts\Build-All.ps1 -CloudOnly`。
触发后不等待云端结果：`.\scripts\Build-All.ps1 -NoWait`（云端仍会在全部构建成功后自动发布 Release）。

本机如需额外的 Gradle 仓库配置，可设置 `CHURCHPRESENTER_GRADLE_INIT` 为本机 init script 路径；脚本不会将该配置上传。
