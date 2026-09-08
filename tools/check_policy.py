"""Check active-state structure and all declared internal Cargo dependency edges."""
import json
from pathlib import Path
import subprocess
from check_structure import check_structure

ROOT = Path(__file__).resolve().parents[1]
STATUSES = {"planned", "investigating", "implementing", "testing", "blocked", "ready_for_cleanup"}


def require(condition, message):
    if not condition:
        raise ValueError(message)


def check_state(state):
    require(isinstance(state, dict), "state must be an object")
    require(type(state.get("schema_version")) is int and state["schema_version"] == 1, "state schema_version must be 1")
    require(isinstance(state.get("active_items"), list), "active_items must be a list")
    ids = set()
    for item in state["active_items"]:
        require(isinstance(item, dict), "active item must be an object")
        for key in ("id", "scope", "summary", "status", "next_action"):
            require(isinstance(item.get(key), str) and item[key].strip(), f"missing/non-text {key}")
        require(item["id"] not in ids, "duplicate active item ID")
        ids.add(item["id"])
        require(item["status"] in STATUSES, "invalid active status")
        for key in ("evidence_references", "completion_conditions"):
            require(isinstance(item.get(key), list) and item[key], f"{key} must be a nonempty list")
            require(all(isinstance(x, str) and x.strip() for x in item[key]), f"invalid {key}")
        for ref in item["evidence_references"]:
            path = (ROOT / ref).resolve()
            require(path.is_relative_to(ROOT) and path.is_file(), f"missing/local evidence reference: {ref}")


def check_dependencies(metadata, policy):
    require(policy.get("schema_version") == 1, "unsupported dependency schema")
    packages = {p["name"]: p for p in metadata["packages"] if p["id"] in metadata["workspace_members"]}
    allowed = policy["allowed_internal_dependencies"]
    require(set(allowed) == set(packages), "allowlist must cover exactly the workspace packages")
    for name, targets in allowed.items():
        require(isinstance(targets, list) and len(targets) == len(set(targets)), "invalid allowlist targets")
        require(set(targets) <= set(packages) - {name}, "unknown/self allowlist target")
        if name not in {"astero-cli", "astero-gui", "astero-gpu-smoke", "astero-core", "astero-debug"}:
            require(not {"astero-core", "astero-debug"}.intersection(targets), "runtime cannot allow core/debug")
    require(set(allowed["astero-gui"]) <= {"astero-core", "astero-debug"}, "GUI must not allow runtime dependencies")
    require("astero-debug" not in allowed["astero-core"], "core cannot allow debug")
    require("astero-libs" not in allowed["astero-hle"], "HLE cannot allow libs")
    dependency_free = policy.get("dependency_free_packages", [])
    require(isinstance(dependency_free, list) and all(isinstance(name, str) and name in packages for name in dependency_free),
            "invalid dependency-free package list")
    for name in dependency_free:
        require(not allowed[name], f"dependency-free package cannot allow internal edges: {name}")
        require(not packages[name]["dependencies"], f"dependency-free package declares dependencies: {name}")
    graph = {name: set() for name in packages}
    for name, package in packages.items():
        for dep in package["dependencies"]:
            target = dep["name"]  # Cargo's original package name, even for renamed dependencies.
            require(target not in policy["forbidden_dependencies"], f"rejected dependency: {target}")
            if target in policy["gui_only_dependencies"] or target.startswith("imgui-"):
                require(name == "astero-gui", f"GUI toolkit dependency outside GUI: {name} -> {target}")
            if target in packages:
                require(target in allowed[name], f"forbidden dependency: {name} -> {target}")
                expected = Path(packages[target]["manifest_path"]).parent.resolve()
                require(dep.get("path") and Path(dep["path"]).resolve() == expected,
                        f"internal dependency must resolve to workspace member: {name} -> {target}")
                graph[name].add(target)
            else:
                require(not target.startswith("astero-"), f"unknown internal dependency: {target}")
    visiting, visited = set(), set()
    def visit(name):
        require(name not in visiting, f"dependency cycle at {name}")
        if name in visited:
            return
        visiting.add(name)
        for target in graph[name]:
            visit(target)
        visiting.remove(name)
        visited.add(name)
    for name in graph:
        visit(name)
    return sum(map(len, graph.values()))


def main():
    check_state(json.loads((ROOT / "PROJECT_STATE.json").read_text(encoding="utf-8")))
    metadata = json.loads(subprocess.check_output(
        ["cargo", "metadata", "--format-version", "1", "--no-deps", "--locked"], cwd=ROOT, text=True))
    edges = check_dependencies(metadata, json.loads(
        (ROOT / "knowledge/architecture/dependency_policy.json").read_text(encoding="utf-8")))
    homes = check_structure(ROOT, json.loads(
        (ROOT / "knowledge/architecture/module_structure.json").read_text(encoding="utf-8")))
    print(f"Policy passed: {len(metadata['workspace_members'])} workspace members, {edges} internal edges; active state valid.")
    print(f"Structure policy passed: {homes} declared module homes.")


if __name__ == "__main__":
    main()
