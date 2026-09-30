//! Strict adapter: same-denomination finance, valued land and costed spot trades.
//! Unsupported positions/events fail before either simulation or reporting publishes.
use crate::{
    accounting::{self, Account, Book, Entry, Flow, Line},
    credit,
    model::*,
    recovery,
    simulation::Simulation,
};
use std::collections::BTreeMap;
type Positions = BTreeMap<(AgentId, Account), i128>;
type Flows = BTreeMap<(AgentId, Account, Flow), i128>;

/// Explicit historical reporting basis for opening a new book, not reconstructed income.
#[derive(Clone, Debug, Default)]
pub struct Opening {
    pub assets: BTreeMap<AssetId, i128>,
    /// Fixed reporting ticks per payment-stock unit for equipment and posted barter.
    pub exchange_values: BTreeMap<ResourceId, i128>,
    pub inventory: BTreeMap<crate::model::Account, i128>,
    pub processes: Option<crate::process_accounting::Costs>,
    pub dues: Option<crate::dues_accounting::Valuation>,
    pub issuance: Option<crate::issuance_accounting::Policy>,
    /// Opt into actual-use capitalization of paid current-period capacity.
    pub services: Option<crate::service_accounting::Costs>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Audit {
    book: Book,
    exchange_values: BTreeMap<ResourceId, i128>,
    boundary: State,
    asset_values: BTreeMap<AssetId, i128>,
    inventory: crate::inventory_accounting::Inventory,
    processes: Option<crate::process_accounting::Costs>,
    dues: Option<crate::dues_accounting::Valuation>,
    issuance: Option<crate::issuance_accounting::Policy>,
    services: Option<crate::service_accounting::Costs>,
}
#[allow(clippy::too_many_arguments)]
fn positions(
    world: &World,
    state: &State,
    coin: ResourceId,
    values: &BTreeMap<AssetId, i128>,
    inventory: &crate::inventory_accounting::Inventory,
    processes: Option<&crate::process_accounting::Costs>,
    dues: Option<&crate::dues_accounting::Valuation>,
    services: Option<&crate::service_accounting::Costs>,
    exchange_values: &BTreeMap<ResourceId, i128>,
) -> Result<Positions, String> {
    if !world
        .resources
        .iter()
        .any(|r| r.id == coin && r.kind == ResourceKind::Stock)
        || (!state.obligations.is_empty() && dues.is_none())
    {
        return Err(
            "financial adapter requires supported positions; land dues require explicit valuation"
                .into(),
        );
    }
    inventory.validate(world, state, coin)?;
    crate::employment::validate(world, state)?;
    for terms in &world.employment {
        crate::reporting_value::value(coin, exchange_values, terms.wage_per_unit.resource, 1)?;
    }
    let mut p = services
        .map(|c| c.positions(world, state))
        .transpose()?
        .unwrap_or_default();
    for (&(agent, r), &q) in &state.balances {
        if q == 0 {
            continue;
        }
        if r == coin {
            accounting::add(&mut p, (agent, Account::Cash), i128::from(q))?;
        } else if world
            .resources
            .iter()
            .any(|x| x.id == r && x.kind == ResourceKind::Stock)
        {
            let h = inventory
                .0
                .get(&(agent, r))
                .ok_or("missing inventory basis")?;
            accounting::add(&mut p, (agent, Account::Inventory(r)), h.cost)?;
        }
    }
    if world.credit.as_ref().is_some_and(|c| {
        c.offers
            .iter()
            .any(|o| o.sale.price.resource != coin || o.loan.denomination != coin)
    }) {
        return Err("mixed asset purchase denominations".into());
    }
    for asset in &world.assets {
        let initial = *values
            .get(&asset.id)
            .ok_or("missing explicit tangible valuation")?;
        let value = state
            .credit
            .values
            .get(&asset.id)
            .map_or(initial, |v| i128::from(*v));
        if initial < 0 || value < 0 {
            return Err("negative tangible valuation".into());
        }
        let owner = state
            .credit
            .loans
            .values()
            .find(|l| {
                l.status == credit::Status::PendingSale
                    && l.collateral.as_ref().is_some_and(|c| c.asset == asset.id)
            })
            .map(|l| l.debtor)
            .or_else(|| credit::owner(world, state, asset.id))
            .ok_or("missing asset owner")?;
        accounting::add(&mut p, (owner, Account::Tangible(asset.id)), value)?;
    }
    for asset in state.equipment.values() {
        let value = *values
            .get(&asset.id)
            .ok_or("missing equipment carrying cost")?;
        if value < 0 || (asset.remaining_uses == 0 && value != 0) {
            return Err("invalid equipment carrying cost".into());
        }
        accounting::add(&mut p, (asset.owner, Account::Tangible(asset.id)), value)?;
    }
    // Retirement either expenses basis or transfers it into recovered stock.
    // Never retain a second carrying value in the provenance archive.
    if state
        .retired_equipment
        .keys()
        .any(|id| values.get(id).is_some_and(|v| *v != 0))
    {
        return Err("retired equipment has nonzero carrying cost".into());
    }
    // This bounded assignment adapter trades whole claims at their fixed reporting
    // value. A different market price needs explicit acquisition-basis accounting.
    for listing in &world.recovery.receivable_listings {
        let resource = state
            .credit
            .loans
            .get(&listing.loan)
            .map(|l| l.denomination)
            .or_else(|| {
                crate::credit::offered_loan(world, listing.loan).map(|a| a.terms.denomination)
            })
            .ok_or("missing listed receivable terms")?;
        if crate::reporting_value::value(coin, exchange_values, resource, 1)?
            != i128::from(listing.coins_per_unit)
        {
            return Err("receivable unit quote must equal fixed reporting claim value".into());
        }
    }
    for l in state.credit.loans.values() {
        if l.denomination != coin && l.collateral.is_some() {
            return Err("noncash collateral needs explicit recovery valuation".into());
        }
        for (agent, a, q) in [
            (l.creditor, Account::LoanReceivable(l.id), l.principal),
            (l.creditor, Account::InterestReceivable(l.id), l.interest),
            (l.debtor, Account::LoanPayable(l.id), -l.principal),
            (l.debtor, Account::InterestPayable(l.id), -l.interest),
        ] {
            accounting::add(
                &mut p,
                (agent, a),
                crate::reporting_value::value(coin, exchange_values, l.denomination, q)?,
            )?;
        }
    }
    for g in &world.recovery.guarantees {
        if recovery::admission::accepted_month(world, &state.credit, g).is_none() {
            continue;
        }
        if let recovery::GuaranteedClaim::Land { agreement, .. } = g.claim {
            let (_, _, denomination) = g
                .claim
                .parties(world)
                .ok_or("missing land guarantee terms")?;
            if denomination != coin {
                let a = world
                    .agreements
                    .iter()
                    .chain(&world.access_offers)
                    .find(|a| a.id == agreement)
                    .ok_or("missing guaranteed land agreement")?;
                let dues = dues.ok_or("physical land guarantee needs dues valuation")?;
                if dues.unit(a, coin)?
                    != crate::reporting_value::value(coin, exchange_values, denomination, 1)?
                {
                    return Err("land guarantee and recourse valuations must agree".into());
                }
            }
        }
    }
    for terms in &world.recovery.proceedings {
        if terms.denomination != coin {
            return Err("mixed estate denominations".into());
        }
        let q = i128::from(
            state
                .credit
                .recovery
                .proceedings
                .get(&terms.id)
                .map_or(0, |c| c.cash),
        );
        accounting::add(&mut p, (terms.estate, Account::Cash), -q)?;
        accounting::add(&mut p, (terms.estate, Account::CustodyCash(terms.id)), q)?;
        accounting::add(
            &mut p,
            (terms.estate, Account::CustodyPayable(terms.id)),
            -q,
        )?;
        accounting::add(&mut p, (terms.debtor, Account::RestrictedCash(terms.id)), q)?;
    }
    if let Some(costs) = processes {
        costs.validate(state)?;
        for (id, (owner, cost)) in &costs.work {
            accounting::add(&mut p, (*owner, Account::WorkInProgress(*id)), *cost)?;
        }
    }
    if let Some(dues) = dues {
        for (key, value) in dues.positions(world, state, coin)? {
            accounting::add(&mut p, key, value)?;
        }
    }
    for (key, value) in crate::forward_accounting::positions(state, coin)? {
        accounting::add(&mut p, key, value)?;
    }
    for (&(id, month), earned) in &state.employment.earned {
        let q = crate::reporting_value::value(
            coin,
            exchange_values,
            earned.claim.transfer.amount.resource,
            earned.claim.outstanding(),
        )?;
        accounting::add(
            &mut p,
            (
                earned.claim.transfer.to,
                Account::WagesReceivable(id, month),
            ),
            q,
        )?;
        accounting::add(
            &mut p,
            (earned.claim.transfer.from, Account::WagesPayable(id, month)),
            -q,
        )?;
    }
    p.retain(|_, v| *v != 0);
    Ok(p)
}
fn flow(
    flows: &mut Flows,
    agent: AgentId,
    account: Account,
    kind: Flow,
    amount: i128,
) -> Result<(), String> {
    accounting::add(flows, (agent, account, kind), amount)
}
fn result(lines: &mut Vec<Line>, agent: AgentId, account: Account, debit: i128) {
    if debit != 0 {
        lines.push(Line {
            agent,
            account,
            debit,
            flow: None,
        });
    }
}
fn disposal(
    lines: &mut Vec<Line>,
    opening: &Positions,
    seller: AgentId,
    asset: AssetId,
    price: i32,
) -> Result<(), String> {
    let basis = opening
        .get(&(seller, Account::Tangible(asset)))
        .copied()
        .unwrap_or(0);
    let gain = i128::from(price)
        .checked_sub(basis)
        .ok_or("disposal overflow")?;
    result(
        lines,
        seller,
        if gain >= 0 {
            Account::DisposalGain
        } else {
            Account::DisposalLoss
        },
        -gain,
    );
    Ok(())
}
impl Audit {
    pub fn new(world: &World, state: &State, denomination: ResourceId) -> Result<Self, String> {
        Self::with_assets(world, state, denomination, BTreeMap::new())
    }
    pub fn with_assets(
        world: &World,
        state: &State,
        denomination: ResourceId,
        asset_values: BTreeMap<AssetId, i128>,
    ) -> Result<Self, String> {
        Self::with_inventory(world, state, denomination, asset_values, BTreeMap::new())
    }
    /// Opening total carrying costs, not unit quotes, for each noncash stock holding.
    pub fn with_inventory(
        world: &World,
        state: &State,
        denomination: ResourceId,
        asset_values: BTreeMap<AssetId, i128>,
        inventory_costs: BTreeMap<crate::model::Account, i128>,
    ) -> Result<Self, String> {
        Self::open_valued(
            world,
            state,
            denomination,
            asset_values,
            inventory_costs,
            None,
        )
    }
    /// Recognize dated dues at fixed reporting ticks per native unit (coins use 1).
    pub fn with_dues(
        world: &World,
        state: &State,
        denomination: ResourceId,
        asset_values: BTreeMap<AssetId, i128>,
        inventory_costs: BTreeMap<crate::model::Account, i128>,
        unit_values: BTreeMap<u32, i128>,
    ) -> Result<Self, String> {
        if unit_values.values().any(|v| *v <= 0) {
            return Err("dues unit values must be positive".into());
        }
        Self::open_valued(
            world,
            state,
            denomination,
            asset_values,
            inventory_costs,
            Some(crate::dues_accounting::Valuation(unit_values)),
        )
    }
    fn open_valued(
        world: &World,
        state: &State,
        denomination: ResourceId,
        asset_values: BTreeMap<AssetId, i128>,
        inventory_costs: BTreeMap<crate::model::Account, i128>,
        dues: Option<crate::dues_accounting::Valuation>,
    ) -> Result<Self, String> {
        Self::with_opening(
            world,
            state,
            denomination,
            Opening {
                assets: asset_values,
                inventory: inventory_costs,
                dues,
                ..Opening::default()
            },
        )
    }
    /// Open with explicit carrying costs for every active process, including zero cost.
    pub fn with_opening(
        world: &World,
        state: &State,
        denomination: ResourceId,
        opening: Opening,
    ) -> Result<Self, String> {
        let Opening {
            assets: asset_values,
            exchange_values,
            inventory: inventory_costs,
            processes,
            dues,
            issuance,
            services,
        } = opening;
        if dues.as_ref().is_some_and(|d| d.0.values().any(|v| *v <= 0)) {
            return Err("dues unit values must be positive".into());
        }
        if exchange_values.iter().any(|(resource, value)| {
            *value <= 0
                || *resource == denomination
                || !world
                    .resources
                    .iter()
                    .any(|r| r.id == *resource && r.kind == ResourceKind::Stock)
        }) {
            return Err(
                "exchange values must price noncash stocks in positive reporting ticks".into(),
            );
        }
        if processes
            .as_ref()
            .and_then(|c| c.earned_royalty_values.as_ref())
            .is_some_and(|prices| {
                prices.iter().any(|(resource, value)| {
                    *value <= 0
                        || *resource == denomination
                        || !world
                            .resources
                            .iter()
                            .any(|r| r.id == *resource && r.kind == ResourceKind::Stock)
                })
            })
        {
            return Err(
                "royalty values must price noncash stocks in positive reporting ticks".into(),
            );
        }
        crate::settlement::validate_world(world, state)?;
        let inventory = crate::inventory_accounting::Inventory::open(
            world,
            state,
            denomination,
            inventory_costs,
        )?;
        let active: std::collections::BTreeSet<_> = state
            .processes
            .values()
            .filter(|p| p.status == Status::Active)
            .map(|p| p.id)
            .collect();
        let costed: std::collections::BTreeSet<_> = processes
            .as_ref()
            .map(|c| c.work.keys().copied().collect())
            .unwrap_or_default();
        if active != costed {
            return Err(
                "opening active work needs historical carrying costs for every active process"
                    .into(),
            );
        }
        if state.phase != Phase::Open {
            return Err("open a reporting book at a month opening".into());
        }
        let mut audit = Self {
            exchange_values: exchange_values.clone(),
            book: Book::open_at(
                denomination,
                state.month.checked_sub(1).ok_or("invalid opening month")?,
                positions(
                    world,
                    state,
                    denomination,
                    &asset_values,
                    &inventory,
                    processes.as_ref(),
                    dues.as_ref(),
                    services.as_ref(),
                    &exchange_values,
                )?,
            )?,
            asset_values,
            inventory,
            processes,
            services,
            issuance,
            dues,
            boundary: state.clone(),
        };
        if let Some(costs) = &audit.processes {
            let weights = costs.output_weights.clone();
            audit = audit.with_output_cost_policy(world, weights)?;
        }
        Ok(audit)
    }
    /// Opt into material-cost production; shares apply to total joint-output cost.
    pub fn with_processes(
        world: &World,
        state: &State,
        denomination: ResourceId,
        asset_values: BTreeMap<AssetId, i128>,
        inventory_costs: BTreeMap<crate::model::Account, i128>,
        output_weights: BTreeMap<DefinitionId, BTreeMap<ResourceId, u32>>,
    ) -> Result<Self, String> {
        Self::with_inventory(world, state, denomination, asset_values, inventory_costs)?
            .with_process_policy(world, output_weights)
    }
    /// Compatibility entry point for stock-only cost shares.
    pub fn with_process_policy(
        self,
        world: &World,
        output_weights: BTreeMap<DefinitionId, BTreeMap<ResourceId, u32>>,
    ) -> Result<Self, String> {
        use crate::process_accounting::Output;
        self.with_output_cost_policy(
            world,
            output_weights
                .into_iter()
                .map(|(id, weights)| {
                    (
                        id,
                        weights
                            .into_iter()
                            .map(|(r, w)| (Output::Stock(r), w))
                            .collect(),
                    )
                })
                .collect(),
        )
    }
    /// Choose total cost shares for stock and durable products at reporting opening.
    /// A single output receives all cost; joint outputs require explicit positive shares.
    pub fn with_output_cost_policy(
        mut self,
        world: &World,
        output_weights: BTreeMap<DefinitionId, BTreeMap<crate::process_accounting::Output, u32>>,
    ) -> Result<Self, String> {
        use crate::process_accounting::Output;
        if self.book.entries().len() != 1 {
            return Err("process policy must be chosen at reporting opening".into());
        }
        for (id, weights) in &output_weights {
            let d = world
                .definitions
                .iter()
                .find(|d| d.id == *id)
                .ok_or("unknown cost-allocation definition")?;
            let mut outputs: std::collections::BTreeSet<_> = d
                .outputs
                .iter()
                .filter(|a| {
                    world
                        .resources
                        .iter()
                        .any(|r| r.id == a.resource && r.kind == ResourceKind::Stock)
                })
                .map(|a| Output::Stock(a.resource))
                .collect();
            if let Some(crate::activities::Outcome::Create(kind)) =
                world.activities.outcomes.get(id)
            {
                outputs.insert(Output::Durable(*kind));
            }
            if d.execution != Execution::Productive
                || weights.is_empty()
                || weights.values().any(|w| *w == 0)
                || weights
                    .keys()
                    .copied()
                    .collect::<std::collections::BTreeSet<_>>()
                    != outputs
            {
                return Err("invalid output cost shares".into());
            }
        }
        let work = self
            .processes
            .as_ref()
            .map(|c| c.work.clone())
            .unwrap_or_default();
        self.processes = Some(crate::process_accounting::Costs {
            output_weights,
            work,
            beneficiary_policy: self.processes.as_ref().and_then(|c| c.beneficiary_policy),
            earned_royalty_values: self
                .processes
                .as_ref()
                .and_then(|c| c.earned_royalty_values.clone()),
        });
        Ok(self)
    }
    /// Choose the issuer convention at opening; absence remains a strict rejection.
    pub fn with_issuance_policy(
        mut self,
        policy: crate::issuance_accounting::Policy,
    ) -> Result<Self, String> {
        if self.book.entries().len() != 1 {
            return Err("issuance policy must be chosen at reporting opening".into());
        }
        self.issuance = Some(policy);
        Ok(self)
    }
    pub fn book(&self) -> &Book {
        &self.book
    }
    /// Finalize only months whose Close boundary has already committed.
    pub fn finalize_through(&mut self, month: u32) -> Result<(), String> {
        if month >= self.boundary.month {
            return Err("cannot finalize an incomplete simulation month".into());
        }
        self.book.finalize_through(month)
    }
    /// Atomic composed execution: accounting failure publishes neither side.
    pub fn step(&mut self, sim: &mut Simulation) -> Result<(), String> {
        let mut next = sim.clone();
        next.step()?;
        self.record(
            &sim.world,
            &sim.state,
            next.ledger.last().ok_or("missing committed batch")?,
            &next.state,
        )?;
        *sim = next;
        Ok(())
    }
    pub fn record(
        &mut self,
        world: &World,
        before: &State,
        batch: &Batch,
        after: &State,
    ) -> Result<(), String> {
        if *before != self.boundary {
            return Err("accounting checkpoint/boundary mismatch".into());
        }
        let outer_before = before;
        let outer_after = after;
        let (prepared, core_settled, verified) = if world.households.is_empty() {
            let mut verified = before.clone();
            crate::settlement::commit(
                world,
                &mut verified,
                batch,
                crate::compute::Backend::Reference,
                crate::settlement::DEFAULT_EFFECT_LIMIT,
            )?;
            (before.clone(), verified.clone(), verified)
        } else {
            crate::households::settled_boundaries(
                world,
                before,
                batch,
                crate::compute::Backend::Reference,
                crate::settlement::DEFAULT_EFFECT_LIMIT,
            )?
        };
        if verified != *after {
            return Err("accounting requires the exact committed state".into());
        }
        let (allocated_services, capacity_allocation_lines) = if let Some(costs) = &self.services {
            let (next, lines) = costs.allocate(
                world,
                outer_before,
                batch
                    .household
                    .as_ref()
                    .map_or(&[][..], |h| h.before.as_slice()),
            )?;
            (Some(next), lines)
        } else {
            (None, vec![])
        };
        let before = &prepared;
        let after = &core_settled;
        let (mut allocated_inventory, allocation_lines) = self.inventory.pool(
            world,
            batch
                .household
                .as_ref()
                .map_or(&[][..], |h| h.before.as_slice()),
            self.book.denomination(),
        )?;
        if batch.transactions.iter().any(|t| {
            (t.process.is_some() && self.processes.is_none())
                || (t.royalty.is_some()
                    && self
                        .processes
                        .as_ref()
                        .is_none_or(|c| c.earned_royalty_values.is_none()))
        }) || (batch.minting.is_some() && self.issuance.is_none())
        {
            return Err("transaction needs an explicit accounting adapter".into());
        }
        let mut asset_values = self.asset_values.clone();
        // Durable estate sales change the same physical object, preserving wear.
        // The buyer's actual price becomes its basis for subsequent depreciation.
        for r in batch.credit.iter().flat_map(|c| &c.recovery) {
            if let recovery::Receipt::Sold {
                asset, proceeds, ..
            } = r
                && before.equipment.contains_key(asset)
            {
                asset_values.insert(*asset, i128::from(*proceeds));
            }
        }
        // Household disposal precedes this Open's scheduled depreciation. The
        // buyer's acquired cost is therefore the basis for subsequent decay/use.
        for r in batch
            .household
            .iter()
            .flat_map(|h| &h.disposals)
            .filter(|r| r.rejection.is_none())
        {
            if r.sale.price.resource != self.book.denomination() {
                return Err("asset disposal needs reporting-denomination payment".into());
            }
            for (asset, value) in r.sale.values()? {
                if before.equipment.contains_key(&asset) {
                    asset_values.insert(asset, i128::from(value));
                }
            }
        }
        let mut equipment_lines = Vec::new();
        for r in batch
            .household
            .iter()
            .flat_map(|h| &h.retirements)
            .filter(|r| r.rejection.is_none())
        {
            let basis = *asset_values
                .get(&r.request.asset)
                .ok_or("missing retired equipment basis")?;
            asset_values.insert(r.request.asset, 0);
            if r.request.mode == crate::households::retirement::Mode::Recover {
                let mut outputs =
                    world.activities.salvage[&r.equipment.as_ref().unwrap().kind].clone();
                outputs.sort_by_key(|a| a.resource);
                let mut units: i128 = outputs.iter().map(|a| i128::from(a.quantity)).sum();
                let mut remaining = basis;
                for a in outputs {
                    if a.resource == self.book.denomination() {
                        return Err("salvage cannot issue reporting currency".into());
                    }
                    let cost = remaining
                        .checked_mul(i128::from(a.quantity))
                        .ok_or("salvage valuation overflow")?
                        / units;
                    units -= i128::from(a.quantity);
                    remaining -= cost;
                    let h = allocated_inventory
                        .0
                        .entry((r.household, a.resource))
                        .or_insert(crate::inventory_accounting::Holding {
                            quantity: 0,
                            cost: 0,
                        });
                    h.quantity = h
                        .quantity
                        .checked_add(a.quantity)
                        .ok_or("salvage quantity overflow")?;
                    h.cost = h.cost.checked_add(cost).ok_or("salvage basis overflow")?;
                }
            } else {
                result(
                    &mut equipment_lines,
                    r.household,
                    Account::DisposalLoss,
                    basis,
                );
            }
        }
        let mut barter_deliveries = vec![];
        let mut production_costs = BTreeMap::new();
        let mut equipment_state = before.clone();
        if batch.phase == Phase::Open {
            crate::activities::age(world, &mut equipment_state);
            for (id, old) in &before.equipment {
                let spent = old.remaining_uses - equipment_state.equipment[id].remaining_uses;
                if spent != 0 {
                    let basis = *asset_values.get(id).ok_or("missing equipment basis")?;
                    let cost = basis
                        .checked_mul(i128::from(spent))
                        .ok_or("equipment decay overflow")?
                        / i128::from(old.remaining_uses);
                    asset_values.insert(*id, basis - cost);
                    result(&mut equipment_lines, old.owner, Account::Depreciation, cost);
                }
            }
        }
        for t in &batch.transactions {
            let purchase = if let Some(trade) = &t.trade {
                let offer = world
                    .offers
                    .iter()
                    .find(|o| o.id == trade.offer)
                    .ok_or("missing equipment offer")?;
                Some((offer.asset, offer.seller, trade.buyer, &offer.price, None))
            } else if let Some(delivery) = &t.delivery {
                if delivery.purchase.is_none() {
                    if self
                        .processes
                        .as_ref()
                        .is_none_or(|c| c.earned_royalty_values.is_none())
                    {
                        return Err(
                            "royalty equipment delivery needs an explicit earned-only policy"
                                .into(),
                        );
                    }
                    let basis = *asset_values
                        .get(&delivery.asset)
                        .ok_or("missing equipment basis")?;
                    result(
                        &mut equipment_lines,
                        delivery.provider,
                        Account::CostOfSales,
                        basis,
                    );
                    asset_values.insert(delivery.asset, 0);
                }
                delivery.purchase.as_ref().map(|purchase| {
                    (
                        delivery.asset,
                        delivery.provider,
                        delivery.buyer,
                        &purchase.price,
                        purchase.advance.as_ref(),
                    )
                })
            } else {
                None
            };
            if let Some((asset, seller, buyer, amount, advance)) = purchase {
                let barter = amount.resource != self.book.denomination();
                let unit_value = if barter {
                    *self
                        .exchange_values
                        .get(&amount.resource)
                        .ok_or("equipment barter needs an explicit exchange value")?
                } else {
                    1
                };
                let basis = *asset_values.get(&asset).ok_or("missing equipment basis")?;
                let price = i128::from(amount.quantity)
                    .checked_mul(unit_value)
                    .ok_or("equipment consideration overflow")?;
                if barter {
                    if advance.is_some() {
                        return Err(
                            "noncash financed equipment requires matching forward valuation".into(),
                        );
                    }
                    barter_deliveries.push(crate::inventory_accounting::PrepaidSale {
                        seller: buyer,
                        buyer: seller,
                        resource: amount.resource,
                        quantity: amount.quantity,
                        value: price,
                    });
                }
                let financed = advance.map_or(0, |c| i128::from(c.advance.quantity));
                let gain = price
                    .checked_sub(basis)
                    .ok_or("equipment disposal overflow")?;
                result(
                    &mut equipment_lines,
                    seller,
                    if gain >= 0 {
                        Account::DisposalGain
                    } else {
                        Account::DisposalLoss
                    },
                    -gain,
                );
                for (agent, debit) in [(buyer, -(price - financed)), (seller, price)]
                    .into_iter()
                    .filter(|_| !barter)
                {
                    equipment_lines.push(Line {
                        agent,
                        account: Account::Cash,
                        debit,
                        flow: Some(Flow::Investing),
                    });
                }
                if let Some(c) = advance {
                    // Supplier is paid directly: no fictitious cash receipt by buyer.
                    equipment_lines.push(Line {
                        agent: c.creditor,
                        account: Account::Cash,
                        debit: -financed,
                        flow: Some(Flow::Operating),
                    });
                }
                asset_values.insert(asset, price);
            }
            // Reconstruct the same equipment binding boundary used by settlement.
            if t.trade.is_some() {
                crate::equipment::apply_trade(world, &mut equipment_state, t)?;
            }
            let mut uses = BTreeMap::new();
            if let Some(id) = t.technique_use.as_ref().and_then(|u| u.asset) {
                let old = equipment_state.equipment[&id].remaining_uses;
                crate::equipment::apply_use(world, before, &mut equipment_state, t)?;
                uses.insert(
                    id,
                    (old, old - equipment_state.equipment[&id].remaining_uses),
                );
            }
            if let Some(change) = &t.process {
                if change.after.status != Status::Aborted
                    && world
                        .activities
                        .required
                        .contains_key(&change.after.definition)
                {
                    let ids = crate::activities::bindings(
                        world,
                        &equipment_state,
                        &change.after,
                        &BTreeMap::new(),
                    )?;
                    let id = ids[0];
                    uses.insert(id, (equipment_state.equipment[&id].remaining_uses, 1));
                }
                crate::activities::apply(world, &mut equipment_state, change)?;
                for (id, (old, spent)) in uses {
                    if spent == 0 {
                        continue;
                    }
                    let basis = *asset_values.get(&id).ok_or("missing equipment basis")?;
                    let cost = basis
                        .checked_mul(i128::from(spent))
                        .ok_or("equipment wear overflow")?
                        .checked_div(i128::from(old))
                        .ok_or("cannot depreciate exhausted equipment")?;
                    accounting::add(&mut production_costs, change.after.id, cost)?;
                    asset_values.insert(id, basis - cost);
                }
            }
        }

        let coin = self.book.denomination();
        let mint_transactions = batch
            .minting
            .as_ref()
            .map(|b| b.transactions.as_slice())
            .unwrap_or(&[]);
        let service_transactions: Vec<_> = mint_transactions
            .iter()
            .filter(|t| {
                t.effects.iter().any(|e| {
                    e.delta < 0
                        && world
                            .resources
                            .iter()
                            .any(|r| r.id == e.account.1 && r.kind == ResourceKind::Capacity)
                })
            })
            .collect();
        let (mut services, mut service_lines) = if let Some(costs) = &allocated_services {
            let (costs, lines, work) =
                costs.settle(world, before, batch, &service_transactions, coin)?;
            for (id, cost) in work {
                accounting::add(&mut production_costs, id, cost)?;
            }
            (Some(costs), lines)
        } else {
            let mut lines = vec![];
            for t in &service_transactions {
                lines.extend(crate::issuance_accounting::services(t, coin)?);
            }
            (None, lines)
        };
        let mut wage_deliveries = vec![];
        if let Some(b) = &batch.employment {
            for r in &b.receipts {
                let t = world
                    .employment
                    .iter()
                    .find(|t| t.id == r.agreement)
                    .ok_or("missing employment terms")?;
                if r.earned > 0 {
                    let value = crate::reporting_value::value(
                        coin,
                        &self.exchange_values,
                        t.wage_per_unit.resource,
                        r.earned,
                    )?;
                    service_lines.push(Line {
                        agent: t.worker,
                        account: Account::ServiceIncome,
                        debit: -value,
                        flow: None,
                    });
                    if let Some(costs) = &mut services {
                        accounting::add(
                            &mut costs.balances,
                            (t.employer, t.capacity.resource),
                            value,
                        )?;
                    } else {
                        service_lines.push(Line {
                            agent: t.employer,
                            account: Account::ServiceExpense,
                            debit: value,
                            flow: None,
                        });
                    }
                }
                if r.paid > 0 && t.wage_per_unit.resource != coin {
                    wage_deliveries.push(crate::inventory_accounting::PrepaidSale {
                        seller: t.employer,
                        buyer: t.worker,
                        resource: t.wage_per_unit.resource,
                        quantity: r.paid,
                        value: crate::reporting_value::value(
                            coin,
                            &self.exchange_values,
                            t.wage_per_unit.resource,
                            r.paid,
                        )?,
                    });
                } else if r.paid > 0 {
                    for (agent, sign) in [(t.employer, -1), (t.worker, 1)] {
                        service_lines.push(Line {
                            agent,
                            account: Account::Cash,
                            debit: sign * i128::from(r.paid),
                            flow: Some(Flow::Operating),
                        });
                    }
                }
            }
        }
        let negotiated = crate::negotiation::transactions(world, before, &batch.negotiation)?;
        let town_trades = match &batch.town_market {
            Some(crate::town_market::Boundary::Market(round)) => round.transactions.as_slice(),
            _ => &[],
        };
        let expired = if batch.phase == Phase::Open {
            crate::activities::expiration(world, before)
        } else {
            vec![]
        };
        let (opening_inventory, expiration_lines) = allocated_inventory.expire(&expired)?;
        let regenerated = if batch.phase == Phase::Open {
            crate::pools::regeneration(world, before)
        } else {
            vec![]
        };
        let opening_inventory = opening_inventory.add_uncosted(&regenerated, coin)?;

        let trades: Vec<_> = batch
            .transactions
            .iter()
            .filter(|t| {
                t.stock_trade.is_some()
                    || town_trades.contains(t)
                    || negotiated.contains(t)
                    || (mint_transactions.contains(t)
                        && !t.effects.iter().any(|e| {
                            world
                                .resources
                                .iter()
                                .any(|r| r.id == e.account.1 && r.kind == ResourceKind::Capacity)
                        }))
            })
            .collect();
        let mut cash_trades = vec![];
        for t in &trades {
            if t.effects
                .iter()
                .any(|e| e.account.1 == coin && e.delta != 0)
            {
                cash_trades.push(*t);
                continue;
            }
            // Venue terms identify the payment stock; actual committed effects
            // determine consideration. Quote attempts and posted prices are not sales.
            let payment = if let Some(trade) = &t.stock_trade {
                let bid = world
                    .bids
                    .iter()
                    .find(|b| b.id == trade.bid)
                    .ok_or("missing barter bid")?;
                crate::currency::terms(world, bid, trade.seller)?
                    .payment
                    .resource
            } else if negotiated.contains(t) {
                world
                    .negotiation
                    .as_ref()
                    .ok_or("missing barter negotiation")?
                    .payment
            } else if town_trades.contains(t) {
                let config = world
                    .town_market
                    .as_ref()
                    .ok_or("missing barter town market")?;
                // Town validation requires every listing to use this common payment.
                crate::marketplace::venue(world, config.venue)
                    .and_then(|v| v.markets.iter().find(|m| m.id == config.market))
                    .ok_or("missing barter town listing")?
                    .payment
            } else {
                return Err("noncash exchange needs supported payment terms".into());
            };
            let unit = *self
                .exchange_values
                .get(&payment)
                .ok_or("barter payment needs an explicit exchange value")?;
            barter_deliveries.extend(crate::inventory_accounting::barter(
                world, t, payment, unit,
            )?);
        }
        let (mut prepaid, forward_lines) =
            crate::forward_accounting::settle(before, after, batch, coin, &self.exchange_values)?;
        prepaid.extend(barter_deliveries);
        prepaid.extend(wage_deliveries);
        let mut inventory_sales = Vec::new();
        for receipt in batch.credit.iter().flat_map(|c| &c.recovery) {
            if let recovery::Receipt::InventorySold {
                proceeding,
                listing,
                bid,
                buyer,
                proceeds,
            } = receipt
            {
                let p = world
                    .recovery
                    .proceedings
                    .iter()
                    .find(|p| p.id == *proceeding)
                    .ok_or("missing inventory estate")?;
                let l = world
                    .recovery
                    .inventory_listings
                    .iter()
                    .find(|l| l.id == *listing)
                    .ok_or("missing inventory lot")?;
                if p.denomination != coin {
                    return Err(
                        "inventory liquidation needs reporting-denomination proceeds".into(),
                    );
                }
                prepaid.push(crate::inventory_accounting::PrepaidSale {
                    seller: p.debtor,
                    buyer: *buyer,
                    resource: l.goods.resource,
                    quantity: l.goods.quantity,
                    value: i128::from(*proceeds),
                });
                inventory_sales.push(recovery::inventory::sale_transaction(world, *bid)?);
            }
        }
        let mut allocation = crate::inventory_accounting::CostAllocation::new(&opening_inventory);
        let (inventory, trade_lines) =
            opening_inventory.settle_allocated(&cash_trades, coin, &prepaid, &mut allocation)?;
        let (inventory, loan_lines, loan_transfers) = crate::loan_accounting::settle(
            world,
            batch.credit.as_ref(),
            before,
            coin,
            &self.exchange_values,
            &inventory,
            &mut allocation,
        )?;
        let (cost_transactions, issuance_lines) = if self.issuance.is_some() {
            crate::issuance_accounting::processes(world, &batch.transactions, coin)?
        } else {
            (batch.transactions.clone(), vec![])
        };
        let process_transactions: Vec<_> = cost_transactions
            .iter()
            .filter(|t| t.process.is_some())
            .collect();
        let attachments: Vec<_> = batch
            .household
            .iter()
            .flat_map(|h| &h.disposals)
            .flat_map(|r| r.processes.iter().cloned())
            .chain(
                batch
                    .credit
                    .iter()
                    .flat_map(|c| c.attachments.iter().cloned()),
            )
            .collect();
        let (transferred_costs, attachment_lines) = if let Some(costs) = &self.processes {
            let (costs, lines) = costs.transfer_attachments(&attachments)?;
            (Some(costs), lines)
        } else {
            (None, vec![])
        };
        let (processes, inventory, process_lines) = if let Some(costs) = &transferred_costs {
            let (costs, inventory, lines) = costs.settle_allocated(
                world,
                &inventory,
                &process_transactions,
                coin,
                &production_costs,
                &mut asset_values,
                &mut allocation,
            )?;
            (Some(costs), inventory, lines)
        } else {
            (None, inventory, vec![])
        };
        let dues_transactions = batch
            .commitments
            .as_ref()
            .or_else(|| batch.credit.as_ref().and_then(|c| c.commitments.as_ref()))
            .map(|c| c.transactions.as_slice())
            .unwrap_or(&[]);
        let (dues_transfers, collection_lines) = if self.issuance.is_some() {
            crate::issuance_accounting::collection(world, before, after, coin, dues_transactions)?
        } else {
            (dues_transactions.to_vec(), vec![])
        };
        let (dues_transfers, guaranteed_dues_lines) = crate::dues_accounting::guarantee_payments(
            world,
            batch.credit.as_ref(),
            &dues_transfers,
            coin,
            &self.exchange_values,
        )?;
        let (dues_transfers, estate_dues_lines) = crate::dues_accounting::estate_payments(
            world,
            batch.credit.as_ref(),
            &dues_transfers,
            coin,
        )?;
        let (inventory, dues_lines) = if let Some(dues) = &self.dues {
            let performance = crate::dues_accounting::without_physical_guarantees(
                world,
                batch.credit.as_ref(),
                after,
                coin,
            )?;
            dues.settle_allocated(
                world,
                before,
                &performance,
                &inventory,
                coin,
                &dues_transfers,
                &mut allocation,
            )?
        } else {
            (inventory, vec![])
        };
        for t in &batch.transactions {
            if (self.dues.is_some() && dues_transactions.contains(t))
                || t.process.is_some()
                || t.trade.is_some()
                || t.delivery.is_some()
                || t.forward.is_some()
                || trades.contains(&t)
                || mint_transactions.contains(t)
                || loan_transfers.contains(t)
                || inventory_sales.contains(t)
            {
                continue;
            }
            let unrecognized: Vec<_> = t
                .effects
                .iter()
                .filter(|e| !expired.contains(e) && !regenerated.contains(e))
                .collect();
            let has_stock = unrecognized.iter().any(|e| {
                e.delta != 0
                    && world
                        .resources
                        .iter()
                        .any(|r| r.id == e.account.1 && r.kind == ResourceKind::Stock)
            });
            if has_stock
                && !batch
                    .credit
                    .as_ref()
                    .is_some_and(|c| c.transactions.contains(t))
            {
                return Err("stock transaction has no recognized financial source".into());
            }
            for e in unrecognized {
                if e.delta != 0
                    && e.account.1 != coin
                    && world
                        .resources
                        .iter()
                        .any(|r| r.id == e.account.1 && r.kind == ResourceKind::Stock)
                {
                    return Err("unpriced stock movement".into());
                }
            }
        }
        // Collection excludes perishable goods from contributions. Its verified
        // positive perishable effects are extra nonrival dwelling entitlements;
        // the actual dwelling use has already borne its wear/production cost.
        let (shared_services, contributions): (Vec<_>, Vec<_>) = batch
            .household
            .as_ref()
            .map_or(&[][..], |h| h.after.as_slice())
            .iter()
            .cloned()
            .partition(|e| world.activities.perishable.contains(&e.account.1));
        let (inventory, pooling_lines) = inventory.pool(world, &contributions, coin)?;
        let inventory = inventory.add_uncosted(&shared_services, coin)?;
        let opening = positions(
            world,
            outer_before,
            coin,
            &self.asset_values,
            &self.inventory,
            self.processes.as_ref(),
            self.dues.as_ref(),
            self.services.as_ref(),
            &self.exchange_values,
        )?;
        let closing = positions(
            world,
            outer_after,
            coin,
            &asset_values,
            &inventory,
            processes.as_ref(),
            self.dues.as_ref(),
            services.as_ref(),
            &self.exchange_values,
        )?;
        let mut delta = closing.clone();
        for (key, value) in &opening {
            accounting::add(&mut delta, key.clone(), -*value)?;
        }
        let mut lines = process_lines;
        let mut flows = Flows::new();
        for l in trade_lines
            .into_iter()
            .chain(dues_lines)
            .chain(loan_lines)
            .chain(equipment_lines)
            .chain(forward_lines)
            .chain(issuance_lines)
            .chain(collection_lines)
            .chain(estate_dues_lines)
            .chain(guaranteed_dues_lines)
            .chain(service_lines)
            .chain(attachment_lines)
            .chain(expiration_lines)
            .chain(allocation_lines)
            .chain(capacity_allocation_lines)
            .chain(pooling_lines)
        {
            if let Some(kind) = l.flow {
                flow(&mut flows, l.agent, l.account, kind, l.debit)?;
            } else {
                lines.push(l);
            }
        }
        // Verified wind-down sales are investments, not pooled gifts. Their
        // payment and ownership were committed together before core execution.
        for r in batch
            .household
            .iter()
            .flat_map(|h| &h.disposals)
            .filter(|r| r.rejection.is_none())
        {
            if r.sale.price.resource != coin {
                return Err("asset disposal needs reporting-denomination payment".into());
            }
            for (asset, value) in r.sale.values()? {
                disposal(&mut lines, &opening, r.household, asset, value)?;
            }
            for (who, amount) in [
                (r.household, r.sale.price.quantity),
                (r.sale.buyer, -r.sale.price.quantity),
            ] {
                flow(
                    &mut flows,
                    who,
                    Account::Cash,
                    Flow::Investing,
                    i128::from(amount),
                )?;
            }
        }
        let mut interest: BTreeMap<_, _> = before
            .credit
            .loans
            .iter()
            .map(|(id, l)| (*id, l.interest))
            .collect();
        if let Some(c) = &batch.credit {
            let loan = |id: &u32| {
                c.after
                    .loans
                    .get(id)
                    .or_else(|| before.credit.loans.get(id))
                    .ok_or("missing reporting loan")
            };
            for e in &c.events {
                use credit::Event;
                match e {
                    Event::Advanced {
                        creditor,
                        debtor,
                        amount,
                        ..
                    } => {
                        if amount.resource != coin {
                            continue;
                        }
                        flow(
                            &mut flows,
                            *creditor,
                            Account::Cash,
                            Flow::Investing,
                            -i128::from(amount.quantity),
                        )?;
                        flow(
                            &mut flows,
                            *debtor,
                            Account::Cash,
                            Flow::Financing,
                            i128::from(amount.quantity),
                        )?;
                    }
                    Event::Accrued {
                        loan: id,
                        interest: q,
                        ..
                    } => {
                        let l = loan(id)?;
                        let previous = interest.entry(*id).or_default();
                        *previous = previous
                            .checked_add(*q)
                            .ok_or("interest reporting overflow")?;
                        result(
                            &mut lines,
                            l.creditor,
                            Account::InterestIncome,
                            -crate::reporting_value::value(
                                coin,
                                &self.exchange_values,
                                l.denomination,
                                *q,
                            )?,
                        );
                        result(
                            &mut lines,
                            l.debtor,
                            Account::InterestExpense,
                            crate::reporting_value::value(
                                coin,
                                &self.exchange_values,
                                l.denomination,
                                *q,
                            )?,
                        );
                    }
                    Event::Paid {
                        loan: id,
                        principal,
                        interest: q,
                    } => {
                        let l = loan(id)?;
                        *interest.entry(*id).or_default() -= *q;
                        if l.denomination != coin {
                            continue;
                        }
                        for (agent, sign, kind) in [
                            (l.creditor, 1, Flow::Investing),
                            (l.debtor, -1, Flow::Financing),
                        ] {
                            flow(
                                &mut flows,
                                agent,
                                Account::Cash,
                                kind,
                                i128::from(*principal) * sign,
                            )?;
                            flow(
                                &mut flows,
                                agent,
                                Account::Cash,
                                Flow::Operating,
                                i128::from(*q) * sign,
                            )?;
                        }
                    }
                    Event::Endowed { agent, amount } => {
                        result(
                            &mut lines,
                            *agent,
                            Account::Capital,
                            -i128::from(amount.quantity),
                        );
                        flow(
                            &mut flows,
                            *agent,
                            Account::Cash,
                            Flow::Financing,
                            i128::from(amount.quantity),
                        )?;
                    }
                    Event::Cashflow { from, to, amount } => {
                        result(
                            &mut lines,
                            *from,
                            Account::TransferExpense,
                            i128::from(amount.quantity),
                        );
                        result(
                            &mut lines,
                            *to,
                            Account::TransferIncome,
                            -i128::from(amount.quantity),
                        );
                        flow(
                            &mut flows,
                            *from,
                            Account::Cash,
                            Flow::Operating,
                            -i128::from(amount.quantity),
                        )?;
                        flow(
                            &mut flows,
                            *to,
                            Account::Cash,
                            Flow::Operating,
                            i128::from(amount.quantity),
                        )?;
                    }
                    Event::Purchased {
                        offer,
                        asset,
                        buyer,
                        price,
                        downpayment,
                        advance,
                    } => {
                        let o = world
                            .credit
                            .as_ref()
                            .and_then(|c| c.offers.iter().find(|o| o.id == *offer))
                            .ok_or("missing purchase offer")?;
                        disposal(&mut lines, &opening, o.sale.seller, *asset, *price)?;
                        flow(
                            &mut flows,
                            *buyer,
                            Account::Cash,
                            Flow::Investing,
                            -i128::from(*downpayment),
                        )?;
                        flow(
                            &mut flows,
                            o.sale.seller,
                            Account::Cash,
                            Flow::Investing,
                            i128::from(*downpayment),
                        )?;
                        if o.loan.creditor != o.sale.seller {
                            flow(
                                &mut flows,
                                o.loan.creditor,
                                Account::Cash,
                                Flow::Investing,
                                -i128::from(*advance),
                            )?;
                            flow(
                                &mut flows,
                                o.sale.seller,
                                Account::Cash,
                                Flow::Investing,
                                i128::from(*advance),
                            )?;
                        }
                    }
                    Event::Enforced {
                        loan: id,
                        value,
                        debt_credit,
                        surplus,
                        ..
                    } => {
                        let l = loan(id)?;
                        let asset = l.collateral.as_ref().ok_or("missing collateral")?.asset;
                        disposal(&mut lines, &opening, l.debtor, asset, *value)?;
                        let q = interest.entry(*id).or_default();
                        *q -= (*q).min(*debt_credit);
                        flow(
                            &mut flows,
                            l.creditor,
                            Account::Cash,
                            Flow::Investing,
                            -i128::from(*surplus),
                        )?;
                        flow(
                            &mut flows,
                            l.debtor,
                            Account::Cash,
                            Flow::Investing,
                            i128::from(*surplus),
                        )?;
                    }
                    Event::Resold {
                        loan: id,
                        buyer,
                        price,
                        debt_credit,
                        surplus,
                        ..
                    } => {
                        let l = loan(id)?;
                        let asset = l.collateral.as_ref().ok_or("missing collateral")?.asset;
                        disposal(&mut lines, &opening, l.debtor, asset, *price)?;
                        let q = interest.entry(*id).or_default();
                        let paid_interest = (*q).min(*debt_credit);
                        *q -= paid_interest;
                        flow(
                            &mut flows,
                            l.creditor,
                            Account::Cash,
                            Flow::Operating,
                            i128::from(paid_interest),
                        )?;
                        flow(
                            &mut flows,
                            l.creditor,
                            Account::Cash,
                            Flow::Investing,
                            i128::from(*debt_credit - paid_interest),
                        )?;
                        flow(
                            &mut flows,
                            l.debtor,
                            Account::Cash,
                            Flow::Investing,
                            i128::from(*surplus),
                        )?;
                        flow(
                            &mut flows,
                            *buyer,
                            Account::Cash,
                            Flow::Investing,
                            -i128::from(*price),
                        )?;
                    }
                    Event::Rejected { .. }
                    | Event::Arrears { .. }
                    | Event::RepossessedForSale { .. }
                    | Event::ResaleNoBuyer { .. }
                    | Event::ResaleDenied { .. }
                    | Event::ResaleBid { .. }
                    | Event::ResaleRejected { .. }
                    | Event::EnforcementDeferred { .. } => {}
                }
            }
            for r in &c.recovery {
                match r {
                    recovery::Receipt::ClaimRelief {
                        proceeding,
                        creditor,
                        written_off: Some(amount),
                        contract: crate::finance::ContractId::Wages(_),
                        ..
                    } => {
                        let p = world
                            .recovery
                            .proceedings
                            .iter()
                            .find(|p| p.id == *proceeding)
                            .ok_or("missing estate")?;
                        let loss = crate::reporting_value::value(
                            coin,
                            &self.exchange_values,
                            amount.resource,
                            amount.quantity,
                        )?;
                        result(&mut lines, *creditor, Account::CreditLoss, loss);
                        result(&mut lines, p.debtor, Account::DebtRelief, -loss);
                    }
                    recovery::Receipt::ClaimRelief { .. } => {}
                    recovery::Receipt::WagesDistributed {
                        proceeding,
                        creditor,
                        paid,
                        ..
                    } => {
                        let p = world
                            .recovery
                            .proceedings
                            .iter()
                            .find(|p| p.id == *proceeding)
                            .ok_or("missing estate")?;
                        if p.denomination != coin {
                            return Err("estate wages require reporting currency".into());
                        }
                        flow(
                            &mut flows,
                            *creditor,
                            Account::Cash,
                            Flow::Operating,
                            i128::from(*paid),
                        )?;
                        flow(
                            &mut flows,
                            p.debtor,
                            Account::RestrictedCash(*proceeding),
                            Flow::Operating,
                            -i128::from(*paid),
                        )?;
                    }
                    recovery::Receipt::Guaranteed {
                        guarantee,
                        claim: recovery::GuaranteedClaim::Loan(id),
                        paid,
                        ..
                    } => {
                        let l = loan(id)?;
                        let g = world
                            .recovery
                            .guarantees
                            .iter()
                            .find(|g| g.id == *guarantee)
                            .ok_or("missing guarantee")?;
                        let q = interest.entry(*id).or_default();
                        let paid_interest = (*q).min(*paid);
                        *q -= paid_interest;
                        if l.denomination != coin {
                            continue;
                        }
                        flow(
                            &mut flows,
                            g.guarantor,
                            Account::Cash,
                            Flow::Investing,
                            -i128::from(*paid),
                        )?;
                        flow(
                            &mut flows,
                            l.creditor,
                            Account::Cash,
                            Flow::Operating,
                            i128::from(paid_interest),
                        )?;
                        flow(
                            &mut flows,
                            l.creditor,
                            Account::Cash,
                            Flow::Investing,
                            i128::from(*paid - paid_interest),
                        )?;
                    }
                    recovery::Receipt::Guaranteed {
                        guarantee,
                        claim: recovery::GuaranteedClaim::Wages { .. },
                        paid,
                        ..
                    } => {
                        let g = world
                            .recovery
                            .guarantees
                            .iter()
                            .find(|g| g.id == *guarantee)
                            .ok_or("missing wage guarantee")?;
                        let (_, creditor, denomination) =
                            g.claim.parties(world).ok_or("missing wage terms")?;
                        if denomination != coin {
                            continue;
                        }
                        flow(
                            &mut flows,
                            g.guarantor,
                            Account::Cash,
                            Flow::Investing,
                            -i128::from(*paid),
                        )?;
                        flow(
                            &mut flows,
                            creditor,
                            Account::Cash,
                            Flow::Operating,
                            i128::from(*paid),
                        )?;
                    }
                    recovery::Receipt::Distributed {
                        proceeding,
                        loan: id,
                        paid,
                        ..
                    } => {
                        let l = loan(id)?;
                        let q = interest.entry(*id).or_default();
                        let paid_interest = (*q).min(*paid);
                        *q -= paid_interest;
                        flow(
                            &mut flows,
                            l.creditor,
                            Account::Cash,
                            Flow::Operating,
                            i128::from(paid_interest),
                        )?;
                        flow(
                            &mut flows,
                            l.creditor,
                            Account::Cash,
                            Flow::Investing,
                            i128::from(*paid - paid_interest),
                        )?;
                        flow(
                            &mut flows,
                            l.debtor,
                            Account::RestrictedCash(*proceeding),
                            Flow::Operating,
                            -i128::from(paid_interest),
                        )?;
                        flow(
                            &mut flows,
                            l.debtor,
                            Account::RestrictedCash(*proceeding),
                            Flow::Financing,
                            -i128::from(*paid - paid_interest),
                        )?;
                    }
                    recovery::Receipt::WrittenOff {
                        loan: id,
                        principal,
                        interest,
                        ..
                    } => {
                        let l = loan(id)?;
                        let loss = i128::from(*principal) + i128::from(*interest);
                        result(&mut lines, l.creditor, Account::CreditLoss, loss);
                        result(&mut lines, l.debtor, Account::DebtRelief, -loss);
                    }
                    recovery::Receipt::InventorySold {
                        proceeding,
                        buyer,
                        proceeds,
                        ..
                    } => {
                        let p = world
                            .recovery
                            .proceedings
                            .iter()
                            .find(|p| p.id == *proceeding)
                            .ok_or("missing inventory estate")?;
                        flow(
                            &mut flows,
                            *buyer,
                            Account::Cash,
                            Flow::Operating,
                            -i128::from(*proceeds),
                        )?;
                        flow(
                            &mut flows,
                            p.debtor,
                            Account::RestrictedCash(*proceeding),
                            Flow::Operating,
                            i128::from(*proceeds),
                        )?;
                    }
                    recovery::Receipt::ReceivableSold {
                        proceeding,
                        buyer,
                        proceeds,
                        ..
                    } => {
                        let p = world
                            .recovery
                            .proceedings
                            .iter()
                            .find(|p| p.id == *proceeding)
                            .ok_or("missing receivable estate")?;
                        if p.denomination != coin {
                            return Err("receivable sale requires reporting currency".into());
                        }
                        flow(
                            &mut flows,
                            *buyer,
                            Account::Cash,
                            Flow::Investing,
                            -i128::from(*proceeds),
                        )?;
                        flow(
                            &mut flows,
                            p.debtor,
                            Account::RestrictedCash(*proceeding),
                            Flow::Investing,
                            i128::from(*proceeds),
                        )?;
                    }
                    recovery::Receipt::Sold {
                        proceeding,
                        asset,
                        buyer,
                        proceeds,
                    } => {
                        let p = world
                            .recovery
                            .proceedings
                            .iter()
                            .find(|p| p.id == *proceeding)
                            .ok_or("missing estate")?;
                        disposal(&mut lines, &opening, p.debtor, *asset, *proceeds)?;
                        flow(
                            &mut flows,
                            *buyer,
                            Account::Cash,
                            Flow::Investing,
                            -i128::from(*proceeds),
                        )?;
                        flow(
                            &mut flows,
                            p.debtor,
                            Account::RestrictedCash(*proceeding),
                            Flow::Investing,
                            i128::from(*proceeds),
                        )?;
                    }
                    recovery::Receipt::Guaranteed {
                        claim: recovery::GuaranteedClaim::Land { .. },
                        ..
                    }
                    | recovery::Receipt::Guaranteed {
                        claim: recovery::GuaranteedClaim::Forward(_),
                        ..
                    }
                    | recovery::Receipt::LandDistributed { .. }
                    | recovery::Receipt::DeliveryRelief { .. }
                    | recovery::Receipt::GuaranteeAdmission { .. }
                    | recovery::Receipt::SaleRejected { .. }
                    | recovery::Receipt::InventorySaleRejected { .. }
                    | recovery::Receipt::ReceivableSaleRejected { .. }
                    | recovery::Receipt::Opened { .. }
                    | recovery::Receipt::OpeningRejected { .. }
                    | recovery::Receipt::Admitted { .. }
                    | recovery::Receipt::ClosureDeferred { .. }
                    | recovery::Receipt::AssetsPending { .. }
                    | recovery::Receipt::Closed { .. } => {}
                }
            }
        }
        // Residual cash movements may only be a verified transfer between owned
        // cash and the same debtor's estate account. No unexplained income plug.
        let mut classified: BTreeMap<_, i128> = BTreeMap::new();
        for ((agent, a, _), q) in &flows {
            accounting::add(&mut classified, (*agent, a.clone()), *q)?;
        }
        let mut residual: BTreeMap<_, i128> = delta
            .iter()
            .filter(|((_, a), _)| a.cash())
            .map(|(k, v)| (k.clone(), *v))
            .collect();
        for (k, v) in &classified {
            accounting::add(&mut residual, k.clone(), -*v)?;
        }
        for p in &world.recovery.proceedings {
            let restricted = residual
                .remove(&(p.debtor, Account::RestrictedCash(p.id)))
                .unwrap_or(0);
            if restricted != 0 {
                flow(
                    &mut flows,
                    p.debtor,
                    Account::RestrictedCash(p.id),
                    Flow::Internal,
                    restricted,
                )?;
                flow(
                    &mut flows,
                    p.debtor,
                    Account::Cash,
                    Flow::Internal,
                    -restricted,
                )?;
                accounting::add(&mut residual, (p.debtor, Account::Cash), restricted)?;
            }
        }
        if residual.values().any(|v| *v != 0) {
            return Err("unclassified cash movement; no financial statements published".into());
        }
        for ((agent, account), debit) in delta {
            if debit != 0 && !account.cash() {
                lines.push(Line {
                    agent,
                    account,
                    debit,
                    flow: None,
                });
            }
        }
        for ((agent, account, kind), debit) in flows {
            if debit != 0 {
                lines.push(Line {
                    agent,
                    account,
                    debit,
                    flow: Some(kind),
                });
            }
        }
        let mut candidate = self.book.clone();
        candidate.post(Entry {
            id: format!("batch:{}", batch.id),
            month: batch.month,
            batch: Some(batch.id),
            description: format!("Verified {:?} financial boundary", batch.phase),
            lines,
        })?;
        let mut reported: Positions = candidate
            .balances()
            .iter()
            .filter(|((_, a), _)| {
                matches!(
                    a.class(),
                    accounting::Class::Asset | accounting::Class::Liability
                )
            })
            .map(|(k, v)| (k.clone(), *v))
            .collect();
        reported.retain(|_, v| *v != 0);
        if reported != closing {
            return Err("journal does not reconcile to authoritative positions".into());
        }
        self.asset_values = asset_values;
        self.processes = processes;
        self.inventory = inventory;
        self.services = services;
        self.book = candidate;
        self.boundary = outer_after.clone();
        Ok(())
    }
}
