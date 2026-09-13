# Contract-aware workshop shift requests

Status: an opt-in staffing pilot; normal-world balance is not established.

`Enterprises.procurement.contract_staffing` lets a firm's due, unsettled service
orders contribute to its desired shift. It defaults false, including old archives.
It is independent of automatic procurement: an explicitly funded order can also
supply demand. Disabling it changes future requests without cancelling contracts,
repayments or refunds.

`--contract-workshop-staffing[=true|false]` overrides the policy for a headless
history. Omission preserves the archived setting, and the switch enables neither
automatic procurement nor credit. The monetary runner accepts the same boolean
flag and holds it constant in all four arms; without the flag it explicitly uses
false, avoiding accidental inheritance from the input checkpoint.

The planner first gathers due contracted work by firm. Desired work is the larger
of demonstrated work with the existing headroom/minimum and contracted work,
capped by leased physical capacity and available site labor. The existing wage
cash cap, shared labor allocation and individual matching still follow. Future,
settled and expired orders do not supply current-month demand. Orders do not
transfer their escrow to operators until actual work earns fees.

This connects two previously separate forecasts. It does not reserve materials,
guarantee customers, or guarantee that paid labor produces output. Larger shifts
can increase paid idle work when inputs are missing, so this remains opt-in until
matched comparisons show useful results. Avoid treating a higher request or payroll
as a production benefit.

The controlled fixture compares identical opening states with low demonstrated
work and the same funded contract. The enabled request should increase while
money is conserved and fees remain unearned at reservation. GPU monthly/batched/
checkpoint continuation then exercises the enabled policy. The GPU fixture passed
(one test), as did strict all-target Clippy. Twelve Python reporting tests passed.
The CLI independence/override check passed (one test), followed by strict
all-target Clippy.

Next compare the independent experiment override off/on
with procurement and credit settings held fixed. Use due-month observations to
separate extra funded labor from extra completed work, fees, refunds and household
food access. The Stage 2 currency gate remains unchanged.


The [eight-arm 50-year comparison](contract-staffing-comparison.md) completed:
contract-aware requests increase completed work, but increase paid idle work too,
and terminal food access worsens in both issuance comparisons. No loans issued.
Keep the policy opt-in and investigate execution feasibility before widening it.
