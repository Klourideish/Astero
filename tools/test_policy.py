import copy
import json
import unittest
from check_policy import ROOT, check_dependencies, check_state


class PolicyTests(unittest.TestCase):
    def setUp(self):
        # Negative tests need a fixture even when approved cleanup leaves no active work.
        self.state = {"schema_version": 1, "active_items": [{
            "id": "policy-fixture", "scope": "repository policy", "summary": "Synthetic validation fixture",
            "status": "testing", "next_action": "Validate the fixture",
            "evidence_references": ["knowledge/architecture/dependency_policy.json"],
            "completion_conditions": ["Policy validation succeeds"],
        }]}
        self.policy = json.loads((ROOT / "knowledge/architecture/dependency_policy.json").read_text())
        self.metadata = {"workspace_members": list(self.policy["allowed_internal_dependencies"]),
                         "packages": [{"id": name, "name": name,
                                       "manifest_path": str(ROOT / "crates" / name / "Cargo.toml"),
                                       "dependencies": []}
                                      for name in self.policy["allowed_internal_dependencies"]]}

    def edge(self, source, destination, **extra):
        package = next(p for p in self.metadata["packages"] if p["name"] == source)
        package["dependencies"].append({"name": destination, "path": str(ROOT / "crates" / destination), **extra})

    def test_loader_cannot_gain_runtime_or_convenience_dependencies(self):
        for dependency in ("astero-memory", "astero-core", "astero-debug", "thiserror"):
            with self.subTest(dependency=dependency):
                metadata = copy.deepcopy(self.metadata)
                package = next(p for p in metadata["packages"] if p["name"] == "astero-loader")
                package["dependencies"].append({"name": dependency, "kind": "dev", "target": "cfg(windows)"})
                with self.assertRaisesRegex(ValueError, "dependency-free package declares dependencies"):
                    check_dependencies(metadata, self.policy)

    def test_loader_dependency_permission_requires_explicit_policy_change(self):
        self.policy["allowed_internal_dependencies"]["astero-loader"].append("astero-memory")
        with self.assertRaisesRegex(ValueError, "dependency-free package cannot allow"):
            check_dependencies(self.metadata, self.policy)

    def test_debug_loader_fixture_edge_is_dev_only(self):
        self.edge("astero-debug", "astero-loader", kind="dev")
        self.assertEqual(check_dependencies(self.metadata, self.policy), 1)
        package = next(p for p in self.metadata["packages"] if p["name"] == "astero-debug")
        for kind in (None, "build"):
            package["dependencies"][0]["kind"] = kind
            with self.assertRaisesRegex(ValueError, "test-only internal dependency"):
                check_dependencies(self.metadata, self.policy)

    def test_composition_edges_remain_downward(self):
        self.edge("astero-core", "astero-loader")
        self.edge("astero-cli", "astero-loader")
        self.assertEqual(check_dependencies(self.metadata, self.policy), 2)
        self.edge("astero-loader", "astero-core")
        with self.assertRaisesRegex(ValueError, "dependency-free package declares dependencies"):
            check_dependencies(self.metadata, self.policy)

    def test_valid(self):
        check_state(self.state)
        self.edge("astero-libs", "astero-hle")
        self.assertEqual(check_dependencies(self.metadata, self.policy), 1)

    def test_invalid_status(self):
        self.state["active_items"][0]["status"] = "completed"
        with self.assertRaises(ValueError):
            check_state(self.state)

    def test_duplicate_id(self):
        self.state["active_items"].append(copy.deepcopy(self.state["active_items"][0]))
        with self.assertRaises(ValueError):
            check_state(self.state)

    def test_missing_required_field(self):
        del self.state["active_items"][0]["next_action"]
        with self.assertRaises(ValueError):
            check_state(self.state)

    def test_forbidden_target_specific_renamed_dev_edge(self):
        self.edge("astero-kernel", "astero-core", rename="app", kind="dev", target="cfg(windows)")
        with self.assertRaises(ValueError):
            check_dependencies(self.metadata, self.policy)

    def test_cycle(self):
        self.policy["allowed_internal_dependencies"]["astero-memory"].append("astero-kernel")
        self.edge("astero-memory", "astero-kernel")
        self.edge("astero-kernel", "astero-memory")
        with self.assertRaises(ValueError):
            check_dependencies(self.metadata, self.policy)

    def test_policy_cannot_allow_runtime_debug(self):
        self.policy["allowed_internal_dependencies"]["astero-memory"].append("astero-debug")
        with self.assertRaises(ValueError):
            check_dependencies(self.metadata, self.policy)

    def test_missing_evidence(self):
        self.state["active_items"][0]["evidence_references"] = ["missing-evidence.md"]
        with self.assertRaises(ValueError):
            check_state(self.state)

    def test_debug_can_observe_core(self):
        self.edge("astero-debug", "astero-core")
        self.assertEqual(check_dependencies(self.metadata, self.policy), 1)

    def test_core_cannot_allow_debug(self):
        self.policy["allowed_internal_dependencies"]["astero-core"].append("astero-debug")
        with self.assertRaises(ValueError):
            check_dependencies(self.metadata, self.policy)

    def test_gui_cannot_allow_emulated_gpu(self):
        self.policy["allowed_internal_dependencies"]["astero-gui"].append("astero-gpu")
        with self.assertRaises(ValueError):
            check_dependencies(self.metadata, self.policy)

    def test_imgui_is_gui_only(self):
        self.edge("astero-core", "imgui")
        with self.assertRaises(ValueError):
            check_dependencies(self.metadata, self.policy)

    def test_rejected_gui_framework(self):
        self.edge("astero-gui", "eframe")
        with self.assertRaises(ValueError):
            check_dependencies(self.metadata, self.policy)

    def test_approved_cleanup_can_leave_no_active_items(self):
        check_state({"schema_version": 1, "active_items": []})

    def test_wrong_schema_and_non_object(self):
        for state in ([], {"schema_version": True, "active_items": []}, {"schema_version": 2, "active_items": []}):
            with self.assertRaises(ValueError):
                check_state(state)

    def test_evidence_cannot_escape_workspace(self):
        self.state["active_items"][0]["evidence_references"] = ["../outside.md"]
        with self.assertRaises(ValueError):
            check_state(self.state)


if __name__ == "__main__":
    unittest.main()
