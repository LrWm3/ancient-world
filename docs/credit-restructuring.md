# Bounded restructuring decisions

The first decision/commit API handles one extension of an existing overdue loan.
It does not supply new principal, forgive debt, change the interest rate or turn
accrued interest into principal. Automatic council/exporter negotiation remains
unfinished; this interface is the boundary those policies will use.

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
default already recorded in Open. No automatic monthly caller has been added.
A later Respond policy must state its timing rather than silently reviving an
already defaulted loan.

Archives default the new receipt collection to empty. Validation checks dates,
amounts, decision criteria, parties and the committed extension entry. Existing
low-level extensions without policy receipts remain readable. The original
contract's repayment source is unchanged, preserving underwriting reservations.

Next integration work is to construct revised evidence from actual delayed
payments or late taxes, decide voluntary acceptance using existing institutions'
interests, and feed the result through the monthly policy boundary. Closed accounts
and recovery after write-off remain separate work. Do not interpret this API alone
as an autonomous debt-workout system.

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
