#!/usr/bin/env python3
from __future__ import annotations

import json
import os
import re
from collections import defaultdict
from dataclasses import dataclass, field
from html import escape
from pathlib import Path
from typing import Iterable

ROOT = Path(__file__).resolve().parents[2]
CONCAT_PATH = ROOT / "concat-report.txt"
DOC_ROOT = ROOT / "docs" / "html"
API_ROOT = DOC_ROOT / "reference" / "api"

SEPARATOR = "=" * 80
VIS_PREFIX = r"(?:pub(?:\([^)]*\))?\s+)?"
DOC_COMMENT_RE = re.compile(r"^\s*///\s?(.*)$")
TYPE_DECL_RE = re.compile(rf"^pub\s+(struct|enum|trait)\s+(\w+)\b")
FN_DECL_RE = re.compile(rf"^{VIS_PREFIX}(?:async\s+)?fn\s+(\w+)\b")
CONST_DECL_RE = re.compile(rf"^pub\s+const\s+(\w+)\b")
TYPE_ALIAS_RE = re.compile(rf"^pub\s+type\s+(\w+)\b")
MOD_DECL_RE = re.compile(rf"^pub\s+mod\s+(\w+)\b")
FIELD_RE = re.compile(rf"^({VIS_PREFIX})(\w+)\s*:\s*(.+?)(?:,)?$")
VARIANT_RE = re.compile(r"^(\w+)\b")
IMPL_FOR_RE = re.compile(r"^impl(?:<[^>]+>)?\s+(.+?)\s+for\s+(.+?)\s*(?:where\b.*)?\{$", re.S)
IMPL_RE = re.compile(r"^impl(?:<[^>]+>)?\s+(.+?)\s*(?:where\b.*)?\{$", re.S)
IDENT_RE = re.compile(r"\b([A-Z][A-Za-z0-9_]*)\b")
HTML_TITLE = "Shell Quest Engine API Reference"


@dataclass
class Member:
    id: str
    name: str
    kind: str
    signature: str
    summary: str
    line: int
    anchor: str
    type_text: str | None = None
    visibility: str | None = None
    related_types: list[str] = field(default_factory=list)

    def to_json(self) -> dict:
        return {
            "id": self.id,
            "name": self.name,
            "kind": self.kind,
            "signature": self.signature,
            "summary": self.summary,
            "line": self.line,
            "anchor": self.anchor,
            "type": self.type_text,
            "visibility": self.visibility,
            "related_types": self.related_types,
        }


@dataclass
class Entity:
    id: str
    kind: str
    name: str
    page: str
    source_path: str
    source_line: int
    summary: str
    doc: list[str]
    signature: str
    module_id: str
    canonical_path: str
    members: list[Member] = field(default_factory=list)
    related_ids: list[str] = field(default_factory=list)
    implements: list[str] = field(default_factory=list)
    implementors: list[str] = field(default_factory=list)
    methods: list[Member] = field(default_factory=list)

    def to_json(self) -> dict:
        return {
            "id": self.id,
            "kind": self.kind,
            "name": self.name,
            "page": self.page,
            "source_path": self.source_path,
            "source_line": self.source_line,
            "summary": self.summary,
            "doc": self.doc,
            "signature": self.signature,
            "module_id": self.module_id,
            "canonical_path": self.canonical_path,
            "members": [member.to_json() for member in self.members],
            "related_ids": self.related_ids,
            "implements": self.implements,
            "implementors": self.implementors,
            "methods": [method.to_json() for method in self.methods],
        }


@dataclass
class ModuleRecord:
    id: str
    name: str
    source_path: str
    page: str
    summary: str
    doc: list[str]
    canonical_path: str
    types: list[str] = field(default_factory=list)
    functions: list[Member] = field(default_factory=list)
    consts: list[Member] = field(default_factory=list)
    type_aliases: list[Member] = field(default_factory=list)
    submodules: list[Member] = field(default_factory=list)

    def to_json(self) -> dict:
        return {
            "id": self.id,
            "name": self.name,
            "source_path": self.source_path,
            "page": self.page,
            "summary": self.summary,
            "doc": self.doc,
            "canonical_path": self.canonical_path,
            "types": self.types,
            "functions": [fn.to_json() for fn in self.functions],
            "consts": [item.to_json() for item in self.consts],
            "type_aliases": [item.to_json() for item in self.type_aliases],
            "submodules": [item.to_json() for item in self.submodules],
        }


def normalize_space(text: str) -> str:
    return re.sub(r"\s+", " ", text).strip()


def summary_from_doc(doc_lines: Iterable[str]) -> str:
    lines = [line.strip() for line in doc_lines if line.strip()]
    if not lines:
        return ""
    first = []
    for line in lines:
        if line == "":
            break
        first.append(line)
    return normalize_space(" ".join(first if first else lines[:1]))


def module_parts_for_path(path: str) -> list[str]:
    path_obj = Path(path)
    parts = list(path_obj.parts)
    if "src" in parts:
        src_index = parts.index("src")
        crate_parts = parts[:src_index]
        mod_parts = parts[src_index + 1 :]
    else:
        crate_parts = parts[:-1]
        mod_parts = [parts[-1]]
    if not mod_parts:
        mod_parts = [path_obj.stem]
    else:
        mod_parts[-1] = Path(mod_parts[-1]).stem
    return list(crate_parts) + list(mod_parts)


def canonical_module_path(path: str) -> str:
    return "::".join(module_parts_for_path(path))


def module_page_for_path(path: str) -> str:
    parts = module_parts_for_path(path)
    return "/".join(["reference", "api", "modules", *parts[:-1], f"{parts[-1]}.html"])


def type_page_for_path(path: str, kind: str, name: str) -> str:
    parts = module_parts_for_path(path)
    return "/".join(["reference", "api", "types", *parts, f"{kind}.{name}.html"])


def entity_id(kind: str, path: str, name: str) -> str:
    return f"{kind}:{path}:{name}"


def module_id(path: str) -> str:
    return f"module:{path}"


def member_anchor(kind: str, name: str) -> str:
    return f"{kind}-{name.replace('_', '-') }"


def strip_comment_prefix(line: str) -> str:
    match = DOC_COMMENT_RE.match(line)
    return match.group(1) if match else line


def scan_line(line: str, state: dict[str, object]) -> int:
    delta = 0
    i = 0
    while i < len(line):
        block_comment = state.get("block_comment", False)
        string_char = state.get("string_char")
        if block_comment:
            if line[i : i + 2] == "*/":
                state["block_comment"] = False
                i += 2
                continue
            i += 1
            continue
        if string_char:
            if line[i] == "\\":
                i += 2
                continue
            if line[i] == string_char:
                state["string_char"] = None
            i += 1
            continue
        if line[i : i + 2] == "//":
            break
        if line[i : i + 2] == "/*":
            state["block_comment"] = True
            i += 2
            continue
        if line[i] == '"':
            state["string_char"] = line[i]
            i += 1
            continue
        if line[i] == "{":
            delta += 1
        elif line[i] == "}":
            delta -= 1
        i += 1
    return delta


def collect_until_terminator(lines: list[str], start: int) -> tuple[int, str]:
    state: dict[str, object] = {"block_comment": False, "string_char": None}
    seen_open = False
    end = start
    header_lines = []
    depth = 0
    while end < len(lines):
        header_lines.append(lines[end])
        delta = scan_line(lines[end], state)
        if "{" in lines[end]:
            seen_open = True
        depth += delta
        if seen_open or ";" in lines[end]:
            break
        end += 1
    return end, "\n".join(header_lines)


def collect_block(lines: list[str], start: int) -> tuple[int, list[str]]:
    state: dict[str, object] = {"block_comment": False, "string_char": None}
    depth = 0
    seen_open = False
    end = start
    collected: list[str] = []
    while end < len(lines):
        line = lines[end]
        collected.append(line)
        delta = scan_line(line, state)
        if "{" in line:
            seen_open = True
        depth += delta
        if seen_open and depth <= 0:
            break
        end += 1
    return end, collected


def collect_signature(lines: list[str], start: int) -> tuple[int, str]:
    end = start
    state: dict[str, object] = {"block_comment": False, "string_char": None}
    depth = 0
    signature_lines: list[str] = []
    seen_open = False
    while end < len(lines):
        line = lines[end]
        signature_lines.append(line)
        delta = scan_line(line, state)
        if "{" in line:
            seen_open = True
        depth += delta
        if ";" in line or (seen_open and depth > 0):
            break
        end += 1
    return end, normalize_space(" ".join(line.strip() for line in signature_lines))


def extract_first_identifier(type_text: str) -> str | None:
    cleaned = re.sub(r"\b(pub|crate|self|super|dyn|impl)\b", " ", type_text)
    cleaned = cleaned.replace("&", " ")
    matches = IDENT_RE.findall(cleaned)
    if not matches:
        return None
    return matches[-1]


def extract_type_names(type_text: str) -> list[str]:
    builtins = {"String", "Vec", "Option", "Result", "Box", "Arc", "Rc", "HashMap", "HashSet", "BTreeMap", "BTreeSet", "PathBuf", "Path", "str", "bool", "u8", "u16", "u32", "u64", "usize", "i8", "i16", "i32", "i64", "isize", "f32", "f64"}
    names = []
    for ident in IDENT_RE.findall(type_text):
        if ident not in builtins and ident not in names:
            names.append(ident)
    return names


def parse_named_fields(entity: Entity, lines: list[str], base_line: int) -> list[Member]:
    members: list[Member] = []
    pending_docs: list[str] = []
    depth = 0
    state: dict[str, object] = {"block_comment": False, "string_char": None}
    for index, line in enumerate(lines):
        stripped = line.strip()
        if depth == 0 and DOC_COMMENT_RE.match(stripped):
            pending_docs.append(strip_comment_prefix(stripped))
            continue
        if depth == 0 and (not stripped or stripped.startswith("#[")):
            continue
        if depth == 0:
            match = FIELD_RE.match(stripped)
            if match and ":" in stripped:
                visibility = normalize_space(match.group(1)) or "private"
                name = match.group(2)
                type_text = match.group(3).rstrip(",")
                members.append(
                    Member(
                        id=f"{entity.id}:field:{name}",
                        name=name,
                        kind="field",
                        signature=normalize_space(stripped.rstrip(",")),
                        summary=summary_from_doc(pending_docs),
                        line=base_line + index,
                        anchor=member_anchor("field", name),
                        type_text=type_text,
                        visibility=visibility,
                    )
                )
                pending_docs = []
        depth += scan_line(line, state)
        if depth < 0:
            depth = 0
    return members


def parse_enum_variants(entity: Entity, lines: list[str], base_line: int) -> list[Member]:
    members: list[Member] = []
    pending_docs: list[str] = []
    depth = 0
    state: dict[str, object] = {"block_comment": False, "string_char": None}
    for index, line in enumerate(lines):
        stripped = line.strip()
        if depth == 0 and DOC_COMMENT_RE.match(stripped):
            pending_docs.append(strip_comment_prefix(stripped))
            continue
        if depth == 0 and (not stripped or stripped.startswith("#[")):
            continue
        if depth == 0:
            match = VARIANT_RE.match(stripped)
            if match:
                name = match.group(1)
                if name not in {"where"}:
                    members.append(
                        Member(
                            id=f"{entity.id}:variant:{name}",
                            name=name,
                            kind="variant",
                            signature=normalize_space(stripped.rstrip(",")),
                            summary=summary_from_doc(pending_docs),
                            line=base_line + index,
                            anchor=member_anchor("variant", name),
                        )
                    )
                    pending_docs = []
        depth += scan_line(line, state)
        if depth < 0:
            depth = 0
    return members


def parse_trait_methods(entity: Entity, lines: list[str], base_line: int) -> list[Member]:
    methods: list[Member] = []
    pending_docs: list[str] = []
    depth = 0
    state: dict[str, object] = {"block_comment": False, "string_char": None}
    index = 0
    while index < len(lines):
        stripped = lines[index].strip()
        if depth == 0 and DOC_COMMENT_RE.match(stripped):
            pending_docs.append(strip_comment_prefix(stripped))
            index += 1
            continue
        if depth == 0 and (not stripped or stripped.startswith("#[")):
            index += 1
            continue
        if depth == 0 and re.match(r"^(?:async\s+)?fn\s+\w+", stripped):
            end, signature = collect_signature(lines, index)
            name_match = re.search(r"fn\s+(\w+)\b", signature)
            if name_match:
                name = name_match.group(1)
                methods.append(
                    Member(
                        id=f"{entity.id}:method:{name}",
                        name=name,
                        kind="method",
                        signature=signature,
                        summary=summary_from_doc(pending_docs),
                        line=base_line + index,
                        anchor=member_anchor("method", name),
                    )
                )
            pending_docs = []
            index = end + 1
            continue
        depth += scan_line(lines[index], state)
        if depth < 0:
            depth = 0
        index += 1
    return methods


def parse_impl_methods(parent_id: str, lines: list[str], base_line: int) -> list[Member]:
    methods: list[Member] = []
    pending_docs: list[str] = []
    depth = 0
    state: dict[str, object] = {"block_comment": False, "string_char": None}
    index = 0
    while index < len(lines):
        stripped = lines[index].strip()
        if depth == 0 and DOC_COMMENT_RE.match(stripped):
            pending_docs.append(strip_comment_prefix(stripped))
            index += 1
            continue
        if depth == 0 and (not stripped or stripped.startswith("#[")):
            index += 1
            continue
        if depth == 0 and re.match(rf"^{VIS_PREFIX}(?:async\s+)?fn\s+\w+", stripped):
            end, signature = collect_signature(lines, index)
            name_match = re.search(r"fn\s+(\w+)\b", signature)
            if name_match:
                name = name_match.group(1)
                methods.append(
                    Member(
                        id=f"{parent_id}:method:{name}:{base_line + index}",
                        name=name,
                        kind="method",
                        signature=signature,
                        summary=summary_from_doc(pending_docs),
                        line=base_line + index,
                        anchor=member_anchor("method", f"{name}-{base_line + index}"),
                        visibility="public" if stripped.startswith("pub") else "private",
                    )
                )
            pending_docs = []
            index = end + 1
            continue
        depth += scan_line(lines[index], state)
        if depth < 0:
            depth = 0
        index += 1
    return methods


def parse_concat_report(path: Path) -> tuple[dict[str, str], dict[str, str]]:
    text = path.read_text(encoding="utf-8")
    lines = text.splitlines()
    meta: dict[str, str] = {}
    files: dict[str, str] = {}
    index = 0
    while index < len(lines):
        if lines[index].startswith(SEPARATOR):
            break
        if ":" in lines[index]:
            key, value = lines[index].split(":", 1)
            meta[key.strip()] = value.strip()
        index += 1

    while index < len(lines):
        if not lines[index].startswith(SEPARATOR):
            index += 1
            continue
        if index + 1 >= len(lines) or not lines[index + 1].startswith("file: "):
            index += 1
            continue
        path_line = lines[index + 1][6:].strip()
        index += 2
        while index < len(lines) and lines[index] != "--- BEGIN CONTENT ---":
            index += 1
        index += 1
        content_lines: list[str] = []
        while index < len(lines) and lines[index] != "--- END CONTENT ---":
            content_lines.append(lines[index])
            index += 1
        files[path_line] = "\n".join(content_lines)
        index += 1
    return meta, files


def parse_rust_snapshot(files: dict[str, str]) -> tuple[dict[str, ModuleRecord], dict[str, Entity], list[dict]]:
    modules: dict[str, ModuleRecord] = {}
    entities: dict[str, Entity] = {}
    impl_blocks: list[dict] = []

    for path, content in files.items():
        if not path.endswith(".rs"):
            continue
        module_record = ModuleRecord(
            id=module_id(path),
            name=Path(path).stem,
            source_path=path,
            page=module_page_for_path(path),
            summary="",
            doc=[],
            canonical_path=canonical_module_path(path),
        )
        modules[module_record.id] = module_record
        lines = content.splitlines()
        top_doc: list[str] = []
        pending_docs: list[str] = []
        index = 0
        depth = 0
        state: dict[str, object] = {"block_comment": False, "string_char": None}
        while index < len(lines):
            line = lines[index]
            stripped = line.strip()
            if depth == 0 and stripped.startswith("//!"):
                top_doc.append(stripped[3:].strip())
                index += 1
                continue
            if depth == 0 and DOC_COMMENT_RE.match(stripped):
                pending_docs.append(strip_comment_prefix(stripped))
                index += 1
                continue
            if depth == 0 and (not stripped or stripped.startswith("#[")):
                index += 1
                continue
            if depth == 0:
                type_match = TYPE_DECL_RE.match(stripped)
                if type_match:
                    kind, name = type_match.groups()
                    sig_end, signature = collect_until_terminator(lines, index)
                    header_lines = lines[index : sig_end + 1]
                    header_text = "\n".join(header_lines)
                    if "{" in header_text and not header_text.rstrip().endswith(";"):
                        block_end, block_lines = collect_block(lines, index)
                    else:
                        block_end = sig_end
                        block_lines = header_lines
                    signature = normalize_space(" ".join(item.strip() for item in header_lines))
                    entity = Entity(
                        id=entity_id(kind, path, name),
                        kind=kind,
                        name=name,
                        page=type_page_for_path(path, kind, name),
                        source_path=path,
                        source_line=index + 1,
                        summary=summary_from_doc(pending_docs),
                        doc=list(pending_docs),
                        signature=signature,
                        module_id=module_record.id,
                        canonical_path=f"{module_record.canonical_path}::{name}",
                    )
                    joined = "\n".join(block_lines)
                    if kind == "struct" and "{" in joined and joined.rstrip().endswith("}"):
                        entity.members = parse_named_fields(entity, block_lines[1:-1], index + 2)
                    elif kind == "enum":
                        entity.members = parse_enum_variants(entity, block_lines[1:-1], index + 2)
                    elif kind == "trait":
                        entity.methods = parse_trait_methods(entity, block_lines[1:-1], index + 2)
                    entities[entity.id] = entity
                    module_record.types.append(entity.id)
                    pending_docs = []
                    index = block_end + 1
                    continue

                fn_match = FN_DECL_RE.match(stripped)
                if fn_match and stripped.startswith("pub"):
                    end, signature = collect_signature(lines, index)
                    name = fn_match.group(1)
                    module_record.functions.append(
                        Member(
                            id=f"{module_record.id}:function:{name}",
                            name=name,
                            kind="function",
                            signature=signature,
                            summary=summary_from_doc(pending_docs),
                            line=index + 1,
                            anchor=member_anchor("fn", name),
                        )
                    )
                    pending_docs = []
                    index = end + 1
                    continue

                const_match = CONST_DECL_RE.match(stripped)
                if const_match:
                    name = const_match.group(1)
                    module_record.consts.append(
                        Member(
                            id=f"{module_record.id}:const:{name}",
                            name=name,
                            kind="const",
                            signature=normalize_space(stripped.rstrip(";")),
                            summary=summary_from_doc(pending_docs),
                            line=index + 1,
                            anchor=member_anchor("const", name),
                        )
                    )
                    pending_docs = []
                    index += 1
                    continue

                alias_match = TYPE_ALIAS_RE.match(stripped)
                if alias_match:
                    name = alias_match.group(1)
                    module_record.type_aliases.append(
                        Member(
                            id=f"{module_record.id}:type:{name}",
                            name=name,
                            kind="type-alias",
                            signature=normalize_space(stripped.rstrip(";")),
                            summary=summary_from_doc(pending_docs),
                            line=index + 1,
                            anchor=member_anchor("type", name),
                        )
                    )
                    pending_docs = []
                    index += 1
                    continue

                mod_match = MOD_DECL_RE.match(stripped)
                if mod_match:
                    name = mod_match.group(1)
                    module_record.submodules.append(
                        Member(
                            id=f"{module_record.id}:mod:{name}",
                            name=name,
                            kind="module",
                            signature=normalize_space(stripped.rstrip(";")),
                            summary=summary_from_doc(pending_docs),
                            line=index + 1,
                            anchor=member_anchor("mod", name),
                        )
                    )
                    pending_docs = []
                    index += 1
                    continue

                if stripped.startswith("impl"):
                    block_end, block_lines = collect_block(lines, index)
                    header = normalize_space(block_lines[0])
                    trait_name = None
                    target_name = None
                    impl_match = IMPL_FOR_RE.match(header)
                    if impl_match:
                        trait_name = extract_first_identifier(impl_match.group(1))
                        target_name = extract_first_identifier(impl_match.group(2))
                    else:
                        direct_match = IMPL_RE.match(header)
                        if direct_match:
                            target_name = extract_first_identifier(direct_match.group(1))
                    impl_blocks.append(
                        {
                            "module_id": module_record.id,
                            "source_path": path,
                            "line": index + 1,
                            "header": header,
                            "trait_name": trait_name,
                            "target_name": target_name,
                            "methods": parse_impl_methods(f"impl:{path}:{index + 1}", block_lines[1:-1], index + 2),
                        }
                    )
                    pending_docs = []
                    index = block_end + 1
                    continue

                pending_docs = []
            depth += scan_line(line, state)
            if depth < 0:
                depth = 0
            index += 1
        module_record.doc = top_doc
        module_record.summary = summary_from_doc(top_doc)
    return modules, entities, impl_blocks


def build_indexes(entities: dict[str, Entity]) -> tuple[dict[str, list[str]], dict[tuple[str, str], str]]:
    by_name: dict[str, list[str]] = defaultdict(list)
    by_file_and_name: dict[tuple[str, str], str] = {}
    for entity in entities.values():
        by_name[entity.name].append(entity.id)
        by_file_and_name[(entity.source_path, entity.name)] = entity.id
    return by_name, by_file_and_name


def resolve_entity(
    name: str,
    source_path: str,
    by_name: dict[str, list[str]],
    by_file_and_name: dict[tuple[str, str], str],
    preferred_kind: str | None = None,
) -> str | None:
    if (source_path, name) in by_file_and_name:
        candidate = by_file_and_name[(source_path, name)]
        if preferred_kind is None or candidate.startswith(f"{preferred_kind}:"):
            return candidate
    matches = by_name.get(name, [])
    if preferred_kind:
        preferred_matches = [entity_id for entity_id in matches if entity_id.startswith(f"{preferred_kind}:")]
        if len(preferred_matches) == 1:
            return preferred_matches[0]
    if len(matches) == 1:
        return matches[0]
    struct_matches = [entity_id for entity_id in matches if entity_id.startswith("struct:")]
    if len(struct_matches) == 1:
        return struct_matches[0]
    return None


def enrich_relationships(modules: dict[str, ModuleRecord], entities: dict[str, Entity], impl_blocks: list[dict]) -> None:
    by_name, by_file_and_name = build_indexes(entities)

    for entity in entities.values():
        related: list[str] = []
        member_pool = entity.members + entity.methods
        for member in member_pool:
            candidate_names = extract_type_names(member.type_text or member.signature)
            for candidate in candidate_names:
                target = resolve_entity(candidate, entity.source_path, by_name, by_file_and_name)
                if target and target != entity.id and target not in related:
                    related.append(target)
                    member.related_types.append(target)
        entity.related_ids = related

    for impl_block in impl_blocks:
        target_name = impl_block.get("target_name")
        if not target_name:
            continue
        target_id = resolve_entity(target_name, impl_block["source_path"], by_name, by_file_and_name)
        if not target_id:
            continue
        entity = entities[target_id]
        if impl_block.get("trait_name"):
            trait_id = resolve_entity(
                impl_block["trait_name"],
                impl_block["source_path"],
                by_name,
                by_file_and_name,
                preferred_kind="trait",
            )
            if trait_id and trait_id != target_id:
                if trait_id not in entity.implements:
                    entity.implements.append(trait_id)
                trait_entity = entities.get(trait_id)
                if trait_entity and target_id not in trait_entity.implementors:
                    trait_entity.implementors.append(target_id)
        else:
            for method in impl_block["methods"]:
                if method.visibility == "public":
                    entity.methods.append(method)
            for method in impl_block["methods"]:
                for candidate in extract_type_names(method.signature):
                    target = resolve_entity(candidate, impl_block["source_path"], by_name, by_file_and_name)
                    if target and target != target_id and target not in entity.related_ids:
                        entity.related_ids.append(target)
                        method.related_types.append(target)


def rel_href(from_page: str, to_page: str) -> str:
    return os.path.relpath(DOC_ROOT / to_page, (DOC_ROOT / from_page).parent).replace(os.sep, "/")


def link_for_entity(from_page: str, entity: Entity) -> str:
    return rel_href(from_page, entity.page)


def render_doc_lines(doc_lines: list[str]) -> str:
    if not doc_lines:
        return "<p><em>No doc comments were present in the concat snapshot.</em></p>"
    paragraphs: list[str] = []
    current: list[str] = []
    for raw in doc_lines:
        line = raw.strip()
        if not line:
            if current:
                paragraphs.append(f"<p>{escape(normalize_space(' '.join(current)))}</p>")
                current = []
            continue
        current.append(line)
    if current:
        paragraphs.append(f"<p>{escape(normalize_space(' '.join(current)))}</p>")
    return "\n".join(paragraphs)


def entity_link(from_page: str, entity: Entity) -> str:
    href = link_for_entity(from_page, entity)
    return f'<a class="entity-ref" data-entity-id="{escape(entity.id)}" href="{escape(href)}"><code>{escape(entity.name)}</code></a>'


def render_related_list(from_page: str, entity_ids: list[str], entities: dict[str, Entity]) -> str:
    if not entity_ids:
        return "<p><em>No related API entities resolved from the snapshot.</em></p>"
    items = []
    for entity_id in sorted(entity_ids):
        entity = entities[entity_id]
        items.append(f"<li>{entity_link(from_page, entity)} <span class=\"entity-meta\">{escape(entity.kind)} · {escape(entity.source_path)}</span></li>")
    return f"<ul class=\"entity-list\">{''.join(items)}</ul>"


def render_members_table(from_page: str, members: list[Member], entities: dict[str, Entity], label: str) -> str:
    if not members:
        return f"<p><em>No {escape(label.lower())} captured from the snapshot.</em></p>"
    rows = []
    for member in members:
        related = ""
        if member.related_types:
            related_links = ", ".join(entity_link(from_page, entities[item]) for item in member.related_types if item in entities)
            related = f"<div class=\"member-related\">Related: {related_links}</div>"
        type_cell = escape(member.type_text or "")
        summary = escape(member.summary or "")
        rows.append(
            "<tr>"
            f"<td id=\"{escape(member.anchor)}\"><code>{escape(member.name)}</code></td>"
            f"<td><code>{escape(member.signature)}</code>{related}</td>"
            f"<td><code>{type_cell}</code></td>"
            f"<td>{summary}</td>"
            "</tr>"
        )
    return (
        "<table class=\"api-table\"><thead><tr><th>Name</th><th>Signature</th><th>Type</th><th>Summary</th></tr></thead>"
        f"<tbody>{''.join(rows)}</tbody></table>"
    )


def render_methods_table(from_page: str, methods: list[Member], entities: dict[str, Entity]) -> str:
    if not methods:
        return "<p><em>No methods captured from the snapshot.</em></p>"
    rows = []
    for method in methods:
        related = ""
        if method.related_types:
            related = "<div class=\"member-related\">Related: " + ", ".join(entity_link(from_page, entities[item]) for item in method.related_types if item in entities) + "</div>"
        rows.append(
            "<tr>"
            f"<td id=\"{escape(method.anchor)}\"><code>{escape(method.name)}</code></td>"
            f"<td><code>{escape(method.signature)}</code>{related}</td>"
            f"<td>{escape(method.summary)}</td>"
            "</tr>"
        )
    return "<table class=\"api-table\"><thead><tr><th>Name</th><th>Signature</th><th>Summary</th></tr></thead><tbody>" + "".join(rows) + "</tbody></table>"


def page_template(title: str, rel_styles: str, body_class: str, body_html: str) -> str:
    return f"""<!DOCTYPE html>
<html lang=\"en\">
<head>
  <meta charset=\"utf-8\" />
  <meta name=\"viewport\" content=\"width=device-width, initial-scale=1.0\" />
  <title>{escape(title)}</title>
  <link rel=\"stylesheet\" href=\"{escape(rel_styles)}\" />
</head>
<body class=\"{escape(body_class)}\">
{body_html}
</body>
</html>
"""


def write_page(page: str, title: str, body_html: str) -> None:
    target = DOC_ROOT / page
    target.parent.mkdir(parents=True, exist_ok=True)
    rel_styles = os.path.relpath(DOC_ROOT / "styles.css", target.parent).replace(os.sep, "/")
    target.write_text(page_template(title, rel_styles, "doc-page api-doc-page", body_html), encoding="utf-8")


def render_type_page(entity: Entity, module: ModuleRecord, entities: dict[str, Entity]) -> None:
    breadcrumbs = [
        ('Home', rel_href(entity.page, 'index.html')),
        ('API Reference', rel_href(entity.page, 'reference/api/index.html')),
        ('Types', rel_href(entity.page, 'reference/api/types/index.html')),
        (module.canonical_path, rel_href(entity.page, module.page)),
    ]
    breadcrumb_html = ' / '.join(f'<a href="{escape(href)}">{escape(label)}</a>' for label, href in breadcrumbs) + f' / {escape(entity.name)}'
    body = f"""
  <header class=\"header\">
    <h1><span class=\"badge {escape(entity.kind)}\">{escape(entity.kind)}</span> {escape(entity.name)}</h1>
    <p class=\"page-subtitle\">Canonical API page from concat snapshot</p>
    <nav class=\"breadcrumb-path\">{breadcrumb_html}</nav>
  </header>
  <div class=\"container\">
    <div class=\"api-layout\">
      <aside class=\"sidebar\">
        <div class=\"sidebar-block\">
          <h3>Canonical ID</h3>
          <p><code>{escape(entity.id)}</code></p>
        </div>
        <div class=\"sidebar-block\">
          <h3>Source</h3>
          <p><code>{escape(entity.source_path)}:{entity.source_line}</code></p>
        </div>
        <div class=\"sidebar-block\">
          <h3>Module</h3>
          <p><a class=\"entity-ref\" data-entity-id=\"{escape(module.id)}\" href=\"{escape(rel_href(entity.page, module.page))}\"><code>{escape(module.canonical_path)}</code></a></p>
        </div>
      </aside>
      <main class=\"content\">
        <section class=\"doc-section\">
          <h2>Overview</h2>
          <p>{escape(entity.summary or f'{entity.name} is documented from the concat snapshot.')}</p>
          <div class=\"signature-block\"><pre><code>{escape(entity.signature)}</code></pre></div>
        </section>
        <section class=\"doc-section\">
          <h2>Source evidence</h2>
          {render_doc_lines(entity.doc)}
        </section>
    """
    if entity.kind == 'struct':
        body += f"""
        <section class=\"doc-section\">
          <h2>Fields / properties</h2>
          {render_members_table(entity.page, entity.members, entities, 'Fields')}
        </section>
        <section class=\"doc-section\">
          <h2>Methods</h2>
          {render_methods_table(entity.page, entity.methods, entities)}
        </section>
        <section class=\"doc-section\">
          <h2>Implements</h2>
          {render_related_list(entity.page, entity.implements, entities)}
        </section>
        """
    elif entity.kind == 'enum':
        body += f"""
        <section class=\"doc-section\">
          <h2>Variants</h2>
          {render_members_table(entity.page, entity.members, entities, 'Variants')}
        </section>
        """
    elif entity.kind == 'trait':
        body += f"""
        <section class=\"doc-section\">
          <h2>Methods</h2>
          {render_methods_table(entity.page, entity.methods, entities)}
        </section>
        <section class=\"doc-section\">
          <h2>Implementors</h2>
          {render_related_list(entity.page, entity.implementors, entities)}
        </section>
        """
    body += f"""
        <section class=\"doc-section\">
          <h2>Related types</h2>
          {render_related_list(entity.page, entity.related_ids, entities)}
        </section>
        <section class=\"doc-section\">
          <h2>See also</h2>
          <ul>
            <li><a href=\"{escape(rel_href(entity.page, 'reference/api/types/index.html'))}\">All canonical type pages</a></li>
            <li><a href=\"{escape(rel_href(entity.page, 'codemap.json'))}\">codemap.json</a></li>
            <li><a href=\"{escape(rel_href(entity.page, 'sitemap.html'))}\">Site map</a></li>
          </ul>
        </section>
      </main>
    </div>
  </div>
  <footer class=\"footer\"><p>{escape(HTML_TITLE)}</p></footer>
    """
    write_page(entity.page, f"{entity.name} — {HTML_TITLE}", body)


def render_module_items(from_page: str, members: list[Member], entities: dict[str, Entity]) -> str:
    if not members:
        return "<p><em>No public items captured in this category.</em></p>"
    rows = []
    for member in members:
        related = ""
        if member.related_types:
            related = "<div class=\"member-related\">Related: " + ", ".join(entity_link(from_page, entities[item]) for item in member.related_types if item in entities) + "</div>"
        rows.append(
            f"<tr><td id=\"{escape(member.anchor)}\"><code>{escape(member.name)}</code></td><td><code>{escape(member.signature)}</code>{related}</td><td>{escape(member.summary)}</td></tr>"
        )
    return "<table class=\"api-table\"><thead><tr><th>Name</th><th>Signature</th><th>Summary</th></tr></thead><tbody>" + "".join(rows) + "</tbody></table>"


def render_module_page(module: ModuleRecord, entities: dict[str, Entity]) -> None:
    breadcrumbs = [
        ('Home', rel_href(module.page, 'index.html')),
        ('API Reference', rel_href(module.page, 'reference/api/index.html')),
        ('Modules', rel_href(module.page, 'reference/api/modules/index.html')),
    ]
    breadcrumb_html = ' / '.join(f'<a href="{escape(href)}">{escape(label)}</a>' for label, href in breadcrumbs) + f' / {escape(module.canonical_path)}'
    type_items = ''.join(
        f'<li>{entity_link(module.page, entities[entity_id])} <span class="entity-meta">{escape(entities[entity_id].kind)}</span></li>'
        for entity_id in sorted(module.types, key=lambda item: (entities[item].kind, entities[item].name))
    ) or '<li><em>No public types captured from this module.</em></li>'
    body = f"""
  <header class=\"header\">
    <h1>Module <code>{escape(module.canonical_path)}</code></h1>
    <p class=\"page-subtitle\">Canonical file-backed API module</p>
    <nav class=\"breadcrumb-path\">{breadcrumb_html}</nav>
  </header>
  <div class=\"container\">
    <main class=\"content\">
      <section class=\"doc-section\">
        <h2>Overview</h2>
        <p>{escape(module.summary or f'Module page derived from {module.source_path}.')}</p>
        <p><strong>Source:</strong> <code>{escape(module.source_path)}</code></p>
      </section>
      <section class=\"doc-section\">
        <h2>Module docs</h2>
        {render_doc_lines(module.doc)}
      </section>
      <section class=\"doc-section\">
        <h2>Public types</h2>
        <ul class=\"entity-list\">{type_items}</ul>
      </section>
      <section class=\"doc-section\">
        <h2>Public functions</h2>
        {render_module_items(module.page, module.functions, entities)}
      </section>
      <section class=\"doc-section\">
        <h2>Public constants</h2>
        {render_module_items(module.page, module.consts, entities)}
      </section>
      <section class=\"doc-section\">
        <h2>Public type aliases</h2>
        {render_module_items(module.page, module.type_aliases, entities)}
      </section>
      <section class=\"doc-section\">
        <h2>Public submodules</h2>
        {render_module_items(module.page, module.submodules, entities)}
      </section>
      <section class=\"doc-section\">
        <h2>See also</h2>
        <ul>
          <li><a href=\"{escape(rel_href(module.page, 'reference/api/types/index.html'))}\">Canonical type pages</a></li>
          <li><a href=\"{escape(rel_href(module.page, 'codemap.json'))}\">codemap.json</a></li>
        </ul>
      </section>
    </main>
  </div>
  <footer class=\"footer\"><p>{escape(HTML_TITLE)}</p></footer>
    """
    write_page(module.page, f"{module.canonical_path} — {HTML_TITLE}", body)


def render_api_indexes(meta: dict[str, str], modules: dict[str, ModuleRecord], entities: dict[str, Entity]) -> None:
    counts = defaultdict(int)
    by_crate: dict[str, list[Entity]] = defaultdict(list)
    for entity in entities.values():
        counts[entity.kind] += 1
        crate = module_parts_for_path(entity.source_path)[0]
        by_crate[crate].append(entity)

    index_body = f"""
  <header class=\"header\">
    <h1>Canonical API Reference</h1>
    <p class=\"page-subtitle\">Generated from concat snapshot: {escape(meta.get('generated_at', 'unknown'))}</p>
    <nav class=\"breadcrumb-path\"><a href=\"{escape(rel_href('reference/api/index.html', 'index.html'))}\">Home</a> / API Reference</nav>
  </header>
  <div class=\"container\">
    <main class=\"content\">
      <section class=\"doc-section\">
        <h2>What this adds</h2>
        <p>This is the canonical API layer driven by <code>concat-report.txt</code>. It provides stable entity IDs, collision-safe page paths, a machine-readable <code>codemap.json</code>, and hoverable references powered by shared browser runtime.</p>
      </section>
      <section class=\"doc-section\">
        <h2>Snapshot metrics</h2>
        <ul>
          <li><strong>Modules:</strong> {len(modules)}</li>
          <li><strong>Structs:</strong> {counts['struct']}</li>
          <li><strong>Enums:</strong> {counts['enum']}</li>
          <li><strong>Traits:</strong> {counts['trait']}</li>
          <li><strong>codemap:</strong> <a href=\"{escape(rel_href('reference/api/index.html', 'codemap.json'))}\">codemap.json</a></li>
        </ul>
      </section>
      <div class=\"quick-nav\">
        <div class=\"nav-card\"><h3><a href=\"{escape(rel_href('reference/api/index.html', 'reference/api/types/index.html'))}\">Types</a></h3><p>Canonical pages for structs, enums, and traits.</p></div>
        <div class=\"nav-card\"><h3><a href=\"{escape(rel_href('reference/api/index.html', 'reference/api/modules/index.html'))}\">Modules</a></h3><p>File-backed public API pages, functions, consts, and submodules.</p></div>
      </div>
    </main>
  </div>
  <footer class=\"footer\"><p>{escape(HTML_TITLE)}</p></footer>
    """
    write_page('reference/api/index.html', f'API Reference — {HTML_TITLE}', index_body)

    type_sections = []
    for crate, crate_entities in sorted(by_crate.items()):
        grouped = defaultdict(list)
        for entity in sorted(crate_entities, key=lambda item: (item.kind, item.name)):
            grouped[entity.kind].append(entity)
        cards = []
        for kind in ['struct', 'enum', 'trait']:
            if not grouped[kind]:
                continue
            items = ''.join(f'<li>{entity_link("reference/api/types/index.html", entity)} <span class="entity-meta">{escape(entity.source_path)}</span></li>' for entity in grouped[kind])
            cards.append(f'<section class="doc-section"><h3>{escape(kind.title())}s ({len(grouped[kind])})</h3><ul class="entity-list">{items}</ul></section>')
        type_sections.append(f'<section class="doc-section"><h2>{escape(crate)}</h2>{"".join(cards)}</section>')
    types_body = f"""
  <header class=\"header\">
    <h1>Canonical Type Pages</h1>
    <nav class=\"breadcrumb-path\"><a href=\"{escape(rel_href('reference/api/types/index.html', 'index.html'))}\">Home</a> / <a href=\"{escape(rel_href('reference/api/types/index.html', 'reference/api/index.html'))}\">API Reference</a> / Types</nav>
  </header>
  <div class=\"container\"><main class=\"content\">{''.join(type_sections)}</main></div>
  <footer class=\"footer\"><p>{escape(HTML_TITLE)}</p></footer>
    """
    write_page('reference/api/types/index.html', f'Types — {HTML_TITLE}', types_body)

    module_groups: dict[str, list[ModuleRecord]] = defaultdict(list)
    for module in modules.values():
        crate = module_parts_for_path(module.source_path)[0]
        module_groups[crate].append(module)
    module_sections = []
    for crate, crate_modules in sorted(module_groups.items()):
        items = ''.join(
            f'<li><a class="entity-ref" data-entity-id="{escape(module.id)}" href="{escape(rel_href("reference/api/modules/index.html", module.page))}"><code>{escape(module.canonical_path)}</code></a> <span class="entity-meta">{escape(module.source_path)}</span></li>'
            for module in sorted(crate_modules, key=lambda item: item.canonical_path)
        )
        module_sections.append(f'<section class="doc-section"><h2>{escape(crate)}</h2><ul class="entity-list">{items}</ul></section>')
    modules_body = f"""
  <header class=\"header\">
    <h1>Canonical Module Pages</h1>
    <nav class=\"breadcrumb-path\"><a href=\"{escape(rel_href('reference/api/modules/index.html', 'index.html'))}\">Home</a> / <a href=\"{escape(rel_href('reference/api/modules/index.html', 'reference/api/index.html'))}\">API Reference</a> / Modules</nav>
  </header>
  <div class=\"container\"><main class=\"content\">{''.join(module_sections)}</main></div>
  <footer class=\"footer\"><p>{escape(HTML_TITLE)}</p></footer>
    """
    write_page('reference/api/modules/index.html', f'Modules — {HTML_TITLE}', modules_body)


def build_codemap(meta: dict[str, str], modules: dict[str, ModuleRecord], entities: dict[str, Entity]) -> dict:
    entities_json = [entity.to_json() for entity in sorted(entities.values(), key=lambda item: (item.kind, item.canonical_path))]
    modules_json = [module.to_json() for module in sorted(modules.values(), key=lambda item: item.canonical_path)]
    names = defaultdict(list)
    for entity in entities.values():
        names[entity.name].append(entity.id)
    return {
        "meta": {
            "source": "concat-report.txt",
            "generated_at": meta.get("generated_at", "unknown"),
            "files": int(meta.get("files", "0") or 0),
            "lines_total": int(meta.get("lines_total", "0") or 0),
            "entity_counts": {
                "modules": len(modules),
                "types": len(entities),
                "structs": sum(1 for entity in entities.values() if entity.kind == "struct"),
                "enums": sum(1 for entity in entities.values() if entity.kind == "enum"),
                "traits": sum(1 for entity in entities.values() if entity.kind == "trait"),
            },
        },
        "entities": entities_json,
        "modules": modules_json,
        "symbols": {name: ids for name, ids in sorted(names.items())},
    }


def write_runtime_assets(codemap: dict) -> None:
    (DOC_ROOT / 'codemap.json').write_text(json.dumps(codemap, indent=2), encoding='utf-8')
    (DOC_ROOT / 'codemap.js').write_text('window.__SQ_CODEMAP__ = ' + json.dumps(codemap, separators=(",", ":")) + ';\n', encoding='utf-8')
    docref_js = r'''
(function () {
  function createCard() {
    const card = document.createElement('div');
    card.className = 'doc-hover-card';
    card.hidden = true;
    document.body.appendChild(card);
    return card;
  }

  function positionCard(card, event) {
    const offset = 18;
    const maxX = window.scrollX + document.documentElement.clientWidth - card.offsetWidth - 12;
    const maxY = window.scrollY + document.documentElement.clientHeight - card.offsetHeight - 12;
    const x = Math.min(event.pageX + offset, maxX);
    const y = Math.min(event.pageY + offset, maxY);
    card.style.left = Math.max(window.scrollX + 12, x) + 'px';
    card.style.top = Math.max(window.scrollY + 12, y) + 'px';
  }

  function getData() {
    if (window.__SQ_CODEMAP__) {
      return Promise.resolve(window.__SQ_CODEMAP__);
    }
    const current = document.currentScript || document.querySelector('script[src$="docref.js"]');
    const jsonPath = current && current.dataset ? current.dataset.codemapJson : null;
    if (!jsonPath) {
      return Promise.resolve(null);
    }
    return fetch(jsonPath).then(function (response) {
      return response.ok ? response.json() : null;
    }).catch(function () {
      return null;
    });
  }

  function buildLookup(data) {
    const byId = new Map();
    const byPage = new Map();
    const byModule = new Map();
    if (data && Array.isArray(data.entities)) {
      data.entities.forEach(function (entity) {
        byId.set(entity.id, entity);
        byPage.set(entity.page, entity);
      });
    }
    if (data && Array.isArray(data.modules)) {
      data.modules.forEach(function (module) {
        byId.set(module.id, module);
        byModule.set(module.page, module);
      });
    }
    return { byId: byId, byPage: byPage, byModule: byModule, raw: data };
  }

  function renderEntity(entity) {
    const summary = entity.summary || (entity.doc && entity.doc[0]) || 'No summary available from the snapshot.';
    const signature = entity.signature ? '<pre><code>' + escapeHtml(entity.signature) + '</code></pre>' : '';
    const source = entity.source_path ? '<div class="doc-hover-source"><strong>Source:</strong> <code>' + escapeHtml(entity.source_path) + (entity.source_line ? ':' + entity.source_line : '') + '</code></div>' : '';
    return '<div class="doc-hover-kind">' + escapeHtml(entity.kind || 'entity') + '</div>' +
      '<div class="doc-hover-name">' + escapeHtml(entity.name || entity.canonical_path || entity.id) + '</div>' +
      '<div class="doc-hover-summary">' + escapeHtml(summary) + '</div>' +
      signature + source;
  }

  function escapeHtml(value) {
    return String(value)
      .replace(/&/g, '&amp;')
      .replace(/</g, '&lt;')
      .replace(/>/g, '&gt;')
      .replace(/"/g, '&quot;')
      .replace(/'/g, '&#39;');
  }

  function normalizeHref(anchor) {
    const href = anchor.getAttribute('href');
    if (!href || href.startsWith('#') || href.startsWith('http://') || href.startsWith('https://') || href.startsWith('mailto:')) {
      return null;
    }
    try {
      const absolute = new URL(href, window.location.href);
      const path = absolute.pathname.replace(/\\/g, '/');
      const marker = '/docs/html/';
      const index = path.lastIndexOf(marker);
      if (index >= 0) {
        return path.slice(index + marker.length);
      }
      const docsIndex = path.indexOf('/reference/');
      if (docsIndex >= 0) {
        return path.slice(docsIndex + 1);
      }
    } catch (error) {
      return null;
    }
    return null;
  }

  function resolveEntity(anchor, lookup) {
    const entityId = anchor.dataset.entityId;
    if (entityId && lookup.byId.has(entityId)) {
      return lookup.byId.get(entityId);
    }
    const page = normalizeHref(anchor);
    if (page && lookup.byPage.has(page)) {
      return lookup.byPage.get(page);
    }
    if (page && lookup.byModule.has(page)) {
      return lookup.byModule.get(page);
    }
    return null;
  }

  getData().then(function (data) {
    if (!data) {
      return;
    }
    const lookup = buildLookup(data);
    window.ShellQuestDocMap = {
      data: data,
      getEntity: function (id) { return lookup.byId.get(id) || null; },
      getSymbol: function (name) { return data.symbols && data.symbols[name] ? data.symbols[name].map(function (id) { return lookup.byId.get(id); }).filter(Boolean) : []; }
    };

    const card = createCard();
    let activeAnchor = null;

    document.querySelectorAll('a.entity-ref, a[data-entity-id]').forEach(function (anchor) {
      const entity = resolveEntity(anchor, lookup);
      if (!entity) {
        return;
      }
      anchor.classList.add('entity-ref');
      anchor.title = (entity.kind || 'entity') + ': ' + (entity.canonical_path || entity.name || entity.id);
      anchor.addEventListener('mouseenter', function (event) {
        activeAnchor = anchor;
        card.innerHTML = renderEntity(entity);
        card.hidden = false;
        positionCard(card, event);
      });
      anchor.addEventListener('mousemove', function (event) {
        if (activeAnchor === anchor && !card.hidden) {
          positionCard(card, event);
        }
      });
      anchor.addEventListener('mouseleave', function () {
        if (activeAnchor === anchor) {
          card.hidden = true;
          activeAnchor = null;
        }
      });
      anchor.addEventListener('blur', function () {
        if (activeAnchor === anchor) {
          card.hidden = true;
          activeAnchor = null;
        }
      });
    });
  });
})();
'''
    (DOC_ROOT / 'docref.js').write_text(docref_js.strip() + '\n', encoding='utf-8')


def inject_runtime_scripts() -> None:
    for html_file in DOC_ROOT.rglob('*.html'):
        text = html_file.read_text(encoding='utf-8')
        if 'docref.js' in text:
            continue
        rel_codemap_js = os.path.relpath(DOC_ROOT / 'codemap.js', html_file.parent).replace(os.sep, '/')
        rel_codemap_json = os.path.relpath(DOC_ROOT / 'codemap.json', html_file.parent).replace(os.sep, '/')
        rel_docref = os.path.relpath(DOC_ROOT / 'docref.js', html_file.parent).replace(os.sep, '/')
        snippet = f'  <script src="{rel_codemap_js}"></script>\n  <script defer src="{rel_docref}" data-codemap-json="{rel_codemap_json}"></script>\n'
        text = text.replace('</body>', snippet + '</body>')
        html_file.write_text(text, encoding='utf-8')


BASE_STYLES = """/* Shell Quest API docs */
:root {
  --primary: #1e3a8a;
  --secondary: #0f766e;
  --accent: #dc2626;
  --bg: #f8fafc;
  --fg: #111827;
  --muted: #6b7280;
  --border: #e5e7eb;
  --link: #2563eb;
  --code-bg: #f3f4f6;
}

* { box-sizing: border-box; }
html, body {
  margin: 0;
  padding: 0;
  font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, Helvetica, Arial, sans-serif;
  color: var(--fg);
  background: var(--bg);
  line-height: 1.6;
}
body { min-height: 100vh; }
.header {
  background: linear-gradient(135deg, var(--primary) 0%, var(--secondary) 100%);
  color: white;
  padding: 2rem 1.25rem;
  border-bottom: 3px solid var(--accent);
}
.header.hero { padding: 3rem 1.25rem; }
.header h1 { margin: 0 0 0.5rem 0; font-size: 2.25rem; }
.page-subtitle, .tagline { margin: 0; opacity: 0.92; }
.breadcrumb-path {
  display: flex;
  flex-wrap: wrap;
  gap: 0.45rem;
  margin-top: 0.9rem;
  font-size: 0.95rem;
}
.breadcrumb-path a { color: white; text-decoration: none; }
.breadcrumb-path a:hover { text-decoration: underline; }
.container { max-width: 1240px; margin: 0 auto; padding: 0 1rem; }
.content {
  background: white;
  margin: 2rem 0;
  padding: 2rem;
  border-radius: 10px;
  box-shadow: 0 1px 3px rgba(15, 23, 42, 0.1);
}
.home-content .quick-nav {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(240px, 1fr));
  gap: 1rem;
  margin: 2rem 0;
}
.nav-card {
  border: 1px solid var(--border);
  border-radius: 8px;
  padding: 1.25rem;
  background: white;
}
.nav-card h3 { margin: 0 0 0.65rem 0; }
.nav-card p { margin: 0; color: var(--muted); }
.doc-section { margin: 2rem 0; }
.doc-section h2 {
  margin: 0 0 1rem 0;
  padding-bottom: 0.45rem;
  border-bottom: 2px solid var(--accent);
  color: var(--primary);
}
.doc-section h3 { color: var(--secondary); margin: 1.25rem 0 0.75rem 0; }
a { color: var(--link); text-decoration: none; }
a:hover { text-decoration: underline; }
code {
  background: var(--code-bg);
  padding: 0.15rem 0.35rem;
  border-radius: 4px;
  font-family: ui-monospace, SFMono-Regular, Menlo, Consolas, monospace;
}
pre {
  background: var(--code-bg);
  border-left: 4px solid var(--primary);
  padding: 1rem;
  border-radius: 6px;
  overflow: auto;
}
pre code { background: transparent; padding: 0; }
.footer {
  padding: 1.25rem;
  text-align: center;
  color: var(--muted);
}
.badge {
  display: inline-block;
  margin-right: 0.4rem;
  padding: 0.2rem 0.45rem;
  border-radius: 999px;
  font-size: 0.8rem;
  text-transform: uppercase;
  letter-spacing: 0.03em;
  background: rgba(255,255,255,0.18);
  color: white;
}
.badge.struct { background: rgba(37, 99, 235, 0.28); }
.badge.enum { background: rgba(13, 148, 136, 0.28); }
.badge.trait { background: rgba(220, 38, 38, 0.28); }
/* API hover + canonical reference */
.api-layout {
  display: grid;
  grid-template-columns: 260px 1fr;
  gap: 2rem;
  align-items: start;
}
.sidebar { width: 260px; }
.sidebar-block {
  background: white;
  border: 1px solid var(--border);
  border-radius: 8px;
  padding: 1rem;
  margin-bottom: 1rem;
}
.sidebar-block h3 { margin: 0 0 0.5rem 0; color: var(--primary); font-size: 0.95rem; }
.api-table {
  width: 100%;
  border-collapse: collapse;
  margin-top: 1rem;
}
.api-table th, .api-table td {
  border: 1px solid var(--border);
  padding: 0.75rem;
  vertical-align: top;
  text-align: left;
}
.api-table th { background: #f8fafc; }
.entity-ref { text-decoration-style: dotted; }
.entity-meta, .member-related { color: var(--muted); font-size: 0.9rem; }
.entity-list { list-style: none; margin: 0; padding: 0; }
.entity-list li { margin: 0.6rem 0; }
.doc-hover-card {
  position: absolute;
  z-index: 9999;
  max-width: 420px;
  padding: 0.85rem 1rem;
  border-radius: 10px;
  border: 1px solid rgba(30, 58, 138, 0.18);
  background: rgba(255, 255, 255, 0.98);
  box-shadow: 0 12px 40px rgba(15, 23, 42, 0.18);
  backdrop-filter: blur(10px);
  pointer-events: none;
}
.doc-hover-kind { font-size: 0.75rem; font-weight: 700; text-transform: uppercase; color: var(--secondary); }
.doc-hover-name { font-weight: 700; font-size: 1rem; margin: 0.25rem 0; }
.doc-hover-summary, .doc-hover-source { font-size: 0.9rem; color: var(--fg); }
.doc-hover-source { color: var(--muted); }
@media (max-width: 960px) {
  .api-layout { grid-template-columns: 1fr; }
  .sidebar { width: auto; }
  .content { padding: 1.25rem; }
}
"""


def render_root_pages(meta: dict[str, str], modules: dict[str, ModuleRecord], entities: dict[str, Entity]) -> None:
    counts = {
        "modules": len(modules),
        "types": len(entities),
        "structs": sum(1 for entity in entities.values() if entity.kind == "struct"),
        "enums": sum(1 for entity in entities.values() if entity.kind == "enum"),
        "traits": sum(1 for entity in entities.values() if entity.kind == "trait"),
    }
    home_body = f"""
  <header class="header hero">
    <h1>Shell Quest API Docs</h1>
    <p class="tagline">Canonical type reference generated from concat-report.txt</p>
  </header>
  <div class="container">
    <main class="content home-content">
      <section class="doc-section">
        <h2>Overview</h2>
        <p>This site is rebuilt from scratch from the concat snapshot and currently focuses on canonical API pages for public types.</p>
        <ul>
          <li><strong>Snapshot:</strong> {escape(meta.get("generated_at", "unknown"))}</li>
          <li><strong>Files:</strong> {meta.get("files", "0")}</li>
          <li><strong>Types:</strong> {counts["types"]} ({counts["structs"]} structs, {counts["enums"]} enums, {counts["traits"]} traits)</li>
          <li><strong>Modules indexed:</strong> {counts["modules"]}</li>
        </ul>
      </section>
      <div class="quick-nav">
        <div class="nav-card"><h3><a href="reference/api/index.html">API Reference</a></h3><p>Canonical API landing page.</p></div>
        <div class="nav-card"><h3><a href="reference/api/types/index.html">Type Pages</a></h3><p>All public structs, enums, and traits.</p></div>
        <div class="nav-card"><h3><a href="reference/api/modules/index.html">Module Pages</a></h3><p>Source modules extracted from the snapshot.</p></div>
        <div class="nav-card"><h3><a href="codemap.json">codemap.json</a></h3><p>Machine-readable entity graph for links and hover cards.</p></div>
      </div>
    </main>
  </div>
  <footer class="footer"><p>{escape(HTML_TITLE)}</p></footer>
    """
    write_page("index.html", f"Home — {HTML_TITLE}", home_body)

    toc_body = f"""
  <header class="header">
    <h1>Table of Contents</h1>
    <nav class="breadcrumb-path"><a href="index.html">Home</a> / TOC</nav>
  </header>
  <div class="container">
    <main class="content">
      <section class="doc-section">
        <h2>Core entry points</h2>
        <ul>
          <li><a href="reference/api/index.html">API Reference Home</a></li>
          <li><a href="reference/api/types/index.html">Canonical Type Pages</a></li>
          <li><a href="reference/api/modules/index.html">Canonical Module Pages</a></li>
          <li><a href="codemap.json">codemap.json</a></li>
        </ul>
      </section>
      <section class="doc-section">
        <h2>Current coverage</h2>
        <ul>
          <li>{counts["structs"]} struct pages</li>
          <li>{counts["enums"]} enum pages</li>
          <li>{counts["traits"]} trait pages</li>
        </ul>
      </section>
    </main>
  </div>
  <footer class="footer"><p>{escape(HTML_TITLE)}</p></footer>
    """
    write_page("toc.html", f"TOC — {HTML_TITLE}", toc_body)

    sitemap_body = f"""
  <header class="header">
    <h1>Site Map</h1>
    <nav class="breadcrumb-path"><a href="index.html">Home</a> / Site Map</nav>
  </header>
  <div class="container">
    <main class="content">
      <section class="doc-section">
        <h2>Generated structure</h2>
        <ul>
          <li>index.html</li>
          <li>toc.html</li>
          <li>codemap.json</li>
          <li>codemap.js</li>
          <li>docref.js</li>
          <li>reference/api/index.html</li>
          <li>reference/api/types/index.html</li>
          <li>reference/api/modules/index.html</li>
        </ul>
      </section>
      <section class="doc-section">
        <h2>Entity totals</h2>
        <ul>
          <li>Total types: {counts["types"]}</li>
          <li>Total modules: {counts["modules"]}</li>
        </ul>
      </section>
    </main>
  </div>
  <footer class="footer"><p>{escape(HTML_TITLE)}</p></footer>
    """
    write_page("sitemap.html", f"Site Map — {HTML_TITLE}", sitemap_body)


def ensure_styles() -> None:
    styles = DOC_ROOT / 'styles.css'
    styles.parent.mkdir(parents=True, exist_ok=True)
    styles.write_text(BASE_STYLES, encoding='utf-8')


def main() -> None:
    meta, files = parse_concat_report(CONCAT_PATH)
    modules, entities, impl_blocks = parse_rust_snapshot(files)
    enrich_relationships(modules, entities, impl_blocks)

    ensure_styles()
    render_root_pages(meta, modules, entities)
    render_api_indexes(meta, modules, entities)
    for module in modules.values():
        render_module_page(module, entities)
    for entity in entities.values():
        render_type_page(entity, modules[entity.module_id], entities)

    codemap = build_codemap(meta, modules, entities)
    write_runtime_assets(codemap)
    inject_runtime_scripts()

    print(json.dumps({
        'modules': len(modules),
        'types': len(entities),
        'structs': sum(1 for entity in entities.values() if entity.kind == 'struct'),
        'enums': sum(1 for entity in entities.values() if entity.kind == 'enum'),
        'traits': sum(1 for entity in entities.values() if entity.kind == 'trait'),
        'codemap': 'docs/html/codemap.json',
        'api_index': 'docs/html/reference/api/index.html',
    }, indent=2))


if __name__ == '__main__':
    main()
