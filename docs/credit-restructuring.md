# Bounded restructuring decisions

The first decision/commit API handles one extension of an existing overdue loan.
It does not supply new principal, forgive debt, change the interest rate or turn
accrued interest into principal. The opt-in commercial policy now negotiates some delayed export loans. Council
negotiation and broader recovery remain unfinished.

A proposal identifies the loan, current month, revised maturity, original repayment
source, revised expected payment month and fresh evidence. Lender and borrower
consent must each identify the corresponding existing account. These are simulation
policy decisions supplied by the caller, not authentication of external users.
The caller remains responsible for constructing evidence from actual observations;
a forecast is never credited as cash.

The resolver requires current arrears, no earlier restructuring and an extension
of at most twelve months. Expected payment must occur after the current boundary
and before revised maturity. Evidence must belong to the borrower and retain the
original source identity. A delayed cargo's new expected arrival is a separate
field, not a new contract ID or a new receivable to pledge.

Net expected receipts after operating costs must cover 1.5 times projected debt
plus competing claims on the same source. Projection includes existing principal,
accrued interest and future simple interest at the unchanged rate. Other active
loans remain reserved; extending a loan does not free its source for new lending.
Expected receipts follow the underwriting convention of already incorporating any
forecast haircut; a separate maximum loss fraction of 0.25 rejects excessive risk.
These coverage and duration limits are conservative toy policies, not fitted
financial estimates or guarantees of repayment.

`History::resolve_credit_restructuring` runs only after this month's debt servicing.
It resolves and commits once per loan/month, checks both accounts still exist,
and retains accepted and declined proposals with their opening loan, forecast,
coverage and reason. The commit takes effect at that explicit call boundary,
after this month's collection; it does not retroactively cancel a payment or a
default already recorded in Open. The commercial caller runs in Reserve after production planning and before new
commercial loans. A default already recorded in Open remains final; negotiation
can only help a still-overdue loan during its grace window.

Archives default the new receipt collection to empty. Validation checks dates,
amounts, decision criteria, parties and the committed extension entry. Existing
low-level extensions without policy receipts remain readable. The original
contract's repayment source is unchanged, preserving underwriting reservations.

## Delayed export policy

`--commercial-credit` also enables the delayed-export caller. It observes existing
buyer-funded payment escrow and remaining cargo, retains the original contract
and due-date identity, and separately forecasts the latest eligible arrival.
Current planned production input commitments remain senior to repayment. The
borrower consents when existing cash cannot cover those commitments plus debt;
the lender consents only while retaining its own input-cost and operating-cash
floor. Both decisions are recorded through the same API, not inferred afterward
from whether an extension happened. These are explicit game policies rather than
models of bargaining personalities.

Flooded or besieged approaches, lost/settled cargo, abandoned exporters and
unfinished staffing-dependent vessel voyages provide no extension evidence.
Future crew funding is not assumed. Partial losses reduce the forecast in direct
proportion to remaining cargo; future loss assumptions haircut it further. A
completed delivery disappears from the forecast and supplies actual cash through
the ordinary export-payment path. New-loan underwriting still excludes overdue
and delayed receipts: observing them for an existing loan does not create new
collateral.

This first caller does not negotiate council tax debt, replace a lost receivable
with a different year's taxes, revive defaulted loans or extend an extension.
Closure handling, numerical-residue classification and recovery remain work.

The analytical fixture checks missing consent, coverage, competing claims, source
identity, repeated extension and corrupt recorded amounts. The History fixture
uses explicit synthetic repayment evidence to isolate the financial boundary:
committing an extension must leave cash and cash-transfer receipts unchanged,
reject replay, and resume collection identically after JSON continuation. That
fixture does not establish that a live delayed-cargo observer supplies good evidence.

Verification: fifteen credit unit tests and twelve active market integration tests
passed (two extended market cases remain ignored). The new integration fixture
confirms unchanged cash, bounded extension, replay rejection, later repayment,
zero monetary residual, matching continuation and rejection of altered consent.

Delayed-export integration verification: all twelve export-contract unit cases
and twelve active market integration tests passed; strict all-target Clippy passed.
The new fixture derives evidence from funded cargo, checks partial loss and read-only
observation, rejects lost/unfinished-voyage evidence, verifies disabled policy and
lender refusal, commits once and collects after actual delivery. Extended GPU
market cases were not rerun for this sparse policy change.
