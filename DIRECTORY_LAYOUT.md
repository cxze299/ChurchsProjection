# 项目目录分类

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
| `.github/`、`.githooks/`、`scripts/`、`config/` | 自动化、构建入口和架构规范 |
| `assets/branding/` | 正式标志和仓库品牌资源 |
| `docs/` | 架构专题、`packaging/` 打包说明、`archive/legacy/` 历史规范 |
| `.local/archive/design/` | 未采用设计稿，本地归档、Git 忽略 |
| `.local/distributables/` | 历史临时软件，本地归档、Git 忽略 |
| `.local/notes/` | 个人笔记，Git 忽略 |

## 构建与维护

- `Build-All.cmd`、Gradle Wrapper 和根目录构建配置仍在原位置。
- 固定入口：`composeApp/build/compose/binaries/main/app/ChurchPresenter/ChurchPresenter.exe`。
- 任务名不变，例如 `:bible-tab:compileKotlin`，源码位于 `modules/features/bible-tab/`。
- 新模块必须同时注册 Gradle、架构层级和物理路径映射。
- CI 根据映射识别变化和测试报告；不能再假设模块都在根目录。
- 缓存、构建产物、依赖与 `.local/` 继续忽略，不上传 GitHub。
