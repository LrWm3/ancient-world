# Institutional operating-budget pilot

`Culture.institution_funding = Operating` enables dated requests for operating
funds. Legacy remains the default during calibration; the balance runner exposes
`--operating-institutions`. This is a game allocation rule, not an estimate of
historical charitable finance.

At Reserve, active institutions with local representatives quote a year of
administrative fees (2 abstract currency units) and component replacement at
current prices, wear and disruption. Existing cash reduces the request. Lost,
destroyed or inaccessible buildings receive no repair allowance. Expansion and
accumulated repair backlog are not funded by this quote. Legacy brick structures
quote their own embodied brick and wear rule.

Requests share a ceiling of 0.5% of opening town cash in proportion to unmet
operating need. Zero demand gets no share; allocations never exceed demand. These
are **conditional ceilings, not escrowed cash**: earlier spending retains its
existing priority. The existing administration action must still receive and use
work in Respond. It transfers at most its captured ceiling, current unmet need
and live town cash. Changed prices do not enlarge a captured request. A depleted
town can pay less; unused ceilings expire rather than moving cash retroactively
ahead of earlier claims.

Payments use the actual representable town withdrawal as the institution's credit.
Each operating request settles at most once, belongs to its captured month and
institution, and persists through checkpoints. Inactive, moved or canceled targets
cannot collect. Legacy plans lacking a funding quote use the existing small-donation
schedule; changing the policy after planning does not change captured requests.

This pilot does not add a second source of money, reserve new personal time, or
change the schedule. Fundraising still competes within the generic cultural action
bundle, and realized collection can retain execution-order effects when live cash
or administration work is scarce. The next review must distinguish requested
funding, work actually provided, money collected, and service delivered. More
money alone cannot fix insufficient usable space; see the controlled comparison
in [institution allocation balance](institution-allocation-balance.md).

## Verification

The operating-budget GPU fixture checks read-only forecasting, missing work, a
changed policy after planning, depleted cash, filled reserves, stale and duplicate
collection, inactive/moved institutions, legacy-plan import, inaccessible buildings
and fractional transfer accounting. A CPU fixture checks proportional sharing,
zero demand, ample/empty budgets and reversed input order.

All six institutional hardware tests and 116 regular library tests pass (107
hardware tests are excluded from the ordinary library run). Full frozen scheduler
monthly/batched/checkpoint equivalence passes on seeds 17, 81 and 256; seed 256
enables operating funding alongside rotating institutional work priority. These
checks establish timing and accounting, not balanced institutional survival.

The funding report accumulates requested amounts, conditional ceilings and actual
payments every month in abstract currency units. Each quarterly request appears
once. Legacy runs have no operating requests, so their zero funding-report totals
do not mean they received no old-style donations; use institutional dues for that
comparison. Requests for a repeated reserve shortfall are counted again next
quarter and must not be interpreted as distinct annual expenses.

The representable-withdrawal correction also applies to legacy donations. The old
formula and work schedule remain, but exact old-version trajectories are not
promised. Matched policy comparisons must use the same new executable.
