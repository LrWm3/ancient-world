"""Synthetic fixtures only; no local session contents belong in the repository."""

from contextlib import redirect_stderr, redirect_stdout
import io
import json
from pathlib import Path
import tempfile
import unittest

from search_codex_traces import extract_text, main


def row(record_type, payload, timestamp="2026-10-10T12:00:00Z"):
    return dict(type=record_type, payload=payload, timestamp=timestamp)


def reasoning(text, timestamp="2026-10-10T12:00:00Z"):
    return row("response_item", dict(type="reasoning", content=[dict(type="reasoning_text", text=text)]), timestamp)


def message(role, text):
    return row("response_item", dict(type="message", role=role, content=[dict(type="output_text", text=text)]))


class TraceSearchTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)

    def write_session(self, records, name="one", archived=False, cwd="/work/ancient-world"):
        directory = self.root / ("archived_sessions" if archived else "sessions/2026/10/10")
        directory.mkdir(parents=True, exist_ok=True)
        path = directory / f"rollout-{name}.jsonl"
        header = row("session_meta", dict(id=name, cwd=cwd))
        context = row("turn_context", dict(model="test-model", turn_id="turn-one"))
        path.write_text("\n".join(json.dumps(item) for item in [header, context, *records]) + "\n", encoding="utf-8")
        return path

    def run_search(self, *args):
        stdout, stderr = io.StringIO(), io.StringIO()
        with redirect_stdout(stdout), redirect_stderr(stderr):
            status = main(["--root", str(self.root), *args])
        return status, stdout.getvalue(), stderr.getvalue()

    def json_search(self, *args):
        status, output, diagnostic = self.run_search("--json", *args)
        return status, [json.loads(line) for line in output.splitlines()], diagnostic

    def test_supported_reasoning_formats_and_ignored_content(self):
        records = [
            reasoning("A readable content block."),
            row("response_item", dict(type="reasoning", summary=[dict(type="summary_text", text="A summary.")], encrypted_content="opaque")),
            row("event_msg", dict(type="agent_reasoning", text="Legacy summary.")),
            row("event_msg", dict(type="agent_reasoning_raw_content", text="Legacy raw text.")),
            row("event_msg", dict(type="item_completed", item=dict(type="Reasoning", summary_text=["Completed summary."], raw_content=["Completed content."]))),
            row("response_item", dict(type="message", role="assistant", channel="analysis", content=[dict(type="text", text="Analysis message.")])),
        ]
        for record in records:
            with self.subTest(record=record):
                kind, source, text, encrypted = extract_text(record)
                self.assertEqual(kind, "reasoning")
                self.assertTrue(text)
                self.assertFalse(encrypted)
        self.assertIsNone(extract_text(row("response_item", dict(type="function_call_output", output="Ignore tool output."))))
        self.assertIsNone(extract_text(message("developer", "Ignore instructions.")))
        self.assertIsNone(extract_text(row("event_msg", dict(type="item_completed", item=None))))

    def test_ciphertext_and_empty_reasoning_are_not_searchable(self):
        self.write_session([
            row("response_item", dict(type="reasoning", encrypted_content="secret keyword", summary=[])),
            row("event_msg", dict(type="item_completed", item=dict(type="Reasoning", summary_text=[], raw_content=[]))),
            row("response_item", dict(type="reasoning", content=[dict(type="encrypted_content", text="secret keyword")], summary=[])),
        ])
        status, results, diagnostic = self.json_search("secret")
        self.assertEqual((status, results), (1, []))
        self.assertIn("1 encrypted-only and 2 empty reasoning records", diagnostic)
        self.assertIn("No readable reasoning", diagnostic)
        self.assertNotIn("secret keyword", diagnostic)

    def test_assistant_opt_in_context_and_duplicates(self):
        path = self.write_session([
            message("user", "Find the scheduling bug."),
            row("event_msg", dict(type="agent_reasoning", text="A duplicate root cause.")),
            reasoning("A duplicate root cause."),
            row("event_msg", dict(type="item_completed", item=dict(type="Reasoning", raw_content=["A duplicate root cause."]))),
            message("assistant", "The root cause is documented."),
            row("event_msg", dict(type="agent_message", message="The root cause is documented.")),
            row("event_msg", dict(type="item_completed", item=dict(type="AgentMessage", content=[dict(type="Text", text="The root cause is documented.")]))),
        ])
        status, results, diagnostic = self.json_search("root cause")
        self.assertEqual(status, 0)
        self.assertEqual(len(results), 1)
        self.assertEqual(results[0]["path"], str(path))
        self.assertEqual(results[0]["line"], 4)
        self.assertEqual(results[0]["user_prompt"], "Find the scheduling bug.")
        self.assertEqual(results[0]["model"], "test-model")
        self.assertIn("4 duplicates skipped", diagnostic)
        _, results, _ = self.json_search("root cause", "--include-messages")
        self.assertEqual({result["kind"] for result in results}, {"reasoning", "assistant"})

    def test_archive_discovery_ranking_and_excluded_unrelated_files(self):
        self.write_session([reasoning("Older match.", "2026-10-01T00:00:00Z")], "old")
        self.write_session([reasoning("Newest match.", "2026-10-12T00:00:00Z")], "new", archived=True)
        (self.root / "history.jsonl").write_text(json.dumps(reasoning("Do not scan history.")))
        _, results, diagnostic = self.json_search("--limit", "1")
        self.assertEqual([item["text"] for item in results], ["Newest match."])
        self.assertIn("2 matches, showing 1", diagnostic)

    def test_literal_regex_all_case_and_exclusion(self):
        self.write_session([reasoning("A+B allocation scheduler"), reasoning("aaab ALLOCATION"), reasoning("unrelated")])
        _, results, _ = self.json_search("a+b")
        self.assertEqual(len(results), 1)
        self.assertTrue(results[0]["text"].startswith("A+B"))
        _, results, _ = self.json_search("a+b", "--regex")
        self.assertEqual([item["text"] for item in results], ["aaab ALLOCATION"])
        _, results, _ = self.json_search("allocation", "scheduler", "--all")
        self.assertEqual(len(results), 1)
        _, results, _ = self.json_search("allocation", "--exclude", "scheduler")
        self.assertEqual([item["text"] for item in results], ["aaab ALLOCATION"])
        status, results, _ = self.json_search("allocation", "--case-sensitive", "--exclude", "scheduler")
        self.assertEqual((status, results), (1, []))

    def test_dates_use_event_timestamps_and_timezone_offsets(self):
        self.write_session([
            reasoning("Included", "2026-10-10T02:00:00+02:00"),
            reasoning("Too early", "2026-10-10T01:59:59+02:00"),
            reasoning("Exclusive end", "2026-10-11T00:00:00Z"),
            reasoning("Unknown date", "invalid"),
        ])
        _, results, diagnostic = self.json_search("--since", "2026-10-10", "--before", "2026-10-11")
        self.assertEqual([item["text"] for item in results], ["Included"])
        self.assertIn("1 records had missing/invalid timestamps", diagnostic)

    def test_repeated_text_in_a_later_turn_remains_searchable(self):
        self.write_session([
            reasoning("Repeated observation", "2026-10-01T00:00:00Z"),
            row("event_msg", dict(type="task_started", turn_id="turn-two")),
            reasoning("Repeated observation", "2026-10-10T00:00:00Z"),
        ])
        _, results, _ = self.json_search("--since", "2026-10-10")
        self.assertEqual(len(results), 1)
        self.assertEqual(results[0]["turn_id"], "turn-two")

    def test_opening_session_filters_and_single_file_root(self):
        path = self.write_session([reasoning("First")], "match-id")
        self.write_session([reasoning("Second")], "other-id", cwd="/elsewhere")
        _, results, _ = self.json_search("--cwd", "ancient-world", "--session", "match")
        self.assertEqual([item["text"] for item in results], ["First"])
        _, results, _ = self.json_search("--root", str(path))
        self.assertEqual([item["text"] for item in results], ["First"])

    def test_themes_ranking_and_minimum_length(self):
        self.write_session([
            reasoning("Plain text with no theme."),
            reasoning("Wait, the root cause was an unexpected race condition."),
            reasoning("A simpler alternative."),
        ])
        _, results, _ = self.json_search("--interesting", "--sort", "interesting")
        self.assertEqual(len(results), 2)
        self.assertEqual(results[0]["themes"], ["reconsideration", "debugging"])
        _, results, _ = self.json_search("--theme", "tradeoffs")
        self.assertEqual([item["text"] for item in results], ["A simpler alternative."])
        _, results, _ = self.json_search("--sort", "longest", "--min-chars", "30")
        self.assertEqual(len(results), 1)

    def test_corrupt_and_unexpected_lines_do_not_hide_later_records(self):
        path = self.write_session([reasoning("First valid")])
        with path.open("ab") as stream:
            stream.write(b'not-json\n\xff\n[]\n{"payload":null}\n')
            stream.write(json.dumps(reasoning("Later valid")).encode() + b"\n")
            stream.write(b'{"partial":')
        _, results, diagnostic = self.json_search()
        self.assertEqual({item["text"] for item in results}, {"First valid", "Later valid"})
        self.assertIn("3 malformed lines", diagnostic)

    def test_excerpt_centers_on_match_and_json_preserves_full_text(self):
        text = "Before " * 150 + "needle" + " After" * 150
        self.write_session([reasoning(text)])
        _, output, _ = self.run_search("needle", "--excerpt-chars", "80")
        self.assertIn("needle", output)
        self.assertNotIn(text, output)
        _, output, _ = self.run_search("needle", "--full")
        self.assertIn(text, output)
        _, results, _ = self.json_search("needle")
        self.assertEqual(results[0]["text"], text)
        self.assertNotIn("rank", results[0])

    def test_invalid_cli_arguments_and_missing_root(self):
        self.write_session([])
        for args in [("--regex", "["), ("--limit", "0"), ("--limit", "bad"),
                     ("--since", "bad"), ("--since", "2026-10-12", "--before", "2026-10-01"),
                     ("--root", str(self.root / "missing"))]:
            with self.subTest(args=args), self.assertRaises(SystemExit) as error:
                self.run_search(*args)
            self.assertEqual(error.exception.code, 2)

    def test_unreadable_file_reports_incomplete_search(self):
        with redirect_stdout(io.StringIO()), redirect_stderr(io.StringIO()) as diagnostic:
            from collections import Counter
            from search_codex_traces import build_parser, read_traces
            stats = Counter()
            self.assertEqual(list(read_traces(self.root / "missing.jsonl", build_parser().parse_args([]), stats)), [])
            self.assertEqual(stats["file_errors"], 1)
        self.assertIn("Warning", diagnostic.getvalue())


if __name__ == "__main__":
    unittest.main()
