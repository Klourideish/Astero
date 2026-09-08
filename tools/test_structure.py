import tempfile
import unittest
from pathlib import Path
from check_structure import ROOT, check_structure


class StructureTests(unittest.TestCase):
    def setUp(self):
        (ROOT / "target").mkdir(exist_ok=True)
        self.temp = tempfile.TemporaryDirectory(dir=ROOT / "target")
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        self.src = self.root / "crates/astero-gpu/src"
        (self.src / "pm4/decode").mkdir(parents=True)
        (self.src / "lib.rs").write_text("pub mod pm4;\n")
        (self.src / "pm4/mod.rs").write_text("pub mod decode;\n")
        (self.src / "pm4/decode/mod.rs").write_text("//! Planned decode home.\n")
        self.policy = {"schema_version": 1, "required_module_roots": {"astero-gpu": ["pm4", "pm4/decode"]},
                       "forbidden_top_level_filenames": ["agc.rs", "pm4.rs"],
                       "discouraged_filenames": ["utils.rs"], "loose_file_exceptions": {}}

    def test_declared_homes_pass(self):
        self.assertEqual(check_structure(self.root, self.policy), 2)

    def test_missing_root_fails(self):
        (self.src / "pm4/decode/mod.rs").unlink()
        with self.assertRaisesRegex(ValueError, "missing module root"):
            check_structure(self.root, self.policy)

    def test_unwired_home_fails(self):
        (self.src / "pm4/mod.rs").write_text("//! No declaration.\n")
        with self.assertRaisesRegex(ValueError, "not declared"):
            check_structure(self.root, self.policy)

    def test_directory_file_collision_fails(self):
        (self.src / "pm4.rs").write_text("//! Wrong home.\n")
        with self.assertRaisesRegex(ValueError, "conflicts"):
            check_structure(self.root, self.policy)

    def test_known_loose_root_fails(self):
        (self.src / "agc.rs").write_text("//! Wrong home.\n")
        with self.assertRaisesRegex(ValueError, "justification"):
            check_structure(self.root, self.policy)

    def test_legitimate_small_leaf_allowed(self):
        (self.src / "pm4/decode/context.rs").write_text("//! Narrow leaf.\n")
        self.assertEqual(check_structure(self.root, self.policy), 2)

    def test_dumping_ground_requires_documented_exception(self):
        path = self.src / "pm4/decode/utils.rs"
        path.write_text("//! Narrow utility with an explicit review record.\n")
        with self.assertRaisesRegex(ValueError, "justification"):
            check_structure(self.root, self.policy)
        (self.root / "decision.md").write_text("A synthetic architectural exception for this test.\n")
        self.policy["loose_file_exceptions"][path.relative_to(self.root).as_posix()] = {
            "reason": "Synthetic bounded exception", "evidence": "decision.md"}
        self.assertEqual(check_structure(self.root, self.policy), 2)

    def test_traversal_rejected(self):
        self.policy["required_module_roots"]["astero-gpu"] = ["../escape"]
        with self.assertRaisesRegex(ValueError, "invalid module home"):
            check_structure(self.root, self.policy)

    def test_undeclared_parent_rejected(self):
        self.policy["required_module_roots"]["astero-gpu"] = ["pm4/decode"]
        with self.assertRaisesRegex(ValueError, "missing parent"):
            check_structure(self.root, self.policy)


if __name__ == "__main__":
    unittest.main()
