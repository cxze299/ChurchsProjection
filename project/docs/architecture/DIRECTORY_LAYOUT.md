# 项目目录分类

完整工程已统一迁入仓库的 `project/`。除特别说明外，下文路径均相对该工程目录。

| 仓库根目录 | 用途 |
|---|---|
| `README.md` | 仓库首页，链接到 project 内的工程和文档 |
| `project/` | 全部工程文件、源码、文档、构建配置及本地缓存 |
| `.github/` | 必须位于仓库根目录的 Actions 和 Issue 模板 |
| `.githooks/` | Git 提交钩子 |
| `.git/` | 本地 Git 仓库数据，不作为项目文件提交 |

Windows 构建入口：`D:\churchpre\project\Build-All.cmd`。
软件入口：`D:\churchpre\project\composeApp\build\compose\binaries\main\app\ChurchPresenter\ChurchPresenter.exe`。

模块已按现有架构层实际迁入 `modules/`。逻辑 Gradle 名称保持不变，完整物理路径由 `config/architecture/module-paths.properties` 统一定义。

## 源码分层

| 目录 | 职责 | 模块数 |
|---|---|---|
| `modules/core/` | 基础与共享模型 | 13 |
| `modules/engines/` | 协议、格式、业务及渲染引擎 | 19 |
| `modules/features/` | 用户功能与界面模块 | 24 |
| `modules/output/` | 实时投影输出管线 | 1 |
| `modules/integrations/` | 跨模块集成与应用配置 | 6 |
| `composeApp/` | 桌面应用入口；保留原位置与固定软件输出路径 | 1 |

## 其他目录

| 目录 | 用途 |
|---|---|
| `platform/desktop/` | macOS 权限与平台配置 |
| `platform/native/` | 必需原生库与工具 |
| `tools/wick-pack/` | 独立辅助工具 |
| `build-logic/`、`gradle/` | 构建约定、版本目录与 Wrapper |
| `scripts/`、`config/` | 工程构建入口、辅助脚本和架构规范；仓库自动化见上表 |
| `assets/branding/` | 正式标志和仓库品牌资源 |
| `docs/` | 架构专题、`packaging/` 打包说明、`archive/legacy/` 历史规范 |
| `.local/archive/design/` | 未采用设计稿，本地归档、Git 忽略 |
| `.local/distributables/` | 历史临时软件，本地归档、Git 忽略 |
| `.local/notes/` | 个人笔记，Git 忽略 |

## 工程根目录（project/）文件与用途

根目录现有文件均与项目有关；按工具发现规则、授权要求和日常入口保留以下文件：

| 文件 | 保留原因 |
|---|---|
| `CHANGELOG.md` | 更新记录 |
| `LICENSE`、`LICENSE.txt` | 项目授权文件，保留现有副本及引用兼容 |
| `AGENTS.md`、`AGENT.md`、`CLAUDE.md` | 自动发现的开发代理指令与规范 |
| `Build-All.cmd` | 本机一键构建入口，调用 scripts 内脚本 |
| `build.gradle.kts`、`settings.gradle.kts`、`gradle.properties` | Gradle 根工程配置 |
| `gradlew`、`gradlew.bat` | Gradle Wrapper 启动器 |
| `.gitignore` | 项目忽略规则 |

已归类文件：

| 目录 | 内容 |
|---|---|
| `docs/README.md` | 全部文档索引 |
| `docs/architecture/` | 目录布局说明 |
| `docs/development/` | 编码规范、开发指南 |
| `docs/product/` | 功能说明、路线图 |
| `docs/api/` | Companion 接口文档 |
| `docs/platform/` | 图形后端说明 |
| `docs/legal/` | 来源引用、FFmpeg / MiniLM / OMT 第三方声明 |
| `scripts/` | PowerShell 构建脚本、质量检查、增量测试、发布说明生成 |

## 构建与维护规则

- `Build-All.cmd`、Gradle Wrapper 和根目录构建配置仍在原位置。
- 固定入口：`composeApp/build/compose/binaries/main/app/ChurchPresenter/ChurchPresenter.exe`。
- 任务名不变，例如 `:bible-tab:compileKotlin`，源码位于 `modules/features/bible-tab/`。
- 新模块必须同时注册 Gradle、架构层级和物理路径映射。
- CI 根据映射识别变化和测试报告；不能再假设模块都在根目录。
- 缓存、构建产物、依赖与 `.local/` 继续忽略，不上传 GitHub。
