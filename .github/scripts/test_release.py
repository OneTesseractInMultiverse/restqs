"""Release policy tests; repository reads are replaced with in-memory fixtures."""

import subprocess
import unittest
from unittest.mock import call, patch

import release


SHA = "1" * 40
OTHER_SHA = "2" * 40
MANIFEST = '[package]\nname = "restqs"\nversion = "0.1.1"\n'


class ReleasePolicyTests(unittest.TestCase):
    def test_manual_validation_selects_tag(self):
        result = release.select_tag(
            "workflow_dispatch", {"inputs": {"tag": "v0.1.1"}}, "refs/heads/main", False
        )
        self.assertEqual(result, "v0.1.1")

    def test_release_event_selects_tag(self):
        result = release.select_tag(
            "release", {"release": {"tag_name": "v0.1.1"}}, "refs/tags/v0.1.1", True
        )
        self.assertEqual(result, "v0.1.1")

    def test_manual_publication_requires_main(self):
        with self.assertRaisesRegex(ValueError, "must run the workflow from main"):
            release.select_tag(
                "workflow_dispatch", {"inputs": {"tag": "v0.1.1"}}, "refs/heads/feature", True
            )

    def test_manual_validation_can_run_on_a_workflow_branch(self):
        result = release.select_tag(
            "workflow_dispatch", {"inputs": {"tag": "v0.1.1"}}, "refs/heads/feature", False
        )
        self.assertEqual(result, "v0.1.1")

    def test_release_event_requires_its_own_tag_ref(self):
        with self.assertRaisesRegex(ValueError, "must run from its own tag"):
            release.select_tag(
                "release", {"release": {"tag_name": "v0.1.1"}}, "refs/heads/main", True
            )

    def test_branch_ref_input_is_rejected(self):
        with self.assertRaisesRegex(ValueError, "Use a tag name"):
            release.select_tag(
                "workflow_dispatch", {"inputs": {"tag": "refs/heads/v0.1.1"}}, "refs/heads/main", False
            )

    def test_raw_revision_input_is_rejected(self):
        with self.assertRaisesRegex(ValueError, "Use a tag name"):
            release.select_tag(
                "workflow_dispatch", {"inputs": {"tag": SHA}}, "refs/heads/main", False
            )

    def test_shell_syntax_is_rejected(self):
        with self.assertRaisesRegex(ValueError, "Use a tag name"):
            release.select_tag(
                "workflow_dispatch", {"inputs": {"tag": 'v0.1.1;$(echo injected)'}}, "refs/heads/main", False
            )

    def test_output_injection_is_rejected(self):
        with self.assertRaisesRegex(ValueError, "Use a tag name"):
            release.select_tag(
                "workflow_dispatch", {"inputs": {"tag": "v0.1.1\nsha=other"}}, "refs/heads/main", False
            )

    def test_prerelease_tag_is_supported(self):
        result = release.select_tag(
            "workflow_dispatch", {"inputs": {"tag": "v0.2.0-rc.1"}}, "refs/heads/main", False
        )
        self.assertEqual(result, "v0.2.0-rc.1")

    def test_build_metadata_is_supported(self):
        result = release.select_tag(
            "workflow_dispatch", {"inputs": {"tag": "v0.2.0+build.1"}}, "refs/heads/main", False
        )
        self.assertEqual(result, "v0.2.0+build.1")

    def test_unexpected_event_is_rejected(self):
        with self.assertRaisesRegex(ValueError, "Only release and workflow_dispatch"):
            release.select_tag("pull_request", {}, "refs/heads/main", False)

    def test_release_tag_moved_before_resolution_is_rejected(self):
        with self.assertRaisesRegex(ValueError, "no longer points to the event commit"):
            release.validate_event_commit("release", SHA, OTHER_SHA)

    def test_unchanged_release_commit_is_accepted(self):
        self.assertIsNone(release.validate_event_commit("release", SHA, SHA))

    def test_manual_dispatch_can_select_an_older_merged_commit(self):
        self.assertIsNone(release.validate_event_commit("workflow_dispatch", OTHER_SHA, SHA))

    def test_manifest_version_mismatch_is_rejected(self):
        with self.assertRaisesRegex(ValueError, "must match"):
            release.release_identity("v0.1.2", SHA, {"package": {"name": "restqs", "version": "0.1.1"}})

    def test_another_package_is_rejected(self):
        with self.assertRaisesRegex(ValueError, "restqs package"):
            release.release_identity("v0.1.1", SHA, {"package": {"name": "other", "version": "0.1.1"}})

    def test_abbreviated_commit_is_rejected(self):
        with self.assertRaisesRegex(ValueError, "full commit SHA"):
            release.release_identity("v0.1.1", SHA[:7], {"package": {"name": "restqs", "version": "0.1.1"}})

    def test_matching_manifest_returns_release_metadata(self):
        result = release.release_identity("v0.1.1", SHA, {"package": {"name": "restqs", "version": "0.1.1"}})
        self.assertEqual(result, {"sha": SHA, "tag": "v0.1.1", "version": "0.1.1"})

    @patch("release.require_main_ancestor")
    @patch("release.git_output", side_effect=[SHA, MANIFEST])
    def test_manual_resolution_returns_peeled_commit(self, _git, _ancestor):
        result = release.resolve_release(
            "workflow_dispatch", {"inputs": {"tag": "v0.1.1"}}, "refs/heads/main", OTHER_SHA, False
        )
        self.assertEqual(result["sha"], SHA)

    @patch("release.require_main_ancestor")
    @patch("release.git_output", side_effect=[SHA, MANIFEST])
    def test_release_resolution_returns_event_commit(self, _git, _ancestor):
        result = release.resolve_release(
            "release", {"release": {"tag_name": "v0.1.1"}}, "refs/tags/v0.1.1", SHA, True
        )
        self.assertEqual(result["sha"], SHA)

    @patch("release.require_main_ancestor")
    @patch("release.git_output", side_effect=[SHA, MANIFEST])
    def test_lookup_uses_only_tag_namespace_and_pinned_manifest(self, git, _ancestor):
        release.resolve_release(
            "workflow_dispatch", {"inputs": {"tag": "v0.1.1"}}, "refs/heads/main", OTHER_SHA, False
        )
        self.assertEqual(git.call_args_list, [
            call("rev-parse", "--verify", "refs/tags/v0.1.1^{commit}"),
            call("show", f"{SHA}:Cargo.toml"),
        ])

    @patch("release.git_output", side_effect=subprocess.CalledProcessError(128, "git"))
    def test_missing_tag_fails_resolution(self, _git):
        with self.assertRaises(subprocess.CalledProcessError):
            release.resolve_release(
                "workflow_dispatch", {"inputs": {"tag": "v0.1.1"}}, "refs/heads/main", SHA, False
            )

    @patch("release.require_main_ancestor", side_effect=subprocess.CalledProcessError(1, "git"))
    @patch("release.git_output", return_value=SHA)
    def test_unmerged_commit_fails_resolution(self, _git, _ancestor):
        with self.assertRaises(subprocess.CalledProcessError):
            release.resolve_release(
                "workflow_dispatch", {"inputs": {"tag": "v0.1.1"}}, "refs/heads/main", SHA, False
            )

    def test_outputs_carry_immutable_commit(self):
        result = release.format_outputs({"sha": SHA, "tag": "v0.1.1", "version": "0.1.1"})
        self.assertEqual(result, f"sha={SHA}\ntag=v0.1.1\nversion=0.1.1\n")

    def test_summary_identifies_checked_revision(self):
        result = release.format_summary({"sha": SHA, "tag": "v0.1.1", "version": "0.1.1"})
        self.assertIn(f"Commit: `{SHA}`", result)


if __name__ == "__main__":
    unittest.main()
