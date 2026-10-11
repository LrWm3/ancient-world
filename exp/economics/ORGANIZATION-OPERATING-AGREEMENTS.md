# Signed organizational operating authority

The shared `agency` controller now supports existing organizations beyond the state
and household. `agency::organization::install_initial` accepts a static operating
agreement at month-one Open. Every named member explicitly signs the same terms;
the state must permit each signer's `GovernOrganization(organization_type)` action.
Installation stages and validates a candidate World before publishing anything.
It neither creates property nor changes financial reporting scopes.

The agreement distinguishes the organization, its consenting members, its initial
office holder and its allowed processes. Membership confers no ownership claim,
wage, personal capacity or automatic property access. Fixed leadership retains the
founder; rotating leadership uses a fixed, sorted roster, anchored at the founder,
and a supplied term length. A dead holder leaves that term vacant rather than
advancing the rotation or rewriting earlier authority. These positions are held
by person agents. Elections and changing membership are not implemented by this
adapter; the existing state and household adapters retain their broader behavior.

The same objective/forecast/program comparison used by states and households now
selects organization work. `WorkTargets` replace the organization's own discretionary
orders at the following Open. The ordinary reservations, legal process permission,
actual labor and input constraints still apply. A governor cannot widen the
agreement's process menu, rewrite the sealed operating objectives, direct another
agent's work or cancel accepted payroll by removing a work target. Accepted terms
and history survive forecasts and in-memory Simulation reconstruction.

This is **initial operating governance, not general founding**. Initial legal
identity, property, employment, members and unanimous consent are supplied. It is
not an API for current members to take over an existing privately owned entity;
installation is restricted to initial setup before the first monthly opening and
rejects a second controller. Ownership, ownership transfers, recruitment, formal
founding and dissolution need their own admission rules. Self-directed personal
policy changes remain outside scope.

Focused controls verify exact lawful consent with atomic rejection, constitutional
bounds versus legal permissions, rotation and vacancy, preserved decision history,
and next-month effect. Existing agency, program-discovery, law and state-governance
regressions also pass. The recurring acceptance scenario uses this adapter for a
fixed-governor workshop and a rotating municipal resource operator.

## Discovered operating and trading programs

`install_discovering_initial` applies the same consent rules to an empty program
menu. Own reserve objectives and signed productive recipes generate persistent
work targets; matching respects objective order. Signed `stock_markets` specify
market side, allowed lot prices, inventory targets/floors and a lot ceiling.
Discovery enumerates bounded portfolios (including stopping all or some markets)
and production/procurement combinations. Oversized catalogs are rejected, not
truncated. Choices depend on forecasts; the catalog depends only on static terms.

`PeerStockQuotes` replaces the organization's peer-stock portfolio next Open and
its issuer-input quotes in slots covered by the signed agreement. An issuer-input
slot supports stock sales only and uses the existing conditional procurement book;
it does not grant permission to buy inputs for the issuer or sell capacity. An
enabled peer-stock book is still required by this bounded adapter. Unmanaged
legacy input quotes, other agents' quotes and the common claim horizon remain.
Stopping a signed slot removes its quote through the same dated execution.
Constitutional authority never grants legal or venue eligibility. Law loss suppresses subsequent execution
without erasing accepted history; stopping purchases does not cancel employment.
Complete programs share the activity validator and publish atomically, so a later
failed command cannot leave an earlier schedule behind.

Declared slots across organizations, together with current quote slots, must fit
the book before authority is installed. This conservative schema prevents a future
Open from failing because separate mandates authorize incompatible resource aliases
or too many quotes. Even mutually exclusive aliases are rejected. Cash, goods and
labor remain subject to ordinary allocation; schema compatibility reserves none.

The [governed acceptance report](ACCEPTANCE-GOVERNED-MARKETS.md) records ten
implementation iterations and the working-capital objective calibration. It does
not establish general price discovery, unrestricted portfolio search, endogenous
employment or self-changing constitutions.

## Sale-linked productive alternatives

A signed stock-sale option can also root productive discovery when its payment
resource serves an organizational reserve or funding-gap objective. The catalog
uses the static union of collective objectives and supplied governor preferences.
A directly matching signed productive recipe receives a buffer target equal to the
sale floor plus the authorized number of lots times listing quantity. Arithmetic
and catalog bounds are checked; existing explicit inventory objectives keep their
own targets. Current stocks, mutable quotes and imagined demand never change the
catalog. Ordinary forecasts choose whether an alternative helps.

This is a persistent buffer, not an optimal production quantity, guaranteed sale
or automatic stop at cash satisfaction. The [acceptance experiment](ACCEPTANCE-SALE-DISCOVERY.md)
compares cold start with finite opening stock and quotes. It does not introduce
negotiated initial terms, unbounded supply-chain search or new legal authority.

## Explicit counterparty expectations

`agency::Config.peer_expectations` defaults to `AcceptedOnly`. The opt-in
`RecentSuccessfulStarts` and `RecentSuccessfulStartsAndBallots` policies freeze
observed peer evidence once per evaluated decision. Each candidate uses the same
snapshot in its isolated forecast; normal admission and settlement determine
whether hypothetical work succeeds. They do not run the peer's optimizer or
publish real peer commands. Incumbent preferences, when supplied, override
collective output-reserve goals for the recurring-work gate.

The existing organization-decision observer includes the snapshot, private ballot
admission receipts, attempted/completed work and closing stocks. The
[acceptance comparison](ACCEPTANCE-PEER-EXPECTATIONS.md) documents horizon/memory
calibration, election and resource controls, and the remaining limits. This policy
is part of the supplied sealed configuration, not a self-directed policy change.
