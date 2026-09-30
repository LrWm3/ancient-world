# Posted guarantee admission

Guarantee terms can now be posted without immediately binding their guarantor.
`Config.posted_guarantees` identifies those entries; older configured guarantees
remain preaccepted. `guarantee_applications` supplies an explicit guarantor-consented
date for each requested acceptance. This is a bounded admission adapter, not an
autonomous underwriter, pricing mechanism or guarantee-premium market.

The common `offers` catalog exposes eligible `Guarantee` terms. Its `prepare`
interface verifies the named guarantor, configured application, date and normal
Acquire settlement. The normal simulation uses the same admission checks and
records acceptance or rejection in the existing credit boundary. An accepted
month and a snapshot of the accepted terms are stored in the recovery book;
there is no second exposure ledger. The snapshot includes the resolved alternative
tender and conversion rate, including rates referenced from land terms. Retained
catalog terms must still match: changing a cap, duration, delay, priority,
recourse or tender after acceptance requires a new agreement. Missing or altered
snapshots fail checkpoint validation and settlement rejects forged admission
receipts atomically. Existing preconfigured, non-posted guarantees keep their
static configuration path.

A state's recognized forms can include `AgreementForm::Guarantee`, and permission
to originate is `Action::Guarantee`. Missing recognition or permission rejects
new admission. A later restriction does not erase accepted obligations. Terminal,
insolvent, closed or winding-down guarantors cannot accept new offers. Being short
of cash does not itself prohibit guaranteeing a future obligation: the cap is a
contingent promise, never proof of funding or an escrow reservation.

Admission occurs at Acquire. Calls retain their existing Due boundary and cannot
be backdated into the admission month. A guarantee accepted in its final valid
month may therefore expire without any callable boundary. Existing delay, cap,
priority, actual funding, storage, recourse and accounting rules still apply.

Household winding-down checks only accepted contingent exposure. A catalog offer
alone does not trap an organization in existence. Once accepted, the guarantee
blocks dissolution through its term; any recourse asset after performance remains
material. Household previews preserve the ordinary preparation, core settlement
and collection boundary. The applications still express supplied organizational
consent; governance does not autonomously select or price guarantees.

Inspection reports the actual acceptance date. Settlement observers expose
`guarantee_admission`, parties, denomination, cap, acceptance and rejection reason.
Origination changes no cash balance. Forged dates, changed receipts and replay
cannot publish an acceptance.

`tests/guaranteed_claims.rs` covers common discovery/preparation, legal refusal,
permission withdrawal after acceptance, delayed performance, final-month expiry,
separate statements, household wind-down, CPU/reference and checkpoint continuation.
Tests also distinguish unaccepted offers from existing agreements and reject a
checkpoint containing acceptance before its Acquire boundary.

Remaining extensions include autonomous benefit/risk assessment, paid guarantees,
negotiated terms and broader tender/security combinations. Explicit loan coin
tender, accepted land coin tender and bounded lien subrogation are covered in
[guaranteed claims](GUARANTEED-CLAIMS.md). Static
constitutions/charters and deferred person self-directed policy changes are unchanged.
