import unittest

from affected_modules import affected, module_of
from changed_tests import test_classes
from module_layout import MODULE_PATHS, ROOT, module_directory


class ArchitectureLayoutTest(unittest.TestCase):
    def test_repository_prefixed_paths_keep_original_modules_and_tasks(self):
        self.assertEqual("project/modules/core/theme", module_directory("theme", ROOT.parent))
        self.assertEqual(":bible-tab", module_of("project/modules/features/bible-tab/src/main/File.kt", {":bible-tab": set()}))
        self.assertEqual({":bible-tab"}, affected(["project/config/architecture/module-paths.properties"], {":bible-tab": set()}))
        groups = test_classes(["project/modules/features/bible-tab/src/test/kotlin/a/BrowseTest.kt"], read=lambda _: "package a\nclass BrowseTest")
        self.assertEqual({":bible-tab:test": ["a.BrowseTest"]}, groups)

    def test_registered_physical_paths_exist_and_resolve_to_original_task_names(self):
        graph = {":" + name: set() for name in MODULE_PATHS}
        for name, directory in MODULE_PATHS.items():
            self.assertTrue((ROOT / directory / "build.gradle.kts").is_file(), name)
            self.assertEqual(":" + name, module_of(directory + "/src/main/File.kt", graph))

    def test_mapping_changes_recheck_all_modules(self):
        graph = {":bible-tab": set(), ":settings": set()}
        self.assertEqual(set(graph), affected(["config/architecture/module-paths.properties"], graph))

    def test_nested_source_paths_keep_original_test_task(self):
        groups = test_classes(
            ["modules/features/bible-tab/src/test/kotlin/a/BrowseTest.kt"],
            read=lambda _: "package a\nclass BrowseTest",
        )
        self.assertEqual({":bible-tab:test": ["a.BrowseTest"]}, groups)
