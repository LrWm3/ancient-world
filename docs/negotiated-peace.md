# Negotiated peace obligations

`offer_peace(war, payer, total, months)` records a three-month offer. The other
council must explicitly call `accept_peace`; offers do not stop fighting. Terms
require 1–60 monthly installments and an affordable first payment. Acceptance
transfers actual council cash, ends the active war, and orders its armies home
using their recorded return duration. People, equipment and remaining food stay
with the traveling army. Existing occupation ends; control already transferred
by an earlier completed conquest is not retroactively changed.

Later installments settle once in Open, before other monthly spending. Receipts
separate cumulative scheduled obligations, actual transfers, and arrears. Payments
can catch up while funds remain available. Three consecutive shortfalls terminate
the ten-year protection treaty and reduce bilateral trust by 25 percentage points.
The ordinary ten-year post-war restriction then also lifts for this breached peace;
normal route, manpower and provisioning requirements still apply. Breach does not
itself launch another war. Obligations and receipts persist in archives.

This is an initial negotiated scenario/API mechanism. Councils do not yet score
and autonomously bargain over competing peace offers. There is no interest,
third-party enforcement, territorial exchange, or collection after breach. Peace
payments have an explicit early claim on council reserves; this is not a general
multi-creditor arbitration system.

## Verification

The GPU-backed fixture `peace_requires_acceptance_and_real_installments_and_survives_checkpoint`
checks wrong-party rejection, no withdrawal on offer alone, affordable acceptance,
no duplicate payment at a boundary, conservation of council cash, serialized
continuation of installments, and loss of protection after three unfunded dues.
It is a controlled mechanism test, not a long-run war-frequency calibration.
