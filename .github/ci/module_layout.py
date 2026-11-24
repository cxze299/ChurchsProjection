"""Physical module paths shared by CI; Gradle project names remain unchanged."""
from pathlib import Path

REPOSITORY_ROOT = Path(__file__).resolve().parents[2]
ROOT = REPOSITORY_ROOT / "project"
MODULE_PATHS = {}
for line in (ROOT / "config/architecture/module-paths.properties").read_text().splitlines():
    if "=" in line and not line.lstrip().startswith("#"):
        name, directory = line.split("=", 1)
        MODULE_PATHS[name.strip()] = directory.strip()


def module_directory(name, root="."):
    directory = MODULE_PATHS.get(name, name)
    # Support older source trees and the existing CI test fixtures as well.
    if (Path(root) / directory).is_dir():
        return directory
    nested = Path("project") / directory
    return nested.as_posix() if (Path(root) / nested).is_dir() else name
