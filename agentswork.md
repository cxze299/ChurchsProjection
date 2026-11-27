# ChurchsProjection 协同工作档案

更新日期：2026-11-26。共同维护者：Codex、Antigravity。项目负责人：cxze299。

本文件汇集本会话的项目需求、用户选择、当前实现、协作规则、构建发布流程及验证记录。仓库根目录 `agentswork.md` 为主要维护文件，`project/agentswork.md` 保存相同内容，便于从工程目录开始工作的代理读取。本文中的源码路径均相对仓库根目录。

## 一、接手项目时先确认

1. 用户最新指令优先于共享档案和仓库说明。阅读本文件、`project/AGENTS.md`、`project/AGENT.md` 与 `project/docs/development/CODING_STANDARDS.md`，再阅读当前任务涉及模块的说明。
2. 检查当前分支、未提交修改及用户最新消息；修改前重新读取目标文件，保留用户与另一位代理已经完成的调整。
3. 在本文“当前协作状态”中登记任务、负责人、涉及文件和验证范围。两个代理对同一文件的修改、Git 提交及分支切换按顺序执行。
4. 同一个工程目录同时只运行一个 Gradle 构建或测试进程。已有云端打包任务运行时，保留其构建分支和任务状态。
5. 完成用户明确要求的全部步骤并验证结果，然后更新 `project/CHANGELOG.md` 和本文件，提交源码并自动合并到当前仓库的 `main`。

## 二、项目身份、来源与目录

| 项目 | 当前信息 |
| --- | --- |
| 对外名称 | ChurchsProjection；部分应用、任务和安装包继续使用 ChurchPresenter 名称 |
| 当前仓库 | https://github.com/cxze299/ChurchsProjection |
| 主分支 | `main` |
| 本机仓库目录 | `D:\churchpre` |
| 工程目录 | `D:\churchpre\project` |
| 固定应用入口 | `D:\churchpre\project\composeApp\build\compose\binaries\main\app\ChurchPresenter\ChurchPresenter.exe` |
| 构建启动器 | `project/Build-All.cmd`；PowerShell 入口为 `project/scripts/Build-All.ps1` |
| 技术组成 | Kotlin、JVM 21、Compose Multiplatform、Gradle 多模块工程 |
| 许可证和来源 | 保留 GPL-3.0、原版权说明及 `project/docs/legal/ATTRIBUTION.md` |

项目最初基于 `cxze299/ChurchPresenter`，用户指定参考上游 ChurchPresenter 的 `v26.17.137`。随后形成独立开源项目，源码与 Release 均维护在当前仓库。用户曾指定 `cxze299/ChurchsProjectionown` 作为迁移目的地，当前目的地统一为 `cxze299/ChurchsProjection`。用户提出的构建参考项目为 https://github.com/xibolun/bible 。

仓库根目录保留 `README.md`、`agentswork.md`、`project/` 和 Git 基础设施 `.git/`、`.github/`、`.githooks/`。完整工程统一放在 `project/`，README 在仓库根目录以中文维护。

| 路径或文件 | 用途 |
| --- | --- |
| `project/modules/core/` | 数据模型、设置、主题、通用界面和基础能力 |
| `project/modules/engines/` | 经文、演示文稿、媒体及输出渲染等执行模块 |
| `project/modules/features/` | 圣经、诗歌、演示文稿、图片等用户功能 |
| `project/modules/output/` | 实时投影输出与预览 |
| `project/modules/integrations/` | 系统设置、输出配置、外部服务等集成 |
| `project/composeApp/` | 桌面应用组合、主窗口、投影窗口与固定输出位置 |
| `project/config/architecture/module-paths.properties` | Gradle 逻辑模块名称到实际目录的对应关系 |
| `project/build-logic/` | 共享 Gradle 构建、运行时裁剪、签名与打包逻辑 |
| `project/scripts/` | 构建、版本、发布说明及辅助脚本 |
| `project/docs/` | 按用途分类的正式说明文档 |
| `project/assets/branding/` | 已采用的标志、应用图标及仓库展示图 |
| `project/.local/` | 忽略提交的临时试验、个人笔记、界面检查图及本地辅助文件 |
| `project/AGENTS.md` | 本地项目范围和协作要求 |
| `project/AGENT.md` | 原工程结构、开发规范和模块说明 |
| `project/CLAUDE.md` | 开发规范及共同协作档案的读取入口 |
| `project/CHANGELOG.md` | 用户可阅读的功能与构建变更记录 |
| `project/build.gradle.kts`、`settings.gradle.kts`、`gradle.properties` | 工程根配置，保持在 `project/` 中 |
| `project/gradlew`、`project/gradlew.bat` | Gradle 启动器 |

详细结构见 `project/docs/architecture/DIRECTORY_LAYOUT.md`。新增或移动模块时同步修改模块路径配置，保留 Gradle 逻辑任务名称及固定软件入口。依赖目录、构建输出、临时记录与正式资源按用途存放，忽略规则只排除与源码交付无关的内容。

## 三、用户已确定的功能与交互

### 3.1 圣经快速输入和书卷候选

- 支持 66 卷书名的拼音首字母。示例 `csj 1 12 16` 表示创世记第 1 章第 12–16 节，`csj 12 2 5` 表示第 12 章第 2–5 节。
- 从两个首字母开始显示候选，按照精确匹配、前缀匹配及保留字母顺序的模糊匹配排列；候选仅剩一个时自动选中对应书卷。
- 输入时保留缩写，实时显示完整书名、章与节范围，方便继续修改。范围上屏完整保留各节。
- 普通单引用沿用该模式的上屏操作；连续上屏中的完整快速输入在停顿约 300 毫秒后自动投影。

### 3.2 经文单击、段选与引用格式

- 第一次普通单击选中一节；第二次单击另一节，选中包含两端的区间；第三次单击开始新的选择。支持反向段选和原有 Ctrl、Shift 选择。
- “选中即上屏”开启时，第一次单击投一节，第二次单击投整段；手动上屏、回车和双击同样保留完整范围。
- 连续上屏时，整段包含的既有单节从累计内容中合并移除，保留其他书卷、章节和译本内容。
- 连续段选按起止节号显示，即使书卷文本中缺少中间节号，也使用例如 `撒母耳记下 8:5-18` 的范围引用。
- 非连续选节合并各段中的相邻节号，例如 `5-11,13-18`；历史记录使用与投影一致的引用。
- “清除经文”清空累计内容和段选起点，并清除相关经文投影；已清除的输入不会在延迟任务结束后再次自动上屏。

- 历史记录支持选中条目后复制引用，保留书名、章节和节号范围。标题栏的引用样式按钮提供四种括号格式，选择会随设置保存。

- 圣经正文搜索和结果高亮允许汉字之间有空格，保留译本文件与经文原文。

### 3.3 多书卷连续上屏

开启“连续上屏”后，以下两种写法均可使用，并可继续输入更多书卷：

```text
mtfy 5 12 5qsl 6 7 8
mtfy 5 12 5 qsl 6 7 8
```

- 书卷之间可直接连接或用空格分隔，不设置段数上限；曾验证连续解析和读取 128 段经文。
- 倒序节号转换为升序。上述示例对应马太福音 5:5–12、启示录 6:7–8。
- 完整输入统一上屏；输入中的书卷、章或节尚未完整，以及范围无效时，保留当前画面。
- 修改当前输入会更新对应的经文组，避免重复追加；清空输入后可开始新的输入组。
- 各段按输入顺序从上向下排列，同一段的不同译本放在一起，沿用对应译本的字体、字号、颜色和引用样式。
- 内容超过投影区域高度时显示右侧滚动条。新增或修改经文后自动滚动到最新内容；平时可手动向上滚动回看。
- 关闭连续上屏后恢复普通替换行为，再次开启开始新的累计组。

### 3.4 经文投影样式

“连续上屏”旁提供“样式设置”按钮。可按译本调整字体、正文字号、引用字号、文字颜色、粗体、斜体、下划线和阴影。连续经文还可设置行距和段间距。字号与间距可拖动滑块或输入数值，设置窗口实时预览草稿，点击“应用”将修改保存到投影屏幕使用的输出配置，并更新已上屏的经文，点击“取消”放弃草稿。

行距为百分比，`0` 使用字体默认行距；显式行距范围为 50–300%。正文字号和引用字号范围为 8–240，段间距范围为 0–200。连续模式未单独配置段间距时沿用已有译本间距。

### 3.5 演示文稿资源与诗歌速放

- 自动递归扫描演示文稿配置目录及其子目录中的 PDF、PowerPoint、Word 等资源。用户已明确只扫描配置目录。
- 支持多个演示文稿目录并合并展示，目录或文件选择后立即刷新；文件与目录统一通过“选择资源”处理。
- 按歌名、文件名前置编号、中文歌名拼音首字母实时检索，兼容大小写、简繁体及多音字。搜索结果右侧提供可拖动滚动条。
- 默认诗歌本可设为配置目录或其中的一级子文件夹。搜索覆盖所有已配置目录并优先显示默认歌本。可用 `002.pps`、`qyed ppsx`、`pps 奇异恩典` 或 `.pdf` 指定扩展名，支持简繁体和编号与歌名组合。
- “诗歌”页进入时同步最新的主目录和附加目录，歌库缓存按完整目录列表匹配。标题搜索支持简繁体和拼音首字母，数字搜索忽略歌号前导零。
- 扫描支持通过符号链接配置的挂载目录，忽略 macOS 的 `._` 伴随文件，并提示无法读取的位置。支持文件名的 Unicode 组合与分解形式。
- 默认关闭“选中即上屏”时，资源选择用于加载预览；显式上屏或提交搜索执行放映。“选中即上屏”开启时按该设置处理资源选择。
- 顶部上屏操作统一使用软件现有按钮；搜索框旁的重复投屏按钮已移除，筛选、资源和放映选项按现有界面控件布局。
- Windows 上可用 Word、WPS 或 LibreOffice 转换 Word 文稿。macOS 上支持从系统或用户“应用程序”目录启动 LibreOffice。

- PowerPoint 渲染保留文稿字符间距，可读取 PPS 和 PPSX 内嵌字体，并为缺失的字形选用系统字体。macOS 渲染前完成字体索引，覆盖本机与用户应用目录下的 Office、WPS 字体及系统下载字体。

### 3.6 投影窗口、屏幕选择与预览

- 支持可移动、可缩放的独立投影窗口，单屏可显示经文、演示文稿和其他资料。
- 主界面显示器按钮打开投影目标菜单，可选择已连接显示器或独立窗口。选中目标后恢复显示当前投影，切换操作对应当前目标。
- Esc 优先隐藏当前投影窗口。再次选择目标或执行上屏可恢复显示。
- 每个应用投影窗口提供右键菜单，支持全屏、窗口模式、置顶和隐藏。macOS 用 `⌃⌘F` 切换全屏，带 `Command` 的按键保留给组合快捷键。其他常用操作包括方向键、PageUp/PageDown、空格、B、T 和 Esc。
- 右侧实时预览显示当前输出，独立窗口、主屏及指定显示器均使用对应输出设置，并随经文、诗歌等内容切换更新。
- 投影工具栏采用图标与文字一体的紧凑按钮，宽侧栏分列，窄侧栏换行，菜单有下拉标识。

### 3.7 设置、多目录、快捷键与更新来源

- 设置中的九类存储部分支持多个目录、添加与移除目录以及指定默认目录，兼容已有单目录设置。
- 演示文稿合并扫描多目录，歌曲合并读取资源，圣经可从多个目录选择和加载；程序单、日历、布告及字幕条使用选定的默认目录读写。
- 设置中提供快捷键编辑入口，可录制、替换和清除功能绑定。排查快捷键时优先检查投影窗口焦点与重复绑定，用户已选择“优先隐藏投影窗口”。
- macOS 菜单与撤销、重做默认使用 `Command`。更新器按 `M-Silicon` 与 `Intel` 安装包文件名选择对应的 DMG。
- macOS 应用声明读取外置磁盘、网络卷、常用用户目录及使用麦克风的用途。Finder 启动时也会查找用户应用目录中的 VLC，以及 Homebrew 的 LibreOffice 启动器。DeckLink 原生构建脚本生成并检查 ARM64 和 Intel 通用库。
- “选中经文或演示文稿资源时直接上屏”由设置控制，默认关闭。
- 软件更新指向当前定制仓库，自动检查默认关闭；保持本项目源码与发布版本的独立维护。

### 3.8 主窗口和边栏默认状态

- 每次启动时收起左侧程序单与右侧投影预览边栏，保留已设置的边栏宽度。窗口模式和最大化模式均使用启动收起状态，运行中可自行展开。
- 主窗口以居中的普通窗口启动，按屏幕可用区域约 80% 计算，常规首选尺寸不超过 1280×800，并结合界面缩放所需最小尺寸调整。
- 窗口及最小尺寸限制在屏幕可用范围内，考虑任务栏、系统菜单和多屏坐标。
- 当前边栏的展开和收起状态使用统一分隔条宽度 36.dp、按钮 32.dp×40.dp、箭头 24.dp，并使用主题 `secondaryContainer` 底色。左右分隔条预留宽度合计为 72.dp。

### 3.9 界面语言与品牌

- 用户要求软件界面采用统一的现有设计语言，新增功能与原有主题控件保持一致，面向用户的提示以中文显示。
- 正式标志采用用户选定的第 05 号“见证之窗”方案，将屏幕边界与十字架组合，风格为现代简洁、温润雅致。
- 主色 `#4E687B`，浅色 `#FAF7F0`；应用图标为 1024×1024，仓库分享图为 1280×640，均有 SVG 和 PNG。
- 品牌文件位于 `project/assets/branding/`，README、应用图标和各平台图标保持一致。GitHub 仓库展示通过 README 标志和分享图维护。
- 标志设计技能来源为 https://github.com/kaankiziltug/logo-design-skill ，当前本机技能名称为 `logo-design`。

## 四、开发与双代理协作规则

### 4.1 任务范围和文件保护

- 两位代理平等协作，根据用户当前任务分工。每项任务仅修改用户明确要求的功能范围；同一任务中的多个明确要求逐项完成并分别验证。
- 已有未提交内容保持原有作者的修改范围。用户撤回或改写代码后，从重新读取的内容继续工作。
- 使用当前可用工作目录；新建工作树需按当前工具与用户要求办理。共享目录中的 Git 索引、提交、切换分支、合并和发布操作串行执行。
- 两位代理领取任务时写明涉及文件；同一文件由当前负责人完成后交接。其他代理可以处理互不重叠的文件和只读检查。
- 所有功能集成在当前软件和固定入口中，开发任务完成后更新 changelog、共享档案和 GitHub。
- 每个完成记录写明实际修改、实际执行的验证与剩余事项；安装包、Mac 实机运行、签名和公证分别记录实际状态。

### 4.2 PowerShell、文件读取和终端输出

- 本机使用 Windows 与 PowerShell 7。命令的变量、路径、循环、条件、重定向及管道使用 PowerShell 语法；优先使用 `-LiteralPath`。
- Git、Python、Node 等跨平台工具可直接调用。用户手动构建保留现有 `Build-All.cmd`，代理在 PowerShell 中调用对应 PowerShell、Python 或 Gradle 入口。
- 命令不能为空；复杂操作使用清晰的多行 PowerShell 或 Python。不要混入其他本地 shell 的赋值、文件操作和重定向语法。
- 先通过 `rg` 或 `Select-String` 定位，再分段读取必要内容；不扫描或读取 `node_modules`、`dist`、`build`、`.git`、`target`、`*.lock`、编译产物和日志文件。
- 构建、测试和包管理使用安静参数或截断输出，每次返回终端日志最多 50 行。原生命令执行后立即保存退出码，再处理输出。
- 删除、移动或覆盖前明确目标路径；递归操作先确认绝对路径位于预期目录。
- 工具支持交互会话时复用会话；按工具公开接口等待提示和读取结果，工作完成后关闭会话。
- 同一工程中的 Gradle 操作串行执行，保留增量编译和构建缓存。源码布局输入由 `project/build-logic/src/main/kotlin/cp.base.gradle.kts` 维护。
- 覆盖固定应用入口需要应用释放相关文件；需要重建单个模块输出时，限定到该模块，保留固定安装目录。

### 4.3 SSH 和远程执行

- 明确区分本机 PowerShell、`ssh.exe` 和远程 shell 三层解析。
- 远程表达式中的变量、命令替换、通配符和引号应在传入 SSH 时保持正确边界，避免被本机提前展开。
- 先确认远程使用的 shell。远程 Linux 可使用其实际 shell 的语法；复杂脚本优先经标准输入或临时脚本传递。
- 涉及多层转义的命令，先检查本地最终参数和远程最终文本，必要时用只读操作验证，再执行修改。

### 4.4 模型分工和验证

- 正文写作、正式说明、生产代码、功能脚本、配置修改和复杂调试由当前主模型完成。
- 简单检索、只读检查、测试执行和测试专用代码可交给子代理；每次子代理调用显式指定模型 `luna`、推理强度 `xhigh`。
- 将任务交给子代理监控后，主模型等待其报告；额度不足可重试，复杂问题由主模型继续处理。
- 不得计算、生成、比较或验证哈希和校验和，也不得创建相应文件。
- 修改后执行适合该范围的编译、测试和必要界面检查，处理出现的问题后再完成交付。
- 正式说明使用规范完整的语言，英文代码标识符保持原名。直接写明动作、对象与结果，避免自造缩略语、抽象管理类比、无关对比、说教、密集引号及多个破折号连接。
- 文档和代码说明当前正确设计；即时诊断过程放在与用户的对话中。最终答复给出完成结果及验证情况。
- 用户在任务中提出补充要求时，先简短回应可立即回答的问题，再继续完成原任务和相关补充。

## 五、构建、版本与发布

### 5.1 三种交付版本

| 平台 | 默认执行位置 | 文件名规则 |
| --- | --- | --- |
| Windows x64 | 本机 Windows | `ChurchPresenter-<版本>-WINDOWS-x64.msi` |
| macOS 苹果 M 系列芯片 | GitHub Actions Mac runner | `ChurchPresenter-<版本>-MACOS-M-Silicon.dmg` |
| macOS Intel | GitHub Actions Mac runner | `ChurchPresenter-<版本>-MACOS-Intel.dmg` |

- 版本从 1.0.0 起按 1.0.x 递增，三平台使用同一份已提交源码和同一版本号；版本生成在 `project/scripts/release_version.py`。
- 用户手动发起构建。默认顺序为本机 Windows 构建、源码提交到构建分支、Windows 安装包上传草稿、调度两个 Mac 构建、等待所需产物齐备并发布正式 Release。
- 发布目的地始终是 `cxze299/ChurchsProjection`。历史旧版本整理已完成，后续发布按现有标签递增。
- `Build-All.cmd` 在任务结束后保留窗口。脚本显示开始时间、结束时间和总耗时，失败或中断也显示计时。
- `-NoWait` 只等待云端任务创建并给出入口，脚本时间不包含随后云端构建。`-CloudOnly` 可显式使用云端 Windows 和两个 Mac 构建。
- 正式 Release 说明使用中文，优先从版本之间新增的中文 changelog 提取更新项，附源码版本对比及平台下载提示。
- Windows 产物通过标准 `:composeApp:createDistributable` 更新固定入口，MSI 位于同一标准输出目录；不创建额外测试应用目录。

### 5.2 触发方式和 Git 合并

- `.github/workflows/build-integrated.yml` 为手动入口，复用 `.github/workflows/build.yml` 完成各平台打包。Mac 两种架构均由 Actions 执行。
- 构建、完整测试、截图和性能检查按当前工作流的 `workflow_dispatch` 手动调度。`auto-merge-builds.yml` 保留构建分支推送后的自动合并；仓库清理与 Dependabot 另有各自维护触发条件。
- 源码推送不自动开始三平台安装包打包，打包结果不作为源码合并的必要条件。
- 源码分支使用 `codex/build-*`；固定构建分支为 `codex/build-release`，用于后续构建缓存复用。正在运行的打包任务继续使用其原有分支。
- 用户已授权提交、推送构建分支并自动合并到 `main`。仅纳入当前任务的文件，处理合并冲突后重新验证涉及内容。
- Git 提交使用用户已有身份，提交和 PR 中保持项目内容说明，不添加代理署名。

### 5.3 本机命令与依赖

本机需要 JDK 21、Python 3、Git 和已登录的 GitHub CLI。既有本机 JDK 记录为 `C:\Users\Administrator\AppData\Local\ChurchPresenterTools\jdk-21.0.13+9`，执行时以实际 `jdk-21*` 工具目录为准。

从 `D:\churchpre` 使用 PowerShell 7 执行：

```powershell
$taskJdk = Get-ChildItem -LiteralPath (Join-Path $env:LOCALAPPDATA 'ChurchPresenterTools') -Directory -Filter 'jdk-21*' | Select-Object -First 1
if (-not $taskJdk) { throw '未找到 JDK 21。' }
$env:JAVA_HOME = $taskJdk.FullName
$taskJdkBin = Join-Path $env:JAVA_HOME 'bin'
if (($env:PATH -split ';') -notcontains $taskJdkBin) {
    $env:PATH = "$taskJdkBin;$env:PATH"
}
& .\project\gradlew.bat -p project --quiet --max-workers=3 :composeApp:compileKotlinJvm
if ($LASTEXITCODE -ne 0) { throw '应用编译未通过。' }
```

手动完整构建可双击现有 `project/Build-All.cmd`；PowerShell 7 中可执行：

```powershell
& .\project\scripts\Build-All.ps1
```

版本参数使用完整字符串，例如 `"-PchurchPresenterVersion=1.0.6"`。只有依赖已完整缓存时才考虑离线构建。英文文案测试使用测试进程的 `user.language=en` 和 `user.country=US`；中文界面测试显式选择中文，并在结束后恢复原语言。

### 5.4 Apple 签名和公证

在当前仓库的 GitHub Actions secrets 中配置工作流要求的凭据，取值由项目负责人管理。

| 用途 | 当前工作流使用的名称 |
| --- | --- |
| Developer ID Application 的 P12 证书及密码 | `MACOS_CERTIFICATE_P12`、`MACOS_CERTIFICATE_PASSWORD` |
| 签名身份和团队 | `MACOS_IDENTITY_NAME`、`MACOS_NOTARIZATION_TEAM_ID` |
| Apple ID 公证方式 | `MACOS_NOTARIZATION_APPLE_ID`、`MACOS_NOTARIZATION_PASSWORD` |
| App Store Connect API 公证方式 | `MACOS_NOTARIZATION_API_KEY_P8`、`MACOS_NOTARIZATION_API_KEY_ID`、`MACOS_NOTARIZATION_API_ISSUER_ID` |

P12 导出文件和 P8 密钥按照工作流要求编码后配置。证书签名、公证提交和票据装订分别依照 `build.yml` 中的步骤执行。未配置签名凭据时的包按其实际签名状态交付，Mac 安装与运行验证分别记录 M 系列芯片和 Intel 机型。

## 六、既有构建优化记录

以下为原协作档案保留的 v1.0.6 构建测量和对应实现说明。体积与时间属于当次构建结果。

### 6.1 体积和时间记录

| 项目 | 优化前记录 | 优化后记录 |
| --- | --- | --- |
| Windows MSI | 617.18 MB | 450.07 MB，减少 167.11 MB（27.1%） |
| Windows 运行目录 | 835.94 MB | 638.58 MB，减少 197.36 MB（23.6%） |
| M-Silicon DMG | 约 490 MB | 405.94 MB |
| Intel DMG | 约 500 MB | 418.68 MB |
| M-Silicon 构建耗时 | 14 分 06 秒 | 7 分 40 秒 |
| JDK 运行时目录 | 155.33 MB | 112.45 MB |
| profiles 模块 JAR | 89.79 MB | 38.12 MB |

### 6.2 打包实现

1. `project/gradle/libs.versions.toml`、`project/build-logic/src/main/kotlin/org/churchpresenter/buildlogic/AppBuild.kt` 维护依赖，已去除未使用的 OpenJFX 依赖和对应启动预热，记录减少约 44.33 MB 解压体积及约 200 毫秒预热工作。
2. `project/build-logic/src/main/kotlin/cp.app-natives.gradle.kts` 中的 `ShrinkPackagedJarsTask` 和 `shrinkPackagedJars` 执行打包期类引用扫描与原生库裁剪，运行于 `createDistributable` 及相关安装包任务之前。测量时扫描 280 个运行时 JAR。
3. 图标库 `material-icons-extended-desktop-1.7.3.jar` 从 11,105 个类保留实际引用的 177 个类，记录体积从 36,883 KB 降至 549 KB。SQLite 驱动从 11,848 KB 降至 797 KB，junixsocket 原生库从 694 KB 降至 92 KB。
4. `project/composeApp/build.gradle.kts` 排除未使用的 Netty QUIC/HTTP3 依赖，并将 DBus 依赖限定为 Linux 目标；运行时以 `includeAllModules = false` 配置所需 JDK 模块。
5. 运行时模块集合包括 `java.se`、`jdk.accessibility`、`jdk.charsets`、`jdk.crypto.cryptoki`、`jdk.crypto.ec`、`jdk.httpserver`、`jdk.jfr`、`jdk.localedata`、`jdk.management`、`jdk.management.jfr`、`jdk.naming.dns`、`jdk.naming.rmi`、`jdk.net`、`jdk.nio.mapmode`、`jdk.random`、`jdk.security.auth`、`jdk.security.jgss`、`jdk.unsupported`、`jdk.unsupported.desktop`、`jdk.xml.dom`、`jdk.zipfs`。`LottieOutputFrames` 的管理接口使用 `jdk.management`。
6. `project/modules/integrations/profiles/src/main/composeResources/files/backgrounds/` 中，截图与资源库测试引用的 17 张参考图片保留原内容，包括 `Free Great Clouds.jpg`、`Abstract Wave.jpg`、`Cross Light.jpg`。另外 4 张大型 PNG 转为 JPEG 并更新 `index.txt`，大图采用 1080p、JPEG `q:v 8` 处理；既有资源库测试记录为 35 项通过。
7. macOS 目标对通用 `libomt.dylib` 按 `arm64` 或 `x86_64` 使用 `lipo -thin` 提取单架构，记录体积从约 17.9 MB 降至约 8.9 MB。ffmpeg 和 OMT 的原生下载缓存使用 Gradle 用户目录下的 `caches/churchpresenter-natives/`。
8. `project/build-logic/src/main/kotlin/cp.app-signing.gradle.kts` 的 `packageFastDmg` 依赖应用目录生成与签名，使用 `ditto` 保留 Framework 结构，写入 Applications 软链接和卷图标，再以 `hdiutil` 的 ULFO/LZFSE 生成 DMG。
9. Mac runner 的源码检出深度为 1，Gradle worker 数为 4；Gradle 和 Kotlin daemon 堆上限使用 2560M，采用 `UseParallelGC`。公证装订重试间隔为 10 秒，最多 15 次。
10. `project/scripts/build_all.py` 使用 8 个上传线程和最多 4 次指数退避重试处理相关源码上传，MSI 上传最多重试 4 次；发布前同步远端标签。既有构建脚本测试记录为 19 项通过。

## 七、当前发布与历史验证

### 7.1 2026-11-26 查询的正式 Release

最新正式版本为 [v1.0.6](https://github.com/cxze299/ChurchsProjection/releases/tag/v1.0.6)，本次从 GitHub 发布信息读取到以下附件：

| 文件 | 字节数 |
| --- | ---: |
| `ChurchPresenter-1.0.6-WINDOWS-x64.msi` | 471934043 |
| `ChurchPresenter-1.0.6-MACOS-M-Silicon.dmg` | 426621417 |
| `ChurchPresenter-1.0.6-MACOS-Intel.dmg` | 440097487 |

既有协作记录载明，本机固定入口曾以版本 1.0.6 构建并检查关于窗口；本机 MSI 记录路径为 `D:\churchpre\project\composeApp\build\compose\binaries\main\msi\ChurchPresenter-1.0.6.msi`。Windows 云端补充构建记录为 Actions 任务 `45787798281`。当前源码、已生成本机应用及 Release 附件的版本状态在每次构建交接时分别更新。

### 7.2 本会话已记录的验证

各行对应功能完成时的验证范围，不将不同任务的测试数量相加作为当前版本测试总数。

| 功能或事项 | 已记录验证 | 已知合并记录 |
| --- | --- | --- |
| 清除经文与工具栏文字 | 41 项相关测试及编译 | PR #25 |
| 源码目录布局与 JVM 编译输入 | 相关模块和应用编译；再次编译使用已有结果 | PR #26 |
| 图标文字一体工具栏 | 17 项交互与布局测试，检查 160 和 280 宽度中文渲染图 | PR #28 |
| 独立窗口和目标屏幕实时预览 | 79 项测试及应用编译 | PR #30 |
| 构建计时 | 15 项脚本测试 | PR #31 |
| 中文 Release 说明 | 18 项脚本测试及实际 changelog 提取 | PR #33 |
| 多书卷输入与纵向滚动 | 66 项测试及应用编译，128 段解析读取和渲染检查 | PR #34 |
| 自动跟随与经文样式设置 | 67 项测试及应用编译，中文设置窗口检查 | PR #36 |
| 引用范围和主窗口启动布局 | 256 项测试及应用编译，边栏展开按钮渲染检查 | PR #38 |
| 当前边栏展开和收起尺寸统一 | 16 项界面与拖拽测试 | 既有共同维护记录 |
| 安装包优化与发布编排 | 19 项构建脚本测试、35 项背景资源库测试 | 既有共同维护记录 |

此前投影恢复记录包括两块显示器和独立窗口的 Esc 隐藏与恢复操作。不同任务的编译、安装包生成、实际硬件运行及 Apple 签名状态分别以各自记录为准。

### 7.3 会话阶段索引

| 阶段 | 用户确定的方向 |
| --- | --- |
| 初始定制 | 从最新上游基础增加拼音经文速投，统一在一个软件内集成功能 |
| 资源与放映 | 独立投影窗口、文档扫描、诗歌速放、默认歌本和格式、实时搜索、目标屏幕选择 |
| 设置完善 | 多目录、自定义快捷键、选中即上屏、独立更新来源、中文界面及 changelog |
| 工程与发布整理 | 独立开源仓库、来源引用、自动合并、完整工程进入 project、中文 README、1.0.x 三平台构建 |
| 品牌确定 | 方案 05 十字架与屏幕组合、统一软件及仓库展示图、1280×640 分享图 |
| 连续经文扩展 | 跨书卷连续输入、区间格式、累计段选、滚动、最新内容自动跟随与样式设置 |
| 启动与构建完善 | 主窗口自适应、边栏默认收起、展开按钮统一、打包体积与 Mac 构建时间优化 |
| 本次协作 | 将完整会话共识汇入双份 agentswork.md，由 Codex 与 Antigravity 共同维护 |

## 八、源码与验证入口

| 功能 | 主要位置 |
| --- | --- |
| 书卷首字母、解析与连续输入 | `project/modules/features/bible-tab/src/main/kotlin/org/churchpresenter/bibletab/` 中的 `ChineseBookInitials.kt`、`BibleViewModelReferenceParse.kt`、`BibleQuickPassages.kt` |
| 段选、引用和累计经文 | 同目录的 `BibleViewModelClicks.kt`、`BibleViewModelSelection.kt`、`BiblePassageProjection.kt`、`BibleTabScope.kt` |
| 连续投影与样式窗口 | `project/modules/engines/presenter/src/main/kotlin/org/churchpresenter/presenter/BibleScrollingPassages.kt`、`BibleSlide.kt`，以及 bible-tab 中的 `BibleProjectionStyleDialog.kt` |
| 设置与经文数据 | `project/modules/core/settings/src/main/kotlin/org/churchpresenter/settings/BibleSettings.kt`、`project/modules/core/core-models/src/main/kotlin/org/churchpresenter/core/models/bible/SelectedVerse.kt` |
| 诗歌搜索和演示文稿 | `project/modules/features/slides/` 中的 `PresentationQuickPlay.kt`、`HymnQuickPlay.kt` |
| 多目录与快捷键设置 | `project/modules/integrations/app-settings/`、`project/composeApp/src/jvmMain/kotlin/org/churchpresenter/app/churchpresenter/dialogs/ShortcutSystemSettings.kt` |
| 投影目标、Esc 与右键菜单 | 桌面目录中的 `PreviewSidebar.kt`、`PresenterWindows.kt`、`OutputShortcut.kt`、`OutputWindowMenu.kt` |
| 右侧实时预览 | `project/modules/output/live-output/src/main/kotlin/org/churchpresenter/liveoutput/preview/` |
| 主窗口与边栏 | 桌面目录中的 `AppRootState.kt`、`MainWindow.kt`、`StartupWindowLogic.kt`、`MainDesktopPanels.kt`、`composables/PanelResizeHandle.kt` |
| 构建、版本与中文发布说明 | `project/scripts/build_all.py`、`release_version.py`、`generate_release_notes.py`，以及 `.github/workflows/` |
| 经文引用回归 | `VerseCitationRangeTest`、`BibleViewModelTest`、`BibleViewModelChapterTest` |
| 连续输入与设置回归 | `BibleQuickPassagesTest`、`BibleTabScopeActionsTest`、`BibleProjectionStyleDialogTest` |
| 渲染回归 | `BibleScrollingPassagesTest`、`BibleLayoutsBoxedAndGridRenderTest`、`BibleStyleTest` |
| 启动布局回归 | `AdaptiveStartupWindowTest`、`MainDesktopPanelsUiTest`、`PanelResizeHandleTest`、`MainLogicTest` |

针对修改选择相关测试执行，使用安静输出并保留退出码。正式截图等可分享资源进入正式目录；本地检查图放在 `project/.local/ui-review/`。

## 九、当前协作状态与后续维护

| 项目 | 状态 |
| --- | --- |
| 最近完成任务 | 2026-11-26，由 Codex 同步本会话的项目信息与共同管理规则 |
| 本次验证 | 两份正文一致；30 项必要信息、44 个源码路径、两段 PowerShell 语法、协作入口及工作流说明检查通过 |
| 本次文件范围 | 根目录与工程目录的 agentswork.md、协作读取入口及 changelog |
| 本次应用功能范围 | 保持现有功能实现，更新共同维护资料 |
| 共享责任 | Codex 与 Antigravity 接手任务前读取本文，完成关键修改后同步更新两份文件 |
| 后续任务来源 | 用户的新需求、实际测试反馈及当前任务中发现的必要问题 |

后续每次交接更新日期、负责人、任务范围、当前分支、验证结果、本机应用状态、各平台发布状态和剩余事项。以 Git 工作目录、当前源码与用户最新指令为工作起点；已完成事项保留在功能和验证索引中，新任务写入当前协作状态。

## 2026-11-27 Rust 歌词解析与 `rust-project/` 原生桌面版完整重写

1. **局部歌词解析模块**：Codex 在 `project/modules/core/song-chords/rust` 添加了多语言段落标题识别与 SPS 文本歌词解析，Windows 上的 2 项 Rust 单元测试通过。
2. **Codex 全局配置优化（解决长任务中断问题）**：Antigravity 将本机 `C:\Users\Administrator\.codex\config.toml` 中的 `model_reasoning_effort` 从 `"low"` 提升为 `"high"`，并将 `[agents] enabled` 设为 `true`（`max_concurrent_threads_per_session = 4`），由 Antigravity 拆解任务并通过 `codex.exe exec` 并行调度与验收。
3. **纯原生 Rust 桌面工程 (`D:\churchpre\rust-project`)**：
   - 按用户选定的技术方案，在仓库根目录新建独立 Cargo Workspace `rust-project/`（与原 Kotlin 工程 `project/` 并存），包含 4 个 crate：
     - `crates/cp-core`：兼容 `~/.churchpresenter/settings.json` 的配置读写、9 类 `ResourceDirectory` 多目录管理、自定义快捷键模型与核心投影状态模型。
     - `crates/cp-bible`：`.spb` 圣经解析、跨译本 `BXXXCXXXVXXX` 并行对齐、66 卷中文拼音首字母速投、单段与无限段跨书卷连续经文解析、两次点击段选、紧凑范围引用、连续上屏单节自动去重（28 项测试通过）。
     - `crates/cp-library`：多音字拼音首字母匹配、前置编号检索、多目录递归扫描、歌本与优先格式排序、`.song` 读写、`.pptx` 文本与图片提取、Word 转 PDF 缓存与 Windows 原生 `Windows.Data.Pdf` 页面渲染（25 项测试通过）。
     - `crates/cp-app`：基于 `eframe` / `egui` 0.31 的双窗口原生 GUI（主控台 + 多屏/独立投影窗口），加载本机微软雅黑字体与应用图标；左右边栏默认收起，收起与展开状态下分隔条统一宽 `36.dp`、切换按钮 `32.dp × 40.dp`、箭头 `24.dp`；包含经文连续上屏与样式设置窗口、诗歌检索预览、多目录与快捷键设置、右侧 16:9 实时预览及带右键菜单与 `F11`/`B`/`T`/`Esc` 快捷键的独立投影窗口（5 项应用状态测试通过）。
   - **构建产物与验证结果**：
     - 全工作区 58 项测试（`cargo test --workspace`）与 `cargo clippy --workspace --all-targets -- -D warnings` 全部 0 警告通过。
     - 单文件 Release 可执行程序：`D:\churchpre\rust-project\target\release\ChurchPresenter.exe`，体积仅 **`9.67 MB`**（无需捆绑 112 MB JVM 运行时，较原 Kotlin 解压目录 `638.58 MB` 缩减 **98.5%**），本机实测启动正常。

4. **Rust 原生版 Material 3 UI 深度重构**：新增 widgets.rs、songs_ui.rs、pictures_ui.rs、nnouncements_ui.rs，补齐 8 套原版配色方案与 6 个完整标签页（圣经+历史记录卡片、诗歌表格+段落卡片、文稿 16:9 缩略图网格、图片 16:9 缩略图网格、布告倒计时+九宫格位置选择器、卡片式设置）。全工作区 62 项测试与 clippy 零警告通过，单文件 D:\churchpre\rust-project\target\release\ChurchPresenter.exe 体积 10.01 MB，本机实测启动正常。
