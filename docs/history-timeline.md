# Settlement history timeline

Open **Civilizations and history → Timeline**. Select a town, move the month
cursor, and compare aligned population, aggregate food-reserve and treasury charts.
All charts use the same time range and cursor; each has a separately labelled
vertical scale. The selected month's occupied/abandoned status is also recorded.

A town selected in the economy tab is also selected in the timeline. Selecting a
place in the timeline or using **Locate** focuses the existing globe and atlas.
**The map remains present-day.** The cursor does not rewind terrain, borders,
routes, or the simulation. Historical map reconstruction remains future work.

Events involving the town as primary site, other site or explicit site subject
appear within six months of the cursor. Switch to the entire town chronicle to
include earlier events, or include routine harvest and market notices (hidden by
default). At most 100 latest matching events are listed. Clicking an event shows
its narrative, recorded causes, linked consequences (up to 30), and location
buttons. These links navigate the recorded causal graph; alignment on a chart
does not imply causality. Attributed religious narratives remain attributed text,
not a claim that supernatural effects occurred.

## What is measured

Each settlement records one observation at the end of a completed social month,
after monthly and scheduled annual updates:

- Population: settlement resident stock in people; travelers are not added back.
- Food: the existing aggregate food reserve, in kg food equivalent. This excludes
  separate crop goods and household inventories, and is **not** a measure of
  household food access, total calories available or agricultural productivity.
- Treasury: settlement cash in abstract currency, not total household wealth.
- Abandoned: the lifecycle flag at that observation's month.

The observations are passive. They do not consume resources, add events, change
random streams or affect simulation decisions. There are no inferred migration,
health or wealth-distribution series in this first view.

## Retention and saves

The latest 600 monthly observations per site are retained in a bounded deque.
This is a rolling 50-year window, not a complete quantitative record of a long
history. Events keep their existing retention. Earlier values are not interpolated
or regenerated. A missing observation is labelled as missing, and chart lines do
not bridge gaps. Old archives deserialize to empty timelines; advancing them
begins recording at their next simulated month, without fabricated founding data.

Observations are part of the ordinary history archive and headless history export.
The GPU simulation layout is unchanged. Archive validation checks chronological
order, dates, capacity, finite values and nonnegative measured stocks. The in-memory
observation payload is approximately 3 MiB at 256 towns and 600 observations each,
before allocator/serialization overhead; JSON archives use more space. No extra
GPU readback is introduced.

## Verification

```sh
mise exec rust@1.89.0 -- cargo test --lib history_timeline
mise exec rust@1.89.0 -- cargo test --test history_timeline -- --ignored --nocapture
```

The CPU fixture checks bounded retention, duplicate-boundary replacement, serialized
continuation, legacy empty defaults, invalid values/dates, and chart construction
through an egui frame. The GPU fixture compares a 24-month history against a
checkpoint resumed at month six and advanced one month at a time. It requires exact
serialized history equality, 24 observations for initial towns, and agreement of
the last population sample with the simulation. It also renders the populated
timeline panel through an egui frame. Native mouse interaction and
historical atlas reconstruction are not covered by these tests.

Validation on 2026-09-10: 36 regular library tests passed (45 hardware tests remained
ignored in that command). The dedicated GPU timeline test passed on the Quadro RTX
5000/Vulkan, including populated-panel construction and exact resumed history
comparison. Formatting, all-target Clippy with warnings denied, and the repository
artifact check passed. No generated visual or experimental artifacts are committed.
