# Local project instructions

- 源码模块按架构层存放在 modules/{core,engines,features,output,integrations}/，新增或移动模块必须同步 config/architecture/module-paths.properties；Gradle 逻辑模块名保持不变，composeApp 固定入口路径保持不变。

- 目录分类以 docs/architecture/DIRECTORY_LAYOUT.md 为准；本地试验软件、未采用设计稿和个人笔记存入 .local/ 并保持忽略，正式品牌资源使用 assets/branding/。不得将 Gradle 模块、平台权限文件或必需原生工具当作无关文件清理。
- 完整工程位于仓库的 project/ 中；仓库根目录保留 README.md 作为项目首页，以及 .git、.github 和 .githooks 等 Git 基础设施。本文中的根目录指 project/ 工程根目录，新说明文档按用途放入 docs，辅助脚本放入 scripts。

- 构建分支更新后自动将当前提交合并到 cxze299/ChurchsProjection 的 main，无需再次询问；打包结果不作为分支合并条件，不自动合并无关依赖更新分支。

- 每次只修改用户当前指定的一个功能；发现其他问题先报告，未经用户明确要求不得顺带修改。
- 禁止无关重构、界面调整、依赖升级或发布配置变更；已有未提交修改不得混入本次功能修改。
- 每次功能修复必须更新 CHANGELOG.md，说明具体修复行为与验证范围，不得把未验证项目写为已通过。

- 所有已授权功能持续集成在当前源码中，统一构建同一个软件。
- 三平台构建由本地 project/Build-All.cmd 手动发起：默认本地构建 Windows，显式调度 Actions 打包 macOS M-Silicon 与 Intel；推送源码不自动打包，三平台使用同一源码与版本号。
- macOS 安装包使用仓库根目录 `.github/workflows/build.yml` 的 macOS runner 构建，交付时分别报告各平台的构建结果与可下载产物；遇到权限、签名或运行环境阻塞时明确说明。
- 使用 `:composeApp:createDistributable` 的标准输出目录。
- 用户已授权完整工程迁入 project/；固定软件入口更新为 `D:\churchpre\project\composeApp\build\compose\binaries\main\app\ChurchPresenter\ChurchPresenter.exe`，后续功能更新同一入口。
- 禁止为各个功能另建 `test-app*` 等独立软件输出目录。
- 遵循项目现有 `AGENT.md`，文件读取先定位再局部读取，忽略依赖、编译产物和日志。
- 终端构建、测试及包管理输出必须保持安静或截断，最多返回 50 行。
- 本项目由 Codex 与 Antigravity 共同管理。接手任务先读取仓库根目录 `agentswork.md` 及其工程内镜像 `project/agentswork.md` 的相关章节；关键修改后同步更新两份内容。用户最新指令优先于共享档案和仓库说明。
- 共同工作时登记任务负责人和涉及文件；同一文件的写入、Git 提交与分支切换、同一工程目录的 Gradle 操作按顺序执行。执行方式、模型分工与交接要求见 [agentswork.md](agentswork.md)。
