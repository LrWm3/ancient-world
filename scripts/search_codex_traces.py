#!/usr/bin/env python3
"""Search readable reasoning in local Codex rollout JSONL files (stdlib only)."""

from __future__ import annotations

import argparse
from collections import Counter
from dataclasses import asdict, dataclass
from datetime import datetime, timezone
import hashlib
import heapq
import json
import os
from pathlib import Path
import re
import sys
from typing import Iterator

DEFAULT_LIMIT = 20
DEFAULT_EXCERPT_CHARS = 700
PROMPT_EXCERPT_CHARS = 240
THEMES = {
    "reconsideration": r"\b(?:wait|reconsider\w*|on second thought|actually|I was wrong|"
    r"my mistake|instead|wrong assumption)\b",
    "uncertainty": r"\b(?:uncertain\w*|unsure|not sure|hypothes\w*|might|perhaps|"
    r"need to (?:check|verify|confirm))\b",
    "tradeoffs": r"\b(?:trade[ -]?offs?|alternative\w*|pros and cons|versus|"
    r"on the other hand|simpler|complexity)\b",
    "debugging": r"\b(?:root cause|bug|regression|race condition|deadlock|"
    r"off.by.one|reproduc\w*|unexpected|fails?|failing)\b",
    "discovery": r"\b(?:aha|realiz\w*|insight|turns out|key (?:point|issue|observation)|"
    r"surpris\w*|counterintuitive)\b",
}
THEME_PATTERNS = {name: re.compile(pattern, re.IGNORECASE) for name, pattern in THEMES.items()}
TEXT_PART_TYPES = {"text", "input_text", "output_text", "summary_text", "reasoning_text"}


@dataclass
class Trace:
    path: str
    line: int
    timestamp: str
    session_id: str
    cwd: str
    model: str
    turn_id: str
    kind: str
    source: str
    text: str
    user_prompt: str


def text_parts(value: object) -> str:
    """Read known text fields only; never stringify ciphertext or arbitrary objects."""
    if isinstance(value, str):
        return value
    if not isinstance(value, list):
        return ""
    parts = []
    for part in value:
        if isinstance(part, str):
            parts.append(part)
        elif isinstance(part, dict) and str(part.get("type", "")).lower() in TEXT_PART_TYPES:
            if isinstance(part.get("text"), str):
                parts.append(part["text"])
    return "\n\n".join(parts)


def extract_text(row: dict) -> tuple[str, str, str, bool] | None:
    """Return (kind, source, text, encrypted_only) for supported records."""
    payload = row.get("payload")
    if not isinstance(payload, dict):
        return None
    record_type = row.get("type")
    kind = payload.get("type")
    source = f"{record_type}.{kind}"
    if record_type == "response_item" and kind == "reasoning":
        content = text_parts(payload.get("content"))
        summary = text_parts(payload.get("summary"))
        text = "\n\n".join(dict.fromkeys(p for p in (summary, content) if p))
        return "reasoning", source, text, bool(payload.get("encrypted_content") and not text)
    if record_type == "response_item" and kind == "message":
        role = payload.get("role")
        if role in ("assistant", "user"):
            channel = payload.get("channel", payload.get("phase"))
            output_kind = "reasoning" if role == "assistant" and channel == "analysis" else role
            return output_kind, source, text_parts(payload.get("content")), False
    if record_type == "event_msg":
        if kind == "item_completed":
            item = payload.get("item")
            if not isinstance(item, dict):
                return None
            source += "." + str(item.get("type"))
            if item.get("type") == "Reasoning":
                pieces = (text_parts(item.get("summary_text")), text_parts(item.get("raw_content")))
                return "reasoning", source, "\n\n".join(dict.fromkeys(p for p in pieces if p)), False
            if item.get("type") in ("AgentMessage", "UserMessage"):
                role = "assistant" if item["type"] == "AgentMessage" else "user"
                return role, source, text_parts(item.get("content")), False
        if kind in ("agent_reasoning", "agent_reasoning_raw_content"):
            return "reasoning", source, text_parts(payload.get("text")), False
        if kind in ("agent_message", "user_message"):
            role = "assistant" if kind == "agent_message" else "user"
            return role, source, text_parts(payload.get("message")), False
    return None


def discover_files(root: Path) -> list[Path]:
    if root.is_file():
        return [root]
    if not root.is_dir():
        raise ValueError(f"Trace root does not exist: {root}")
    session_dirs = [root / name for name in ("sessions", "archived_sessions") if (root / name).is_dir()]
    paths = {path for directory in session_dirs or [root] for path in directory.rglob("rollout-*.jsonl")}
    if not paths:
        raise ValueError(f"No rollout-*.jsonl files found under {root}")
    return sorted(paths)


def read_traces(path: Path, args: argparse.Namespace, stats: Counter) -> Iterator[Trace]:
    session_id, cwd, model, turn_id, prompt = path.stem, "", "", "", ""
    seen = set()
    stats["files_scanned"] += 1
    try:
        # Binary line iteration lets us skip invalid UTF-8 without losing later records.
        with path.open("rb") as stream:
            for line_number, line in enumerate(stream, 1):
                if not line.strip():
                    continue
                try:
                    row = json.loads(line)
                except (ValueError, UnicodeError):
                    stats["malformed_lines"] += 1
                    continue
                if not isinstance(row, dict) or not isinstance(row.get("payload"), dict):
                    stats["unsupported_records"] += 1
                    continue
                payload = row["payload"]
                if row.get("type") == "session_meta":
                    session_id = string_field(payload, "id") or string_field(payload, "session_id") or session_id
                    cwd = string_field(payload, "cwd")
                    # These filters describe the session's opening metadata.
                    if (args.session and args.session not in session_id) or (args.cwd and args.cwd not in cwd):
                        stats["files_filtered"] += 1
                        return
                elif row.get("type") == "turn_context":
                    model = string_field(payload, "model")
                    turn_id = string_field(payload, "turn_id")
                elif row.get("type") == "event_msg" and payload.get("type") == "task_started":
                    turn_id = string_field(payload, "turn_id")
                extracted = extract_text(row)
                if extracted is None:
                    continue
                kind, source, text, encrypted_only = extracted
                if kind == "user":
                    if text.strip():
                        prompt = text
                    continue
                if args.session and args.session not in session_id or args.cwd and args.cwd not in cwd:
                    continue
                if kind == "reasoning":
                    stats["reasoning_records"] += 1
                    if encrypted_only:
                        stats["encrypted_only_records"] += 1
                    elif not text.strip():
                        stats["empty_reasoning_records"] += 1
                if not text.strip():
                    continue
                # Codex can record the same text as an event and a response item.
                key = (turn_id, kind, hashlib.sha256(text.encode("utf-8", errors="surrogatepass")).digest())
                if key in seen:
                    stats["duplicates_skipped"] += 1
                    continue
                seen.add(key)
                stats[f"readable_{kind}_records"] += 1
                if kind == "assistant" and not args.include_messages:
                    continue
                yield Trace(str(path), line_number, string_field(row, "timestamp"), session_id,
                            cwd, model, turn_id, kind, source, text, prompt)
    except OSError as error:
        stats["file_errors"] += 1
        print(f"Warning: {path}: {error}", file=sys.stderr)


def string_field(value: dict, key: str) -> str:
    field = value.get(key)
    return field if isinstance(field, str) else ""


def parse_time(value: str) -> datetime:
    try:
        parsed = datetime.fromisoformat(value.replace("Z", "+00:00"))
        return parsed.replace(tzinfo=timezone.utc) if parsed.tzinfo is None else parsed.astimezone(timezone.utc)
    except (ValueError, OverflowError) as error:
        raise argparse.ArgumentTypeError(f"Invalid ISO date/time: {value}") from error


def positive_int(value: str) -> int:
    try:
        number = int(value)
    except ValueError as error:
        raise argparse.ArgumentTypeError("Expected a positive integer") from error
    if number <= 0:
        raise argparse.ArgumentTypeError("Expected a positive integer")
    return number


def matching_traces(paths: list[Path], args: argparse.Namespace, stats: Counter) -> Iterator[dict]:
    for path in paths:
        for trace in read_traces(path, args, stats):
            try:
                timestamp = parse_time(trace.timestamp)
            except argparse.ArgumentTypeError:
                stats["invalid_timestamps"] += 1
                if args.since or args.before:
                    continue
                timestamp = datetime.min.replace(tzinfo=timezone.utc)
            if args.since and timestamp < args.since or args.before and timestamp >= args.before:
                continue
            if len(trace.text) < args.min_chars:
                continue
            hits = [pattern.search(trace.text) for pattern in args.patterns]
            if hits and not (all(hits) if args.match_all else any(hits)):
                continue
            if any(pattern.search(trace.text) for pattern in args.exclude_patterns):
                continue
            themes = {name: pattern.search(trace.text) for name, pattern in THEME_PATTERNS.items()}
            if args.themes and not any(themes[name] for name in args.themes):
                continue
            # Center the excerpt on a query match, then on a selected theme.
            positions = [hit.start() for hit in hits if hit]
            if not positions:
                positions = [themes[name].start() for name in args.themes if themes[name]]
            result = asdict(trace)
            result.update(themes=[name for name, hit in themes.items() if hit],
                          match_offset=min(positions, default=0), characters=len(trace.text))
            score = len(result["themes"])
            rank = (timestamp, trace.path, trace.line)
            if args.sort == "longest":
                rank = (len(trace.text), *rank)
            elif args.sort == "interesting":
                rank = (score, *rank)
            result["rank"] = rank
            stats["matches"] += 1
            yield result


def excerpt(text: str, width: int, offset: int = 0) -> str:
    start = max(0, min(offset - width // 3, len(text) - width))
    end = start + width
    return ("…" if start else "") + text[start:end] + ("…" if end < len(text) else "")


def terminal_text(text: str) -> str:
    # Logs are data: suppress terminal control sequences while retaining formatting.
    return "".join(char if char.isprintable() or char in "\n\t" else f"\\u{ord(char):04x}" for char in text)


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter,
        epilog="""Examples:
  %(prog)s 'root cause'
  %(prog)s --interesting --sort interesting --cwd ancient-world
  %(prog)s --theme reconsideration --include-messages
  %(prog)s --regex 'race condition|deadlock' --since 2026-10-01
  %(prog)s allocation scheduler --all --json

Only readable text is searchable; encrypted reasoning is counted, never decoded.
Theme matches are keyword heuristics, not a judgment of reasoning quality.""")
    parser.add_argument("query", nargs="*", help="Literal search phrases (OR by default)")
    parser.add_argument("--root", type=Path, default=Path(os.environ.get("CODEX_HOME") or "~/.codex"),
                        help="Codex home, rollout directory, or single JSONL file (default: CODEX_HOME or ~/.codex)")
    parser.add_argument("--regex", action="store_true", help="Treat queries and exclusions as Python regexes")
    parser.add_argument("--case-sensitive", action="store_true")
    parser.add_argument("--all", dest="match_all", action="store_true", help="Require every query phrase")
    parser.add_argument("--exclude", action="append", default=[], help="Reject matching text; repeatable")
    parser.add_argument("--theme", action="append", choices=sorted(THEMES), default=[], help="Require any selected theme; repeatable")
    parser.add_argument("--interesting", action="store_true", help="Require any built-in theme")
    parser.add_argument("--list-themes", action="store_true", help="Show theme regexes and exit")
    parser.add_argument("--include-messages", action="store_true", help="Also search recorded assistant commentary and answers")
    parser.add_argument("--cwd", help="Case-sensitive substring of the session's opening working directory")
    parser.add_argument("--session", help="Case-sensitive substring of the session ID")
    parser.add_argument("--since", type=parse_time, help="Inclusive ISO date/time; dates and naive times use UTC")
    parser.add_argument("--before", type=parse_time, help="Exclusive ISO date/time; dates and naive times use UTC")
    parser.add_argument("--min-chars", type=positive_int, default=1, help="Minimum text length")
    parser.add_argument("--limit", type=positive_int, default=DEFAULT_LIMIT, help=f"Maximum results (default: {DEFAULT_LIMIT})")
    parser.add_argument("--sort", choices=("recent", "longest", "interesting"), default="recent",
                        help="Rank globally by timestamp, length, or number of themes (default: recent)")
    parser.add_argument("--excerpt-chars", type=positive_int, default=DEFAULT_EXCERPT_CHARS)
    parser.add_argument("--full", action="store_true", help="Print full matching text instead of excerpts")
    parser.add_argument("--json", action="store_true", help="Emit JSONL with full text and preceding user prompt")
    return parser


def main(argv: list[str] | None = None) -> int:
    parser = build_parser()
    args = parser.parse_args(argv)
    if args.list_themes:
        for name, pattern in THEMES.items():
            print(f"{name}: {pattern}")
        return 0
    if args.since and args.before and args.since >= args.before:
        parser.error("--since must be earlier than --before")
    flags = 0 if args.case_sensitive else re.IGNORECASE
    try:
        compile_pattern = lambda value: re.compile(value if args.regex else re.escape(value), flags)
        args.patterns = [compile_pattern(value) for value in args.query]
        args.exclude_patterns = [compile_pattern(value) for value in args.exclude]
        paths = discover_files(args.root.expanduser().resolve())
    except (ValueError, re.error, OSError) as error:
        parser.error(str(error))
    args.themes = list(THEMES) if args.interesting else args.theme
    stats = Counter()
    results = heapq.nlargest(args.limit, matching_traces(paths, args, stats), key=lambda item: item["rank"])
    for result in results:
        result.pop("rank")
        if args.json:
            print(json.dumps(result, ensure_ascii=True))
            continue
        print(terminal_text(f"{result['path']}:{result['line']}"))
        print(terminal_text(f"  {result['timestamp'] or '(undated)'} | {result['kind']} | "
                            f"{result['characters']} chars | themes: {', '.join(result['themes']) or 'none'}"))
        print(terminal_text(f"  session: {result['session_id']} | cwd: {result['cwd']} | model: {result['model']}"))
        if result["user_prompt"]:
            prompt = excerpt(" ".join(result["user_prompt"].split()), PROMPT_EXCERPT_CHARS)
            print(terminal_text(f"  user: {prompt}"))
        body = result["text"] if args.full else excerpt(result["text"], args.excerpt_chars, result["match_offset"])
        print(terminal_text(body))
        print()
    print(f"Scanned {stats['files_scanned']} files ({stats['files_filtered']} filtered); "
          f"readable: {stats['readable_reasoning_records']} reasoning, {stats['readable_assistant_records']} assistant; "
          f"{stats['encrypted_only_records']} encrypted-only and {stats['empty_reasoning_records']} empty reasoning records; "
          f"{stats['duplicates_skipped']} duplicates skipped; {stats['malformed_lines']} malformed lines; "
          f"{stats['file_errors']} file errors; {stats['matches']} matches, showing {len(results)}.", file=sys.stderr)
    if not stats["readable_reasoning_records"]:
        print("No readable reasoning found in the selected sessions. Encrypted payloads cannot be searched.", file=sys.stderr)
        if not args.include_messages:
            print("Use --include-messages to search recorded assistant commentary and answers.", file=sys.stderr)
    if stats["invalid_timestamps"]:
        print(f"{stats['invalid_timestamps']} records had missing/invalid timestamps; "
              "they sort oldest and are excluded when date filters are used.", file=sys.stderr)
    if stats["file_errors"]:
        return 2
    return 0 if results else 1


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except BrokenPipeError:
        # Also redirect the final interpreter flush when piping to head.
        os.dup2(os.open(os.devnull, os.O_WRONLY), sys.stdout.fileno())
        raise SystemExit(0)
    except KeyboardInterrupt:
        raise SystemExit(130)
