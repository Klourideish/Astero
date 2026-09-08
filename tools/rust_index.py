"""Bounded lexical Rust item inventory; not a compiler, macro expander or name resolver."""
from dataclasses import dataclass
import re


@dataclass
class Token:
    text: str
    start: int
    end: int


def lex(source):
    tokens, comments = [], []
    i = 0
    while i < len(source):
        start = i
        if source[i].isspace():
            i += 1
            continue
        if source.startswith("//", i):
            end = source.find("\n", i)
            i = len(source) if end < 0 else end
            comments.append((start, i, source[start:i]))
            continue
        if source.startswith("/*", i):
            depth, i = 1, i + 2
            while i < len(source) and depth:
                if source.startswith("/*", i):
                    depth, i = depth + 1, i + 2
                elif source.startswith("*/", i):
                    depth, i = depth - 1, i + 2
                else:
                    i += 1
            if depth:
                raise ValueError("unterminated block comment")
            comments.append((start, i, source[start:i]))
            continue
        raw = re.match(r'(?:br|cr|r)(#{0,255})"', source[i:])
        if raw:
            close = '"' + raw[1]
            end = source.find(close, i + len(raw[0]))
            if end < 0:
                raise ValueError("unterminated raw string")
            i = end + len(close)
            tokens.append(Token("LITERAL", start, i))
            continue
        string = re.match(r'(?:b|c)?"', source[i:])
        if string:
            i += len(string[0])
            while i < len(source) and source[i] != '"':
                i += 2 if source[i] == "\\" else 1
            if i >= len(source):
                raise ValueError("unterminated string")
            i += 1
            tokens.append(Token("LITERAL", start, i))
            continue
        char = re.match(r"b?'(?:\\(?:u\{[0-9a-fA-F]+\}|x[0-9a-fA-F]{2}|.)|[^'\\\n])'", source[i:])
        if char:
            i += len(char[0])
            tokens.append(Token("LITERAL", start, i))
            continue
        word = re.match(r"(?:r#)?[A-Za-z_][A-Za-z_0-9]*|[0-9][A-Za-z_0-9]*|::|->|=>", source[i:])
        i += len(word[0]) if word else 1
        tokens.append(Token(source[start:i], start, i))
    return tokens, comments


def scan(source):
    tokens, comments = lex(source)
    pairs, stack = {}, []
    for i, token in enumerate(tokens):
        if token.text in ("(", "[", "{"):
            stack.append(i)
        elif token.text in (")", "]", "}"):
            if not stack or tokens[stack[-1]].text != {")": "(", "]": "[", "}": "{"}[token.text]:
                raise ValueError("unbalanced delimiters")
            begin = stack.pop()
            pairs[begin] = i
    if stack:
        raise ValueError("unbalanced delimiters")
    records, modules, notes = [], [], []
    line = lambda offset: source.count("\n", 0, offset) + 1

    def terminator(i, end):
        angle = 0
        while i < end:
            text = tokens[i].text
            if text == "<":
                angle += 1
            elif text == ">" and angle:
                angle -= 1
            elif text in (";", "{") and not angle:
                return i
            if i in pairs:
                i = pairs[i] + 1
            else:
                i += 1
        raise ValueError("unsupported unterminated item header")

    def walk(begin, end, module=(), owner=(), inherited=()):
        i = begin
        while i < end:
            attrs = list(inherited)
            while i < end and tokens[i].text == "#":
                j = i + 1
                if tokens[j].text == "!":
                    j += 1
                if tokens[j].text != "[":
                    raise ValueError("unsupported attribute")
                attrs.append(source[tokens[j].end:tokens[pairs[j]].start].strip())
                i = pairs[j] + 1
            if i >= end:
                break
            start = i
            visibility = "internal"
            if tokens[i].text == "pub":
                visibility = "public"
                i += 1
                if tokens[i].text == "(":
                    visibility = source[tokens[i].start:tokens[pairs[i]].end]
                    i = pairs[i] + 1
            while tokens[i].text in ("async", "unsafe", "default") or (tokens[i].text == "const" and tokens[i+1].text == "fn"):
                i += 1
            kind = tokens[i].text
            if kind == "extern":
                i += 1
                if tokens[i].text == "LITERAL":
                    i += 1
                kind = tokens[i].text
            if kind not in {"fn", "struct", "enum", "trait", "type", "const", "static", "mod", "impl", "use"}:
                notes.append({"line": line(tokens[start].start), "reason": "unsupported item; no symbols inferred"})
                stop = terminator(i, end)
                i = pairs[stop] + 1 if tokens[stop].text == "{" else stop + 1
                if i < end and tokens[i].text == ";":
                    i += 1
                continue
            stop = terminator(i + 1, end)
            body = tokens[stop].text == "{"
            finish = pairs[stop] if body else stop
            # const/static initializers can contain blocks followed by more expression tokens.
            if kind in {"const", "static", "type", "use"}:
                j = stop
                while j < end and tokens[j].text != ";":
                    j = pairs[j] + 1 if j in pairs else j + 1
                if j == end:
                    raise ValueError("unterminated declaration")
                finish, body = j, False
            name_at = i + 1
            if kind == "static" and tokens[name_at].text == "mut":
                name_at += 1
            name = tokens[name_at].text
            test_only = any(re.search(r"\bcfg\s*\(\s*test\s*\)", attr) for attr in attrs)
            if kind == "mod":
                if any(re.search(r"\bpath\s*=", attr) for attr in attrs):
                    raise ValueError("#[path] modules need explicit extractor support")
                modules.append({"module": list(module + (name,)), "inline": body, "attributes": attrs,
                                "lines": {"start": line(tokens[start].start), "end": line(tokens[finish].end-1)}})
                if body:
                    walk(stop + 1, finish, module + (name,), (), tuple(attrs))
            elif kind == "impl":
                header = " ".join(t.text for t in tokens[i+1:stop])
                walk(stop + 1, finish, module, ("impl[" + header + "]",), tuple(attrs))
            elif kind != "use":
                record = {"symbol": name, "symbol_kind": kind, "module_suffix": list(module),
                          "owner": list(owner), "visibility": visibility, "attributes": attrs,
                          "test_only": test_only, "is_test": kind == "fn" and "test" in attrs,
                          "has_body": body, "start": tokens[start].start, "end": tokens[finish].end,
                          "lines": {"start": line(tokens[start].start), "end": line(tokens[finish].end-1)}}
                record["identifiers"] = sorted({t.text for t in tokens[stop:finish+1]
                                                if re.fullmatch(r"[A-Za-z_][A-Za-z_0-9]*", t.text) and t.text != "LITERAL"})
                records.append(record)
                if kind == "trait" and body:
                    walk(stop + 1, finish, module, ("trait[" + name + "]",), tuple(attrs))
            i = finish + 1
            if i < end and tokens[i].text == ";":
                i += 1
    walk(0, len(tokens))
    # Only triple-slash/inner line-doc Rust fences are supported, never arbitrary Markdown.
    doc_tests, opened = [], None
    for start, end, comment in comments:
        if not (comment.startswith("///") and not comment.startswith("////") or comment.startswith("//!")):
            continue
        content = comment[3:].strip()
        if content.startswith("```"):
            if opened:
                doc_tests.append({"lines": {"start": opened[0], "end": line(end)}, "end": end,
                                  "test_type": opened[1]})
                opened = None
            elif content in {"```", "```rust", "```compile_fail", "```no_run"}:
                opened = (line(start), content[3:] or "rust")
    return {"symbols": records, "modules": modules, "notes": notes, "doctests": doc_tests}
