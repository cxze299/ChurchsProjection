import io
import unittest
from contextlib import redirect_stdout
from unittest.mock import patch

import generate_release_notes as notes


class ChineseReleaseNotesTest(unittest.TestCase):
    def test_only_added_chinese_changelog_bullets_are_included(self):
        diff = "--- old\n+++ new\n@@ -1 +1 @@\n- 旧内容\n+- 修复投影预览。\n+- 修复投影预览。\n+- English title\n+### 日期\n context\n+- 增加构建计时。"
        with patch.object(notes, "run", return_value=diff) as run:
            self.assertEqual(["修复投影预览。", "增加构建计时。"], notes.chinese_changelog_entries("v1", "HEAD"))
        self.assertIn("v1", run.call_args.args[0])
        self.assertIn("HEAD", run.call_args.args[0])

    def test_generated_description_is_chinese_and_respects_platform_results(self):
        output = io.StringIO()
        argv = ["notes", "v1.0.2", "v1.0.3", "false", "true", "true", "false", "HEAD"]
        with patch.object(notes.sys, "argv", argv), patch.object(notes, "run", return_value="v1.0.2"), patch.object(notes, "chinese_changelog_entries", return_value=["修复右侧预览。"]), redirect_stdout(output):
            notes.main()
        body = output.getvalue()
        self.assertIn("**更新内容**", body)
        self.assertIn("修复右侧预览。", body)
        self.assertIn("/compare/v1.0.2...v1.0.3", body)
        self.assertIn("苹果 M 系列芯片", body)
        self.assertIn("ChurchPresenter-1.0.3-MACOS-Intel.dmg", body)
        self.assertNotIn("WINDOWS-x64.msi", body)
        self.assertNotIn("DEBIAN-x64.deb", body)
        self.assertNotIn("Download", body)

    def test_no_changelog_changes_uses_chinese_fallback(self):
        output = io.StringIO()
        with patch.object(notes.sys, "argv", ["notes", "v1.0.2", "v1.0.3"]), patch.object(notes, "run", return_value="v1.0.2"), patch.object(notes, "chinese_changelog_entries", return_value=[]), redirect_stdout(output):
            notes.main()
        self.assertIn("详细变更请查看下方源码版本对比", output.getvalue())


if __name__ == "__main__":
    unittest.main()
