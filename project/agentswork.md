# Agents 协同工作档案 (`agentswork.md`)

> **文档定位**：本文件为 **Codex** 与 **Antigravity** 共同维护 `ChurchsProjection` 项目的协作档案与变更台账（仓库根目录 `agentswork.md` 与工程根目录 `project/agentswork.md` 保持同步）。两位 Agent 在接手任务前应首先阅读本文件及 `project/AGENTS.md`，在完成任务后将关键架构变更、构建配置、版本号与已验证结论同步更新至本文件。

---

## 一、 协同管理规范（Codex & Antigravity 共同遵守）

1. **目录与固定软件入口**：
   - 仓库根目录仅保留 `README.md`、`agentswork.md`、`.git/`、`.github/`、`.githooks/` 及完整工程目录 `project/`。
   - 工程根目录为 `D:\churchpre\project`（含 `AGENTS.md`、`AGENT.md`、`agentswork.md`、`CHANGELOG.md`、`Build-All.cmd` 等）。
   - **唯一固定本地软件入口**：
     `D:\churchpre\project\composeApp\build\compose\binaries\main\app\ChurchPresenter\ChurchPresenter.exe`
   - 严禁创建 `test-app*` 等额外独立输出目录；所有功能更新与修复统一编译并更新到该固定入口。
2. **本地 JDK 与构建环境**：
   - 本机 JDK 21 路径：`C:\Users\Administrator\AppData\Local\ChurchPresenterTools\jdk-21.0.13+9`。
   - 在 PowerShell 中执行 Gradle 命令时需显式指定 `$env:JAVA_HOME = "C:\Users\Administrator\AppData\Local\ChurchPresenterTools\jdk-21.0.13+9"`。
   - **PowerShell 参数转义注意**：传入包含小数点的 Gradle 属性（如版本号）时，**必须加引号**（例如 `"-PchurchPresenterVersion=1.0.6"`），否则 PowerShell 会按 `.` 拆分参数导致 `jpackage` 报错 `Illegal version for 'Exe': '1'`。
   - 若本机网络访问 Maven Central 偶发 TLS 握手超时（如拉取未缓存的 `detekt` 插件），日常构建与打包可加 `--offline` 使用本地 Gradle 缓存。
   - 运行依赖英文文案断言的单元测试（如 `:profiles:test --tests "*LocalLibrary*"`）时，因宿主 Windows 默认语言为 `zh_CN`，需传入 `JAVA_TOOL_OPTIONS="-Duser.language=en -Duser.country=US"`。
3. **版本控制与自动合并流程**：
   - 每次修改完成后自动提交 Git，推送到 `codex/build-*` 分支（日常发布固定复用 `codex/build-release` 分支以保留 GitHub Actions Gradle 缓存），由 `.github/workflows/auto-merge-builds.yml` 自动快进/合并到 `cxze299/ChurchsProjection` 的 `main` 分支。
   - 严禁在 Commit message 或 PR 中添加 `Co-Authored-By:` 等任何 Agent 署名。
   - 每次功能或构建修复必须同步更新 `project/CHANGELOG.md` 与 `agentswork.md`。
4. **手动构建原则**：
   - 所有 GitHub Actions 构建、测试与巡检工作流均为**纯手动触发（`workflow_dispatch`）**，推送代码不自动触发三平台安装包构建。
   - 版本标签在云端创建后，本地构建前必须执行 `git fetch origin --tags`（已内置于 `build_all.py`），确保本地 `appVersion()` 解析出与云端一致的最新版本号。

---

## 二、 2026-11-26 操作与优化全量同步记录

### 1. 安装包与运行时体积缩减（功能 100% 保持不变，首次构建耗时最短）

#### 实测体积缩减效果
| 平台 / 产物 | 优化前体积 | 优化后体积 | 缩减幅度 |
| :--- | :--- | :--- | :--- |
| **Windows MSI 安装包** (`ChurchPresenter-1.0.6.msi`) | `617.18 MB` | **`450.07 MB`** | **-167.11 MB (-27.1%)** |
| **Windows 解压运行目录** (`app/` + `runtime/`) | `835.94 MB` | **`638.58 MB`** | **-197.36 MB (-23.6%)** |
| **macOS Apple Silicon DMG** (`MACOS-M-Silicon.dmg`) | `~490 MB` | **`405.94 MB`** | **~ -84 MB (-17.1%)** |
| **macOS Intel DMG** (`MACOS-Intel.dmg`) | `~500 MB` | **`418.68 MB`** | **~ -81 MB (-16.2%)** |

#### 具体源码与构建改动清单
1. **移除废弃的 OpenJFX 依赖与启动预热**（节省约 `44.33 MB` 解压体积 / `~22 MB` 安装包体积，消除 200ms 启动阻塞）：
   - `project/gradle/libs.versions.toml`、`project/build-logic/src/main/kotlin/org/churchpresenter/buildlogic/AppBuild.kt`：移除未使用的 `javafx-base`、`javafx-controls`、`javafx-graphics`、`javafx-media`、`javafx-swing` 依赖注入。
   - `project/composeApp/src/jvmMain/kotlin/org/churchpresenter/app/churchpresenter/MainWindow.kt`：移除 `preWarmJavaFX()` 调用，并清理废弃的 `JavaFxInit.kt` 与 `JavaFxScreenRaceTest.kt`。
2. **新增打包期字节码常量池闭包扫描与跨平台原生库裁剪任务 `ShrinkPackagedJarsTask`**（位于 `project/build-logic/src/main/kotlin/cp.app-natives.gradle.kts`）：
   - `@CacheableTask` 任务 `shrinkPackagedJars` 自动挂载在所有 `AbstractJPackageTask` / `createDistributable` 打包前执行：
     - **图标库按需裁剪**：扫描运行时全部 280 个 JAR 的 `.class` 常量池引用闭包，将 `material-icons-extended-desktop-1.7.3.jar` 从 `11,105` 个类（`36,883 KB`）裁剪为实际引用的 `177` 个类（**`549 KB`**），节省 **`36.3 MB`**。
     - **跨平台 Native 库剥离**：剥离 `sqlite-jdbc-3.54.1.0.jar` 中非当前目标 OS/架构的驱动（从 `11,848 KB` 降至 **`797 KB`**，节省 **`11.0 MB`**），以及 `junixsocket-native-common-2.10.1.jar` 中非当前平台的库（从 `694 KB` 降至 **`92 KB`**）。
3. **排除未使用的 Netty QUIC / HTTP3 多平台原生库与非 Linux DBus 依赖**（位于 `project/composeApp/build.gradle.kts`）：
   - 全局排除 Ktor 传递引入但未使用的 `io.netty:netty-codec-native-quic`、`netty-codec-classes-quic`、`netty-codec-http3`（节省 **`11.6 MB`**）。
   - `:composeApp` 直接声明的 `dbus-java-core` 与 `dbus-java-transport-native-unixsocket` 限定为仅 Linux 目标打包。
4. **JDK 21 运行时模块精确裁剪 (`jlink`) 与启动崩溃修复**（位于 `project/composeApp/build.gradle.kts`）：
   - 将 `includeAllModules = true` 改为 `includeAllModules = false`，通过扫描全部 280 个运行时 JAR 对 JDK 21 全部 70 个 `.jmod` 的包引用关系，显式保留所有必需运行时模块（剔除 `jdk.compiler`、`jdk.jshell`、`jdk.jdi`、`jdk.javadoc`、`jdk.jdeps`、`jdk.editpad`、`jdk.jpackage`、`jdk.jlink` 等编译与调试工具模块），使 `runtime/` 目录从 `155.33 MB` 降至 **`112.45 MB`**（节省 **`42.88 MB`**）。
   - **关键修复记录**：`LottieOutputFrames.kt:35` 使用了 `com.sun.management.OperatingSystemMXBean`（位于 `jdk.management` 模块而非 `java.management`），必须包含完整运行时模块列表：
     ```kotlin
     modules(
         "java.se",
         "jdk.accessibility",
         "jdk.charsets",
         "jdk.crypto.cryptoki",
         "jdk.crypto.ec",
         "jdk.httpserver",
         "jdk.jfr",
         "jdk.localedata",
         "jdk.management",
         "jdk.management.jfr",
         "jdk.naming.dns",
         "jdk.naming.rmi",
         "jdk.net",
         "jdk.nio.mapmode",
         "jdk.random",
         "jdk.security.auth",
         "jdk.security.jgss",
         "jdk.unsupported",
         "jdk.unsupported.desktop",
         "jdk.xml.dom",
         "jdk.zipfs",
     )
     ```
5. **内置背景图库视觉无损压缩（严格保留截图测试基线）**（位于 `project/modules/integrations/profiles/src/main/composeResources/files/backgrounds/`）：
   - 定位出 17 张被 Roborazzi 截图测试与 `LocalLibraryContentTest` 引用的基线图片（`Free Great Clouds.jpg`、`Abstract Wave.jpg`、`Cross Light.jpg` 等）**保持原字节完全不变**。
   - 将其余 4 张超大摄影类 `.png` 背景图（`16.48 MB`）转为 `.jpg` 并同步更新 `index.txt`，对超大分辨率背景图按 1080p JPEG `q:v 8` 优化，使 `profiles` 模块 JAR 从 `89.79 MB` 降至 **`38.12 MB`**（节省 **`51.67 MB`**）。`:profiles:test --tests "*LocalLibrary*"` 全部 35 项测试 100% 通过。
6. **macOS 原生库单架构裁剪 (`lipo -thin`) 与原生缓存持久化**（位于 `project/build-logic/src/main/kotlin/cp.app-natives.gradle.kts`）：
   - 在 macOS 打包阶段对通用二进制 `libomt.dylib`（`17.9 MB`）按目标架构（`arm64` 或 `x86_64`）执行 `lipo -thin` 单架构提取，体积减半至 **`~8.9 MB`**。
   - 将 `ffmpeg` 与 `omt` 下载缓存目录从 `build/` 迁移至 `~/.gradle/caches/churchpresenter-natives/`，避免 `clean` 后重复下载。

---

### 2. macOS 双架构 DMG 构建提速（M-Silicon 从 14m06s 降至 7m40s）

1. **新增单遍 `ULFO` (LZFSE) 快速打包任务 `packageFastDmg`**（位于 `project/build-logic/src/main/kotlin/cp.app-signing.gradle.kts`）：
   - 原本 `:composeApp:packageDmg` 会先由 `jpackage` 单线程执行一遍高耗时的 `hdiutil UDZO` (zlib) 压缩（耗时 3~5 分钟），随后 `fixDmgIcon` 又挂载、拷出并重新压缩一遍 DMG。
   - 新增 `packageFastDmg` 任务：直接依赖 `createDistributable`（已完成 `.app` 构建与 `signMacosApp` 签名），使用 `/usr/bin/ditto` 保留 macOS Framework 符号链接与签名结构拷入临时目录，写入 `/Applications` 软链接与 `.VolumeIcon.icns` 后，直接调用 `hdiutil create -format ULFO`（Apple 原生 LZFSE 算法）一次性生成最终 DMG。
2. **GitHub Actions macOS Runner 参数调优**（位于 `.github/workflows/build.yml`）：
   - `build-windows`、`build-macos`、`build-macos-x64` 的 `actions/checkout` 统一设为 `fetch-depth: 1`。
   - macOS 构建命令切换为 `:composeApp:packageFastDmg --parallel --build-cache -Dorg.gradle.workers.max=4`，并将 Gradle 与 Kotlin Daemon 堆上限设为 `2560M`（配合 `-XX:+UseParallelGC`），避免在 3~4 核 macOS Runner 上触发内存交换抖动。
   - Apple 公证票据装订（`xcrun stapler staple`）重试间隔从 `30s`（10 次）缩短为 `10s`（15 次）。

---

### 3. 左右边栏收起与展开按钮尺寸统一

- **改动文件**：
  - `project/composeApp/src/jvmMain/kotlin/org/churchpresenter/app/churchpresenter/composables/PanelResizeHandle.kt`
  - `project/composeApp/src/jvmMain/kotlin/org/churchpresenter/app/churchpresenter/MainDesktopPanels.kt`
  - `project/composeApp/src/jvmTest/kotlin/org/churchpresenter/app/churchpresenter/MainDesktopPanelsUiTest.kt`
- **具体规格**：
  - 左右边栏（程序单边栏与投影预览边栏）在**收起（collapsed）**与**展开（expanded）**两种状态下：
    - 分隔条容器宽度统一为 `36.dp`（`PANEL_RESIZE_HANDLE_EXPANDED_WIDTH = 36.dp`，`PANEL_RESIZE_HANDLE_COLLAPSED_WIDTH = 36.dp`）。
    - 切换按钮尺寸统一为 `32.dp × 40.dp`，统一使用圆角 `8.dp` 与 `MaterialTheme.colorScheme.secondaryContainer` 醒目底色。
    - 箭头图标尺寸统一为 `24.dp`。
  - 同步将 `MainDesktopPanels.kt` 中左右面板最大宽度预留空间常量 `PANEL_HANDLES_RESERVED_DP` 从 `40` 更新为 `72`（`36.dp × 2`）。
  - 通过 `MainDesktopPanelsUiTest` 与 `PanelResizeHandleTest` 共 16 项 UI 与拖拽测试验证。

---

### 4. 工作流全手动化与 `build_all.py` 构建发布编排加固

1. **所有工作流统一为纯手动触发 (`workflow_dispatch`)**：
   - 移除了 `.github/workflows/nightly.yml`、`render-benchmark.yml`、`omt-linux.yml`、`monkey.yml`、`mutation-test.yml`、`soak.yml` 中的 `schedule` 与 `pull_request` 自动触发器，连同 `build.yml`、`test.yml`、`screenshots.yml` 全部仅保留手动触发（仅 `auto-merge-builds.yml` 保留对 `codex/build-*` 分支的自动合并）。
2. **`project/scripts/build_all.py` 优化**：
   - **先本地后云端**：先在本机执行 `build_windows(version)`（同时更新固定入口 `ChurchPresenter.exe` 与 `msi`），若本地文件被占用可立即中断报错，确认本地构建成功后再上传源码并调度云端 Mac 构建。
   - **8 线程并发幂等重试上传源码**：`publish_source()` 使用 `ThreadPoolExecutor(max_workers=8)` 并发上传变更的 Git Blobs，并为 `git/blobs` 与 `git/trees` 增加 4 次指数退避重试。
   - **自动同步远程版本标签**：在 `prepare_windows_release()` 中自动执行 `git fetch origin --tags --quiet`，防止云端创建 Release 标签后本地因缺少标签导致 `appVersion()` 回退到旧版本号（如 `1.0.4`）。
   - **MSI 上传重试**：为 `gh release upload` 增加 4 次自动重试。

---

## 三、 当前版本与状态快照（2026-11-26）

- **当前最新版本**：`1.0.6`（Git 标签 `v1.0.6`）
- **本地固定入口状态**：
  - 路径：`D:\churchpre\project\composeApp\build\compose\binaries\main\app\ChurchPresenter\ChurchPresenter.exe`
  - 版本：已使用 `-PchurchPresenterVersion=1.0.6` 重新编译并验证启动正常（关于窗口显示 `1.0.6`），大小与模块裁剪均已生效。
- **本地 Windows MSI 安装包**：
  - 路径：`D:\churchpre\project\composeApp\build\compose\binaries\main\msi\ChurchPresenter-1.0.6.msi`（`450.07 MB`）。
- **远端仓库与 GitHub Release `v1.0.6` 状态**：
  - 分支：`codex/build-reference-startup-ui-20261126` 已通过 `Compare and merge builds to main` 自动同步至 `main`。
  - GitHub Release `v1.0.6` 已包含：
    - `ChurchPresenter-1.0.6-MACOS-M-Silicon.dmg` (`405.94 MB`)
    - `ChurchPresenter-1.0.6-MACOS-Intel.dmg` (`418.68 MB`)
    - Windows 安装包 `ChurchPresenter-1.0.6-WINDOWS-x64.msi`（因本机直传 450MB 到 `uploads.github.com` 遇到 HTTP 408 超时，已通过 GitHub Actions Run `45787798281` 在云端构建并自动挂载至 Release `v1.0.6`）。
