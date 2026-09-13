# Council credit after loss of the tax base

A focused matched fixture exercises actual annual tax collection, credit
servicing and default. It initializes a small hardware-generated world, collects
taxes at month 12 and observes those receipts. At month 13 an explicit ten-unit
bridge loan is contracted against month-24 taxes, due in month 25 with three
months' grace and no interest.

The fixture moves existing council cash away before borrowing and transfers the
loan proceeds to another existing town account. This is an explicit test
expenditure; it does not claim to model completed administration or relief.
Collection can therefore succeed only if the council receives additional cash.

Matched branches retain or abandon the council's controlled settlements after
disbursement. Abandonment preserves their retained cash but removes the taxable
sites. At month 24 the real annual collection runs after Open servicing; month-25
collection then has access to those completed receipts. New lending and issuance
do not rescue either branch. Collection uses a full available-cash share to
isolate receipt failure from operating-reserve policy.

Assertions require:

- A positive observed tax forecast before lending, and a zero current forecast
  after loss of the controlled sites.
- Positive real tax receipts in the retained-base branch and zero in the lost-base
  branch.
- Debt still outstanding at the month-24 collection boundary.
- Repayment in the retained-base branch and default in the lost-base branch.
- No new loans or issuance, and unchanged global money residual at every tested
  boundary.
- Identical serialized continuation for both branches.

This test covers contractual consequences of a tax-base shock. It does not
demonstrate automatic loan selection, the usefulness of the original expenditure,
or population benefits. Those remain separate acceptance claims.

Verification: `cargo test --lib council_tax_base_loss_changes_collection_and_default_without_creating_cash -- --ignored`
ran one hardware fixture and passed. The world uses terrain resolution 32 and
ecology resolution 16; only the selected monthly credit and annual tax operations
advance after initialization. It is not a complete living-history experiment.
