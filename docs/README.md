# 📚 ChurchPresenter Documentation

| File | What it holds |
|------|---------------|
| [CODING_STANDARDS.md](development/CODING_STANDARDS.md) | The style rules: imports, string resources, Material 3, type names |
| [DEVELOPMENT_GUIDE.md](development/DEVELOPMENT_GUIDE.md) | Workflow, verification commands, audit status, contributing |
| [AGENT.md](../AGENT.md) | Architecture, modules, commands, screenshot and test rules (read by coding agents, useful to anyone) |
| [FEATURES.md](product/FEATURES.md) | Every feature and where its source lives |
| `<module>/AGENT.md` | Each module's own layout, commands and rules |
| [BUILD_INSTALLERS.md](packaging/BUILD_INSTALLERS.md), [QUICK_START_INSTALLERS.md](packaging/QUICK_START_INSTALLERS.md) | Building the installers |
| [GRAPHICS_BACKEND.md](platform/GRAPHICS_BACKEND.md) | Choosing the GPU backend on a machine, and what it does to screen capture |
| [目录布局](architecture/DIRECTORY_LAYOUT.md) | 根目录保留文件、源码模块、文档与工具分类 |
| [Companion API](api/COMPANION_API.md) | Companion REST / WebSocket 接口 |
| [路线图](product/ROADMAP.md) | 功能规划 |
| [来源引用](legal/ATTRIBUTION.md) | 项目来源及开源引用 |
| 第三方声明：[FFmpeg](legal/THIRD_PARTY_FFMPEG.md)、[MiniLM](legal/THIRD_PARTY_MINILM.md)、[OMT](legal/THIRD_PARTY_OMT.md) | 第三方组件授权说明 |

## Before every commit

```bash
bash scripts/cleanup_check.sh          # wildcard imports, Material 2, prints, fully qualified names, unused code
./gradlew :composeApp:detekt   # CI's first gate — run it last
```

## File locations

- **String resources:** `modules/core/strings/src/main/composeResources/values/strings.xml` (English only)
- **Constants:** `modules/core/settings/src/main/kotlin/org/churchpresenter/settings/utils/Constants.kt` (anything
  persisted or shared with the settings)

Historical coding standards are kept in `docs/archive/legacy/`. Personal notes and unselected logo drafts are machine-local under `.local/` and are excluded from Git.

See [DIRECTORY_LAYOUT.md](architecture/DIRECTORY_LAYOUT.md) for the directory categories.
