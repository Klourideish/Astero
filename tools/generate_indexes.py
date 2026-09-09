"""Deterministic repository indexes. Run normally to write; --check rejects any drift."""
import argparse
import ast
import hashlib
import json
from pathlib import Path
import re
import subprocess
from rust_index import scan

ROOT = Path(__file__).resolve().parents[1]
DEST = Path("knowledge/indexes")
STATUSES = {"planned", "scaffolded", "implemented", "tested", "runtime_validated"}
FILES = {"implementation": "implementation.json", "subsystems": "subsystems.json", "modules": "modules.json",
         "nids": "nids.json", "abi": "abi.json", "diagnostics": "diagnostics.json", "tests": "tests.json", "sources": "SOURCE_INDEX.json"}
MARKDOWN = {"implementation": "IMPLEMENTATION_INDEX.md", "subsystems": "SUBSYSTEM_INDEX.md", "modules": "MODULE_INDEX.md",
            "nids": "NID_INDEX.md", "abi": "ABI_INDEX.md", "diagnostics": "DIAGNOSTIC_INDEX.md", "tests": "TEST_INDEX.md"}


def require(value, message):
    if not value:
        raise ValueError(message)


def read(path):
    return path.read_text(encoding="utf-8").replace("\r\n", "\n")


def digest(text):
    return hashlib.sha256(text.encode("utf-8")).hexdigest()


def json_bytes(value):
    return (json.dumps(value, ensure_ascii=True, indent=2) + "\n").encode("utf-8")


def relative(root, path):
    path = path.resolve()
    require(path.is_relative_to(root.resolve()), f"path outside repository: {path.name}")
    return path.relative_to(root.resolve()).as_posix()


def workspace(root):
    return json.loads(subprocess.check_output(["cargo", "metadata", "--no-deps", "--locked", "--format-version", "1"], cwd=root, text=True))


def ownership(root):
    owners = {}
    for line in read(root / "knowledge/architecture/crate_boundaries.md").splitlines():
        cells = [c.strip() for c in line.split("|")]
        if len(cells) == 5 and cells[1].startswith("astero-"):
            owners[cells[1]] = (cells[2], cells[3])
    return owners


def docs_for(root, crate_dir):
    docs = {"knowledge/architecture/crate_boundaries.md", "knowledge/architecture/module_structure.md"}
    README = crate_dir / "README.md"
    docs.add(relative(root, README))
    for target in re.findall(r"\]\(([^)]+)\)", read(README)):
        target = target.split("#")[0]
        if target and not re.match(r"[a-zA-Z]+:", target):
            path = (crate_dir / target).resolve()
            if path.is_relative_to(root.resolve()) and path.is_file() and path.suffix == ".md":
                docs.add(relative(root, path))
    return sorted(docs)


def build(root=ROOT, metadata=None):
    root = root.resolve()
    metadata = workspace(root) if metadata is None else metadata
    owners = ownership(root)
    homes = json.loads(read(root / "knowledge/architecture/module_structure.json"))["required_module_roots"]
    bundle = {kind: [] for kind in FILES}
    source_map, scanned, seen = {}, {}, set()

    def source_record(path, crate, category):
        file = relative(root, path)
        if file not in source_map:
            text = read(path)
            parsed = scan(text)
            scanned[file] = (text, parsed)
            source_map[file] = {"id": "source:" + file, "crate": crate, "file": file, "category": category,
                                "language": "rust", "line_count": len(text.splitlines()), "source_hash": digest(text),
                                "module_ids": [], "symbols": [], "extraction_notes": parsed["notes"]}
        return source_map[file]

    def visit(path, crate, target, parts, category, docs, inherited=()):
        file = relative(root, path)
        key = (file, target, parts)
        require(key not in seen, f"duplicate/cyclic module resolution: {key}")
        seen.add(key)
        src = source_record(path, crate, category)
        text, parsed = scanned[file]
        prefix = f"{crate}/{target}/"
        sub = parts[0] if parts else "root"
        module_name = "::".join(parts) or "crate"
        mid = "module:" + prefix + module_name
        leading = " ".join(line[3:].strip() for line in text.splitlines() if line.startswith("//!"))
        module = {"id": mid, "crate": crate, "target": target, "module": module_name, "category": category,
                  "subsystem": sub, "file": file, "lines": {"start": 1, "end": src["line_count"]} if src["line_count"] else None,
                  "source_id": src["id"], "implementation_home": relative(root, path.parent), "responsibility": leading or owners[crate][0],
                  "forbidden_responsibility": owners[crate][1], "knowledge": docs,
                  "implementation_ids": [], "status": "scaffolded", "confidence": "source_backed",
                  "declared_home": file in {f"crates/{c}/src/{h}/mod.rs" for c, paths in homes.items() for h in paths},
                  "attributes": list(inherited)}
        bundle["modules"].append(module)
        src["module_ids"].append(mid)
        symbol_rows = []
        for symbol in parsed["symbols"]:
            full_parts = parts + tuple(symbol["module_suffix"])
            modname = "::".join(full_parts) or "crate"
            symbol_mid = "module:" + prefix + modname
            qualified = "::".join(full_parts + tuple(symbol["owner"]) + (symbol["symbol"],))
            logical = prefix + qualified
            row = {"id": "impl:" + logical, "crate": crate, "target": target, "category": category,
                   "subsystem": full_parts[0] if full_parts else "root", "module": modname, "module_id": symbol_mid,
                   "symbol": symbol["symbol"], "symbol_kind": symbol["symbol_kind"], "owner": symbol["owner"],
                   "file": file, "lines": symbol["lines"], "visibility": symbol["visibility"],
                   "status": "scaffolded" if symbol["symbol_kind"] == "fn" and not symbol["has_body"] else "implemented",
                   "confidence": "source_backed", "symbol_hash": digest(text[symbol["start"]:symbol["end"]]),
                   "attributes": list(inherited) + symbol["attributes"], "related_tests": [], "related_nids": [],
                   "related_abi": [], "related_diagnostics": [], "knowledge": docs}
            src["symbols"].append({k: row[k] for k in ("symbol", "symbol_kind", "module", "owner", "lines", "visibility")})
            is_testing = target.startswith("test:") or symbol["test_only"] or any("cfg(test)" == a.replace(" ", "") for a in inherited)
            if symbol["is_test"]:
                test = {k: row[k] for k in ("crate", "category", "subsystem", "module", "module_id", "file", "lines")}
                test.update(id="test:" + logical, name=symbol["symbol"], test_type="integration" if target.startswith("test:") else "unit",
                            language="rust", evidence_level="test_declared_not_executed", related_implementations=[],
                            lexical_identifiers=symbol["identifiers"], reference_confidence="lexical_only_not_resolved")
                bundle["tests"].append(test)
            elif not is_testing:
                bundle["implementation"].append(row)
            symbol_rows.append((symbol, logical, symbol_mid))
        # Inline modules have their own logical owner but share one physical file.
        for child in parsed["modules"]:
            child_parts = parts + tuple(child["module"])
            if child["inline"]:
                inline = dict(module)
                inline.update(id="module:" + prefix + "::".join(child_parts), module="::".join(child_parts),
                              subsystem=child_parts[0], lines=child["lines"], declared_home=False,
                              attributes=list(inherited) + child["attributes"], implementation_ids=[])
                bundle["modules"].append(inline)
                src["module_ids"].append(inline["id"])
            else:
                base = path.parent if path.name in {"mod.rs", "lib.rs", "main.rs"} or path == Path(next(t["src_path"] for p in metadata["packages"] if p["name"] == crate for t in p["targets"] if target == t["kind"][0] + ":" + t["name"])) else path.with_suffix("")
                stem = base.joinpath(*child["module"])
                candidates = [p for p in (stem.with_suffix(".rs"), stem / "mod.rs") if p.is_file()]
                require(len(candidates) == 1, f"missing/ambiguous module {file}: {'::'.join(child_parts)}")
                visit(candidates[0], crate, target, child_parts, category, docs, tuple(inherited) + tuple(child["attributes"]))
        ordinals = {}
        for doc in parsed["doctests"]:
            following = [(s, logical, owner) for s, logical, owner in symbol_rows if s["start"] >= doc["end"]]
            if not following:
                src["extraction_notes"].append({"line": doc["lines"]["start"], "reason": "unattached doctest fence"})
                continue
            symbol, logical, doc_mid = min(following, key=lambda item: item[0]["start"])
            ordinals[logical] = ordinals.get(logical, 0) + 1
            bundle["tests"].append({"id": "test:" + logical + ":doctest:" + str(ordinals[logical]),
                "name": symbol["symbol"] + " doctest " + str(ordinals[logical]), "crate": crate, "category": category,
                "subsystem": sub, "module": module_name, "module_id": doc_mid, "file": file, "lines": doc["lines"],
                "test_type": "doctest:" + doc["test_type"], "language": "rust", "evidence_level": "test_declared_not_executed",
                "related_implementations": ["impl:" + logical], "lexical_identifiers": [], "reference_confidence": "attached_doc_fence"})

    for package in sorted(metadata["packages"], key=lambda p: p["name"]):
        if package["id"] not in metadata["workspace_members"]:
            continue
        crate = package["name"]
        require(crate in owners, f"missing ownership record: {crate}")
        crate_dir = Path(package["manifest_path"]).parent
        category = crate.removeprefix("astero-")
        docs = docs_for(root, crate_dir)
        for path in sorted(crate_dir.rglob("*.rs")):
            if not {"target", ".git"}.intersection(path.relative_to(crate_dir).parts):
                source_record(path, crate, category)
        for target in sorted(package["targets"], key=lambda t: (t["kind"], t["name"])):
            visit(Path(target["src_path"]), crate, target["kind"][0] + ":" + target["name"], (), category, docs)

    # Python tools use the standard library AST; test discovery does not imply passing tests.
    for path in sorted((root / "tools").glob("*.py")):
        text, file = read(path), relative(root, path)
        tree = ast.parse(text)
        mid = "module:tools/" + path.stem
        bundle["modules"].append({"id": mid, "crate": None, "target": "python", "module": path.stem,
            "category": "tooling", "subsystem": "indexes" if "index" in path.stem else "policy", "file": file,
            "source_id": "source:" + file, "implementation_home": "tools", "lines": {"start": 1, "end": max(1, len(text.splitlines()))},
            "status": "implemented", "confidence": "python_ast", "responsibility": ast.get_docstring(tree) or "Repository validation tools",
            "forbidden_responsibility": "Emulator runtime mechanisms", "knowledge": ["knowledge/architecture/validation.md"],
            "implementation_ids": [], "declared_home": False, "attributes": []})
        symbols = []
        def python_items(nodes, parent=""):
            for node in nodes:
                if isinstance(node, (ast.ClassDef, ast.FunctionDef, ast.AsyncFunctionDef)):
                    symbols.append({"symbol": node.name, "symbol_kind": type(node).__name__, "module": path.stem,
                                    "owner": [parent] if parent else [], "visibility": "internal",
                                    "lines": {"start": node.lineno, "end": node.end_lineno}})
                    if isinstance(node, ast.ClassDef):
                        python_items(node.body, node.name)
                    elif parent and node.name.startswith("test_") and path.name.startswith("test_"):
                        bundle["tests"].append({"id": f"test:tools/{path.stem}/{parent}::{node.name}", "name": node.name,
                            "crate": None, "category": "tooling", "subsystem": "indexes" if "index" in path.stem else "policy",
                            "module": path.stem, "module_id": mid, "file": file, "lines": {"start": node.lineno, "end": node.end_lineno},
                            "test_type": "unittest", "language": "python", "evidence_level": "test_declared_not_executed",
                            "related_implementations": [], "lexical_identifiers": [], "reference_confidence": "not_resolved"})
        python_items(tree.body)
        source_map[file] = {"id": "source:" + file, "crate": None, "category": "tooling", "language": "python", "file": file,
                            "line_count": len(text.splitlines()), "source_hash": digest(text), "module_ids": [mid],
                            "symbols": symbols, "extraction_notes": []}
    bundle["sources"] = sorted(source_map.values(), key=lambda s: s["id"])
    for src in bundle["sources"]:
        src["module_ids"] = sorted(set(src["module_ids"]))
        if not src["module_ids"]:
            src["symbols"] = [{"symbol": s["symbol"], "symbol_kind": s["symbol_kind"], "owner": s["owner"],
                               "module": None, "visibility": s["visibility"], "lines": s["lines"]}
                              for s in scanned[src["file"]][1]["symbols"]]
            src["extraction_notes"].append({"line": 1, "reason": "unlinked Rust source; no module path inferred"})
        # Multiple Cargo targets can legitimately reuse one support file.
        src["symbols"] = sorted({json.dumps(s, sort_keys=True): s for s in src["symbols"]}.values(), key=lambda s: (s["lines"]["start"], s["module"], s["symbol"]))
    module_lookup = {m["id"]: m for m in bundle["modules"]}
    for src in bundle["sources"]:
        src["module_paths"] = sorted({module_lookup[mid]["module"] for mid in src["module_ids"]})
        src["subsystems"] = sorted({module_lookup[mid]["subsystem"] for mid in src["module_ids"]})
    apply_links(root, bundle)
    for module in bundle["modules"]:
        module["implementation_ids"] = sorted(r["id"] for r in bundle["implementation"] if r["module_id"] == module["id"])
        if module["category"] != "tooling":
            descendants = [r for r in bundle["implementation"] if r["crate"] == module["crate"] and r["target"] == module["target"]
                           and (module["module"] == "crate" or r["module"] == module["module"] or r["module"].startswith(module["module"] + "::"))]
            module["status"] = "implemented" if any(r["status"] != "scaffolded" for r in descendants) else "scaffolded"
    for crate, category, sub in sorted({(m["crate"] or "tools", m["category"], m["subsystem"]) for m in bundle["modules"]}):
        selected = [m for m in bundle["modules"] if (m["crate"] or "tools", m["category"], m["subsystem"]) == (crate, category, sub)]
        bundle["subsystems"].append({"id": f"subsystem:{crate}/{sub}", "crate": None if crate == "tools" else crate,
            "category": category, "subsystem": sub, "module_ids": sorted(m["id"] for m in selected),
            "implementation_ids": sorted({i for m in selected for i in m["implementation_ids"]}),
            "responsibility": next((m["responsibility"] for m in selected if m["module"] == sub), selected[0]["responsibility"]),
            "forbidden_responsibility": owners[crate][1] if crate in owners else "Emulator runtime mechanisms",
            "knowledge": sorted({d for m in selected for d in m["knowledge"]}),
            "status": "implemented" if any(m["status"] != "scaffolded" for m in selected) else "scaffolded",
            "confidence": "documented_ownership"})
    return {kind: {"schema_version": 1, "generator_version": 1, "kind": kind, "record_count": len(records),
                   "records": sorted(records, key=lambda r: r["id"])} for kind, records in bundle.items()}


def apply_links(root, bundle):
    config = json.loads(read(root / "tools/index_links.json"))
    require(config.get("schema_version") == 1, "unsupported links schema")
    implementations = {r["id"]: r for r in bundle["implementation"]}
    tests = {r["id"]: r for r in bundle["tests"]}
    for link in config["test_links"]:
        require(link["implementation"] in implementations, "unresolved implementation link")
        row = implementations[link["implementation"]]
        for test in link["tests"]:
            require(test in tests, "unresolved test link")
            row["related_tests"].append(test)
            tests[test]["related_implementations"].append(row["id"])
    for test in tests.values():
        for ref in test["related_implementations"]:
            require(ref in implementations, "unresolved attached doctest")
            implementations[ref]["related_tests"].append(test["id"])
    for review in config["reviews"]:
        require(review["implementation"] in implementations, "unresolved review")
        row = implementations[review["implementation"]]
        require(review["status"] in STATUSES and review["evidence"] and review["scope"], "invalid status review")
        require(review["source_fingerprints"], "review needs pinned source evidence")
        fresh = row["symbol_hash"] == review["symbol_hash"]
        for file, expected in review["source_fingerprints"].items():
            path = check_path(root, file)
            fresh = fresh and digest(read(path)) == expected
        row.update(validation_evidence=review["evidence"], validation_scope=review["scope"],
                   validation_state="current_source" if fresh else "stale_source")
        if fresh:
            row["status"] = review["status"]
    for category in ("nids", "abi", "diagnostics"):
        for entry in config[category]:
            require(not set(entry).intersection({"file", "lines", "crate", "category", "subsystem", "module", "symbol", "status"}),
                    "semantic links must not override generated locations/status")
            require(entry["implementation"] in implementations, f"unresolved {category} symbol")
            owner = implementations[entry["implementation"]]
            row = {k: owner[k] for k in ("crate", "category", "subsystem", "module", "file", "lines", "symbol", "status")}
            row.update(entry)
            row["confidence"] = "reviewed_relationship"
            row["related_tests"] = sorted(set(owner["related_tests"]))
            bundle[category].append(row)
            owner["related_" + category].append(row["id"])
    for row in implementations.values():
        for field in ("related_tests", "related_nids", "related_abi", "related_diagnostics"):
            row[field] = sorted(set(row[field]))
    for row in tests.values():
        row["related_implementations"] = sorted(set(row["related_implementations"]))


REQUIRED = {
    "implementation": ["id", "crate", "category", "subsystem", "module", "module_id", "symbol", "symbol_kind", "file", "lines", "status", "confidence", "related_tests", "related_nids", "related_abi", "related_diagnostics"],
    "subsystems": ["id", "crate", "category", "subsystem", "module_ids", "implementation_ids", "responsibility", "forbidden_responsibility", "knowledge", "status"],
    "modules": ["id", "crate", "module", "category", "subsystem", "file", "lines", "source_id", "implementation_ids", "responsibility", "forbidden_responsibility", "knowledge", "status", "declared_home"],
    "sources": ["id", "crate", "file", "category", "language", "line_count", "source_hash", "module_ids", "module_paths", "subsystems", "symbols", "extraction_notes"],
    "tests": ["id", "name", "crate", "file", "lines", "category", "subsystem", "module_id", "test_type", "language", "evidence_level", "related_implementations", "lexical_identifiers", "reference_confidence"],
    "nids": ["id", "nid", "guest_library", "guest_module", "implementation", "file", "lines", "status", "registration_status", "related_abi", "related_tests", "evidence", "provenance"],
    "abi": ["id", "guest_concept", "guest_library", "implementation", "file", "lines", "subsystem", "status", "related_tests", "evidence"],
    "diagnostics": ["id", "implementation", "file", "lines", "subsystem", "status", "consumers", "surfaces", "related_tests", "evidence", "description"],
}


def schema():
    # Portable JSON Schema for consumers; the repo validator additionally checks graph/source consistency.
    string = {"type": "string"}
    strings = {"type": "array", "items": string, "uniqueItems": True}
    properties = {key: string for key in ("id", "category", "subsystem", "module", "module_id", "symbol", "symbol_kind", "file", "confidence", "responsibility", "forbidden_responsibility", "source_id", "source_hash", "name", "test_type", "language", "evidence_level", "reference_confidence", "guest_library", "guest_module", "implementation", "provenance", "guest_concept", "description")}
    properties.update({key: strings for key in ("related_tests", "related_nids", "related_abi", "related_diagnostics", "related_implementations", "module_ids", "module_paths", "subsystems", "implementation_ids", "knowledge", "lexical_identifiers", "consumers", "surfaces", "evidence", "validation_evidence")})
    properties.update(crate={"type": ["string", "null"]}, status={"enum": sorted(STATUSES)},
        lines={"type": ["object", "null"], "required": ["start", "end"], "properties": {"start": {"type": "integer", "minimum": 1}, "end": {"type": "integer", "minimum": 1}}, "additionalProperties": False},
        line_count={"type": "integer", "minimum": 0}, declared_home={"type": "boolean"},
        symbols={"type": "array", "items": {"type": "object"}}, extraction_notes={"type": "array", "items": {"type": "object"}},
        registration_status={"enum": ["planned", "unregistered", "registered"]},
        nid={"type": "object", "minProperties": 1, "properties": {"numeric_hex": {"type": "string", "pattern": "^0x[0-9a-fA-F]+$"}, "encoded": string}, "additionalProperties": False})
    return {"$schema": "https://json-schema.org/draft/2020-12/schema", "title": "Astero generated navigation envelope v1",
        "oneOf": [{"type": "object", "required": ["schema_version", "generator_version", "kind", "record_count", "records"],
                   "properties": {"schema_version": {"const": 1}, "generator_version": {"const": 1}, "kind": {"const": kind},
                       "record_count": {"type": "integer", "minimum": 0}, "records": {"type": "array", "items": {"$ref": "#/$defs/" + kind}}},
                   "additionalProperties": False} for kind in FILES],
        "$defs": {kind: {"type": "object", "required": required, "properties": properties} for kind, required in REQUIRED.items()}}


def check_path(root, file):
    require(isinstance(file, str) and file and "\\" not in file and ":" not in file and not file.startswith("/") and ".." not in Path(file).parts,
            f"non-relative repository path: {file}")
    path = (root / file).resolve()
    require(path.is_relative_to(root.resolve()) and path.is_file(), f"missing indexed file: {file}")
    return path


def validate(root, bundle):
    definitions = schema()["$defs"]
    rows = {kind: envelope["records"] for kind, envelope in bundle.items()}
    require(set(rows) == set(FILES), "missing index kind")
    ids = {}
    for kind, records in rows.items():
        envelope = bundle[kind]
        require(envelope["schema_version"] == 1 and envelope["generator_version"] == 1 and envelope["kind"] == kind,
                "invalid envelope schema")
        require(envelope["record_count"] == len(records), "incorrect record count")
        for row in records:
            require(set(REQUIRED[kind]) <= set(row), f"incomplete {kind} record")
            require(row["id"] not in ids, "duplicate logical ID: " + row["id"])
            ids[row["id"]] = row
            for field, rule in definitions[kind]["properties"].items():
                if field not in row:
                    continue
                value = row[field]
                if "enum" in rule:
                    require(value in rule["enum"], f"invalid {field} vocabulary")
                if rule.get("type") == "string":
                    require(isinstance(value, str), f"invalid {field}")
                if rule.get("type") == "array":
                    require(isinstance(value, list), f"invalid {field}")
                    if rule.get("items", {}).get("type") == "string":
                        require(all(isinstance(v, str) for v in value) and len(value) == len(set(value)), f"invalid/duplicate {field}")
            if "nid" in row:
                nid = row["nid"]
                require(isinstance(nid, dict) and bool(nid) and set(nid) <= {"numeric_hex", "encoded"}, "invalid NID representation")
                require(all(isinstance(value, str) and value for value in nid.values()), "invalid NID value")
                if "numeric_hex" in nid:
                    require(re.fullmatch(r"0x[0-9a-fA-F]+", nid["numeric_hex"]), "invalid numeric NID")
            if "file" in row:
                path = check_path(root, row["file"])
                count = len(read(path).splitlines())
                if row.get("lines") is not None:
                    bounds = row["lines"]
                    require(set(bounds) == {"start", "end"} and type(bounds["start"]) is int and type(bounds["end"]) is int
                            and 1 <= bounds["start"] <= bounds["end"] <= count, "invalid source line range")
                if kind == "sources":
                    require(row["line_count"] == count and row["source_hash"] == digest(read(path)), "stale source fingerprint")
                    for symbol in row["symbols"]:
                        require(1 <= symbol["lines"]["start"] <= symbol["lines"]["end"] <= count, "invalid symbol span")
            for field in ("knowledge", "evidence", "validation_evidence"):
                for file in row.get(field, []):
                    check_path(root, file)
    references = {"related_tests": "test:", "related_nids": "nid:", "related_abi": "abi:", "related_diagnostics": "diagnostic:",
                  "related_implementations": "impl:", "implementation_ids": "impl:", "module_ids": "module:", "consumers": "impl:", "surfaces": "impl:"}
    for kind, records in rows.items():
        for row in records:
            for field, prefix in references.items():
                for ref in row.get(field, []):
                    require(ref in ids and ref.startswith(prefix), f"unresolved {field}: {ref}")
            for field, prefix in (("module_id", "module:"), ("source_id", "source:"), ("implementation", "impl:")):
                if field in row:
                    ref = row[field]
                    require(ref in ids and ref.startswith(prefix), f"unresolved {field}")
                    if "file" in row:
                        require(ids[ref]["file"] == row["file"], "source/module owner mismatch")
            if kind == "implementation":
                module = ids[row["module_id"]]
                require(module["crate"] == row["crate"] and module["module"] == row["module"], "implementation owner mismatch")
                src = ids[module["source_id"]]
                require(any(s["symbol"] == row["symbol"] and s["owner"] == row["owner"] and s["lines"] == row["lines"] for s in src["symbols"]), "implementation is not a discovered source symbol")
            if kind == "modules" and row["category"] != "tooling":
                descendants = [r for r in rows["implementation"] if r["crate"] == row["crate"] and r["target"] == row["target"] and
                               (row["module"] == "crate" or r["module"] == row["module"] or r["module"].startswith(row["module"] + "::"))]
                expected = "implemented" if any(r["status"] != "scaffolded" for r in descendants) else "scaffolded"
                require(row["status"] == expected, "scaffolded/implemented module mismatch")
    homes = json.loads(read(root / "knowledge/architecture/module_structure.json"))["required_module_roots"]
    module_files = {r["file"] for r in rows["modules"]}
    require(all(f"crates/{crate}/src/{home}/mod.rs" in module_files for crate, paths in homes.items() for home in paths), "known module home missing")
    # Never emit host-specific paths, even inside unexpected metadata fields.
    def portable(value):
        if isinstance(value, str):
            require(not re.match(r"^(?:[A-Za-z]:[\\/]|/|\\\\)", value), "absolute host path in index")
        elif isinstance(value, dict):
            for v in value.values(): portable(v)
        elif isinstance(value, list):
            for v in value: portable(v)
    portable(bundle)


def render(bundle):
    outputs = {filename: json_bytes(bundle[kind]) for kind, filename in FILES.items()}
    outputs["INDEX_SCHEMA.json"] = json_bytes(schema())
    for kind, filename in MARKDOWN.items():
        records = bundle[kind]["records"]
        lines = ["# " + kind.title() + " index", "", "Generated by `python tools/generate_indexes.py`; do not edit.", "",
                 f"Records: **{len(records)}**. [Machine index]({FILES[kind]}). [Scope and search](README.md).", "",
                 "Discovery and lexical references do not prove passing tests, runtime support or semantic correctness.", ""]
        if kind == "nids":
            lines += ["NID rows distinguish source observation from registration. Owner implementation status does not establish an HLE provider.", ""]
        if not records:
            lines += ["Zero reviewed Astero records. Schema reserved; no external/prototype data imported.", ""]
        else:
            lines += ["| Logical ID | Owner / symbol | Status / evidence | Source / knowledge |", "|---|---|---|---|"]
            for row in records:
                label = row.get("symbol", row.get("name", row.get("module", row.get("subsystem", ""))))
                owner = (row.get("crate") or "tools") + " / " + label
                if "file" in row:
                    bounds = row.get("lines")
                    location = f"[{row['file']}](../../{row['file']})" + (f" : {bounds['start']}-{bounds['end']}" if bounds else "")
                else:
                    location = "; ".join(f"[{d}](../../{d})" for d in row.get("knowledge", []))
                values = [row["id"], owner, row.get("status", row.get("evidence_level", "source_backed")), location]
                if kind == "nids":
                    values[2] = "owner " + values[2] + "; " + row["registration_status"]
                    values[3] += "; " + row["provenance"]
                lines.append("| " + " | ".join(v.replace("|", "\\|").replace("\n", " ") for v in values) + " |")
            lines.append("")
        outputs[filename] = "\n".join(lines).encode("utf-8")
    return outputs


def check_generated(root=ROOT, metadata=None):
    bundle = build(root, metadata)
    validate(root, bundle)
    for filename, expected in render(bundle).items():
        path = root / DEST / filename
        require(path.is_file() and read(path).encode("utf-8") == expected,
                f"stale/missing generated index: {filename}; run python tools/generate_indexes.py")
    return bundle


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--check", action="store_true", help="validate source and reject stale indexes without writing")
    args = parser.parse_args()
    if args.check:
        bundle = check_generated()
    else:
        bundle = build()
        validate(ROOT, bundle)
        (ROOT / DEST).mkdir(parents=True, exist_ok=True)
        for filename, content in render(bundle).items():
            (ROOT / DEST / filename).write_bytes(content)
    print("Indexes " + ("verified" if args.check else "generated") + ": " + ", ".join(f"{k}={v['record_count']}" for k, v in bundle.items()))


if __name__ == "__main__":
    main()
