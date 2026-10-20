# Searching local Codex traces

`scripts/search_codex_traces.py` searches readable reasoning text in Codex rollout
JSONL files using Python 3.10+ and its standard library. It reads local files and
prints results; it does not call a service or modify the logs.

```sh
# Search a phrase, case-insensitively.
python3 scripts/search_codex_traces.py 'root cause'

# Browse possible discoveries, reconsiderations, debugging, uncertainty or tradeoffs.
python3 scripts/search_codex_traces.py --interesting --sort interesting

# Narrow to this project and a particular theme.
python3 scripts/search_codex_traces.py --cwd ancient-world --theme reconsideration

# Include recorded assistant commentary and answers, labeled separately.
python3 scripts/search_codex_traces.py --include-messages --theme debugging

# Search a UTC time window with a regex.
python3 scripts/search_codex_traces.py --regex 'allocation|scheduler' \
  --since 2026-10-01 --before 2026-10-15

# Require both phrases and remove unwanted matches.
python3 scripts/search_codex_traces.py allocation scheduler --all --exclude 'test fixture'

# Inspect longer passages or a specific session.
python3 scripts/search_codex_traces.py --min-chars 1000 --sort longest --limit 5 --full
python3 scripts/search_codex_traces.py --session SESSION_ID_FRAGMENT --include-messages

# Export full matches to an ignored local output directory.
mkdir -p output
python3 scripts/search_codex_traces.py --include-messages --interesting --json \
  > output/codex-matches.jsonl
```

The default root is `$CODEX_HOME`, if set, otherwise `~/.codex`. Both `sessions/`
and `archived_sessions/` are searched. `--root` also accepts a directory containing
`rollout-*.jsonl` files or one explicit JSONL file. Unrelated files such as
`auth.json`, `history.jsonl`, plugin data and databases are not searched.

Results show the source file and physical line number, timestamp, session ID,
opening working directory, model, theme matches and preceding user prompt.
Text excerpts center on the first query match, or a selected theme if there is
no query match. `--full` prints the complete passage. `--json` emits one JSON
object per match with full text and prompt, plus the turn ID and record source;
diagnostics go to stderr. A source reference can be opened with, for example,
`less +123 /path/to/rollout-file.jsonl`.

Queries are literal phrases combined with OR unless `--all` is specified.
Queries and exclusions become Python regular expressions with `--regex`.
`--theme` can be repeated; any selected theme qualifies, in addition to the query.
`--interesting` enables every theme. `--list-themes` prints the actual patterns.
These are keyword heuristics that can miss interesting passages and select
ordinary ones. Sorting by `interesting` counts distinct matching themes; it
does not assess correctness or reasoning quality. Themes remain case-insensitive
even when queries use `--case-sensitive`.

All selected files are scanned so the default 20 results are ranked globally by
event timestamp. `--sort longest` ranks by text length; ties use recency.
Only the best `--limit` results are retained, along with per-session duplicate
hashes and the current input record. Scanning several gigabytes can take time.
Session and opening-directory filters can skip files after reading their metadata.
Date filtering uses each event's timestamp, including events in resumed sessions;
`--since` is inclusive and `--before` exclusive. Dates and times without an offset
use UTC. Undated records sort last and are excluded when date filters are active.

Supported reasoning representations are response-item summaries/content,
assistant messages explicitly marked `analysis`, legacy `agent_reasoning` and
`agent_reasoning_raw_content` events, and completed `Reasoning` items. Message
search supports assistant response items, `agent_message` events and completed
`AgentMessage` items. User messages supply context and are never search candidates;
tool outputs and instructions are excluded. Identical text within a session turn
is deduplicated separately for reasoning and assistant messages, keeping the first
occurrence. This also collapses repeated text when a session records both an
event and a response item. For older logs without turn IDs, this deduplication
applies across the whole session.

Some Codex versions save only encrypted reasoning and empty reasoning fields.
The script counts those records but cannot search or recover their contents.
`--include-messages` provides access to recorded commentary and answers in those
sessions. Diagnostics describe all readable records in selected sessions before
query/date filtering; encrypted/empty counts describe records, not unique model
responses. Malformed lines, including incomplete trailing writes, are counted and
skipped. File read errors are reported and make the exit status indicate an
incomplete search. Exit codes: 0 for matches, 1 for no matches, 2 for input or file
errors, and 130 for interruption.

Run focused checks with:

```sh
python3 -m unittest discover -s scripts -p 'test_search_codex_traces.py'
```

Tests use synthetic records for schema variants, encrypted-only sessions,
duplicate events, archive discovery, query and date filtering, ranking, corrupt
lines and CLI validation. Local session contents and generated search results
must stay out of Git.

Verification on 2026-10-15: all 13 focused tests passed. A local scan read 563
session files with no file errors and found no readable reasoning; the optional
assistant-message search returned matches with valid file/line references.
Seven malformed lines were skipped during the full scan. These observations
describe that local history, not every Codex installation or log version.
