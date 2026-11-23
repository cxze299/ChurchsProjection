import json
import unittest
from unittest.mock import patch

import build_all


class SourcePublishingTest(unittest.TestCase):
    def test_verified_tree_uses_a_new_branch_without_force_push(self):
        writes = []

        def fake_api(repo, path, payload=None):
            if payload is not None:
                writes.append((path, payload))
            if path.startswith("git/ref/"):
                return {"object": {"sha": "base"}}
            if path == "git/commits/base":
                return {"tree": {"sha": "tree"}}
            if path.startswith("git/trees/"):
                return {"tree": [{"path": "app.kt", "mode": "100644", "type": "blob", "sha": "blob"}]}
            return {"sha": "commit"}

        def fake_command(args, **kwargs):
            if args[0:2] == ["git", "ls-tree"]:
                return b"100644 blob blob\tapp.kt\0"
            if args[0:2] == ["git", "rev-parse"]:
                return b"tree\n"
            if args[0] == "gh":
                self.assertFalse(json.loads(kwargs["input"])["force"])
            return b""

        with patch.object(build_all, "api", side_effect=fake_api), patch.object(build_all, "command", side_effect=fake_command):
            branch, commit = build_all.publish_source("owner/repo", "builds/base")
        self.assertTrue(branch.startswith("codex/build-"))
        self.assertEqual("commit", commit)
        self.assertEqual(["git/commits", "git/refs"], [path for path, _ in writes])

    def test_tree_mismatch_stops_before_creating_a_commit_or_branch(self):
        def fake_api(repo, path, payload=None):
            self.assertIsNone(payload)
            if path.startswith("git/ref/"):
                return {"object": {"sha": "base"}}
            if path == "git/commits/base":
                return {"tree": {"sha": "remote-tree"}}
            return {"tree": []}

        def fake_command(args, **kwargs):
            return b"different-tree" if args[1] == "rev-parse" else b""

        with patch.object(build_all, "api", side_effect=fake_api), patch.object(build_all, "command", side_effect=fake_command):
            with self.assertRaisesRegex(RuntimeError, "源码校验失败"):
                build_all.publish_source("owner/repo", "builds/base")


if __name__ == "__main__":
    unittest.main()
