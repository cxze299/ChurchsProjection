# ChurchsProjection

<p align="center">
  <img src="project/assets/branding/app-icon.png" width="144" height="144" alt="ChurchsProjection — 见证之窗" />
</p>

ChurchsProjection 由 cxze299 独立维护，基于 [ChurchPresenter](https://github.com/ChurchPresenter/ChurchPresenter) 开发。
在原有功能基础上，增加了经文速投、诗歌搜索、自定义快捷键、多资源目录，以及 Windows 与 macOS 同步打包等功能。项目来源说明见[来源引用](project/docs/legal/ATTRIBUTION.md)。

本项目保留上游的 GPL-3.0 许可证及版权声明。

这是一款面向教会的免费开源桌面投影软件，将诗歌、经文、幻灯片、视频、字幕和直播图文集中在同一个应用中，支持 Windows、macOS 和 Linux。
Linux 版本通过 `.deb` 安装包安装，适用于 Ubuntu 22.04、Debian 12、Linux Mint 21 及更新版本。

软件采用运行于 JVM 的 Compose Multiplatform 开发。[功能说明](project/docs/product/FEATURES.md)列出了各项功能及其源码位置。

## 协作开发

Codex 与 Antigravity 共同维护本项目。接手任务前先阅读[协作工作档案](agentswork.md)和[项目开发要求](project/AGENTS.md)，再检查相关源码。关键修改完成后同步更新根目录 `agentswork.md`、工程目录 `project/agentswork.md` 和变更记录。

---

## 快速开始

### 环境要求

运行 Gradle 需要 JDK 17 或更新版本。工程使用 JDK 21 编译；如果本机尚未安装，Gradle 会自动下载所需工具链。

### 获取源码

项目不包含 Git 子模块，直接克隆即可：

```shell
git clone https://github.com/cxze299/ChurchsProjection.git
cd ChurchsProjection/project
```

### 构建与运行

完整工程位于 `project/`。请在 IDE 中打开该目录，并在该目录内执行下方命令。Windows 用户也可以在仓库目录中双击 `project\Build-All.cmd`，手动发起构建。

可以使用 IDE 工具栏中的运行配置，也可以通过终端启动：

```shell
./gradlew :composeApp:run        # macOS / Linux
```

```shell
.\gradlew.bat :composeApp:run    # Windows
```

#### 强制启用开发备用投影窗口

在仅有一块显示器且未连接 DeckLink 设备的机器上，开发版本会自动打开一个额外的小型投影窗口，用于替代第二块显示器。
如果需要在打包版本或正式版本中使用同样的窗口，例如在没有第二块显示器时演示或测试投影，可以在启动应用前设置环境变量：

```shell
CHURCHPRESENTER_FORCE_DEV_WINDOW=true ./ChurchPresenter   # macOS / Linux
```

```shell
set CHURCHPRESENTER_FORCE_DEV_WINDOW=true && ChurchPresenter.exe   # Windows
```

也可以使用 JVM 系统属性 `-Dchurchpresenter.forceDevWindow=true`，例如通过 `JAVA_TOOL_OPTIONS` 传入。
该设置仅控制是否显示备用窗口，不改变应用在更新检查、崩溃报告或使用情况统计中的版本身份。

### 测试

以下命令均在 `project/` 目录内执行：

```shell
./gradlew :composeApp:check     # 编译并运行应用的全部单元测试
./gradlew :settings:test       # 运行指定模块的测试，其他模块同理
bash scripts/test-changed.sh   # 仅运行与当前修改有关的测试
```

### 内置 ffmpeg

软件通过 **ffmpeg** 打开摄像头和采集卡，并随应用提供所需程序，无需用户自行安装。
ffmpeg 二进制文件不直接提交到仓库：`:composeApp:fetchBundledFfmpeg` 会下载 `gradle/ffmpeg-builds.properties` 中固定版本的程序，校验 SHA-256，然后将其写入 `composeApp/src/jvmMain/appResources/<os>/`。
打包任务和 `run` 任务均使用该位置，并自动依赖下载任务，无需手动处理。

在 macOS 上，如果配置了签名身份，`:composeApp:signBundledFfmpeg` 会在打包前启用强化运行时，并使用 `platform/desktop/macos/ffmpeg.entitlements` 对 ffmpeg 签名。
Compose 不会自动签名 `appResources` 下的内容，缺少这一步会导致 DMG 公证失败。

目标平台未配置下载地址时，软件会尝试使用本机已经安装的 ffmpeg；可以在“设置 → 投影 → 摄像头采集”中查看实际使用的程序。
许可证及源码提供说明见 [FFmpeg 第三方声明](project/docs/legal/THIRD_PARTY_FFMPEG.md)。

---

## 项目结构

完整工程位于仓库的 `project/` 中。工程内的源码模块按架构层级归入 `modules/`，平台相关文件位于 `platform/`。
完整目录说明见[目录布局](project/docs/architecture/DIRECTORY_LAYOUT.md)。Gradle 逻辑任务名称保持不变，应用输出仍位于工程内的 `composeApp/build/` 下。

[`composeApp/`](project/composeApp/src) 是桌面应用主体，应用代码位于 [`jvmMain`](project/composeApp/src/jvmMain/kotlin)，测试位于 [`jvmTest`](project/composeApp/src/jvmTest/kotlin)。

其他包含 `build.gradle.kts` 的源码模块均属于同一个 Gradle 工程，不单独维护 Wrapper。
在工程根目录 `project/` 使用统一的 `./gradlew` 即可构建和测试这些模块。各模块的 `AGENT.md` 记录了对应的结构和开发约定。

| 模块 | 用途 |
|---|---|
| [`modules/core/settings/`](project/modules/core/settings) | 持久化设置：设置数据类、负责加载、迁移和保存 `settings.json` 的 `SettingsManager`，以及默认值常量。 |
| [`modules/core/diagnostics/`](project/modules/core/diagnostics) | 崩溃报告：将日志写入 `~/.churchpresenter/crash-reports/`，向 Sentry 转发事件，并对上报事件中的个人身份信息进行脱敏。 |
| [`modules/core/core-models/`](project/modules/core/core-models) | 共享数据模型，包括程序单项目、场景、问题、歌词，以及诗歌模型和 `.song` 文件格式。 |
| [`modules/core/theme/`](project/modules/core/theme) | 应用外观，包括九套配色、语义颜色、字体和形状规范。 |
| [`modules/core/bible/`](project/modules/core/bible) | 圣经数据：加载 `.spb` 译本，管理书卷、两套编号体系及经文搜索。 |
| [`modules/engines/bible-formats/`](project/modules/engines/bible-formats) | 圣经下载与格式转换：提供 eBible、Zefania、Beblia 下载目录，并将 USFX 和 Zefania XML 转换为 `.spb`。 |
| [`modules/engines/bible-engine/`](project/modules/engines/bible-engine) | 圣经查找引擎，从语音中识别经文引用。 |
| [`modules/core/song-chords/`](project/modules/core/song-chords) | 诗歌文本语法：和弦、段落标题、移调，以及将粘贴的和弦谱转换为 `[G]lyric` 行内标记。不依赖其他模块，供应用和转换器共用。 |
| [`modules/features/songlibrary/`](project/modules/features/songlibrary) | 诗歌库管理器：在可编辑表格中管理诗歌库目录内的全部诗歌，从“帮助”菜单打开。 |
| [`modules/features/calendar/`](project/modules/features/calendar) | 日历管理器：在月历中安排聚会，并将各场聚会的流程载入程序单；从“帮助”菜单打开，通过端到端加密中继与手机配对。 |
| [`modules/engines/presentation-engine/`](project/modules/engines/presentation-engine) | 在 JVM 内完成 PPTX、PPT、Keynote 和 PDF 的解析、计时及动画处理。 |
| [`modules/engines/lottieGenerator/`](project/modules/engines/lottieGenerator) | 独立的 Compose Desktop 应用，用于生成 Lottie JSON 格式的动态字幕条，也可从字幕条设置中打开。 |
| [`modules/features/converter/`](project/modules/features/converter) | 诗歌与圣经格式转换器，从“帮助”菜单打开；也可通过 `./gradlew :converter:run` 单独运行。 |
| [`modules/features/crossword/`](project/modules/features/crossword) | 填字游戏制作工具，不直接编入主应用；构建任务会将编码后的题目复制到应用资源中，使用 `./gradlew :crossword:run` 打开。 |
| [`modules/features/planning-center/`](project/modules/features/planning-center) | Planning Center Online 客户端，处理 OAuth 授权、通过 Services API 读取聚会计划，以及接收授权回调。各教会自行提供免费的 PCO 开发者凭据，客户端不向服务端写回内容。 |
| [`modules/engines/companion-satellite/`](project/modules/engines/companion-satellite) | 使用纯 Kotlin 实现的 Bitfocus Companion Satellite 协议客户端。 |
| [`modules/engines/atem/`](project/modules/engines/atem) | Blackmagic ATEM 协议客户端，处理与切换台之间从握手到媒体池上传的 UDP 通信。测试使用依据真实设备通信构建的本地模拟切换台，无需连接设备。 |
| [`modules/engines/ndi/`](project/modules/engines/ndi) | 双向 NDI：将输出发布为网络 NDI 源，或将接收的 NDI 内容显示在画布中。NDI Runtime 需单独免费下载，应用在启动时自动检测，方式与 VLC 类似。 |
| [`modules/engines/omt/`](project/modules/engines/omt) | 双向 Open Media Transport。其库采用 MIT 许可证，构建时下载固定版本并随应用分发，方式与 ffmpeg 类似。 |

各模块均可通过 `./gradlew :<module>:test` 运行对应的测试。

---

## 文档

[文档索引](project/docs/README.md)列出了项目文档，常用文档包括：

- [开发指南](project/docs/development/DEVELOPMENT_GUIDE.md)：开发流程、验证命令和贡献方式。
- [编码规范](project/docs/development/CODING_STANDARDS.md)：代码风格要求。
- [开发代理规范](project/AGENT.md)：架构、模块、命令，以及截图和测试约定。
- [安装包构建说明](project/docs/packaging/BUILD_INSTALLERS.md)和[安装包快速开始](project/docs/packaging/QUICK_START_INSTALLERS.md)：如何构建安装包。
- [图形后端说明](project/docs/platform/GRAPHICS_BACKEND.md)：选择 GPU 后端，以及后端对屏幕采集的影响。
- [Companion 接口文档](project/docs/api/COMPANION_API.md)：面向移动 Companion 应用的 REST 和 WebSocket API。

### 提交前检查

在 `project/` 目录内执行：

```bash
bash scripts/cleanup_check.sh   # 检查通配符导入、Material 2、控制台输出、完整限定名及未使用代码
./gradlew :composeApp:detekt    # CI 的首道检查，提交前最后运行
```

---

## 开源许可

项目采用 GPL-3.0 许可证，详见[许可证正文](project/LICENSE.txt)。
随应用分发的 ffmpeg、OMT 库及 Wick 句向量模型分别遵循各自的条款，详见 [FFmpeg 第三方声明](project/docs/legal/THIRD_PARTY_FFMPEG.md)、[OMT 第三方声明](project/docs/legal/THIRD_PARTY_OMT.md)和 [MiniLM 第三方声明](project/docs/legal/THIRD_PARTY_MINILM.md)。
