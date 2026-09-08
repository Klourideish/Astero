"""Index invariants use a small synthetic repository, not emulator implementations."""
import json
from pathlib import Path
import tempfile
import unittest
from generate_indexes import ROOT, DEST, build, check_generated, digest, render, schema, validate
from rust_index import scan


class IndexTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        self.write("knowledge/architecture/crate_boundaries.md", "| astero-demo | Synthetic contracts | Runtime execution |\n")
        self.write("knowledge/architecture/module_structure.md", "Synthetic ownership\n")
        self.write("knowledge/architecture/validation.md", "Synthetic test evidence\n")
        self.write("knowledge/architecture/module_structure.json", json.dumps({"required_module_roots": {"astero-demo": ["empty"]}}))
        self.write("tools/index_links.json", json.dumps({"schema_version": 1, "test_links": [], "reviews": [], "nids": [], "abi": [], "diagnostics": []}))
        self.write("crates/astero-demo/README.md", "# Synthetic crate\n")
        self.write("crates/astero-demo/Cargo.toml", "# Metadata supplied by the fixture\n")
        self.write("crates/astero-demo/src/lib.rs", "//! Synthetic owner\npub mod empty;\nmod work;\n")
        self.write("crates/astero-demo/src/empty/mod.rs", "//! Planned home; no mechanisms.\n")
        self.write("crates/astero-demo/src/work.rs", "pub struct Thing;\nimpl Thing {\n pub fn run(&self) -> u8 { 1 }\n}\n#[cfg(test)]\nmod tests {\n #[test]\n fn exercise() { assert_eq!(super::Thing.run(), 1); }\n}\n")
        self.metadata = {"workspace_members": ["demo"], "packages": [{"id": "demo", "name": "astero-demo",
            "manifest_path": str(self.root / "crates/astero-demo/Cargo.toml"),
            "targets": [{"kind": ["lib"], "name": "astero_demo", "src_path": str(self.root / "crates/astero-demo/src/lib.rs")}]}]}

    def write(self, name, text):
        path = self.root / name
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(text, encoding="utf-8")

    def bundle(self):
        return build(self.root, self.metadata)

    def generated(self):
        data = self.bundle()
        validate(self.root, data)
        for filename, content in render(data).items():
            path = self.root / DEST / filename
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_bytes(content)
        return data

    def test_deterministic_regeneration_and_portable_paths(self):
        first = self.generated()
        self.assertEqual(render(first), render(self.bundle()))
        self.assertEqual(first, check_generated(self.root, self.metadata))
        self.assertNotIn(str(self.root), json.dumps(first))
        self.assertNotIn("generated_at", json.dumps(first))

    def test_locations_refresh_after_line_and_file_moves(self):
        first = self.bundle()
        before = next(r for r in first["implementation"]["records"] if r["symbol"] == "run")
        old = self.root / "crates/astero-demo/src/work.rs"
        text = old.read_text()
        self.write("crates/astero-demo/src/work/mod.rs", "\n\n" + text)
        old.unlink()  # Explicit file in the isolated temporary fixture only.
        after = next(r for r in self.bundle()["implementation"]["records"] if r["symbol"] == "run")
        self.assertEqual(before["id"], after["id"])
        self.assertEqual(before["symbol_hash"], after["symbol_hash"])
        self.assertEqual(after["lines"]["start"], before["lines"]["start"] + 2)
        self.assertNotEqual(before["file"], after["file"])
        validate(self.root, self.bundle())

    def test_stale_generated_content_requires_regeneration(self):
        self.generated()
        self.write("crates/astero-demo/src/work.rs", "pub struct Changed;\n")
        with self.assertRaisesRegex(ValueError, "stale/missing generated"):
            check_generated(self.root, self.metadata)
        self.generated()
        check_generated(self.root, self.metadata)

    def test_paths_must_exist_and_be_relative(self):
        for file in ("/host/file.rs", "C:/host/file.rs", "../escape.rs", "missing.rs"):
            data = self.bundle()
            data["implementation"]["records"][0]["file"] = file
            with self.subTest(file=file), self.assertRaises(ValueError):
                validate(self.root, data)

    def test_invalid_line_ranges_and_stale_hashes_fail(self):
        for bounds in ({"start": 0, "end": 1}, {"start": 3, "end": 2}, {"start": 1, "end": 9999}):
            data = self.bundle()
            data["implementation"]["records"][0]["lines"] = bounds
            with self.assertRaisesRegex(ValueError, "line range"):
                validate(self.root, data)
        data = self.bundle()
        data["sources"]["records"][0]["source_hash"] = "bad"
        with self.assertRaisesRegex(ValueError, "stale source"):
            validate(self.root, data)

    def test_known_empty_homes_are_scaffolded_even_for_empty_files(self):
        self.write("crates/astero-demo/src/empty/mod.rs", "")
        data = self.bundle()
        validate(self.root, data)
        empty = next(r for r in data["modules"]["records"] if r["declared_home"])
        self.assertEqual(empty["status"], "scaffolded")
        self.assertIsNone(empty["lines"])
        empty["status"] = "implemented"
        with self.assertRaisesRegex(ValueError, "scaffolded/implemented"):
            validate(self.root, data)

    def test_duplicate_ids_and_status_vocabulary_are_enforced(self):
        data = self.bundle()
        data["implementation"]["records"][1]["id"] = data["implementation"]["records"][0]["id"]
        with self.assertRaisesRegex(ValueError, "duplicate logical ID"):
            validate(self.root, data)
        data = self.bundle()
        data["implementation"]["records"][0]["status"] = "probably works"
        with self.assertRaisesRegex(ValueError, "status vocabulary"):
            validate(self.root, data)

    def test_implementation_owner_must_resolve_to_its_source(self):
        data = self.bundle()
        data["implementation"]["records"][0]["module_id"] = next(r["id"] for r in data["modules"]["records"] if r["declared_home"])
        with self.assertRaisesRegex(ValueError, "owner mismatch"):
            validate(self.root, data)

    def test_known_home_omission_is_detected(self):
        data = self.bundle()
        data["modules"]["records"] = [r for r in data["modules"]["records"] if not r["declared_home"]]
        data["modules"]["record_count"] -= 1
        with self.assertRaises(ValueError):
            validate(self.root, data)

    def test_normalized_line_endings_preserve_output(self):
        before = render(self.bundle())
        path = self.root / "crates/astero-demo/src/work.rs"
        path.write_bytes(path.read_text().replace("\n", "\r\n").encode("utf-8"))
        self.assertEqual(before, render(self.bundle()))

    def test_unlinked_files_are_visible_without_invented_module_paths(self):
        self.write("crates/astero-demo/src/orphan.rs", "pub fn orphan() {}\n")
        data = self.bundle()
        orphan = next(r for r in data["sources"]["records"] if r["file"].endswith("orphan.rs"))
        self.assertEqual(orphan["module_ids"], [])
        self.assertEqual(orphan["symbols"][0]["symbol"], "orphan")
        self.assertTrue(orphan["extraction_notes"])
        self.assertFalse(any(r["symbol"] == "orphan" for r in data["implementation"]["records"]))

    def test_empty_semantic_categories_keep_their_schemas(self):
        data = self.bundle()
        for kind in ("nids", "abi", "diagnostics"):
            self.assertEqual(data[kind]["records"], [])
            self.assertIn("implementation", schema()["$defs"][kind]["required"])
        self.assertIn("numeric_hex", schema()["$defs"]["nids"]["properties"]["nid"]["properties"])

    def test_reviewed_status_expires_without_inventing_new_evidence(self):
        data = self.bundle()
        row = next(r for r in data["implementation"]["records"] if r["symbol"] == "run")
        config = json.loads((self.root / "tools/index_links.json").read_text())
        config["reviews"] = [{"implementation": row["id"], "symbol_hash": row["symbol_hash"],
            "source_fingerprints": {row["file"]: digest((self.root / row["file"]).read_text())},
            "status": "tested", "evidence": ["knowledge/architecture/validation.md"], "scope": "Synthetic fixture evidence"}]
        self.write("tools/index_links.json", json.dumps(config))
        row = next(r for r in self.bundle()["implementation"]["records"] if r["symbol"] == "run")
        self.assertEqual(row["status"], "tested")
        path = self.root / row["file"]
        path.write_text(path.read_text().replace("{ 1 }", "{ 2 }"))
        row = next(r for r in self.bundle()["implementation"]["records"] if r["symbol"] == "run")
        self.assertEqual(row["status"], "implemented")
        self.assertEqual(row["validation_state"], "stale_source")

    def test_semantic_links_cannot_override_source_locations(self):
        data = self.bundle()
        config = json.loads((self.root / "tools/index_links.json").read_text())
        config["diagnostics"] = [{"id": "diagnostic:fake", "implementation": data["implementation"]["records"][0]["id"], "lines": {"start": 1, "end": 2}}]
        self.write("tools/index_links.json", json.dumps(config))
        with self.assertRaisesRegex(ValueError, "must not override"):
            self.bundle()

    def test_repo_indexes_match_current_sources(self):
        check_generated(ROOT)


class RustExtractionTests(unittest.TestCase):
    def test_literals_nested_comments_and_function_bodies_are_opaque(self):
        text = '/* outer /* fn fake() {} */ end */\npub fn real() { let s = r###"fn fake() { }"###; let c = \'{\'; let b = b"}"; /* } */ }\n'
        result = scan(text)
        self.assertEqual([r["symbol"] for r in result["symbols"]], ["real"])
        self.assertEqual(result["symbols"][0]["lines"], {"start": 2, "end": 2})

    def test_impl_trait_inline_modules_and_multiline_spans(self):
        result = scan('pub trait View { fn get(&self); }\nimpl View for Thing {\n fn get(&self) {\n }\n}\n#[cfg(test)] mod tests { #[test] fn check() {} }')
        get = [r for r in result["symbols"] if r["symbol"] == "get"]
        self.assertEqual(len(get), 2)
        self.assertNotEqual(get[0]["owner"], get[1]["owner"])
        self.assertEqual(get[1]["lines"], {"start": 3, "end": 4})
        test = next(r for r in result["symbols"] if r["symbol"] == "check")
        self.assertTrue(test["is_test"] and test["test_only"])
        self.assertEqual(test["module_suffix"], ["tests"])

    def test_macro_items_are_reported_without_expansion(self):
        result = scan('macro_rules! factory { () => { fn invented() {} }; }\nfactory!();\npub struct Real;')
        self.assertEqual([r["symbol"] for r in result["symbols"]], ["Real"])
        self.assertEqual(len(result["notes"]), 2)

    def test_path_attributes_and_malformed_delimiters_fail_explicitly(self):
        for source in ('#[path="elsewhere.rs"] mod other;', '#[cfg_attr(unix, path="other.rs")] mod other;', 'fn broken() {'):
            with self.subTest(source=source), self.assertRaises(ValueError):
                scan(source)

    def test_doc_fences_are_not_scanned_as_rust_implementations(self):
        result = scan('/// ```compile_fail\n/// fn pretend() {}\n/// ```\npub struct Real;\n')
        self.assertEqual([r["symbol"] for r in result["symbols"]], ["Real"])
        self.assertEqual(result["doctests"][0]["lines"], {"start": 1, "end": 3})
