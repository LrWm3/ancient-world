//! One-time liquid portfolio import from a completed main-world boundary.
//! Physical custody is kept separate from household beneficial ownership.
use super::{LaborBindings, LaborReplay};
use crate::{
    compute::Backend,
    financial_reporting::Audit,
    host::{Boundary, Settlement},
    model::{AgentId, ResourceId, ResourceKind, State, World},
    simulation::Simulation,
};
use ::ancient_world::{civilization::History, economy::FOOD};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GoodBinding {
    /// Stable main catalog ID. "food" reads Stocks.stock[1], never goods[FOOD].
    pub good: String,
    pub resource: ResourceId,
    pub ticks_per_kg: u32,
    /// Explicit opening carrying cost in experimental currency ticks per stock
    /// tick. This is a reporting assumption, not a source price or past income.
    pub carrying_cost_per_tick: u64,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WalletBinding {
    pub household: u32,
    pub agent: AgentId,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StockBindings {
    pub site: u32,
    /// One distinct custodian of the town's physical stock and working cash.
    /// This is not a council treasury or a person's beneficial ownership share.
    pub town_agent: AgentId,
    pub currency: ResourceId,
    pub ticks_per_money_unit: u32,
    pub goods: Vec<GoodBinding>,
    /// Exactly one distinct account per household currently at this site.
    pub wallets: Vec<WalletBinding>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub enum Source {
    TownGood(String),
    TownCash,
    HouseholdCash(u32),
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct ImportedBalance {
    pub source: Source,
    pub agent: AgentId,
    pub resource: ResourceId,
    pub source_quantity: f64,
    pub ticks_per_source_unit: u32,
    pub quantity: i32,
    /// Source units that were not imported; no reassignment or later top-up.
    pub remainder: f64,
    pub carrying_cost: i128,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct StockReceipt {
    pub completed_boundary: Boundary,
    pub first_replay_boundary: Boundary,
    pub balances: Vec<ImportedBalance>,
    /// Source evidence only; this does not install claims into engine accounting.
    pub claims: super::claims::ClaimInventory,
}

fn convert(quantity: f64, scale: u32) -> Result<(i32, f64), String> {
    if !scale.is_power_of_two() || !quantity.is_finite() || quantity < 0. {
        return Err("stock import requires nonnegative finite quantities and dyadic units".into());
    }
    // Multiplication by a power of two is exact in the admitted i32 range.
    let whole = (quantity * f64::from(scale)).floor();
    if whole > f64::from(i32::MAX) {
        return Err("imported balance exceeds engine quantity range".into());
    }
    let ticks = whole as i32;
    Ok((ticks, quantity - f64::from(ticks) / f64::from(scale)))
}

impl StockBindings {
    /// Bootstrap a private economy, never overwrite a running one. The first
    /// labor frame must FOLLOW the completed inventory boundary: end-of-month
    /// stock cannot fund production that already happened in that same month.
    /// The caller supplies a catalog, not a Simulation or fixture balances.
    pub fn start(
        &self,
        completed: &History,
        first: &History,
        labor: LaborBindings,
        world: World,
        backend: Backend,
    ) -> Result<(LaborReplay, StockReceipt), String> {
        if completed.month.checked_add(1) != Some(first.month)
            || self.site != labor.site
            || labor.source_identity(completed)? != labor.source_identity(first)?
        {
            return Err("stock import requires consecutive boundaries from the same source".into());
        }
        if !world.assets.is_empty() || !world.rights.is_empty() {
            return Err("stock import does not import land, equipment or use rights".into());
        }
        let used_controller = world.agency.values().any(|c| !c.is_unreviewed());
        if used_controller
            || world.governance_observation.is_some()
            || world.discovery.as_ref().is_some_and(|d| {
                d.through != 0
                    || !d.receipts.is_empty()
                    || !d.financial.is_empty()
                    || !d.supply.is_empty()
            })
        {
            return Err("stock import requires fresh controllers and discovery history".into());
        }
        if labor
            .people
            .iter()
            .any(|b| b.agent == self.town_agent && b.source.is_some())
        {
            return Err("town custody cannot be assigned to a named labor participant".into());
        }
        let site = completed
            .sites
            .get(self.site as usize)
            .filter(|s| s.id == self.site && s.founded <= completed.month)
            .ok_or("missing or future-founded stock source site")?;
        if let Some(participation) = &completed.participation {
            participation
                .validate(completed)
                .map_err(|e| e.to_string())?;
            if participation.month != Some(completed.month)
                || participation
                    .commitments
                    .iter()
                    .any(|c| c.site == self.site && !c.settled)
            {
                return Err("stock import requires completed source participation".into());
            }
        }
        let catalog = completed
            .economy_catalog
            .as_ref()
            .ok_or("missing source goods catalog")?;
        catalog.validate().map_err(|e| e.to_string())?;
        // A later catalog edit cannot silently change an imported good's meaning.
        let next_catalog = first
            .economy_catalog
            .as_ref()
            .ok_or("missing next goods catalog")?;
        next_catalog.validate().map_err(|e| e.to_string())?;
        if serde_json::to_value(catalog).map_err(|e| e.to_string())?
            != serde_json::to_value(next_catalog).map_err(|e| e.to_string())?
        {
            return Err("source goods catalog changed across stock import".into());
        }
        // FOOD is a virtual market slot backed exclusively by Stocks.stock[1].
        if site.economy.goods[FOOD] != 0. {
            return Err("ambiguous food custody in virtual goods slot".into());
        }
        let agents: BTreeSet<_> = world.agents.iter().map(|a| a.id).collect();
        let stock_resources: BTreeSet<_> = world
            .resources
            .iter()
            .filter(|r| r.kind == ResourceKind::Stock)
            .map(|r| r.id)
            .collect();
        if !agents.contains(&self.town_agent) || !stock_resources.contains(&self.currency) {
            return Err("missing town custodian or stock currency".into());
        }
        let mut balances = Vec::new();
        let mut good_ids = BTreeSet::new();
        let mut resources = BTreeSet::from([self.currency]);
        for binding in &self.goods {
            let index = catalog.index(&binding.good).ok_or("unknown source good")?;
            if !good_ids.insert(index)
                || !resources.insert(binding.resource)
                || !stock_resources.contains(&binding.resource)
            {
                return Err("duplicate goods, aliased units or non-stock import target".into());
            }
            let source_quantity = f64::from(if index == FOOD {
                site.stocks.stock[1]
            } else {
                site.economy.goods[index]
            });
            let (quantity, remainder) = convert(source_quantity, binding.ticks_per_kg)?;
            balances.push(ImportedBalance {
                source: Source::TownGood(binding.good.clone()),
                agent: self.town_agent,
                resource: binding.resource,
                source_quantity,
                ticks_per_source_unit: binding.ticks_per_kg,
                quantity,
                remainder,
                carrying_cost: i128::from(quantity) * i128::from(binding.carrying_cost_per_tick),
            });
        }
        for (index, &q) in site.economy.goods.iter().enumerate() {
            if !q.is_finite() || q < 0. || (q > 0. && !good_ids.contains(&index)) {
                return Err("invalid or unmapped town goods balance".into());
            }
        }
        let food = site.stocks.stock[1];
        if !food.is_finite() || food < 0. || (food > 0. && !good_ids.contains(&FOOD)) {
            return Err("invalid or unmapped town food balance".into());
        }
        let society = completed
            .society
            .as_ref()
            .ok_or("household identities are required")?;
        let wallets = society
            .household_economy
            .as_ref()
            .ok_or("household wallets are required")?;
        if wallets.accounts.len() != society.households.len()
            || wallets.started > wallets.observed
            || wallets.observed > completed.month
            || society
                .households
                .iter()
                .enumerate()
                .any(|(id, h)| h.id as usize != id)
        {
            return Err("invalid household wallet identities or boundary".into());
        }
        wallets.validate(completed).map_err(|e| e.to_string())?;
        let expected: BTreeSet<_> = society
            .households
            .iter()
            .filter(|h| h.site == self.site)
            .map(|h| h.id)
            .collect();
        let mut seen = BTreeSet::new();
        let mut custodians = BTreeSet::from([self.town_agent]);
        let mut cash = vec![(
            Source::TownCash,
            self.town_agent,
            f64::from(site.economy.finance[0]),
        )];
        for binding in &self.wallets {
            if !expected.contains(&binding.household)
                || !seen.insert(binding.household)
                || !agents.contains(&binding.agent)
                || !custodians.insert(binding.agent)
                || labor
                    .people
                    .iter()
                    .any(|b| b.agent == binding.agent && b.source.is_some())
            {
                return Err("invalid, duplicate or merged household wallet binding".into());
            }
            let household = &society.households[binding.household as usize];
            if household.founded > completed.month {
                return Err("future-founded household wallet".into());
            }
            cash.push((
                Source::HouseholdCash(binding.household),
                binding.agent,
                wallets.accounts[binding.household as usize].cash,
            ));
        }
        if seen != expected {
            return Err("stock import must bind every local household wallet".into());
        }
        for (source, agent, source_quantity) in cash {
            let (quantity, remainder) = convert(source_quantity, self.ticks_per_money_unit)?;
            balances.push(ImportedBalance {
                source,
                agent,
                resource: self.currency,
                source_quantity,
                ticks_per_source_unit: self.ticks_per_money_unit,
                quantity,
                remainder,
                carrying_cost: i128::from(quantity),
            });
        }
        balances.sort_by_key(|b| (b.agent, b.resource));
        let claims = self.inspect_claims(completed)?;
        claims.require_liquid_only()?;
        let mut state = State::empty();
        let mut costs = BTreeMap::new();
        for b in &balances {
            if b.quantity > 0 {
                state.balances.insert((b.agent, b.resource), b.quantity);
                if b.resource != self.currency {
                    costs.insert((b.agent, b.resource), b.carrying_cost);
                }
            }
        }
        let sim = Simulation::new(world, state, backend)?;
        let audit = Audit::with_inventory(
            &sim.world,
            &sim.state,
            self.currency,
            BTreeMap::new(),
            costs,
        )?
        .with_process_policy(&sim.world, BTreeMap::new())?;
        let first_replay_boundary = Boundary {
            site: self.site,
            month: first.month,
        };
        let economy = Settlement::attach(first_replay_boundary, sim, audit)?;
        let replay = LaborReplay::attach(labor, first, economy)?;
        Ok((
            replay,
            StockReceipt {
                completed_boundary: Boundary {
                    site: self.site,
                    month: completed.month,
                },
                first_replay_boundary,
                balances,
                claims,
            },
        ))
    }
}
