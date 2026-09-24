# Goals: agents, institutions and markets

This document records the experiment's intended direction. These are design goals,
not claims of implemented behavior or a commitment to implement everything at once.
The [integration matrix](INTEGRATION-STATUS.md) records supported combinations;
the [design](DESIGN.md) describes the transaction and process architecture.

## Current position

The [README progress summary](README.md#current-progress--2026-09-23) separates
implemented pilots from these longer-term goals. Recent consolidation shares loan,
land and forward claim execution and adds alternative-tender allocation, capped
guarantees, configured recovery with land/forward admission, and funded liquidation.
Those are
bounded mechanisms, not completed organizational governance or a universal legal
and financial system. Remaining work is ordered in
[contract consolidation](CONTRACT-CONSOLIDATION.md).

## Purpose and consistency

Rebuild economic and institutional behavior from generic agents, explicit
agreements and planning from each agent's needs, resources and opportunities.
Ancient World's confusing mixture of aggregate populations and individuals is a
problem this experiment should address. A household, firm or state is an actual
organizational agent with relationships and delegated resources. It is not a
second population that independently reproduces its members' labor, consumption,
assets or obligations. Aggregate reports are derived views.

Use small one-person and 32-person experiments to establish a consistent loop:

```text
formation agreement -> constitution -> authorized governance
                    -> charter and decision policies
                    -> allocation of resources and delegated work
                    -> feasible options using assets, rights and markets
                    -> transactions, outcomes and revised projections
```

Needs, commitments and forecasts inform action selection at every level. Keep
physical-world detail limited while establishing this structure. Existing forwards,
credit and market pilots exercise whether the primitives support useful behavior;
they do not by themselves establish that all pilots compose. Broader capabilities
from Ancient World can be recreated incrementally on this foundation.

## Distinct concepts and authority

| Concept | Meaning and intended boundary |
| --- | --- |
| Law | Rules within a jurisdiction governing recognition, founding, agreements, rights and permissible actions |
| Constitution | A founding template defining rules, parameter slots and permitted policy choices |
| Charter | The parameter values filled in for a particular constitution; static initially |
| Governance | Persons and procedures authorized to set policies within the founding rules, including succession, elections, rotation and terms |
| Decision policy | Operational objectives, priorities, horizons and allocation/tiebreak rules used to select actions |
| Membership | An agreement specifying participation, contributions, benefits, eligibility and exit obligations |
| Ownership | Claims over an organization or its assets, with transfer and distribution rights where permitted |
| Valuation | A balance sheet or forecast of value; it neither grants control nor implies something can be sold |

As a first idea, treat the constitution as a template and the charter as its
filled-in parameters. For example, a constitution might permit a monthly labor
contribution percentage, with a household's charter supplying 20%. Both remain
static after founding for now. This is a provisional distinction, not a settled
requirement for a separate charter amendment system. Revisiting charter changes
and who may authorize them is later design work.

Leadership selection and operational decision policy are separate. Leaders set
policies within the constitution and its fixed charter parameters; they do not
choose every operational action. Persons should occupy governance and policy-setting roles, including below
the state level. Owning an organization need not imply governing it, working for
it or being a member. An owner may establish dividend rules and delegate governance
to a person or member collective without working there. A household may have no
owner and still own assets, make projections and maintain a balance sheet.

Initially, commercial constitutions should come from predefined templates and
remain immutable, including after sale or transfer of ownership. A transfer must
be permitted by the constitution and the relevant marketplace. Transfer does not
change the static charter. Any future charter amendment mechanism must respect
the constitution and applicable law.

## Law, recognition and delegated institutions

Start with states defining law and organizations filling in their charters at
founding and setting policies within legal limits. Law determines which organizations may form, their founding
requirements, recognized agreement shapes, interpretation of rights and permitted
actions after founding. Different states may choose different rules:

- Retain land ownership and permit leases, or require purchase for productive use.
- Recognize households with annual rotating or elected heads, but disallow
  hereditary succession in recognized households.
- Permit generic firms with licensed activities, or require particular kinds of
  organizations for particular activities.
- Transfer estates to the state unless valid household or next-of-kin agreements
  specify otherwise, or require asset sales followed by debt payment and transfer
  of remaining proceeds to the state. Individual agreements operate within the
  chosen legal range.

This authority is not unique to states. Provinces, towns, religions and other
organizations may establish subordinate rules within their mandates. Define scope
and precedence explicitly rather than letting each layer grant unrestricted powers.
State rulemaking is the initial focus; delegation and conflicts across layers can
be developed later. Current state-provided services are acceptable scaffolding,
but should be delegable rather than intrinsic powers of a special state class.

Eventually states should also have assets, membership, constitutions, governance,
charters, objectives and goals. Autonomous state planning is a later priority than
getting lawful formation and bounded organizational decisions working.

## Marketplaces and price formation

Markets should be institutions or agents with an explicit catalog and eligibility
requirements. The first locality rule to exercise is proximity to a town containing
the market, observed at month start. Admission, legal permission to trade, and
having the resources to settle are distinct checks. Locality must not become an
implicit global market or instantaneous transport mechanism.

Eligible agents at any level should be able to submit bids and asks: persons,
households, firms, cooperatives and states. The first target is a bounded monthly
matching mechanism in which a matched bid and ask establish that month's posted
price. State-defined limits may constrain prices initially, but administered prices
should not be the only source of prices. Multiple matches, price selection, order
expiry and the no-match case need explicit rules before implementation; an old
price observation must not masquerade as a new trade.

Record dated prices, traded volume and unfilled demand/supply so future projections
can use actual market observations. Separate valuation, order generation, quote
adaptation (including optional ZIP), matching, allocation and settlement. Later
mechanisms can include fuller bid/ask books and auctions without changing the
agent interface or accounting model.

Labor can be offered and bought through this interface, with dated capacity,
provider eligibility and delivery obligations. It requires no exclusive price
mechanism. Eventually the catalog should support resources, assets, rights,
agreements and membership, including transactions currently mediated by the state.
Transferability and lawful terms remain specific to what is exchanged.

Cross-level exchange should support opportunities such as a shipbuilding cooperative
asking for a boat price and a state bidding to buy, or a posted state ship order
motivating eligible people to found a cooperative. This is a future formation
planning case, not a requirement for an unlimited planner now. Arbitrage should
be possible when admission, resources, timing and transaction costs permit it.

## Household and organizational labor agreements

Rework the representative household policy from directing only spare labor to an
explicit founding-agreement contribution: **20% of each member's available monthly
labor is directed by the household**. Determine the contribution at a dated
reservation boundary, retain its contributor and subtract it from that person's
independently spendable capacity. Multiple memberships cannot create duplicate
hours. Rounding, insufficient capacity, existing commitments and release of unused
hours require explicit policies and receipts.

The household allocates contributed work under its charter and decision policy.
Make allocation tiebreaks configurable rather than implicitly following signatory
order. Excess-labor delegation remains a legitimate alternative policy, but is no
longer the representative target for exercising constitutions and governance.
Other institutions may require a fixed hour or small dues instead of a percentage.

The household's target action scope can include actions available through its
constituents. This does not transfer every personal license to every worker:
record who performs the work, under whose authority, with which skills and rights.
Other organizations may restrict contributed hours to named member activities or
hold rights enabling work only on behalf of that organization. General permission
inheritance is therefore an organizational rule, not a universal shortcut.

For example, a workshop collective may receive four hours each from 80 members
in return for access to tools and space. Within its constitution it could obtain
a group license, produce weapons for the army, organize training, pay members for
production or dedicate one hour per member to maintenance and return the rest.
Helping Alice fulfill personal orders may be allowed while building Alice's home
is outside its mandate. Rights, materials, skills and bounded labor still constrain
all accepted work.

Membership recruitment can be a market offer and maintaining sufficient membership
can be an organizational need. Later membership negotiation may allow counteroffers
on contributions and benefits within constitutional and legal limits. Admission
and formation have requirements; falling below viability requirements has explicit
dissolution consequences. Nested organization membership should eventually support
state coalitions as well as households of persons, without duplicating resources
at either level.

## Swappable decisions and self-governance

Separate candidate search from the policy used to assess and rank candidates.
Attach decision policies to individual and organizational agents so they can differ
within one world. Preserve the same observations, feasible action checks,
reservations and settlement when comparing policies.

Examples to exercise include feeding members first, then paying or reserving their
long-term obligations and completing committed work, then maximizing net wealth;
and maximizing projected gain over six months without giving those outcomes extra
priority. An objective may take risks, but cannot bypass law, mandates or resource
constraints. Decisions and consequences should reveal those tradeoffs.

Individuals should also eventually be able to revise their own policy at explicit
intervals. A personal charter, if used, would initially remain static like an
organizational charter; charter revision and personality-driven self-governance
can come later. The initial
positive-outcome policy is adequate provided its selection, parameters, revision
boundary and history are explicit and swappable. Organizations change policies
through authorized governors; changing policy does not erase existing agreements
or retrospectively alter completed allocation.

## Death, dissolution and estates

A person's death or an organization's dissolution should enter an explicit estate
or liquidation flow. Preserve identity, assets, outstanding claims, collateral and
process obligations while legal and contractual rules determine administration,
transfer, sale and distribution. Death must not silently delete debt or create
spendable inheritance before settlement. Estate handling is still a goal for
persons and households; existing terminal-state or household closure behavior is
not a complete implementation of this flow. The implemented
[loan-estate recovery](CONTRACT-RECOVERY.md) starts from configured authorization
and observed arrears. It does not yet connect death/dissolution to estate creation,
inheritance, household administration or admission of every contractual claim.

## Incremental development and evidence

Prioritize a small integrated formation/governance/allocation/market loop over
additional physical detail. Market price formation and the household redesign are
major next directions; precise implementation order can follow bounded experiments.
The existing deprivation, planner-ablation, uncertainty and competing-offer review
items remain useful validation work within this broader direction.

Demonstrate each extension under controlled alternatives: identical openings with
different laws, constitutions, leaders or operational policies. Track requested,
reserved and completed work separately, with explanations for rejected actions,
unfilled orders and unmet needs. A finite resource must have one authoritative
owner or reservation chain even when several organizational layers plan around it.

Require accounting conservation, lawful authority, fulfillment or explicit failure
of commitments, replay and continuation consistency. Also inspect whether the
agents actually meet needs and remain viable; conservation alone does not show
that the economy works. Scale from one or a few people to 32 only after the small
case is understandable. Document unsupported combinations rather than implying
that individually tested pilots form a complete economy.

## Verification through demanding financial systems

The [verification stress-test program](VERIFICATION-STRESS-TEST.md) develops these
goals through minting, markets, forwards and lending, then increasingly demanding
financial and institutional compositions. Modern institutions are experimental
tests of the primitives, not promised Ancient World content. Alternative species
and civilizations should vary needs, knowledge, settlement and organizational
norms through definitions and policies. Each stage requires mechanical, agentic
and composition evidence rather than merely a catalog entry.
