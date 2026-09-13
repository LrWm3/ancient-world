# Local institutional credit offers

This opt-in extension lets an operational institution offer existing surplus cash
to its home council's tax-bridge request. It addresses a missing type of lender;
it does not relax the borrower's coverage, exposure, pledged-source or default
checks. Existing councils-only policies and archives keep it disabled.

Enable it with `--council-credit --institution-credit-lenders`. Either flag accepts
`=false`; omitting a flag preserves the archived setting. Disabling institutional
offers does not cancel or stop servicing loans already made.

## Offer and operating reserve

The institution must be active and operational, at a live home site, with its
leader both a member and a locally present eligible adult under the existing
cultural presence rules. Its borrower is that site's current council. This first
version makes no remote institutional offers or assumptions about distant branches.
Foreign council offers still require the existing eligible direct route.

Protect the greater of the council credit policy's reserve floor (default 100
currency units) and the institution's annual operating target. The target is the
same administration-and-repair quote used by institutional operating funding,
including current building materials, wear, disruption and replacement prices.
Offer only the policy's surplus share (default 25%) of the cash above that reserve.
Expected donations are not funds, and the quote is not new construction capacity.

This is a simple institutional lending preference, not a negotiated internal ballot.
It does not reserve a separate named administrative work action. Preserving a
reserve at origination does not guarantee that subsequent unrelated spending will
leave it untouched. Extra discretionary projects are not automatically priced into
the annual operating target.

## Allocation and accounting

Collect eligible institutional offers with foreign council offers before resolving
the council's shared cash gap. Split that gap across eligible lenders, then use
existing joint source, borrower and lender caps. An extra lender cannot pledge
the same tax collection twice. Request IDs distinguish institutional and council
lenders even when their numeric owner IDs coincide.

Loans debit the existing institutional treasury and credit the council's existing
treasury through the mixed-precision cash adapter. The general credit ledger
already supports this account type, repayment, arrears, default, and retained
institutional estates. The institution does not gain donation income by lending;
principal and interest remain separate credit receipts.

## Comparison protocol

`python3 scripts/monetary_experiment.py --compare-institution-lenders` adds credit
and combined arms with this switch enabled alongside the four existing arms. Supply
`--checkpoint LABEL=PATH`, `--years` and an ignored `--output` directory as usual.
The four original arms explicitly disable these offers when this comparison is
selected. Report loan counts by original lender kind as well as council construction
outcomes, completed work, access, defaults and cash. No effect is an acceptable
result; it does not justify consuming the institution's operating reserve.

Verification passed: one selected GPU lending fixture, 19 active market tests,
one CLI switch test, two institutional-funding tests (including named GPU-backed
administration), seven Python reporting tests, and strict all-target Clippy. The
lending fixture checks inactive/missing-member institutions, exhausted reserve,
insufficient future receipts, conserved money, and serialized continuation with
new offers subsequently disabled. It demonstrates availability and accounting,
not an institution-financed improvement in completed council services.

The seed-1024 six-arm comparison is running separately; its balance outcome is not
yet established.
