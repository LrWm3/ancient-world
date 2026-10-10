# Finite household, state and personal circulation

Status: implemented calibration control, not a sustainable-economy claim.
`scenario::financed_circulation()` combines endogenous citizenship, household and
land agreements, state governance, loan-term search, farming, paid mint work,
public food sales and annual dues. It starts with no selected operational programs,
signed agreements, dated mint orders or counterparty quotes.

The state begins with zero coins and sixteen wheat; the independent worker has
zero coins and three wheat. The supplier retains six coins and ten metal. Two
prospective household members each have five monthly hours; their household
agreement delegates one hour each. Cultivation takes six hours, so shared labor
remains necessary. The state protects four wheat. Loan search tries four months,
then eight, at zero interest. These are explicit experimental assumptions.

The calibrated mint recipe, labor listing and independent worker each use five
hours. Household members cannot individually offer a five-hour lot from their four
private hours. The comparison restores all three quantities to two. This changes
technology, lot size and worker capacity; it does not compare allocation policies
against an identical resource pool. Prices retain the existing four-coin labor
lot and three-coin/three-wheat food lot.

The fourteen-month comparisons produce:

| Case | Repaid loan | Mint completions | Crop completions | Supplier / grower / worker nutrition deficits |
| --- | --- | --- | --- | --- |
| Five-hour calibration | 6 coins, eight months | 2 | 10 | 1 / 1 / 5 |
| Two-hour comparison | 6 coins, eight months | 2 | 10 | 1 / 1 / 11 |
| No supplier ore | 6 coins, eight months | 0 | 10 | 1 / 1 / 11 |
| No lender coins | None | 0 | 10 | 1 / 1 / 11 |

Every case discovers one household and one lease and pays its one-wheat annual
obligation in month fourteen. In the five-hour case the worker earns four coins
in each of months three and four, then buys three wheat in each of months four
and five. Opening resources fund each boundary; the first wage cannot finance a
purchase until the following month. Eight earned coins minus six spent leave two.
Three opening wheat plus six purchased wheat satisfy nine of fourteen monthly
nutrition units. The other two people retain their initial one-unit deficits.

The no-ore case deliberately retains its successful loan repayment. Public sales
and other projected objectives can make finance acceptable without any completed
minting. Repayment is evidence of financial performance, not proof that the
underwritten productive activity occurred. The no-cash control permits continued
household farming and annual dues but no loan, mint or worker food purchase.

CPU execution reconstructed at every phase agrees with uninterrupted reference
execution on world, state, ledger, reports and separately maintained double-entry
books. Tests check actual accepted trades, loan principal/term/status, a nonempty
paid annual obligation, completed processes and cash/food conservation.

Run from `exp/economics`:

```sh
cargo +1.92.0 run --release --locked --example discovered_economy -- --financed-circulation
cargo +1.92.0 test --release --locked --test discovered_circulation
```

Five worker deficits remain. Paid mint demand is finite, and ending coins do not
buy a whole food lot. The household also finishes with food that this narrow
adapter does not offer to the independent worker: its quotes cover person supply
of mint inputs and public sales of wheat, not general household/person food asks.
The opt-in [private circulation follow-up](PRIVATE-CIRCULATION.md) now covers
person/household food asks and collective bids on that listing, with matched
public-stock and lot-size controls. This original fixture retains its public-only
settings. Neither greater crop output nor successful aggregate accounting alone
establishes access to food for every person.
