"""M28 narrowly scoped Windows FFI exception; not a general unsafe policy relaxation."""
from pathlib import Path
import unittest

ROOT = Path(__file__).resolve().parents[1]

class NativePolicyTests(unittest.TestCase):
    def test_memory_unsafe_opt_in_is_only_private_windows_leaf(self):
        manifest = (ROOT / "crates/astero-memory/Cargo.toml").read_text(encoding="utf-8")
        self.assertIn('unsafe_code = "deny"', manifest)
        source = ROOT / "crates/astero-memory/src"
        opt_ins = [str(p.relative_to(source)).replace("\\", "/") for p in source.rglob("*.rs")
                   if "allow(unsafe_code)" in p.read_text(encoding="utf-8")]
        self.assertEqual(opt_ins, ["mapping/windows_native/mod.rs"])
        module = (source / opt_ins[0]).read_text(encoding="utf-8")
        self.assertIn('cfg(all(windows, target_arch = "x86_64"))', module)
        self.assertIn("mod platform;", module)
        self.assertNotIn("pub mod platform", module)
        self.assertIn('unsafe_code = "forbid"', (ROOT / "Cargo.toml").read_text(encoding="utf-8"))
