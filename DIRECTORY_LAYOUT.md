# 项目目录分类

Gradle 模块保留根目录路径，避免破坏模块依赖、资源加载和构建配置。

| 分类 | 存放位置与用途 | Git |
|---|---|---|
| 应用入口 | `composeApp/` | 保留 |
| 功能界面 | `bible-tab/`、`songs/`、`slides/`、`media/`、`schedule/`、`calendar/`、`announcements/`、`canvas/` 等 Gradle 模块 | 保留 |
| 业务与渲染引擎 | `*-engine/`、`presenter/`、`live-output/`、`core-models/`、`settings/` 等 | 保留 |
| 界面与本地化资源 | `theme/`、`shared-ui/`、`icons/`、`strings/`、`profiles/` | 保留 |
| 服务及设备集成 | `server/`、`server-ui/`、`ndi/`、`omt/`、`obs/`、`atem/`、`native/` 等 | 保留 |
| 构建与自动化 | `gradle/`、`build-logic/`、`.github/`、`.githooks/`、`scripts/`、根目录 Gradle 文件及 `Build-All.*` | 保留 |
| 桌面系统集成 | `desktop/macos/` 权限声明、`native/` 必需库与工具 | 保留 |
| 质量检查 | 模块测试、截图基准、`qa/`、`config/`、`detekt-rules/`、检查脚本 | 保留 |
| 当前正式标志 | `assets/branding/`；平台图标仍位于构建要求的原有资源路径 | 保留 |
| 项目说明 | 根目录 README、许可证、来源声明、开发规范、changelog；专题文档位于 `docs/` | 保留 |
| 打包说明 | `docs/packaging/` | 保留 |
| 历史规范 | `docs/archive/legacy/`，仅供历史参考；当前规范仍以根目录文件为准 | 保留 |
| 未采用的标志方案 | `.local/archive/design/logo-concepts/` | 忽略，仅本机归档 |
| 历史临时软件 | `.local/distributables/test-app/`、`.local/distributables/test-app-latest/` | 忽略，仅本机归档 |
| 个人笔记 | `.local/notes/` | 忽略，仅本机保留 |
| 缓存、临时文件及构建产物 | `.gradle/`、`.kotlin/`、`**/build/`、`node_modules/`、`dist/`、`target/`、Python 缓存等 | 忽略 |

清理只移除当前 Git 树中的无关文件，保留提交历史，不重写 Git 历史。
`.gitignore` 不会自动取消已跟踪文件，因此草稿及个人笔记已明确从 Git 索引移除。
本地归档不会随一键构建的源码快照上传。
