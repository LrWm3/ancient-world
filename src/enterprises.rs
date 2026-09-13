//! Household-owned toll manufacturers leasing a subset of communal workshop equipment.
//! Cash is separate; materials, products and embodied equipment stay in canonical town stocks.
use crate::{
    civilization::History,
    household_economy::{deposit, withdraw, HouseholdAccount},
};
use anyhow::{ensure, Result};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
const MIN_OBSERVED_WORKER_MONTHS: f64 = 1e-6;
const SHORTAGE_MEMORY_RETENTION: f64 = 0.75;
const SHORTAGE_NEW_WEIGHT: f64 = 0.25;
const WAGE_PROFITABILITY_HEADROOM: f64 = 0.9;
const MIN_WAGE_MULTIPLIER: f64 = 0.6;
const MAX_WAGE_MULTIPLIER: f64 = 1.125;
const WAGE_RAISE_SHORTAGE_THRESHOLD: f64 = 0.15;
const WAGE_RAISE_MIN_UTILIZATION: f64 = 0.8;
const PAYROLL_RESERVE_MONTHS: f64 = 3.;
const SHORTAGE_WAGE_RAISE_RATE: f64 = 0.08;
const WAGE_CUT_UTILIZATION_THRESHOLD: f64 = 0.5;
const WAGE_CUT_RETENTION: f64 = 0.97;
const STAFF_GRANT_TOLERANCE: f64 = 1e-5;
const STAFF_RECEIPT_TOLERANCE: f64 = 1e-4;
const STAFF_COST_TOLERANCE: f64 = 1e-5;
const CAPITAL_RELATIVE_TOLERANCE: f64 = 1e-7;
const COMPLETED_WORK_TOLERANCE: f64 = 1e-4;
const RENT_REFERENCE_WORK_MONTHS_PER_UNIT: f64 = 0.08;
const SURPLUS_DIVIDEND_FRACTION: f64 = 0.05;
const WAGE_REFERENCE_FOOD_KG_PER_MONTH: f64 = 18.;
const MIN_WAGE_FOOD_PRICE: f32 = 0.01;
const REOPEN_DELAY_MONTHS: u32 = 6;
const MAX_LEASED_WORKSHOP_FRACTION: f64 = 0.5;
const MIN_FOUNDING_LEASE_UNITS: f64 = 0.02;
const MIN_FOUNDING_DEMONSTRATED_WORK: f32 = 0.02;
const MIN_WORKSHOP_SHARE_DENOMINATOR: f64 = 0.001;
const SHIFT_DEMONSTRATED_HEADROOM: f64 = 1.1;
const MIN_SHIFT_WORKER_MONTHS: f64 = 0.05;
const FOUNDING_CAPITAL_CASH_FRACTION: f64 = 0.25;
const DISTRESS_MIN_STAFFING_FRACTION: f64 = 0.25;
const DISTRESS_MIN_LEASE_UNITS: f64 = 0.001;
const PAYROLL_WEIGHT_FLOOR: f64 = 1e-12;
const SERVICE_MAX_TOWN_CASH_FRACTION: f64 = 0.2;
const IDLE_COMPLETION_THRESHOLD: f64 = 0.15;
const DISTRESS_CLOSURE_MONTHS: u32 = 3;
const IDLE_CLOSURE_MONTHS: u32 = 6;
const MIN_IDLE_FUNDED_WORK: f64 = 0.01;

pub(crate) const SERVICE_QUOTE_MULTIPLIER: f64 = 1.25;

pub mod orders;

pub(crate) fn workshop_reference_wage(town: &crate::civilization::Site) -> f64 {
    WAGE_REFERENCE_FOOD_KG_PER_MONTH
        * town.economy.prices[crate::economy::FOOD].max(MIN_WAGE_FOOD_PRICE) as f64
}

/// Posted next-month labor offer; service prices remain independent of wage bids.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WagePolicy {
    pub multiplier: f64,
    pub pending: Option<f64>,
    pub shortage: f64,
    pub observed: Option<u32>,
}
impl Default for WagePolicy {
    fn default() -> Self {
        Self {
            multiplier: 1.,
            pending: None,
            shortage: 0.,
            observed: None,
        }
    }
}
impl WagePolicy {
    fn observe(
        &mut self,
        month: u32,
        work: [f64; 3],
        cash: f64,
        reference: f64,
        paid_invoice: f64,
    ) {
        let [expected, hired, completed] = work;
        if self.observed == Some(month) {
            return;
        }
        self.observed = Some(month);
        let vacancy = if expected > MIN_OBSERVED_WORKER_MONTHS {
            (1. - hired / expected).clamp(0., 1.)
        } else {
            0.
        };
        self.shortage = SHORTAGE_MEMORY_RETENTION * self.shortage + SHORTAGE_NEW_WEIGHT * vacancy;
        let utilization = if hired > MIN_OBSERVED_WORKER_MONTHS {
            (completed / hired).clamp(0., 1.)
        } else {
            0.
        };
        let payroll = hired * reference * self.multiplier;
        // Raises need demonstrated productive work, payment and a cash buffer.
        // The independent 1.25x service quote limits the affordable wage.
        let profitable_ceiling =
            (SERVICE_QUOTE_MULTIPLIER * utilization * WAGE_PROFITABILITY_HEADROOM)
                .clamp(MIN_WAGE_MULTIPLIER, MAX_WAGE_MULTIPLIER);
        let next = if self.shortage > WAGE_RAISE_SHORTAGE_THRESHOLD
            && utilization >= WAGE_RAISE_MIN_UTILIZATION
            && paid_invoice >= payroll
            && cash >= PAYROLL_RESERVE_MONTHS * payroll
        {
            (self.multiplier * (1. + SHORTAGE_WAGE_RAISE_RATE * self.shortage))
                .min(profitable_ceiling)
                .max(self.multiplier)
        } else if utilization < WAGE_CUT_UTILIZATION_THRESHOLD || paid_invoice < payroll {
            self.multiplier * WAGE_CUT_RETENTION
        } else {
            self.multiplier
        };
        self.pending = Some(next.clamp(MIN_WAGE_MULTIPLIER, MAX_WAGE_MULTIPLIER));
    }
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Firm {
    #[serde(default)]
    pub financing: crate::credit::accounts::Financing,
    #[serde(default)]
    pub wage_policy: Option<WagePolicy>,
    /// None preserves legacy wage-indexed fees; refined contracts fix this at posting.
    #[serde(default)]
    pub service_rate: Option<f64>,
    #[serde(default)]
    pub staffing: Option<crate::workshop_resolution::Staffing>,
    pub id: u32,
    pub site: u32,
    pub family: u32,
    pub owner: u32,
    pub founded: u32,
    pub closed: Option<u32>,
    pub closing_reason: Option<String>,
    /// Operating lease on existing equipment; not a second material inventory.
    pub leased_units: f64,
    pub cash: f64,
    pub capital: f64,
    pub revenue: f64,
    pub wages: f64,
    pub rent: f64,
    pub dividends: f64,
    pub liquidation: f64,
    /// Unpaid fees are written off, not spendable debt or invented revenue.
    pub written_off: f64,
    pub completed_work: f64,
    pub paid_work: f64,
    #[serde(default)]
    pub last_requested_work: f64,
    pub last_funded_work: f64,
    pub last_completed_work: f64,
    pub wage_rate: f64,
    pub idle_months: u32,
    pub distressed_months: u32,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Enterprises {
    #[serde(default)]
    pub procurement: orders::Procurement,
    #[serde(default)]
    pub orders: Vec<orders::ServiceOrder>,
    pub enabled: bool,
    pub firms: Vec<Firm>,
}
impl Default for Enterprises {
    fn default() -> Self {
        Self {
            enabled: true,
            procurement: Default::default(),
            orders: vec![],
            firms: vec![],
        }
    }
}
impl Enterprises {
    pub fn validate(&self, h: &History) -> Result<()> {
        orders::validate(self, h)?;
        let mut occupied = BTreeSet::new();
        for (i, f) in self.firms.iter().enumerate() {
            if let Some(p) = &f.wage_policy {
                ensure!(
                    p.multiplier.is_finite()
                        && (MIN_WAGE_MULTIPLIER..=MAX_WAGE_MULTIPLIER).contains(&p.multiplier)
                        && p.pending.is_none_or(|v| v.is_finite()
                            && (MIN_WAGE_MULTIPLIER..=MAX_WAGE_MULTIPLIER).contains(&v))
                        && p.shortage.is_finite()
                        && (0. ..=1.).contains(&p.shortage)
                        && p.observed.is_none_or(|m| m <= h.month),
                    "invalid enterprise wage policy"
                );
            }
            ensure!(
                f.service_rate.is_none_or(|v| v.is_finite() && v >= 0.),
                "invalid enterprise service quote"
            );
            ensure!(
                f.id as usize == i
                    && (f.site as usize) < h.sites.len()
                    && f.family < 4
                    && h.society
                        .as_ref()
                        .is_some_and(|s| (f.owner as usize) < s.households.len())
                    && f.founded <= h.month
                    && f.closed.is_none_or(|m| m >= f.founded && m <= h.month),
                "invalid enterprise identity or clock"
            );
            if let Some(staff) = &f.staffing {
                ensure!(
                    staff.boundary.month <= h.month
                        && staff.boundary.site == f.site
                        && staff.boundary.subject == f.id
                        && staff.boundary.system == crate::resolution::System::Workshop,
                    "invalid workshop resolution boundary"
                );
                ensure!(
                    staff.expected.is_finite()
                        && staff.granted.is_finite()
                        && staff.expected >= 0.
                        && staff.granted >= 0.
                        && staff.granted <= staff.expected + STAFF_GRANT_TOLERANCE
                        && (staff.granted - f.last_funded_work).abs() < STAFF_RECEIPT_TOLERANCE,
                    "invalid workshop time grant"
                );
                ensure!(
                    staff.wages.iter().all(|(id, w)| h
                        .society
                        .as_ref()
                        .is_some_and(|s| (*id as usize) < s.households.len())
                        && w.is_finite()
                        && *w >= 0.),
                    "invalid workshop wage recipients"
                );
                if staff.mode == crate::resolution::Mode::Individual {
                    ensure!(
                        (staff.wages.iter().map(|(_, w)| w).sum::<f64>() - staff.granted).abs()
                            < STAFF_COST_TOLERANCE,
                        "workshop wages lack actual participants"
                    );
                    if let Some(p) = h
                        .participation
                        .as_ref()
                        .filter(|p| p.month == Some(staff.boundary.month))
                    {
                        let mut ids = BTreeSet::new();
                        ensure!(
                            staff.commitments.iter().all(|id| ids.insert(id)
                                && p.commitments.get(*id as usize).is_some_and(|c| c.activity
                                    == crate::participation::Activity::Workshop
                                    && c.site == f.site
                                    && c.month == staff.boundary.month
                                    && (!staff.settled || c.settled))),
                            "invalid workshop personal commitment"
                        );
                    }
                }
            }
            ensure!(
                f.closed.is_some() || occupied.insert((f.site, f.family)),
                "overlapping workshop leases"
            );
            ensure!(
                [
                    f.leased_units,
                    f.cash,
                    f.capital,
                    f.revenue,
                    f.wages,
                    f.rent,
                    f.dividends,
                    f.liquidation,
                    f.written_off,
                    f.completed_work,
                    f.paid_work,
                    f.last_funded_work,
                    f.last_completed_work,
                    f.wage_rate
                ]
                .iter()
                .all(|x| x.is_finite() && *x >= 0.),
                "invalid enterprise account"
            );
            ensure!(f.financing.validate(), "invalid enterprise financing");
            ensure!(
                (f.cash - f.capital - f.revenue - f.financing.net_cash()
                    + f.wages
                    + f.rent
                    + f.dividends
                    + f.liquidation)
                    .abs()
                    < CAPITAL_RELATIVE_TOLERANCE * (1. + f.capital + f.revenue),
                "enterprise cash ledger does not reconcile"
            );
            ensure!(
                f.completed_work <= f.paid_work + COMPLETED_WORK_TOLERANCE * (1. + f.paid_work)
                    && f.last_completed_work <= f.last_funded_work + COMPLETED_WORK_TOLERANCE,
                "unfunded enterprise work"
            );
        }
        if !self.firms.is_empty() {
            let accounts = &h
                .society
                .as_ref()
                .and_then(|s| s.household_economy.as_ref())
                .ok_or_else(|| anyhow::anyhow!("employers require household accounts"))?
                .accounts;
            let mut equity = vec![[0.; 2]; accounts.len()];
            for f in &self.firms {
                ensure!((f.owner as usize) < equity.len(), "missing owner wallet");
                equity[f.owner as usize][0] += f.capital;
                equity[f.owner as usize][1] += f.liquidation;
            }
            for (a, claim) in accounts.iter().zip(equity) {
                ensure!(
                    (a.capital_invested - claim[0]).abs()
                        <= CAPITAL_RELATIVE_TOLERANCE * (1. + claim[0])
                        && (a.capital_returned - claim[1]).abs()
                            <= CAPITAL_RELATIVE_TOLERANCE * (1. + claim[1]),
                    "household equity does not match owned businesses"
                );
            }
        }
        Ok(())
    }
}
fn close(f: &mut Firm, owner: &mut HouseholdAccount, month: u32, reason: &str, retain: bool) {
    if !retain {
        owner.cash += f.cash;
        owner.capital_returned += f.cash;
        f.liquidation += f.cash;
        f.cash = 0.;
    }
    f.closed = Some(month);
    f.closing_reason = Some(reason.into());
}
// Round a worker allowance down: no f32 GPU work can exceed prepaid f64 wages.
fn work_floor(v: f64) -> f32 {
    let mut rounded = v.max(0.) as f32;
    if rounded as f64 > v {
        rounded = f32::from_bits(rounded.to_bits().saturating_sub(1));
    }
    rounded
}
// One slot per workshop family makes summation independent of firm storage order.
// Requests are already affordable after rent; do not reserve workers for unfunded shifts.
fn allocate_work(requests: [f64; 4], capacity: f64) -> [f32; 4] {
    let total = requests.iter().sum::<f64>();
    let factor = if total > 0. {
        (capacity.max(0.) / total).min(1.)
    } else {
        0.
    };
    requests.map(|request| work_floor(request * factor))
}
fn viable_entry(expected_work: f64, paid_shift: f64, units: f64) -> bool {
    expected_work * SERVICE_QUOTE_MULTIPLIER
        > paid_shift + units * RENT_REFERENCE_WORK_MONTHS_PER_UNIT
}

// Equity investment is risk capital, not earned profit. Losses must be recovered
// before dividends resume; the operating reserve is an additional independent cap.
fn distributable_profit(f: &Firm) -> f64 {
    let earned = (f.revenue + f.financing.net_interest() - f.wages - f.rent - f.dividends).max(0.);
    let surplus = (f.cash - PAYROLL_RESERVE_MONTHS * f.last_funded_work * f.wage_rate).max(0.)
        * SURPLUS_DIVIDEND_FRACTION;
    earned.min(surplus)
}
impl History {
    pub fn enterprise_summary(&self) -> serde_json::Value {
        let firms = self
            .enterprises
            .as_ref()
            .map(|e| e.firms.as_slice())
            .unwrap_or(&[]);
        let orders = self
            .enterprises
            .as_ref()
            .map(|e| e.orders.as_slice())
            .unwrap_or(&[]);
        serde_json::json!({"procurement": self.enterprises.as_ref().map(|e| &e.procurement),
            "service_orders": orders.len(),
            "service_escrow": orders.iter().map(|o| o.escrow).sum::<f64>(),
            "service_order_paid": orders.iter().map(|o| o.paid).sum::<f64>(),
            "service_order_refunded": orders.iter().map(|o| o.refunded).sum::<f64>(),
            "enabled":self.enterprises.as_ref().is_some_and(|e|e.enabled),
            "founded":firms.len(),"active":firms.iter().filter(|f|f.closed.is_none()).count(),
            "closed":firms.iter().filter(|f|f.closed.is_some()).count(),
            "cash":firms.iter().map(|f|f.cash).sum::<f64>(),
            "capital":firms.iter().map(|f|f.capital).sum::<f64>(),
            "revenue":firms.iter().map(|f|f.revenue).sum::<f64>(),
            "wages":firms.iter().map(|f|f.wages).sum::<f64>(),
            "rent":firms.iter().map(|f|f.rent).sum::<f64>(),
            "dividends":firms.iter().map(|f|f.dividends).sum::<f64>(),
            "written_off":firms.iter().map(|f|f.written_off).sum::<f64>(),
            "paid_work":firms.iter().map(|f|f.paid_work).sum::<f64>(),
            "completed_work":firms.iter().map(|f|f.completed_work).sum::<f64>()})
    }
    pub(crate) fn prepare_enterprises(&mut self) {
        let offers = self.workshop_offers();
        let refine = self
            .resolution
            .as_ref()
            .is_some_and(|r| r.workshop_individual);
        for s in &mut self.sites {
            s.economy.enterprise_lease = [0.; 4];
            s.economy.enterprise_plan = [0.; 4];
            s.economy.enterprise_used = [0.; 4];
            s.economy.enterprise_productivity = [0.; 4];
        }
        let Some(mut enterprises) = self.enterprises.take() else {
            return;
        };
        let Some(society) = self.society.as_mut() else {
            self.enterprises = Some(enterprises);
            return;
        };
        let Some(e) = society.household_economy.as_mut() else {
            self.enterprises = Some(enterprises);
            return;
        };
        e.accounts
            .resize(society.households.len(), HouseholdAccount::default());
        let mut residents = vec![vec![]; self.sites.len()];
        for hh in &society.households {
            if hh.vacant_since.is_none()
                && self.people[hh.head as usize].died.is_none()
                && !society.relocation.away(hh.id)
                && !society.relocation.lost_households.contains(&hh.id)
            {
                residents[hh.site as usize].push(hh.id as usize);
            }
        }
        let mut notices = vec![];
        // Leases do not follow an owner who leaves. Cash follows the household's real wallet.
        for f in enterprises.firms.iter_mut().filter(|f| f.closed.is_none()) {
            let site = &self.sites[f.site as usize];
            let reason = if !enterprises.enabled {
                Some("operator policy ended")
            } else if site.abandoned || site.stocks.stock[0] <= 0. {
                Some("settlement abandoned")
            } else if society.households[f.owner as usize].vacant_since.is_some()
                || self.people[society.households[f.owner as usize].head as usize]
                    .died
                    .is_some()
            {
                Some("ownership account has no living representative")
            } else if !residents[f.site as usize].contains(&(f.owner as usize)) {
                Some("owner departed")
            } else if site.economy.workshop_types[0][3] < 0.5 {
                Some("specialized workshop service ended")
            } else {
                None
            };
            if let Some(reason) = reason {
                close(
                    f,
                    &mut e.accounts[f.owner as usize],
                    self.month,
                    reason,
                    self.credit.operator_has_debt(f.id),
                );
                notices.push(("enterprise_closed",f.site,format!("Workshop operator {} closed: {}; cash retained for outstanding debt or returned to household {} and equipment lease released",f.id,reason,f.owner)));
            }
        }
        // Founders supply actual wallet capital. No city grant, loan or new workshop is implied.
        if enterprises.enabled && self.month > 0 && self.month % 3 == 0 {
            for (site, ids) in residents.iter().enumerate() {
                let town = &self.sites[site];
                if town.abandoned || town.economy.workshop_types[0][3] < 0.5 {
                    continue;
                }
                let rate = WAGE_REFERENCE_FOOD_KG_PER_MONTH
                    * town.economy.prices[crate::economy::FOOD].max(MIN_WAGE_FOOD_PRICE) as f64;
                for family in 0..4 {
                    if enterprises.firms.iter().any(|f| {
                        f.site as usize == site
                            && f.family as usize == family
                            && (f.closed.is_none()
                                || f.closed.is_some_and(|m| {
                                    self.month.saturating_sub(m) < REOPEN_DELAY_MONTHS
                                }))
                    }) {
                        continue;
                    }
                    let units = (town.economy.workshop_types[family][0] as f64
                        * MAX_LEASED_WORKSHOP_FRACTION)
                        .min(1.);
                    if units < MIN_FOUNDING_LEASE_UNITS
                        || town.economy.workshop_types[family][2] < MIN_FOUNDING_DEMONSTRATED_WORK
                    {
                        continue;
                    }
                    // Use only locally observed work and the current service quote.
                    // Do not repeatedly finance a shift whose forecast fees cannot cover
                    // wages and rent even before uncertainty or customer nonpayment.
                    let observed = town.economy.workshop_types[family][2] as f64;
                    let share = units
                        / (town.economy.workshop_types[family][0] as f64)
                            .max(MIN_WORKSHOP_SHARE_DENOMINATOR);
                    let expected = (observed * share)
                        .min(units * crate::production::WORKSHOP_WORKER_MONTHS_PER_UNIT as f64);
                    let shift = (expected * SHIFT_DEMONSTRATED_HEADROOM)
                        .max(MIN_SHIFT_WORKER_MONTHS)
                        .min(units * crate::production::WORKSHOP_WORKER_MONTHS_PER_UNIT as f64);
                    if !viable_entry(expected, shift, units) {
                        continue;
                    }
                    let owner = ids.iter().copied().max_by(|&a, &b| {
                        e.accounts[a]
                            .cash
                            .total_cmp(&e.accounts[b].cash)
                            .then_with(|| b.cmp(&a))
                    });
                    let Some(owner) = owner else {
                        continue;
                    };
                    let capital = e.accounts[owner].cash * FOUNDING_CAPITAL_CASH_FRACTION;
                    if capital
                        < PAYROLL_RESERVE_MONTHS
                            * rate
                            * (shift + units * RENT_REFERENCE_WORK_MONTHS_PER_UNIT)
                    {
                        continue;
                    }
                    e.accounts[owner].cash -= capital;
                    e.accounts[owner].capital_invested += capital;
                    let id = enterprises.firms.len() as u32;
                    enterprises.firms.push(Firm {
                        financing: Default::default(),
                        wage_policy: None,
                        service_rate: None,
                        staffing: None,
                        id,
                        site: site as u32,
                        family: family as u32,
                        owner: owner as u32,
                        founded: self.month,
                        closed: None,
                        closing_reason: None,
                        leased_units: units,
                        cash: capital,
                        capital,
                        revenue: 0.,
                        wages: 0.,
                        rent: 0.,
                        dividends: 0.,
                        liquidation: 0.,
                        written_off: 0.,
                        completed_work: 0.,
                        paid_work: 0.,
                        last_requested_work: 0.,
                        last_funded_work: 0.,
                        last_completed_work: 0.,
                        wage_rate: rate,
                        idle_months: 0,
                        distressed_months: 0,
                    });
                    notices.push(("enterprise_founded",site as u32,format!("Household {} invested {:.2} in workshop operator {} (family {}), leasing {:.3} existing units",owner,capital,id,family,units)));
                }
            }
        }
        let capacities: Vec<f64> = self
            .sites
            .iter()
            .map(|s| {
                (s.economy.labor[3].max(0.).min(crate::labor::available(
                    s,
                    true,
                    self.living.is_some(),
                ))) as f64
            })
            .collect();
        let mut requests = vec![[0.; 4]; self.sites.len()];
        let mut desired_work = vec![0.; enterprises.firms.len()];
        for f in enterprises.firms.iter_mut().filter(|f| f.closed.is_none()) {
            let site = f.site as usize;
            let family = f.family as usize;
            let town = &mut self.sites[site];
            let reference = workshop_reference_wage(town);
            if refine {
                let policy = f.wage_policy.get_or_insert_with(Default::default);
                if let Some(next) = policy.pending.take() {
                    policy.multiplier = next;
                }
                f.wage_rate = reference * policy.multiplier;
                f.service_rate = Some(reference * SERVICE_QUOTE_MULTIPLIER);
            } else {
                f.wage_rate = reference;
                f.service_rate = None;
            }
            let units = f
                .leased_units
                .min(town.economy.workshop_types[family][0] as f64);
            let rent_request =
                (units * RENT_REFERENCE_WORK_MONTHS_PER_UNIT * reference).min(f.cash);
            let rent = deposit(&mut town.economy.finance[0], rent_request);
            f.cash -= rent;
            f.rent += rent;
            let lease_share = units
                / (town.economy.workshop_types[family][0] as f64)
                    .max(MIN_WORKSHOP_SHARE_DENOMINATOR);
            let desired = (units * crate::production::WORKSHOP_WORKER_MONTHS_PER_UNIT as f64)
                .min(
                    (town.economy.workshop_types[family][2] as f64
                        * lease_share
                        * SHIFT_DEMONSTRATED_HEADROOM)
                        .max(MIN_SHIFT_WORKER_MONTHS),
                )
                .min(capacities[site]);
            desired_work[f.id as usize] = desired;
            requests[site][family] = desired.min(f.cash / f.wage_rate);
            f.last_requested_work = requests[site][family];
            town.economy.enterprise_lease[family] = units as f32;
        }
        let grants = requests
            .iter()
            .zip(capacities)
            .map(|(&requests, capacity)| allocate_work(requests, capacity))
            .collect::<Vec<_>>();
        let mut order: Vec<_> = enterprises
            .firms
            .iter()
            .enumerate()
            .filter(|(_, f)| f.closed.is_none())
            .map(|(i, f)| ((f.site, f.family, f.id), i))
            .collect();
        order.sort_unstable();
        let mut matched = std::collections::BTreeMap::new();
        if refine {
            for site in 0..self.sites.len() {
                let jobs: Vec<_> = order
                    .iter()
                    .map(|(_, i)| &enterprises.firms[*i])
                    .filter(|f| f.site as usize == site)
                    .map(|f| crate::workshop_resolution::Job {
                        boundary: crate::resolution::Boundary {
                            month: self.month,
                            system: crate::resolution::System::Workshop,
                            site: f.site,
                            subject: f.id,
                            revision: 0,
                        },
                        expected: grants[site][f.family as usize],
                        wage: f.wage_rate,
                        family: f.family,
                    })
                    .collect();
                let eligible: Vec<_> = offers[site]
                    .iter()
                    .filter(|o| residents[site].contains(&(o.household as usize)))
                    .copied()
                    .collect();
                if let Some(pool) = &mut self.participation {
                    for staff in crate::workshop_resolution::resolve_market(pool, &jobs, &eligible)
                    {
                        matched.insert(staff.boundary.subject, staff);
                    }
                }
            }
        }
        for (_, i) in order {
            let f = &mut enterprises.firms[i];
            let site = f.site as usize;
            let boundary = crate::resolution::Boundary {
                month: self.month,
                system: crate::resolution::System::Workshop,
                site: f.site,
                subject: f.id,
                revision: crate::resolution::revision([
                    f.cash.to_bits(),
                    f.wage_rate.to_bits(),
                    (grants[site][f.family as usize] as f64).to_bits(),
                ]),
            };
            f.staffing = if refine {
                matched.remove(&f.id)
            } else if self.resolution.is_some() {
                let expected = grants[site][f.family as usize] as f64;
                Some(crate::workshop_resolution::Staffing {
                    boundary,
                    mode: crate::resolution::Mode::Aggregate,
                    expected,
                    granted: expected,
                    commitments: vec![],
                    wages: vec![],
                    settled: false,
                })
            } else {
                None
            };
        }
        for f in enterprises.firms.iter_mut().filter(|f| f.closed.is_none()) {
            let site = f.site as usize;
            let family = f.family as usize;
            let town = &mut self.sites[site];
            let ids = &residents[site];
            let units = town.economy.enterprise_lease[family] as f64;
            let desired = desired_work[f.id as usize];
            let work = f
                .staffing
                .as_ref()
                .map_or(grants[site][family] as f64, |s| {
                    work_floor(s.granted) as f64
                });
            let payroll = (work * f.wage_rate).min(f.cash);
            f.cash -= payroll;
            f.wages += payroll;
            f.paid_work += work;
            f.last_funded_work = work;
            f.last_completed_work = 0.;
            f.distressed_months = if work < desired * DISTRESS_MIN_STAFFING_FRACTION
                || units < DISTRESS_MIN_LEASE_UNITS
            {
                f.distressed_months + 1
            } else {
                0
            };
            town.economy.enterprise_plan[family] = work as f32;
            town.economy.enterprise_productivity[family] = f
                .staffing
                .as_ref()
                .zip(self.participation.as_ref())
                .map_or(0., |(staff, pool)| {
                    crate::workshop_resolution::productivity_bonus(pool, staff, family)
                });
            let weights = ids
                .iter()
                .map(|&id| {
                    if let Some(s) = f
                        .staffing
                        .as_ref()
                        .filter(|s| s.mode == crate::resolution::Mode::Individual)
                    {
                        s.wages
                            .iter()
                            .filter(|(household, _)| *household as usize == id)
                            .map(|(_, w)| *w)
                            .sum()
                    } else if e.occupational_payroll {
                        e.accounts[id].livelihood.unwrap_or([1.; 4])[3]
                    } else {
                        1.
                    }
                })
                .collect::<Vec<_>>();
            let total = weights.iter().sum::<f64>();
            let mut remaining = payroll;
            let last_paid = weights.iter().rposition(|w| *w > 0.);
            for (j, &id) in ids.iter().enumerate() {
                let wage = if weights[j] <= 0. {
                    0.
                } else if Some(j) == last_paid {
                    remaining
                } else {
                    (payroll * weights[j] / total.max(PAYROLL_WEIGHT_FLOOR)).min(remaining)
                };
                remaining -= wage;
                e.accounts[id].cash += wage;
                e.accounts[id].wages += wage;
                e.accounts[id].employer_income += wage;
            }
        }
        for f in enterprises.firms.iter_mut().filter(|f| f.closed.is_none()) {
            let revision = crate::workshop_resolution::staffing_revision(f);
            if let Some(staff) = &mut f.staffing {
                staff.boundary.revision = revision;
            }
        }
        self.enterprises = Some(enterprises);
        for (kind, site, text) in notices {
            self.event(kind, Some(site), None, text);
        }
    }
    pub(crate) fn settle_enterprises(&mut self) {
        let Some(mut enterprises) = self.enterprises.take() else {
            return;
        };
        let Some(e) = self
            .society
            .as_mut()
            .and_then(|s| s.household_economy.as_mut())
        else {
            self.enterprises = Some(enterprises);
            return;
        };
        let (covered, earned) = orders::settle(&mut enterprises, &mut self.sites, self.month);
        let mut invoices = vec![0.; self.sites.len()];
        for f in enterprises.firms.iter_mut().filter(|f| f.closed.is_none()) {
            let work =
                self.sites[f.site as usize].economy.enterprise_used[f.family as usize] as f64;
            f.last_completed_work = work;
            f.completed_work += work;
            invoices[f.site as usize] += (work - covered[f.id as usize]).max(0.)
                * f.service_rate
                    .unwrap_or(f.wage_rate * SERVICE_QUOTE_MULTIPLIER);
        }
        // Gather per-town invoices first. A common affordability factor prevents first-operator priority.
        let mut funds = vec![0.; self.sites.len()];
        for (site, invoice) in invoices.iter().enumerate() {
            let pool = &mut self.sites[site].economy.finance[0];
            funds[site] = withdraw(
                pool,
                invoice.min(*pool as f64 * SERVICE_MAX_TOWN_CASH_FRACTION),
            );
        }
        let mut remaining = funds.clone();
        let mut notices = vec![];
        let mut last = vec![None; self.sites.len()];
        for (i, f) in enterprises
            .firms
            .iter()
            .enumerate()
            .filter(|(_, f)| f.closed.is_none())
        {
            last[f.site as usize] = Some(i);
        }
        for (i, f) in enterprises
            .firms
            .iter_mut()
            .enumerate()
            .filter(|(_, f)| f.closed.is_none())
        {
            let site = f.site as usize;
            let invoice = (f.last_completed_work - covered[f.id as usize]).max(0.)
                * f.service_rate
                    .unwrap_or(f.wage_rate * SERVICE_QUOTE_MULTIPLIER);
            let received = if invoices[site] <= 0. {
                0.
            } else if last[site] == Some(i) {
                remaining[site]
            } else {
                (funds[site] * invoice / invoices[site]).min(remaining[site])
            };
            remaining[site] -= received;
            f.written_off += (invoice - received).max(0.);
            let received = received + earned[f.id as usize];
            f.cash += received;
            f.revenue += received;
            let surplus = distributable_profit(f);
            f.cash -= surplus;
            f.dividends += surplus;
            e.accounts[f.owner as usize].cash += surplus;
            e.accounts[f.owner as usize].dividends += surplus;
            f.idle_months = if f.last_completed_work
                < f.last_funded_work.max(MIN_IDLE_FUNDED_WORK) * IDLE_COMPLETION_THRESHOLD
            {
                f.idle_months + 1
            } else {
                0
            };
            if let (Some(policy), Some(staff), Some(service)) =
                (&mut f.wage_policy, &f.staffing, f.service_rate)
            {
                if staff.mode == crate::resolution::Mode::Individual {
                    policy.observe(
                        self.month,
                        [staff.expected, f.last_funded_work, f.last_completed_work],
                        f.cash,
                        service / SERVICE_QUOTE_MULTIPLIER,
                        received,
                    );
                }
            }
            let reason = if f.distressed_months >= DISTRESS_CLOSURE_MONTHS {
                Some("working capital exhausted")
            } else if f.idle_months >= IDLE_CLOSURE_MONTHS {
                Some("six months without sufficient orders or inputs")
            } else {
                None
            };
            if let Some(reason) = reason {
                close(
                    f,
                    &mut e.accounts[f.owner as usize],
                    self.month,
                    reason,
                    self.credit.operator_has_debt(f.id),
                );
                notices.push((f.site,format!("Workshop operator {} closed: {}; revenue {:.2}, wages {:.2}, rent {:.2}, liquidation {:.2}; lease returned to communal production",f.id,reason,f.revenue,f.wages,f.rent,f.liquidation)));
            }
        }
        self.enterprises = Some(enterprises);
        for (site, text) in notices {
            self.event("enterprise_closed", Some(site), None, text);
        }
    }
}
impl crate::gpu::Generator {
    pub fn set_enterprises(&mut self, enabled: bool) -> Result<()> {
        ensure!(
            self.progress.stage == crate::gpu::Stage::Boundary,
            "enterprise policy requires completed boundary"
        );
        self.validate_living_boundary()?;
        let h = self
            .civilizations
            .as_mut()
            .ok_or_else(|| anyhow::anyhow!("found civilizations first"))?;
        ensure!(
            h.society
                .as_ref()
                .is_some_and(|s| s.household_economy.is_some()),
            "enable household economy first"
        );
        let starting_baseline = h.enterprises.is_none();
        let e = h.enterprises.get_or_insert_with(Default::default);
        if starting_baseline || e.enabled != enabled {
            e.enabled = enabled;
            h.event("enterprise_policy",None,None,format!("Private workshop operators {}; existing accounts settle at the next monthly boundary",if enabled {"enabled"} else {"disabled"}));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        catalog::Catalog,
        config::Config,
        economy::EconomyCatalog,
        gpu::{ContextGpu, Generator},
    };
    #[test]
    fn wage_offers_wait_for_next_posting_and_need_productive_affordable_shortage() {
        let mut productive = WagePolicy::default();
        productive.observe(1, [4., 1., 1.], 1000., 10., 12.5);
        assert_eq!(productive.multiplier, 1.);
        assert!(productive.pending.unwrap() > 1.);
        let before = serde_json::to_value(&productive).unwrap();
        productive.observe(1, [2., 1., 0.], 0., 10., 0.);
        assert_eq!(before, serde_json::to_value(&productive).unwrap());
        let mut idle = WagePolicy::default();
        idle.observe(1, [2., 1., 0.], 1000., 10., 0.);
        assert!(idle.pending.unwrap() < 1.);
        let mut poor = WagePolicy::default();
        poor.observe(1, [4., 1., 1.], 0., 10., 12.5);
        assert_eq!(poor.pending, Some(1.));
        let mut resumed: WagePolicy = serde_json::from_value(before).unwrap();
        for month in 2..500 {
            for p in [&mut productive, &mut resumed] {
                p.multiplier = p.pending.take().unwrap();
                p.observe(month, [2., 1., 1.], 1000., 10., 12.5);
            }
        }
        assert_eq!(
            serde_json::to_value(&productive).unwrap(),
            serde_json::to_value(resumed).unwrap()
        );
        assert!(productive.pending.unwrap() <= 1.125);
    }

    #[test]
    #[ignore = "requires hardware GPU"]
    fn posted_wage_changes_payroll_but_not_the_service_quote() {
        let mut g = world();
        install(&mut g);
        g.enable_politics().unwrap();
        let h = g.civilizations.as_mut().unwrap();
        h.enable_individual_demography().unwrap();
        h.set_workshop_refinement(true).unwrap();
        h.month = 3;
        h.begin_service_reservations();
        h.prepare_enterprises();
        h.settle_enterprises();
        h.settle_workshop_resolutions().unwrap();
        assert!(!h.enterprises.as_ref().unwrap().firms.is_empty());
        h.month = 4;
        h.begin_service_reservations();
        let mut higher = h.clone();
        for f in &mut h.enterprises.as_mut().unwrap().firms {
            f.wage_policy.as_mut().unwrap().pending = Some(1.);
        }
        for f in &mut higher.enterprises.as_mut().unwrap().firms {
            f.wage_policy.as_mut().unwrap().pending = Some(1.1);
        }
        h.prepare_enterprises();
        higher.prepare_enterprises();
        for (low, high) in h
            .enterprises
            .as_ref()
            .unwrap()
            .firms
            .iter()
            .zip(&higher.enterprises.as_ref().unwrap().firms)
        {
            if low.closed.is_none() {
                assert!((high.wage_rate / low.wage_rate - 1.1).abs() < 1e-6);
                assert_eq!(low.service_rate, high.service_rate);
            }
        }
        // Hold completed production identical to isolate the contracted fee.
        for (low, high) in h.sites.iter_mut().zip(&mut higher.sites) {
            for family in 0..4 {
                let used = low.economy.enterprise_plan[family]
                    .min(high.economy.enterprise_plan[family])
                    * 0.5;
                low.economy.enterprise_used[family] = used;
                high.economy.enterprise_used[family] = used;
            }
        }
        h.settle_enterprises();
        higher.settle_enterprises();
        for (low, high) in h
            .enterprises
            .as_ref()
            .unwrap()
            .firms
            .iter()
            .zip(&higher.enterprises.as_ref().unwrap().firms)
        {
            assert!((low.revenue - high.revenue).abs() < 1e-6);
        }
        h.settle_workshop_resolutions().unwrap();
        higher.settle_workshop_resolutions().unwrap();
        h.enterprises.as_ref().unwrap().validate(h).unwrap();
        higher
            .enterprises
            .as_ref()
            .unwrap()
            .validate(&higher)
            .unwrap();
    }

    #[test]
    #[ignore = "requires hardware GPU"]
    fn coworkers_learn_only_from_actual_shared_work_and_resume_identically() {
        let mut g = world();
        install(&mut g);
        g.enable_politics().unwrap();
        let h = g.civilizations.as_mut().unwrap();
        h.enable_individual_demography().unwrap();
        h.set_workshop_refinement(true).unwrap();
        h.month = 3;
        h.begin_service_reservations();
        let ids: Vec<_> = h
            .participation
            .as_ref()
            .unwrap()
            .residents
            .values()
            .filter(|r| {
                r.presence == crate::participation::Presence::Resident(0) && r.capacity > 0.
            })
            .take(2)
            .map(|r| r.person)
            .collect();
        assert_eq!(ids.len(), 2);
        for r in h.participation.as_mut().unwrap().residents.values_mut() {
            r.capacity = 0.;
            r.care = 0.;
        }
        for &id in &ids {
            h.participation
                .as_mut()
                .unwrap()
                .residents
                .get_mut(&id)
                .unwrap()
                .capacity = 0.001;
        }
        let mentor = h
            .participation
            .as_mut()
            .unwrap()
            .residents
            .get_mut(&ids[0])
            .unwrap();
        mentor.workshop_practice = [24.; 4];
        mentor.workshop_completed = 96.;
        h.prepare_enterprises();
        let mut idle = h.clone();
        let mut resumed: History =
            serde_json::from_value(serde_json::to_value(&*h).unwrap()).unwrap();
        for world in [&mut *h, &mut resumed] {
            for site in &mut world.sites {
                site.economy.enterprise_used = site.economy.enterprise_plan.map(|w| w * 0.5);
            }
            world.settle_enterprises();
            world.settle_workshop_resolutions().unwrap();
        }
        for site in &mut idle.sites {
            site.economy.enterprise_used = [0.; 4];
        }
        idle.settle_enterprises();
        idle.settle_workshop_resolutions().unwrap();
        assert_eq!(
            serde_json::to_value(&*h).unwrap(),
            serde_json::to_value(resumed).unwrap()
        );
        let pool = h.participation.as_ref().unwrap();
        assert!(pool.residents[&ids[1]]
            .workshop_learning
            .iter()
            .any(|v| *v > 0.));
        assert_eq!(pool.residents[&ids[0]].workshop_learning, [0.; 4]);
        assert_eq!(
            idle.participation.as_ref().unwrap().residents[&ids[1]].workshop_learning,
            [0.; 4]
        );
        let before = serde_json::to_value(&h.participation).unwrap();
        h.settle_workshop_resolutions().unwrap();
        assert_eq!(before, serde_json::to_value(&h.participation).unwrap());
        h.enterprises.as_ref().unwrap().validate(h).unwrap();
    }

    #[test]
    fn competing_workshops_share_labor_before_payroll() {
        assert_eq!(allocate_work([6., 3., 0., 0.], 3.), [2., 1., 0., 0.]);
        assert_eq!(allocate_work([3., 6., 0., 0.], 3.), [1., 2., 0., 0.]);
        assert_eq!(allocate_work([0.; 4], 3.), [0.; 4]);
        assert_eq!(allocate_work([6.; 4], 0.), [0.; 4]);
        // Cash-limited claims leave workers available to other operators.
        assert_eq!(allocate_work([0.5, 1., 0., 0.], 3.), [0.5, 1., 0., 0.]);
        for capacity in [0.001, 0.1, 1., 10., 100.] {
            let requests = [0.13, 7.7, 12.1, 0.003];
            let grants = allocate_work(requests, capacity);
            assert!(grants.iter().map(|&x| x as f64).sum::<f64>() <= capacity);
            for (grant, request) in grants.into_iter().zip(requests) {
                assert!(grant as f64 <= request);
            }
        }
    }
    #[test]
    #[ignore = "requires hardware GPU"]
    fn credit_account_transfers_preserve_cash_and_operator_income() {
        use crate::credit::{Account, SHARED_CURRENCY};
        let mut g = world();
        install(&mut g);
        let h = g.civilizations.as_mut().unwrap();
        h.month = 3;
        h.prepare_enterprises();
        let firm = h
            .enterprises
            .as_ref()
            .unwrap()
            .firms
            .iter()
            .find(|f| f.closed.is_none() && f.cash > 1.)
            .unwrap()
            .id;
        let from = Account::Operator(firm);
        let to = Account::Town(0);
        let baseline = h.money_residual();
        let revenue = h.enterprises.as_ref().unwrap().firms[firm as usize].revenue;
        let paid = h
            .transfer_credit_cash(from, to, SHARED_CURRENCY, 1., 0.)
            .unwrap();
        assert_eq!(paid.amount(), 1.);
        assert_eq!(
            h.enterprises.as_ref().unwrap().firms[firm as usize].revenue,
            revenue
        );
        h.enterprises.as_ref().unwrap().validate(h).unwrap();
        assert!((h.money_residual() - baseline).abs() < 1e-10);
        let before = serde_json::to_value(&*h).unwrap();
        assert!(h
            .transfer_credit_cash(to, Account::Operator(u32::MAX), SHARED_CURRENCY, 1., 0.)
            .is_err());
        assert_eq!(before, serde_json::to_value(&*h).unwrap());
        let mut restored: History = serde_json::from_value(before).unwrap();
        for history in [&mut *h, &mut restored] {
            let receipt = history
                .transfer_credit_cash(to, from, SHARED_CURRENCY, 0.75, 0.25)
                .unwrap();
            assert_eq!(receipt.principal, 0.75);
            assert_eq!(receipt.interest, 0.25);
            history
                .enterprises
                .as_ref()
                .unwrap()
                .validate(history)
                .unwrap();
            assert!((history.money_residual() - baseline).abs() < 1e-10);
            let f = &history.enterprises.as_ref().unwrap().firms[firm as usize];
            assert_eq!(f.revenue, revenue);
            assert_eq!(f.financing.net_cash(), 0.);
            assert_eq!(f.financing.net_interest(), 0.25);
        }
        assert_eq!(
            serde_json::to_value(h).unwrap(),
            serde_json::to_value(restored).unwrap()
        );
    }

    #[test]
    #[ignore = "requires hardware GPU"]
    fn council_tax_base_loss_changes_collection_and_default_without_creating_cash() {
        use crate::credit::{Account, RepaymentSource, Status, Terms, SHARED_CURRENCY};
        let mut g = world();
        let h = g.civilizations.as_mut().unwrap();
        // Observe real annual taxes before contracting a small bridge.
        h.month = 12;
        for site in &mut h.sites {
            site.stocks.stock[3] = 0.;
        }
        h.social_year();
        h.observe_credit_taxes();
        h.month = 13;
        let evidence = h.council_credit_evidence(0).unwrap();
        assert!(evidence.expected_receipts > 20.);
        let opening = h.credit_account_cash(Account::Council(0)).unwrap();
        h.transfer_credit_cash(
            Account::Council(0),
            Account::Town(0),
            SHARED_CURRENCY,
            opening,
            0.,
        )
        .unwrap();
        h.commit_credit_loan(
            Terms {
                lender: Account::Town(0),
                borrower: Account::Council(0),
                currency: SHARED_CURRENCY,
                source: RepaymentSource::AnnualTax {
                    council: 0,
                    collection_month: 24,
                },
                annual_simple_rate: 0.,
                maturity_month: 25,
                grace_months: 3,
            },
            10.,
        )
        .unwrap();
        // A declared fixture expenditure, not simulated administration: move
        // the bridge to a different existing account so it must be repaid from taxes.
        h.transfer_credit_cash(
            Account::Council(0),
            Account::Town(1),
            SHARED_CURRENCY,
            10.,
            0.,
        )
        .unwrap();
        assert_eq!(h.credit_account_cash(Account::Council(0)).unwrap(), 0.);
        h.credit.servicing_policy.available_cash_share = 1.;
        let baseline = h.clone();
        let residual = baseline.money_residual();
        for lost_base in [false, true] {
            let mut branch = baseline.clone();
            if lost_base {
                let controlled: Vec<_> = branch
                    .sites
                    .iter()
                    .filter(|s| branch.controller(s.id) == 0)
                    .map(|s| s.id)
                    .collect();
                for id in controlled {
                    branch.sites[id as usize].abandoned = true;
                }
                assert_eq!(
                    branch.council_credit_evidence(0).unwrap().expected_receipts,
                    0.
                );
            }
            let mut resumed: History =
                serde_json::from_value(serde_json::to_value(&branch).unwrap()).unwrap();
            for world in [&mut branch, &mut resumed] {
                for month in 14..=28 {
                    world.month = month;
                    world.service_credit_month().unwrap();
                    if month == 24 {
                        // Actual Respond-phase collection follows Open servicing.
                        world.social_year();
                        world.observe_credit_taxes();
                        let paid: f64 = world
                            .society
                            .as_ref()
                            .unwrap()
                            .council_funding
                            .taxes
                            .iter()
                            .filter(|r| r.council == 0)
                            .map(|r| f64::from(r.paid))
                            .sum();
                        if lost_base {
                            assert_eq!(paid, 0.);
                        } else {
                            assert!(paid > 10.);
                        }
                        assert!(world.credit.loans[0].outstanding_principal > 0.);
                    }
                    world.validate_credit().unwrap();
                    assert!((world.money_residual() - residual).abs() < 1e-7);
                }
                assert_eq!(
                    world.credit.loans[0].status,
                    if lost_base {
                        Status::Defaulted
                    } else {
                        Status::Repaid
                    }
                );
                assert_eq!(world.credit.loans.len(), 1);
                assert_eq!(world.credit.issuance.total_issued(), 0.);
            }
            assert_eq!(
                serde_json::to_value(branch).unwrap(),
                serde_json::to_value(resumed).unwrap()
            );
        }
    }

    #[test]
    #[ignore = "requires hardware GPU"]
    fn persisted_credit_commits_cash_debt_and_failed_collection_together() {
        use crate::credit::{Account, RepaymentSource, Status, Terms, SHARED_CURRENCY};
        let mut g = world();
        install(&mut g);
        let h = g.civilizations.as_mut().unwrap();
        h.month = 3;
        h.prepare_enterprises();
        let firm = h
            .enterprises
            .as_ref()
            .unwrap()
            .firms
            .iter()
            .find(|f| f.closed.is_none() && f.cash > 10.)
            .unwrap()
            .id;
        let mut old = serde_json::to_value(&*h).unwrap();
        old.as_object_mut().unwrap().remove("credit");
        let migrated: History = serde_json::from_value(old).unwrap();
        assert!(migrated.credit.loans.is_empty());
        let before = serde_json::to_value(&*h).unwrap();
        let terms = Terms {
            lender: Account::Operator(firm),
            borrower: Account::Town(0),
            currency: SHARED_CURRENCY,
            source: RepaymentSource::Export {
                contract: 0,
                payment_month: 15,
            },
            annual_simple_rate: 0.12,
            maturity_month: 15,
            grace_months: 3,
        };
        assert!(h.commit_credit_loan(terms.clone(), f64::NAN).is_err());
        assert_eq!(serde_json::to_value(&*h).unwrap(), before);
        let baseline = h.money_residual();
        let id = h.commit_credit_loan(terms, 10.).unwrap().unwrap();
        let principal = h.credit.loans[id as usize].original_principal;
        assert!(principal > 9.99 && principal <= 10.);
        assert_eq!(principal, h.credit.cash_receipts[0].transfer.amount());
        h.validate_credit().unwrap();
        assert!((h.money_residual() - baseline).abs() < 1e-10);
        let mut resumed: History =
            serde_json::from_value(serde_json::to_value(&*h).unwrap()).unwrap();
        for history in [&mut *h, &mut resumed] {
            history.month = 15;
            let paid = history.pay_credit_loan(id, principal).unwrap();
            assert!(paid > 0. && paid <= 10.);
            assert!(history.credit.loans[id as usize].outstanding_principal > 0.);
            assert_eq!(history.credit.loans[id as usize].status, Status::Arrears);
            history.validate_credit().unwrap();
            history
                .enterprises
                .as_ref()
                .unwrap()
                .validate(history)
                .unwrap();
            assert!((history.money_residual() - baseline).abs() < 1e-10);
        }
        assert_eq!(
            serde_json::to_value(&*h).unwrap(),
            serde_json::to_value(resumed).unwrap()
        );
        let mut corrupted = h.clone();
        corrupted.credit.cash_receipts.remove(0);
        assert!(corrupted.validate_credit().is_err());
    }

    #[test]
    #[ignore = "requires hardware GPU"]
    fn funded_service_orders_pay_completed_work_and_refund_without_double_invoicing() {
        let mut g = world();
        install(&mut g);
        let h = g.civilizations.as_mut().unwrap();
        h.month = 3;
        h.begin_service_reservations();
        h.prepare_enterprises();
        h.settle_enterprises();
        let firm = h
            .enterprises
            .as_ref()
            .unwrap()
            .firms
            .iter()
            .find(|f| f.closed.is_none())
            .unwrap()
            .id;
        let site = h.enterprises.as_ref().unwrap().firms[firm as usize].site as usize;
        // Existing town cash funds the fee; this is not an arrival import.
        let transfer = withdraw(&mut h.sites[1].economy.finance[0], 100.);
        let added = deposit(&mut h.sites[site].economy.finance[0], transfer);
        h.society.as_mut().unwrap().councils[0].treasury += transfer - added;
        let money = h.money_residual();
        let before = serde_json::to_value(&*h).unwrap();
        for (work, due) in [(f64::NAN, 4), (0., 4), (1., 3), (1., 16)] {
            assert!(h.fund_workshop_order(firm, work, due).is_err());
            assert_eq!(before, serde_json::to_value(&*h).unwrap());
        }
        let id = h.fund_workshop_order(firm, 0.01, 4).unwrap();
        assert!(h.fund_workshop_order(firm, 0.01, 4).is_err());
        assert!((h.money_residual() - money).abs() < 1e-7);
        let posted = h.clone();
        for fraction in [0., 0.5, 1.] {
            let mut world = posted.clone();
            world.month = 4;
            world.begin_service_reservations();
            world.prepare_enterprises();
            let order = &world.enterprises.as_ref().unwrap().orders[id as usize];
            let work = order.funded_work * fraction;
            let family = world.enterprises.as_ref().unwrap().firms[firm as usize].family as usize;
            assert!(
                world.enterprises.as_ref().unwrap().firms[firm as usize].last_funded_work >= work
            );
            world.sites[site].economy.enterprise_used[family] = work as f32;
            // Production completion is an explicit fixture; this tests fee
            // settlement, not whether the GPU manufactured goods for that work.
            let revenue = world.enterprises.as_ref().unwrap().firms[firm as usize].revenue;
            let mut resumed: History =
                serde_json::from_value(serde_json::to_value(&world).unwrap()).unwrap();
            world.settle_enterprises();
            resumed.settle_enterprises();
            assert_eq!(
                serde_json::to_value(&world).unwrap(),
                serde_json::to_value(&resumed).unwrap()
            );
            let enterprises = world.enterprises.as_ref().unwrap();
            let order = &enterprises.orders[id as usize];
            assert_eq!(order.settled, Some(4));
            assert!((order.paid - order.completed_work * order.price_per_work).abs() < 1e-9);
            assert!((enterprises.firms[firm as usize].revenue - revenue - order.paid).abs() < 1e-5);
            assert!((order.funded - order.paid - order.refunded - order.escrow).abs() < 1e-9);
            enterprises.validate(&world).unwrap();
            assert!((world.money_residual() - money).abs() < 1e-6);
            let mut corrupt = world.clone();
            corrupt.enterprises.as_mut().unwrap().orders[id as usize].escrow += 1.;
            assert!(corrupt
                .enterprises
                .as_ref()
                .unwrap()
                .validate(&corrupt)
                .is_err());
        }
        for abandoned in [false, true] {
            let mut cancelled = posted.clone();
            cancelled.month = 4;
            if abandoned {
                cancelled.sites[site].abandoned = true;
            } else {
                cancelled.enterprises.as_mut().unwrap().firms[firm as usize].closed = Some(4);
            }
            cancelled.settle_enterprises();
            let order = &cancelled.enterprises.as_ref().unwrap().orders[id as usize];
            assert_eq!(order.paid, 0.);
            assert_eq!(order.settled, Some(4));
            assert!((order.funded - order.refunded - order.escrow).abs() < 1e-9);
            assert!((cancelled.money_residual() - money).abs() < 1e-6);
        }
        // Missing the contracted month cannot pay for unrelated later work.
        let mut late = posted.clone();
        late.month = 5;
        late.settle_enterprises();
        let order = &late.enterprises.as_ref().unwrap().orders[id as usize];
        assert_eq!(order.paid, 0.);
        assert_eq!(order.settled, Some(5));
        assert!((late.money_residual() - money).abs() < 1e-6);
    }

    #[test]
    #[ignore = "requires hardware GPU"]
    fn funded_service_receipt_supports_only_bounded_operator_credit() {
        let mut g = world();
        install(&mut g);
        let h = g.civilizations.as_mut().unwrap();
        h.month = 3;
        h.begin_service_reservations();
        h.prepare_enterprises();
        h.settle_enterprises();
        let firm = h
            .enterprises
            .as_ref()
            .unwrap()
            .firms
            .iter()
            .find(|f| f.closed.is_none())
            .unwrap()
            .id;
        let f = &mut h.enterprises.as_mut().unwrap().firms[firm as usize];
        let site = f.site as usize;
        // Explicit opening capitalization fixture: move the remaining invested
        // cash back to its owner, preserving both cash and equity ledgers.
        let returned = f.cash;
        assert!(returned <= f.capital);
        f.cash = 0.;
        f.capital -= returned;
        let owner = &mut h
            .society
            .as_mut()
            .unwrap()
            .household_economy
            .as_mut()
            .unwrap()
            .accounts[f.owner as usize];
        owner.cash += returned;
        owner.capital_invested -= returned;
        // A deliberately valuable service contract, not a new default fee.
        f.service_rate = Some(1000.);
        let transfer = withdraw(&mut h.sites[1].economy.finance[0], 1000.);
        let added = deposit(&mut h.sites[site].economy.finance[0], transfer);
        h.society.as_mut().unwrap().councils[0].treasury += transfer - added;
        let money = h.money_residual();
        h.fund_workshop_order(firm, 0.25, 4).unwrap();
        h.month = 4;
        h.credit.commercial_policy.enabled = true;
        h.credit.commercial_policy.service_orders = true;
        let mut production_opening = h.clone();
        production_opening.month = 3;
        let evidence = h.service_order_credit_evidence(0.05).unwrap();
        assert_eq!(evidence.len(), 1);
        assert!(evidence[0].expected_receipts > evidence[0].operating_costs);
        assert!(evidence[0].operating_costs > 0.);
        h.enterprises.as_ref().unwrap().validate(h).unwrap();
        let mut disabled = h.clone();
        disabled.credit.commercial_policy.service_orders = false;
        assert_eq!(disabled.commercial_credit_month().unwrap(), 0);
        let mut lost = h.clone();
        lost.sites[site].economy.workshop_types
            [h.enterprises.as_ref().unwrap().firms[firm as usize].family as usize][0] = 0.;
        assert!(lost.service_order_credit_evidence(0.05).unwrap().is_empty());
        assert_eq!(lost.commercial_credit_month().unwrap(), 0);
        let mut costly = h.clone();
        costly.sites[site].economy.prices[crate::economy::FOOD] = 1000.;
        assert_eq!(costly.commercial_credit_month().unwrap(), 0);
        assert_eq!(h.commercial_credit_month().unwrap(), 1);
        let loan = &h.credit.loans[0];
        assert_eq!(loan.terms.borrower, crate::credit::Account::Operator(firm));
        assert_eq!(loan.terms.lender, crate::credit::Account::Town(site as u32));
        assert!(loan.original_principal <= evidence[0].operating_costs);
        assert_eq!(h.enterprises.as_ref().unwrap().orders[0].paid, 0.);
        assert!((h.money_residual() - money).abs() < 1e-6);
        h.validate_credit().unwrap();
        h.enterprises.as_ref().unwrap().validate(h).unwrap();
        let mut resumed: History =
            serde_json::from_value(serde_json::to_value(&*h).unwrap()).unwrap();
        assert_eq!(resumed.commercial_credit_month().unwrap(), 0);
        assert_eq!(
            serde_json::to_value(&*h).unwrap(),
            serde_json::to_value(&resumed).unwrap()
        );
        // Now exercise the full monthly coordinator and actual GPU production
        // from the pre-Reserve snapshot, without supplying completion amounts.
        g.civilizations = Some(production_opening);
        let path =
            std::env::temp_dir().join(format!("service-credit-{}.world", std::process::id()));
        g.save(&path).unwrap();
        let mut control = Generator::load(g.gpu.clone(), &path).unwrap();
        let mut checkpoint = Generator::load(g.gpu.clone(), &path).unwrap();
        std::fs::remove_file(path).unwrap();
        control
            .civilizations
            .as_mut()
            .unwrap()
            .credit
            .commercial_policy
            .service_orders = false;
        g.advance_history(1).unwrap();
        control.advance_history(1).unwrap();
        checkpoint.advance_history(1).unwrap();
        let funded = g.civilizations.as_ref().unwrap();
        let unfunded = control.civilizations.as_ref().unwrap();
        let operator = &funded.enterprises.as_ref().unwrap().firms[firm as usize];
        let other = &unfunded.enterprises.as_ref().unwrap().firms[firm as usize];
        assert!(operator.last_completed_work > other.last_completed_work);
        assert!(operator.last_funded_work > other.last_funded_work);
        assert!(funded.enterprises.as_ref().unwrap().orders[0].paid > 0.);
        assert_eq!(unfunded.enterprises.as_ref().unwrap().orders[0].paid, 0.);
        assert_eq!(
            serde_json::to_value(funded).unwrap(),
            serde_json::to_value(checkpoint.civilizations.as_ref().unwrap()).unwrap()
        );
        assert!((funded.money_residual() - money).abs() < 1e-6);
    }

    #[test]
    #[ignore = "requires hardware GPU"]
    fn automatic_service_procurement_reserves_surplus_once_and_resumes() {
        let mut g = world();
        install(&mut g);
        let h = g.civilizations.as_mut().unwrap();
        h.month = 3;
        h.begin_service_reservations();
        h.plan_production();
        h.prepare_enterprises();
        h.settle_enterprises();
        let firm = h
            .enterprises
            .as_ref()
            .unwrap()
            .firms
            .iter()
            .find(|f| f.closed.is_none())
            .unwrap()
            .id;
        let site = h.enterprises.as_ref().unwrap().firms[firm as usize].site as usize;
        let moved = withdraw(&mut h.sites[1].economy.finance[0], 1000.);
        let deposited = deposit(&mut h.sites[site].economy.finance[0], moved);
        h.society.as_mut().unwrap().councils[0].treasury += moved - deposited;
        assert_eq!(h.procure_workshop_services().unwrap(), 0);
        h.enterprises.as_mut().unwrap().procurement.enabled = true;
        let mut no_cash = h.clone();
        no_cash
            .enterprises
            .as_mut()
            .unwrap()
            .procurement
            .cash_reserve = 1e9;
        assert_eq!(no_cash.procure_workshop_services().unwrap(), 0);
        let mut no_demand = h.clone();
        for s in &mut no_demand.sites {
            s.economy.orders.fill(0.);
        }
        assert_eq!(no_demand.procure_workshop_services().unwrap(), 0);
        let money = h.money_residual();
        let opening_cash = f64::from(h.sites[site].economy.finance[0]);
        let reserve = h.commercial_input_costs()[site].max(100.);
        assert!(h.procure_workshop_services().unwrap() > 0);
        let e = h.enterprises.as_ref().unwrap();
        let spent: f64 = e.procurement.claims.iter().map(|c| c.funded).sum();
        assert!(spent <= (opening_cash - reserve).max(0.) * 0.25);
        assert!(e.orders.iter().all(|o| o.due == 4 && o.paid == 0.));
        assert!((h.money_residual() - money).abs() < 1e-6);
        e.validate(h).unwrap();
        let before = serde_json::to_value(&*h).unwrap();
        assert_eq!(h.procure_workshop_services().unwrap(), 0);
        assert_eq!(before, serde_json::to_value(&*h).unwrap());
        let mut corrupt = h.clone();
        corrupt.enterprises.as_mut().unwrap().procurement.claims[0].funded += 1.;
        assert!(corrupt
            .enterprises
            .as_ref()
            .unwrap()
            .validate(&corrupt)
            .is_err());
        let path =
            std::env::temp_dir().join(format!("service-procurement-{}.world", std::process::id()));
        g.save(&path).unwrap();
        let mut resumed = Generator::load(g.gpu.clone(), &path).unwrap();
        std::fs::remove_file(path).unwrap();
        g.advance_history(3).unwrap();
        for _ in 0..3 {
            resumed.advance_history(1).unwrap();
        }
        let h = g.civilizations.as_ref().unwrap();
        assert_eq!(
            h.enterprises.as_ref().unwrap().procurement.last_month,
            Some(6)
        );
        assert!(h.enterprises.as_ref().unwrap().orders[0].settled.is_some());
        assert_eq!(
            serde_json::to_value(h).unwrap(),
            serde_json::to_value(resumed.civilizations.as_ref().unwrap()).unwrap()
        );
        assert!((h.money_residual() - money).abs() < 1e-6);
    }

    #[test]
    #[ignore = "requires hardware GPU; normal-fee credit comparison"]
    fn normal_service_fee_credit_comparison() {
        for seed in [17, 81, 256] {
            for requested_work in [0.25, 1., 2.] {
                let mut g = world_seed(seed);
                install(&mut g);
                let h = g.civilizations.as_mut().unwrap();
                h.month = 3;
                h.begin_service_reservations();
                h.prepare_enterprises();
                h.settle_enterprises();
                let f = h
                    .enterprises
                    .as_mut()
                    .unwrap()
                    .firms
                    .iter_mut()
                    .find(|f| f.closed.is_none())
                    .unwrap();
                let firm = f.id;
                let site = f.site as usize;
                // Matched opening liquidity gap, with the original service quote.
                // As in the causal fixture, returned capital remains owned cash.
                let returned = f.cash;
                assert!(returned <= f.capital);
                f.cash = 0.;
                f.capital -= returned;
                let owner = &mut h
                    .society
                    .as_mut()
                    .unwrap()
                    .household_economy
                    .as_mut()
                    .unwrap()
                    .accounts[f.owner as usize];
                owner.cash += returned;
                owner.capital_invested -= returned;
                let transfer = withdraw(&mut h.sites[1].economy.finance[0], 1000.);
                let added = deposit(&mut h.sites[site].economy.finance[0], transfer);
                h.society.as_mut().unwrap().councils[0].treasury += transfer - added;
                h.fund_workshop_order(firm, requested_work, 4).unwrap();
                h.credit.commercial_policy.enabled = true;
                h.credit.commercial_policy.service_orders = true;
                let opening_money = h.money_residual();
                let mut forecast = h.clone();
                forecast.month = 4;
                let evidence = forecast.service_order_credit_evidence(0.05).unwrap();
                assert_eq!(evidence.len(), 1);
                let path = std::env::temp_dir().join(format!(
                    "normal-service-credit-{}-{seed}.world",
                    std::process::id()
                ));
                g.save(&path).unwrap();
                let mut control = Generator::load(g.gpu.clone(), &path).unwrap();
                std::fs::remove_file(path).unwrap();
                control
                    .civilizations
                    .as_mut()
                    .unwrap()
                    .credit
                    .commercial_policy
                    .service_orders = false;
                for (enabled, world) in [(true, &mut g), (false, &mut control)] {
                    let mut maximum_money_error = 0_f64;
                    for _ in 0..12 {
                        world.advance_history(1).unwrap();
                        let h = world.civilizations.as_ref().unwrap();
                        h.validate_credit().unwrap();
                        h.enterprises.as_ref().unwrap().validate(h).unwrap();
                        maximum_money_error =
                            maximum_money_error.max((h.money_residual() - opening_money).abs());
                    }
                    let h = world.civilizations.as_ref().unwrap();
                    let f = &h.enterprises.as_ref().unwrap().firms[firm as usize];
                    let order = &h.enterprises.as_ref().unwrap().orders[0];
                    assert!(order.settled.is_some());
                    assert!(maximum_money_error < 1e-6);
                    println!(
                        "normal_service_credit {}",
                        serde_json::json!({
                            "seed": seed, "credit": enabled, "requested_work": requested_work,
                            "quoted_fee": order.price_per_work,
                            "expected_receipts": evidence[0].expected_receipts,
                            "operating_costs": evidence[0].operating_costs,
                            "loans": h.credit.loans.len(),
                            "principal": h.credit.loans.iter().map(|l| l.original_principal).sum::<f64>(),
                            "order_paid": order.paid, "order_refunded": order.refunded,
                            "order_escrow": order.escrow,
                            "completed_work": f.completed_work, "wages": f.wages,
                            "closed": f.closed, "maximum_money_error": maximum_money_error,
                        })
                    );
                }
            }
        }
    }

    fn world() -> Generator {
        world_seed(Config::default().seed)
    }
    fn world_seed(seed: u32) -> Generator {
        let mut g = Generator::new(
            pollster::block_on(ContextGpu::headless()).unwrap(),
            Config {
                resolution: 32,
                ecology_resolution: 16,
                seed,
                ..Default::default()
            },
            Catalog::bundled().unwrap(),
        )
        .unwrap();
        g.found_civilizations(5).unwrap();
        g.enable_society().unwrap();
        g
    }
    /// Matched reservation-only counterfactuals: synthetic installed port at the
    /// workshop site isolates priority from route geography. No harvest/population
    /// conclusions are inferred from this single-boundary experiment.
    #[test]
    #[ignore = "requires hardware GPU"]
    fn service_scarcity_priority_comparison() {
        for seed in [17, 81, 256] {
            let mut g = world_seed(seed);
            install(&mut g);
            g.enable_shipping().unwrap();
            let h = g.civilizations.as_mut().unwrap();
            h.month = 3;
            h.prepare_enterprises();
            assert_eq!(h.enterprises.as_ref().unwrap().firms.len(), 1);
            // Explicit fixture capital import, identical in every arm. No transactions
            // may create money after this baseline is captured.
            h.sites[0].economy.finance[0] = 10000.;
            h.shipping.as_mut().unwrap().ports = vec![crate::shipping::Port {
                fleet: Some(Default::default()),
                work: None,
                site: 0,
                access: vec![],
                water_cell: 0,
                access_km: 0.,
                assets: [200., 10., 100.],
                commissioned: Some(3),
                flood_months: 0,
            }];
            let base = h.clone();
            let money = |h: &History| {
                h.sites
                    .iter()
                    .map(|s| s.economy.finance[0] as f64)
                    .sum::<f64>()
                    + h.society
                        .as_ref()
                        .unwrap()
                        .household_economy
                        .as_ref()
                        .unwrap()
                        .accounts
                        .iter()
                        .map(|a| a.cash)
                        .sum::<f64>()
                    + h.enterprises
                        .as_ref()
                        .unwrap()
                        .firms
                        .iter()
                        .map(|f| f.cash)
                        .sum::<f64>()
            };
            println!(
                "seed,adults,illness,culture,order,ceiling,cultural,enterprise,crew,freight,idle"
            );
            for adults in [2., 8., 20., 100.] {
                for illness in [0., 0.5] {
                    for cultural in [false, true] {
                        let mut outcomes = Vec::new();
                        for order in [0, 1, 2] {
                            let mut h = base.clone();
                            h.sites[0].demography.ages[1] = adults;
                            h.sites[0].demography.health[0] = illness;
                            h.sites[0].economy.external[3] = 0.;
                            h.sites[0].economy.enterprise_plan = [0.; 4];
                            let ceiling = crate::labor::available(&h.sites[0], true, false);
                            let before = money(&h);
                            if cultural && order != 2 {
                                h.reserve_cultural_work();
                            }
                            if order == 0 {
                                h.prepare_enterprises();
                                h.prepare_vessels();
                            } else {
                                h.prepare_vessels();
                                h.prepare_enterprises();
                            }
                            if cultural && order == 2 {
                                h.reserve_cultural_work();
                            }
                            let enterprise = h.sites[0].economy.enterprise_plan.iter().sum::<f32>();
                            let crew = h.vessel_work(0);
                            let culture = if cultural {
                                h.culture.as_ref().unwrap().labor_budget[0]
                            } else {
                                0.
                            };
                            let used = enterprise + crew + culture;
                            outcomes.push([culture, enterprise, crew]);
                            assert!(used <= ceiling + 1e-5);
                            assert!((money(&h) - before).abs() < 1e-6);
                            println!("{seed},{adults},{illness},{cultural},{order},{ceiling:.4},{culture:.4},{enterprise:.4},{crew:.4},{:.2},{:.4}",
                            h.shipping.as_ref().unwrap().ports[0].capacity(), (ceiling-used).max(0.));
                            if adults == 8. && illness == 0. && !cultural {
                                if order == 0 {
                                    assert!(crew < 0.5);
                                } else {
                                    assert!(crew > 0.099 && crew <= 0.100001);
                                }
                            }
                        }
                        // Negative control: when everyone fits, reordering must
                        // not alter these allocations. With culture off, orders
                        // 1 and 2 are the same intervention and must agree.
                        for outcome in &outcomes[1..] {
                            if adults >= 20. {
                                for (a, b) in outcome.iter().zip(outcomes[0]) {
                                    assert!((*a - b).abs() < 1e-5);
                                }
                            }
                        }
                        if !cultural {
                            assert_eq!(outcomes[1], outcomes[2]);
                        }
                    }
                }
            }
        }
    }
    // Explicit fixture imports of installed capital and metal, plus a transfer of existing cash.
    fn install(g: &mut Generator) {
        let mut catalog = EconomyCatalog::bundled().unwrap();
        catalog.recipes.retain(|r| r.output[3] > 0.);
        catalog.production.adaptive_labor = false;
        g.configure_economy(catalog.clone()).unwrap();
        let h = g.civilizations.as_mut().unwrap();
        let count = h.society.as_ref().unwrap().households.len();
        let owner = h
            .society
            .as_ref()
            .unwrap()
            .households
            .iter()
            .find(|hh| hh.site == 0)
            .unwrap()
            .id as usize;
        let e = &mut h.sites[0].economy;
        for (k, mass) in [(2, 10000.), (0, 100.), (5, 150.), (3, 10.)] {
            e.initial[k] += mass;
            if k == 2 {
                e.goods[k] += mass;
            }
            for j in 0..3 {
                e.external[j] += mass * catalog.composition(k)[j];
            }
        }
        e.workshop = [100., 150., 10., 1.];
        e.workshop_types[0][3] = 1.;
        e.workshop_types[1] = [5., 5., 4., 0.];
        e.labor[3] = 10.;
        let paid = withdraw(&mut e.finance[0], f64::INFINITY);
        let accounts = &mut h
            .society
            .as_mut()
            .unwrap()
            .household_economy
            .as_mut()
            .unwrap()
            .accounts;
        accounts.resize(count, HouseholdAccount::default());
        accounts[owner].cash += paid;
        accounts[owner].wages += paid;
    }
    #[test]
    #[ignore = "requires hardware GPU"]
    fn individual_staffing_absence_changes_grants_and_pays_actual_households() {
        let mut g = world();
        install(&mut g);
        g.enable_politics().unwrap();
        let h = g.civilizations.as_mut().unwrap();
        h.enable_individual_demography().unwrap();
        h.set_workshop_refinement(true).unwrap();
        h.resolution.as_mut().unwrap().compare = true;
        h.month = 3;
        h.begin_service_reservations();
        let person = h
            .participation
            .as_ref()
            .unwrap()
            .residents
            .values()
            .find(|r| r.presence == crate::participation::Presence::Resident(0) && r.capacity > 0.)
            .unwrap()
            .person;
        for r in h.participation.as_mut().unwrap().residents.values_mut() {
            r.capacity = 0.;
            r.care = 0.;
        }
        h.participation
            .as_mut()
            .unwrap()
            .residents
            .get_mut(&person)
            .unwrap()
            .capacity = 0.4;
        // Opening household circumstances and completed practice reach actual
        // candidates; retail resets these food records only after staffing.
        let household = h.participation.as_ref().unwrap().residents[&person]
            .household
            .unwrap();
        let mut pressured = h.clone();
        let account = &mut pressured
            .society
            .as_mut()
            .unwrap()
            .household_economy
            .as_mut()
            .unwrap()
            .accounts[household as usize];
        account.food_site = Some(0);
        account.need = (account.cash + 1000.) * 100.;
        account.hunger = 1.;
        let mut secure = pressured.clone();
        let account = &mut secure
            .society
            .as_mut()
            .unwrap()
            .household_economy
            .as_mut()
            .unwrap()
            .accounts[household as usize];
        // Keep cash identical: otherwise enterprise founding capital changes too.
        account.need = 0.;
        account.hunger = 0.;
        let offer = |world: &History| {
            world.workshop_offers()[0]
                .iter()
                .find(|o| o.person == person)
                .unwrap()
                .at_wage(36., 0)
        };
        assert!(offer(&pressured).fraction > offer(&secure).fraction);
        let score_before = offer(&secure).score;
        secure
            .participation
            .as_mut()
            .unwrap()
            .residents
            .get_mut(&person)
            .unwrap()
            .workshop_completed = 12.;
        assert!(offer(&secure).score > score_before);
        // Demand caps can hide a willingness difference; isolate a labor shortage.
        for world in [&mut pressured, &mut secure] {
            world
                .participation
                .as_mut()
                .unwrap()
                .residents
                .get_mut(&person)
                .unwrap()
                .capacity = 0.001;
        }
        pressured.prepare_enterprises();
        secure.prepare_enterprises();
        let paid = |world: &History| {
            world
                .enterprises
                .as_ref()
                .unwrap()
                .firms
                .iter()
                .map(|f| f.last_funded_work)
                .sum::<f64>()
        };
        assert!(
            paid(&pressured) > paid(&secure),
            "pressured {}, secure {}",
            paid(&pressured),
            paid(&secure)
        );

        let mut absent = h.clone();
        absent
            .participation
            .as_mut()
            .unwrap()
            .residents
            .get_mut(&person)
            .unwrap()
            .capacity = 0.;
        let mut resumed: History =
            serde_json::from_value(serde_json::to_value(&*h).unwrap()).unwrap();
        h.prepare_enterprises();
        resumed.prepare_enterprises();
        absent.prepare_enterprises();
        assert_eq!(
            serde_json::to_value(&*h).unwrap(),
            serde_json::to_value(resumed).unwrap()
        );
        let mut stale = h.clone();
        stale.enterprises.as_mut().unwrap().firms[0].last_requested_work += 1.;
        assert!(stale.settle_workshop_resolutions().is_err());
        let grants = h
            .enterprises
            .as_ref()
            .unwrap()
            .firms
            .iter()
            .map(|f| f.last_funded_work)
            .sum::<f64>();
        assert!(grants > 0. && grants <= 0.4);
        assert_eq!(
            absent
                .enterprises
                .as_ref()
                .unwrap()
                .firms
                .iter()
                .map(|f| f.last_funded_work)
                .sum::<f64>(),
            0.
        );
        let household = h.participation.as_ref().unwrap().residents[&person]
            .household
            .unwrap() as usize;
        let accounts = &h
            .society
            .as_ref()
            .unwrap()
            .household_economy
            .as_ref()
            .unwrap()
            .accounts;
        assert!(accounts[household].employer_income > 0.);
        assert!(accounts
            .iter()
            .enumerate()
            .filter(|(i, _)| *i != household)
            .all(|(_, a)| a.employer_income == 0.));
        assert!(h.check_workshop_reservation_boundary().is_err());
        // Controlled completed-work input; production itself is exercised by seed runs.
        for site in &mut h.sites {
            site.economy.enterprise_used = site.economy.enterprise_plan.map(|w| w * 0.5);
        }
        h.settle_enterprises();
        h.settle_workshop_resolutions().unwrap();
        let actual = h.participation.as_ref().unwrap().residents[&person].workshop_completed;
        assert!((actual - grants * 0.5).abs() < 1e-6);
        let learned = h.participation.as_ref().unwrap().residents[&person].workshop_practice;
        for (family, practice) in learned.iter().enumerate() {
            let completed = h
                .enterprises
                .as_ref()
                .unwrap()
                .firms
                .iter()
                .filter(|f| f.family as usize == family)
                .map(|f| f.last_completed_work)
                .sum::<f64>();
            assert!((practice - completed).abs() < 1e-6);
        }
        assert!(h
            .resolution
            .as_ref()
            .unwrap()
            .receipts
            .iter()
            .any(|r| r.boundary.system == crate::resolution::System::Workshop));
        h.enterprises.as_ref().unwrap().validate(h).unwrap();
    }
    #[test]
    fn entry_requires_forecast_operating_profit_not_just_a_rich_founder() {
        assert!(!viable_entry(0.02, 0.05, 0.05));
        assert!(!viable_entry(1., 1.1, 2.));
        assert!(viable_entry(1., 1.1, 1.));
        assert!(!viable_entry(0., 0., 0.));
    }
    #[test]
    fn dividends_require_earned_profit_and_leave_operating_reserve() {
        let mut f: Firm = serde_json::from_value(serde_json::json!({
            "id":0,"site":0,"family":0,"owner":0,"founded":0,"closed":null,
            "closing_reason":null,"leased_units":1.,"cash":100.,"capital":100.,
            "revenue":0.,"wages":0.,"rent":0.,"dividends":0.,"liquidation":0.,
            "written_off":0.,"completed_work":0.,"paid_work":0.,"last_funded_work":1.,
            "last_completed_work":0.,"wage_rate":10.,"idle_months":0,"distressed_months":0
        }))
        .unwrap();
        assert_eq!(distributable_profit(&f), 0.);
        f.revenue = 50.;
        f.wages = 30.;
        f.rent = 5.;
        f.cash = 115.;
        assert_eq!(distributable_profit(&f), 4.25);
        f.dividends = 14.;
        assert_eq!(distributable_profit(&f), 1.);
        f.cash = 20.;
        assert_eq!(distributable_profit(&f), 0.);
        f.cash = 100.;
        f.wages = 60.;
        assert_eq!(distributable_profit(&f), 0.);
    }
    #[test]
    fn allowances_round_down_and_old_state_has_no_employers() {
        for work in [0., 1e-10, 0.1, 1., 12345.6789] {
            assert!(work_floor(work) as f64 <= work);
        }
        let old:HouseholdAccount=serde_json::from_value(serde_json::json!({"cash":0.,"wages":0.,"dividends":0.,"food_spending":0.,"estate_returned":0.,"need":0.,"common_food":0.,"purchased_food":0.,"hunger":0.})).unwrap();
        assert_eq!(
            old.capital_invested + old.capital_returned + old.employer_income,
            0.
        );
    }
    #[test]
    #[ignore = "requires hardware GPU"]
    fn employer_transfers_and_failure_reconcile_without_duplicating_equipment() {
        let mut g = world();
        install(&mut g);
        let h = g.civilizations.as_mut().unwrap();
        h.month = 3;
        let before = h.economy_residuals();
        let equipment = h.sites[0].economy.workshop;
        h.prepare_enterprises();
        let firms = &h.enterprises.as_ref().unwrap().firms;
        assert_eq!(
            firms.len(),
            1,
            "cash {} rate {}",
            h.society
                .as_ref()
                .unwrap()
                .household_economy
                .as_ref()
                .unwrap()
                .accounts
                .iter()
                .map(|a| a.cash)
                .fold(0., f64::max),
            h.sites[0].economy.prices[crate::economy::FOOD] * 18.
        );
        assert!(firms[0].wages > 0.);
        assert!(firms[0].capital > 0.);
        assert_eq!(equipment, h.sites[0].economy.workshop);
        let f = &firms[0];
        let funded = f.last_funded_work;
        assert!(funded > 0. && funded <= 10.);
        h.sites[0].economy.enterprise_used[1] = (funded * 0.5) as f32;
        h.settle_enterprises();
        h.enterprises.as_ref().unwrap().validate(h).unwrap();
        h.society
            .as_ref()
            .unwrap()
            .household_economy
            .as_ref()
            .unwrap()
            .validate(h)
            .unwrap();
        assert!((h.economy_residuals()[4] - before[4]).abs() < 1e-6);
        // No output means no customer fees; idle wages and rent consume finite reserves.
        for _ in 0..8 {
            h.month += 1;
            h.prepare_enterprises();
            h.settle_enterprises();
        }
        let first = &h.enterprises.as_ref().unwrap().firms[0];
        assert!(first.closed.is_some());
        assert_eq!(first.cash, 0.);
        assert!(first.liquidation <= first.capital + first.revenue);
        assert_eq!(h.sites[0].economy.workshop, equipment);
        assert!((h.economy_residuals()[4] - before[4]).abs() < 1e-6);
        h.enterprises.as_ref().unwrap().validate(h).unwrap();
        let mut bad = h.enterprises.as_ref().unwrap().clone();
        bad.firms[0].cash += 1.;
        assert!(bad.validate(h).is_err());
    }
    #[test]
    #[ignore = "requires hardware GPU"]
    fn vacant_owner_closes_operator_and_returns_only_existing_cash() {
        let mut g = world();
        install(&mut g);
        let h = g.civilizations.as_mut().unwrap();
        h.month = 3;
        h.prepare_enterprises();
        h.settle_enterprises();
        assert_eq!(h.enterprises.as_ref().unwrap().firms.len(), 1);
        let owner = h.enterprises.as_ref().unwrap().firms[0].owner;
        let cash = h.enterprises.as_ref().unwrap().firms[0].cash;
        let before = h.household_account(owner).unwrap().cash;
        let residual = h.economy_residuals();
        let equipment = h.sites[0].economy.workshop;
        h.month = 4;
        let hh = &mut h.society.as_mut().unwrap().households[owner as usize];
        hh.vacant_since = Some(4);
        h.people[hh.head as usize].died = Some(4);
        h.prepare_enterprises();
        let f = &h.enterprises.as_ref().unwrap().firms[0];
        assert_eq!(f.closed, Some(4));
        assert_eq!(f.cash, 0.);
        assert_eq!(h.household_account(owner).unwrap().cash, before + cash);
        assert_eq!(h.sites[0].economy.workshop, equipment);
        assert_eq!(h.sites[0].economy.enterprise_lease, [0.; 4]);
        assert!((h.economy_residuals()[4] - residual[4]).abs() < 1e-6);
        h.enterprises.as_ref().unwrap().validate(h).unwrap();
        h.society
            .as_ref()
            .unwrap()
            .household_economy
            .as_ref()
            .unwrap()
            .validate(h)
            .unwrap();
        let returned = h.household_account(owner).unwrap().cash;
        // A rich vacant account cannot open another company at the next entry date.
        h.month = 12;
        h.prepare_enterprises();
        assert_eq!(h.household_account(owner).unwrap().cash, returned);
        assert_eq!(h.enterprises.as_ref().unwrap().firms.len(), 1);
        // Subsistence remains an aggregate resident entitlement. A dead head does
        // not freeze dependents' access to the existing estate wallet.
        let plans = h.prepare_household_retail();
        assert!(h.household_account(owner).unwrap().need > 0.);
        assert!(!plans.is_empty());
        h.society
            .as_ref()
            .unwrap()
            .household_economy
            .as_ref()
            .unwrap()
            .validate(h)
            .unwrap();
    }
    #[test]
    #[ignore = "requires hardware GPU"]
    fn experienced_work_has_bounded_productivity_with_finite_inputs() {
        let mut g = world();
        install(&mut g);
        g.enable_politics().unwrap();
        let h = g.civilizations.as_mut().unwrap();
        h.month = 3;
        h.enable_individual_demography().unwrap();
        h.set_workshop_refinement(true).unwrap();
        h.begin_service_reservations();
        for r in h.participation.as_mut().unwrap().residents.values_mut() {
            r.workshop_completed = 120.;
            r.workshop_practice = [0., 120., 0., 0.];
        }
        h.prepare_enterprises();
        assert!(h.sites[0].economy.enterprise_productivity[1] > 0.);
        assert!(h.sites[0].economy.enterprise_productivity[1] < 0.75);
        let paid = h.enterprises.as_ref().unwrap().firms[0].last_funded_work;
        assert!(paid > 0.);
        // Continue the ordinary monthly pipeline, which refreshes assignments.
        h.settle_enterprises();
        // Preparatory fixture reservation must be settled before a new month.
        h.settle_workshop_resolutions().unwrap();
        let path =
            std::env::temp_dir().join(format!("worker-experience-{}.world", std::process::id()));
        g.save(&path).unwrap();
        let mut novice = Generator::load(g.gpu.clone(), &path).unwrap();
        std::fs::remove_file(path).unwrap();
        for r in novice
            .civilizations
            .as_mut()
            .unwrap()
            .participation
            .as_mut()
            .unwrap()
            .residents
            .values_mut()
        {
            r.workshop_completed = 0.;
            r.workshop_practice = [0.; 4];
            r.workshop_learning = [0.; 4];
        }
        g.advance_history(1).unwrap();
        novice.advance_history(1).unwrap();
        let h = g.civilizations.as_ref().unwrap();
        let e = &h.sites[0].economy;
        assert!(e.enterprise_productivity[1] > 0.);
        let n = &novice.civilizations.as_ref().unwrap().sites[0].economy;
        assert_eq!(n.enterprise_productivity[1], 0.);
        assert!((n.enterprise_plan[1] - e.enterprise_plan[1]).abs() < 1e-4);
        assert!(
            e.workshop_types[1][2] >= n.workshop_types[1][2] && e.logistics[2] > n.logistics[2],
            "expert output {} vs novice {}, unused time {} vs {}",
            e.workshop_types[1][2],
            n.workshop_types[1][2],
            e.logistics[2],
            n.logistics[2]
        );
        assert!(e.enterprise_used[1] <= e.enterprise_plan[1] + 1e-4);
        assert!(h.economy_residuals().iter().all(|r| r.abs() < 0.02));
    }

    #[test]
    #[ignore = "requires hardware GPU"]
    fn institutional_estates_retain_claims_and_return_only_actual_residual_cash() {
        use crate::credit::{Account, RepaymentSource, Status, Terms, SHARED_CURRENCY};
        use crate::culture::{Institution, InstitutionKind};
        let mut g = world();
        let base = g.civilizations.as_mut().unwrap();
        base.month = 3;
        let id = base.culture.as_ref().unwrap().institutions.len() as u32;
        base.culture
            .as_mut()
            .unwrap()
            .institutions
            .push(Institution {
                id,
                name: "Estate fixture".into(),
                kind: InstitutionKind::Scholarly,
                site: 0,
                tradition: None,
                members: vec![base.civilizations[0].leader],
                leader: base.civilizations[0].leader,
                treasury: 0.,
                active: true,
                founded: 3,
                knowledge: Default::default(),
                property: vec![],
                dues: 0.,
                expenses: 0.,
                capacity: None,
            });
        let terms = |lender, borrower, source| Terms {
            lender,
            borrower,
            currency: SHARED_CURRENCY,
            source: RepaymentSource::ServiceOrder {
                order: source,
                payment_month: 15,
            },
            annual_simple_rate: 0.,
            maturity_month: 15,
            grace_months: 1,
        };
        for insolvent in [false, true] {
            let mut h = base.clone();
            let residual = h.money_residual();
            for lender in [1, 2] {
                h.commit_credit_loan(
                    terms(
                        Account::Town(lender),
                        Account::Institution(id),
                        lender as u64,
                    ),
                    20.,
                )
                .unwrap()
                .unwrap();
            }
            if insolvent {
                h.transfer_credit_cash(
                    Account::Institution(id),
                    Account::Council(0),
                    SHARED_CURRENCY,
                    30.,
                    0.,
                )
                .unwrap();
            } else {
                h.transfer_credit_cash(
                    Account::Town(3),
                    Account::Institution(id),
                    SHARED_CURRENCY,
                    20.,
                    0.,
                )
                .unwrap();
            }
            // Moving an active institution does not liquidate its treasury or
            // alter the identities of its loans.
            let mut relocated = h.clone();
            relocated.culture.as_mut().unwrap().institutions[id as usize].site = 1;
            let before = serde_json::to_value(&relocated).unwrap();
            relocated.settle_credit_estates().unwrap();
            assert_eq!(before, serde_json::to_value(&relocated).unwrap());
            h.culture.as_mut().unwrap().institutions[id as usize].active = false;
            let opening = serde_json::to_value(&h).unwrap();
            assert!(h
                .commit_credit_loan(terms(Account::Town(1), Account::Institution(id), 3), 1.)
                .is_err());
            assert_eq!(opening, serde_json::to_value(&h).unwrap());
            let mut resumed: History = serde_json::from_value(opening).unwrap();
            for world in [&mut h, &mut resumed] {
                let town_cash = world.sites[0].economy.finance[0] as f64;
                world.settle_credit_estates().unwrap();
                let n = &world.culture.as_ref().unwrap().institutions[id as usize];
                assert!(!n.active);
                assert_eq!(n.dues, 0.); // Loan principal is not institutional income.
                assert!((world.money_residual() - residual).abs() < 1e-8);
                let payments: Vec<_> = world
                    .credit
                    .cash_receipts
                    .iter()
                    .filter(|r| !r.disbursement)
                    .map(|r| r.transfer.amount())
                    .collect();
                if insolvent {
                    assert_eq!(payments, vec![5., 5.]);
                    assert_eq!(n.expenses, 0.);
                    assert_eq!(world.sites[0].economy.finance[0] as f64, town_cash);
                    assert!(world
                        .credit
                        .loans
                        .iter()
                        .all(|l| l.outstanding_principal == 15. && l.status == Status::Performing));
                } else {
                    assert_eq!(payments, vec![20., 20.]);
                    assert!(world
                        .credit
                        .loans
                        .iter()
                        .all(|l| l.status == Status::Repaid));
                    let received = world.sites[0].economy.finance[0] as f64 - town_cash;
                    assert_eq!(received, n.expenses);
                    assert!((received + n.treasury - 20.).abs() < 1e-8);
                }
                world.validate_credit().unwrap();
                let once = serde_json::to_value(&world).unwrap();
                world.settle_credit_estates().unwrap();
                assert_eq!(once, serde_json::to_value(&world).unwrap());
            }
            assert_eq!(
                serde_json::to_value(h).unwrap(),
                serde_json::to_value(resumed).unwrap()
            );
        }
        let mut h = base.clone();
        let residual = h.money_residual();
        h.transfer_credit_cash(
            Account::Town(3),
            Account::Institution(id),
            SHARED_CURRENCY,
            20.,
            0.,
        )
        .unwrap();
        h.commit_credit_loan(terms(Account::Institution(id), Account::Town(1), 4), 10.)
            .unwrap()
            .unwrap();
        h.culture.as_mut().unwrap().institutions[id as usize].active = false;
        h.settle_credit_estates().unwrap();
        assert_eq!(h.credit.ownership.assignments().len(), 1);
        assert_eq!(
            h.credit_owner_at(0, h.month).unwrap(),
            Account::Institution(id)
        );
        h.month = 15;
        assert_eq!(h.credit_owner_at(0, h.month).unwrap(), Account::Town(0));
        let paid = h.pay_credit_loan(0, 10.).unwrap();
        assert_eq!(
            h.credit.cash_receipts.last().unwrap().transfer.to,
            Account::Town(0)
        );
        assert!(paid > 9.99);
        let n = &h.culture.as_ref().unwrap().institutions[id as usize];
        let pending = n.treasury;
        let expenses = n.expenses;
        let town_cash = h.sites[0].economy.finance[0] as f64;
        h.settle_credit_estates().unwrap();
        let n = &h.culture.as_ref().unwrap().institutions[id as usize];
        let received = h.sites[0].economy.finance[0] as f64 - town_cash;
        assert_eq!(n.expenses - expenses, received);
        assert!((pending - n.treasury - received).abs() < 1e-8);
        assert!(!n.active);
        h.validate_credit().unwrap();
        assert!((h.money_residual() - residual).abs() < 1e-8);

        // Force the annual cultural shutdown condition and verify that its
        // treasury survives until the explicit Respond estate window.
        let mut h = base.clone();
        h.commit_credit_loan(terms(Account::Town(1), Account::Institution(id), 6), 20.)
            .unwrap()
            .unwrap();
        h.sites[0].abandoned = true;
        h.month = 12;
        h.culture_month();
        let n = &h.culture.as_ref().unwrap().institutions[id as usize];
        assert!(!n.active);
        assert_eq!(n.treasury, 20.);
        assert_eq!(n.expenses, 0.);
        h.settle_credit_estates().unwrap();
        assert_eq!(h.credit.loans[0].status, Status::Repaid);
        assert_eq!(
            h.culture.as_ref().unwrap().institutions[id as usize].treasury,
            0.
        );
        h.validate_credit().unwrap();
    }

    #[test]
    #[ignore = "requires hardware GPU"]
    fn operator_estates_pay_claims_before_owners_and_receive_later_repayments() {
        use crate::credit::{Account, RepaymentSource, Status, Terms, SHARED_CURRENCY};
        let mut g = world();
        install(&mut g);
        let base = g.civilizations.as_mut().unwrap();
        base.month = 3;
        base.prepare_enterprises();
        base.settle_enterprises();
        assert_eq!(base.enterprises.as_ref().unwrap().firms.len(), 1);
        let terms = |lender, borrower, source| Terms {
            lender,
            borrower,
            currency: SHARED_CURRENCY,
            source: RepaymentSource::ServiceOrder {
                order: source,
                payment_month: 15,
            },
            annual_simple_rate: 0.,
            maturity_month: 15,
            grace_months: 1,
        };
        // Live and previously defaulted creditors share the same opening
        // estate cash; default must neither erase priority nor mint repayment.
        {
            let mut h = base.clone();
            let residual = h.economy_residuals()[4];
            h.commit_credit_loan(terms(Account::Town(1), Account::Operator(0), 80), 20.)
                .unwrap();
            let mut later = terms(Account::Town(2), Account::Operator(0), 81);
            later.maturity_month = 30;
            h.commit_credit_loan(later, 20.).unwrap();
            let cash = h.enterprises.as_ref().unwrap().firms[0].cash;
            h.transfer_credit_cash(
                Account::Operator(0),
                Account::Council(0),
                SHARED_CURRENCY,
                cash,
                0.,
            )
            .unwrap();
            h.month = 17;
            h.credit.servicing_policy.available_cash_share = 0.;
            h.service_credit_month().unwrap();
            assert_eq!(h.credit.loans[0].status, Status::Defaulted);
            let default_record = serde_json::to_value(&h.credit.loans[0]).unwrap();
            h.transfer_credit_cash(
                Account::Council(0),
                Account::Operator(0),
                SHARED_CURRENCY,
                10.,
                0.,
            )
            .unwrap();
            h.enterprises.as_mut().unwrap().enabled = false;
            h.prepare_enterprises();
            let mut resumed: History =
                serde_json::from_value(serde_json::to_value(&h).unwrap()).unwrap();
            for world in [&mut h, &mut resumed] {
                world.settle_credit_estates().unwrap();
                assert_eq!(
                    world.credit.recoveries.last().unwrap().transfer.amount(),
                    5.
                );
                assert_eq!(world.credit.loans[1].outstanding_principal, 15.);
                assert_eq!(world.enterprises.as_ref().unwrap().firms[0].cash, 0.);
                assert_eq!(
                    serde_json::to_value(&world.credit.loans[0]).unwrap(),
                    default_record
                );
                let once = serde_json::to_value(&world).unwrap();
                world.settle_credit_estates().unwrap();
                assert_eq!(serde_json::to_value(&world).unwrap(), once);
                world.validate_credit().unwrap();
                assert!((world.economy_residuals()[4] - residual).abs() < 1e-8);
            }
            assert_eq!(
                serde_json::to_value(&h).unwrap(),
                serde_json::to_value(&resumed).unwrap()
            );
        }
        // A cashless closed creditor can pass its claim to its owner; a firm
        // owing live debt or an unrecovered default must retain that asset.
        for indebted in [false, true] {
            let mut h = base.clone();
            let residual = h.money_residual();
            h.commit_credit_loan(terms(Account::Operator(0), Account::Town(0), 90), 10.)
                .unwrap();
            if indebted {
                h.commit_credit_loan(terms(Account::Town(1), Account::Operator(0), 91), 20.)
                    .unwrap();
            }
            let cash = h.enterprises.as_ref().unwrap().firms[0].cash;
            h.transfer_credit_cash(
                Account::Operator(0),
                Account::Council(0),
                SHARED_CURRENCY,
                cash,
                0.,
            )
            .unwrap();
            let owner = h.enterprises.as_ref().unwrap().firms[0].owner;
            h.enterprises.as_mut().unwrap().enabled = false;
            h.prepare_enterprises();
            h.settle_credit_estates().unwrap();
            assert_eq!(
                h.credit.ownership.assignments().len(),
                usize::from(!indebted)
            );
            assert_eq!(h.credit_owner_at(0, h.month).unwrap(), Account::Operator(0));
            let once = serde_json::to_value(&h).unwrap();
            h.settle_credit_estates().unwrap();
            assert_eq!(once, serde_json::to_value(&h).unwrap());
            if indebted {
                h.month = 17;
                h.credit.servicing_policy.available_cash_share = 0.;
                h.service_credit_month().unwrap();
                assert_eq!(h.credit.loans[1].status, Status::Defaulted);
                // Later incoming cash is still an estate asset: default must
                // not make it available to the household ahead of creditors.
                h.transfer_credit_cash(
                    Account::Council(0),
                    Account::Operator(0),
                    SHARED_CURRENCY,
                    5.,
                    0.,
                )
                .unwrap();
                let household_cash = h
                    .society
                    .as_ref()
                    .unwrap()
                    .household_economy
                    .as_ref()
                    .unwrap()
                    .accounts[owner as usize]
                    .cash;
                h.settle_credit_estates().unwrap();
                assert_eq!(h.enterprises.as_ref().unwrap().firms[0].cash, 0.);
                assert_eq!(h.credit.recoveries.last().unwrap().transfer.amount(), 5.);
                assert_eq!(
                    h.society
                        .as_ref()
                        .unwrap()
                        .household_economy
                        .as_ref()
                        .unwrap()
                        .accounts[owner as usize]
                        .cash,
                    household_cash
                );
                assert!(h.credit.ownership.assignments().is_empty());
                h.transfer_credit_cash(
                    Account::Council(0),
                    Account::Operator(0),
                    SHARED_CURRENCY,
                    15.,
                    0.,
                )
                .unwrap();
                h.settle_credit_estates().unwrap();
                assert_eq!(h.credit.recoveries.last().unwrap().transfer.amount(), 15.);
                assert_eq!(h.credit.ownership.assignments().len(), 1);
                assert_eq!(h.credit_owner_at(0, h.month).unwrap(), Account::Operator(0));
                assert_eq!(
                    h.credit_owner_at(0, h.month + 1).unwrap(),
                    Account::Household(owner)
                );
                h.validate_credit().unwrap();
            } else {
                let mut restored: History = serde_json::from_value(once).unwrap();
                for world in [&mut h, &mut restored] {
                    world.month += 1;
                    assert_eq!(
                        world.credit_owner_at(0, world.month).unwrap(),
                        Account::Household(owner)
                    );
                    let paid = world.pay_credit_loan(0, 5.).unwrap();
                    assert!(paid > 0.);
                    assert_eq!(
                        world.credit.cash_receipts.last().unwrap().transfer.to,
                        Account::Household(owner)
                    );
                    assert_eq!(world.enterprises.as_ref().unwrap().firms[0].cash, 0.);
                    world.validate_credit().unwrap();
                    world
                        .society
                        .as_ref()
                        .unwrap()
                        .household_economy
                        .as_ref()
                        .unwrap()
                        .validate(world)
                        .unwrap();
                }
                assert_eq!(
                    serde_json::to_value(&h).unwrap(),
                    serde_json::to_value(&restored).unwrap()
                );
            }
            assert!((h.money_residual() - residual).abs() < 1e-8);
        }
        for insolvent in [false, true] {
            let mut h = base.clone();
            let residual = h.money_residual();
            for lender in [1, 2] {
                h.commit_credit_loan(
                    terms(Account::Town(lender), Account::Operator(0), lender as u64),
                    20.,
                )
                .unwrap()
                .unwrap();
            }
            if insolvent {
                let cash = h.enterprises.as_ref().unwrap().firms[0].cash;
                h.transfer_credit_cash(
                    Account::Operator(0),
                    Account::Council(0),
                    SHARED_CURRENCY,
                    cash - 10.,
                    0.,
                )
                .unwrap();
            }
            let opening = h.enterprises.as_ref().unwrap().firms[0].cash;
            let owner = h.enterprises.as_ref().unwrap().firms[0].owner as usize;
            let returned = h
                .society
                .as_ref()
                .unwrap()
                .household_economy
                .as_ref()
                .unwrap()
                .accounts[owner]
                .capital_returned;
            h.enterprises.as_mut().unwrap().enabled = false;
            h.prepare_enterprises();
            assert!(h.enterprises.as_ref().unwrap().firms[0].closed.is_some());
            assert_eq!(h.enterprises.as_ref().unwrap().firms[0].cash, opening);
            assert_eq!(
                h.society
                    .as_ref()
                    .unwrap()
                    .household_economy
                    .as_ref()
                    .unwrap()
                    .accounts[owner]
                    .capital_returned,
                returned
            );
            let snapshot = serde_json::to_value(&h).unwrap();
            assert!(h
                .commit_credit_loan(terms(Account::Town(1), Account::Operator(0), 3), 1.)
                .is_err());
            assert_eq!(snapshot, serde_json::to_value(&h).unwrap());
            let mut resumed: History = serde_json::from_value(snapshot).unwrap();
            for world in [&mut h, &mut resumed] {
                world.settle_credit_estates().unwrap();
                let expected = if insolvent { 5. } else { 20. };
                let payments: Vec<_> = world
                    .credit
                    .cash_receipts
                    .iter()
                    .filter(|r| !r.disbursement)
                    .map(|r| r.transfer.amount())
                    .collect();
                assert_eq!(payments, vec![expected, expected]);
                if insolvent {
                    assert!(world
                        .credit
                        .loans
                        .iter()
                        .all(|l| l.outstanding_principal == 15. && l.status == Status::Performing));
                } else {
                    assert!(world
                        .credit
                        .loans
                        .iter()
                        .all(|l| l.status == Status::Repaid));
                }
                assert_eq!(world.enterprises.as_ref().unwrap().firms[0].cash, 0.);
                let distributed = world
                    .society
                    .as_ref()
                    .unwrap()
                    .household_economy
                    .as_ref()
                    .unwrap()
                    .accounts[owner]
                    .capital_returned
                    - returned;
                assert!((distributed - (opening - 2. * expected)).abs() < 1e-8);
                world.validate_credit().unwrap();
                world.enterprises.as_ref().unwrap().validate(world).unwrap();
                assert!((world.money_residual() - residual).abs() < 1e-8);
                let once = serde_json::to_value(&world).unwrap();
                world.settle_credit_estates().unwrap();
                assert_eq!(once, serde_json::to_value(&world).unwrap());
            }
            assert_eq!(
                serde_json::to_value(h).unwrap(),
                serde_json::to_value(resumed).unwrap()
            );
        }
        // A closed creditor's identity survives: later repayment enters its
        // account, then reaches the existing owner without reopening the firm.
        let mut h = base.clone();
        let residual = h.money_residual();
        h.commit_credit_loan(terms(Account::Operator(0), Account::Town(0), 4), 10.)
            .unwrap()
            .unwrap();
        h.enterprises.as_mut().unwrap().enabled = false;
        h.prepare_enterprises();
        let owner = h.enterprises.as_ref().unwrap().firms[0].owner as usize;
        let returned = h
            .society
            .as_ref()
            .unwrap()
            .household_economy
            .as_ref()
            .unwrap()
            .accounts[owner]
            .capital_returned;
        h.month = 15;
        let paid = h.pay_credit_loan(0, 10.).unwrap();
        // Town cash uses f32, so assert against the actual exact-transfer receipt.
        assert!(paid > 9.99 && paid <= 10.);
        assert_eq!(h.enterprises.as_ref().unwrap().firms[0].cash, paid);
        h.settle_credit_estates().unwrap();
        assert_eq!(h.enterprises.as_ref().unwrap().firms[0].cash, 0.);
        assert!(h.enterprises.as_ref().unwrap().firms[0].closed.is_some());
        assert_eq!(
            h.society
                .as_ref()
                .unwrap()
                .household_economy
                .as_ref()
                .unwrap()
                .accounts[owner]
                .capital_returned,
            returned + paid
        );
        h.validate_credit().unwrap();
        h.enterprises.as_ref().unwrap().validate(&h).unwrap();
        assert!((h.money_residual() - residual).abs() < 1e-8);

        // Exercise the actual Reserve closure and settlement hooks too.
        let h = g.civilizations.as_mut().unwrap();
        h.commit_credit_loan(terms(Account::Town(1), Account::Operator(0), 5), 20.)
            .unwrap()
            .unwrap();
        h.enterprises.as_mut().unwrap().enabled = false;
        let path =
            std::env::temp_dir().join(format!("operator-estate-{}.world", std::process::id()));
        g.save(&path).unwrap();
        let mut resumed = Generator::load(g.gpu.clone(), &path).unwrap();
        std::fs::remove_file(path).unwrap();
        for world in [&mut g, &mut resumed] {
            world.advance_history(1).unwrap();
            let h = world.civilizations.as_ref().unwrap();
            assert_eq!(h.credit.loans[0].status, Status::Repaid);
            assert!(h.enterprises.as_ref().unwrap().firms[0].closed.is_some());
            assert_eq!(h.enterprises.as_ref().unwrap().firms[0].cash, 0.);
            assert_eq!(h.sites[0].economy.enterprise_used[1], 0.);
            h.validate_credit().unwrap();
            h.enterprises.as_ref().unwrap().validate(h).unwrap();
            assert!(h.economy_residuals().iter().all(|v| v.abs() < 0.02));
        }
        assert_eq!(
            serde_json::to_vec(&g.civilizations).unwrap(),
            serde_json::to_vec(&resumed.civilizations).unwrap()
        );
    }

    #[test]
    #[ignore = "requires hardware GPU"]
    fn credit_funds_gpu_work_but_cannot_replace_materials() {
        use crate::credit::{Account, RepaymentSource, Terms, SHARED_CURRENCY};
        let mut g = world();
        install(&mut g);
        let h = g.civilizations.as_mut().unwrap();
        h.month = 3;
        h.prepare_enterprises();
        h.settle_enterprises();
        assert_eq!(h.enterprises.as_ref().unwrap().firms.len(), 1);
        let firm = &mut h.enterprises.as_mut().unwrap().firms[0];
        firm.leased_units = 5.;
        let capital = firm.cash;
        assert!(capital > 0.);
        let cash_residual = h.money_residual();
        // Move existing capital to a lender. Both arms start with the same
        // inventories and cash ownership; only the loan reverses this transfer.
        let moved = h
            .transfer_credit_cash(
                Account::Operator(0),
                Account::Council(0),
                SHARED_CURRENCY,
                capital,
                0.,
            )
            .unwrap();
        assert_eq!(moved.amount(), capital);
        assert_eq!(h.enterprises.as_ref().unwrap().firms[0].cash, 0.);
        let baseline = h.clone();
        let revenue = h.enterprises.as_ref().unwrap().firms[0].revenue;
        // Explicit service-order fixture: this tests execution downstream of a
        // committed loan, not automatic underwriting or repayment forecasting.
        h.commit_credit_loan(
            Terms {
                lender: Account::Council(0),
                borrower: Account::Operator(0),
                currency: SHARED_CURRENCY,
                source: RepaymentSource::ServiceOrder {
                    order: 0,
                    payment_month: 15,
                },
                annual_simple_rate: 0.12,
                maturity_month: 15,
                grace_months: 3,
            },
            capital,
        )
        .unwrap()
        .unwrap();
        assert_eq!(h.enterprises.as_ref().unwrap().firms[0].revenue, revenue);
        assert!((h.money_residual() - cash_residual).abs() < 1e-9);
        h.validate_credit().unwrap();
        let path =
            std::env::temp_dir().join(format!("enterprise-credit-{}.world", std::process::id()));
        g.save(&path).unwrap();
        let mut resumed = Generator::load(g.gpu.clone(), &path).unwrap();
        let mut unfunded = Generator::load(g.gpu.clone(), &path).unwrap();
        let mut missing_material = Generator::load(g.gpu.clone(), &path).unwrap();
        std::fs::remove_file(path).unwrap();
        unfunded.civilizations = Some(baseline);
        // Move the actual metal to another settlement: credit cannot fabricate
        // a recipe input. Preserve the world's material inventory.
        let h = missing_material.civilizations.as_mut().unwrap();
        let metal = h.sites[0].economy.goods[2];
        h.sites[0].economy.goods[2] = 0.;
        h.sites[1].economy.goods[2] += metal;
        for world in [&mut g, &mut resumed, &mut unfunded, &mut missing_material] {
            world.advance_history(1).unwrap();
            let h = world.civilizations.as_ref().unwrap();
            h.validate_credit().unwrap();
            assert!(h.economy_residuals().iter().all(|v| v.abs() < 0.02));
        }
        let output = |world: &Generator| {
            world.civilizations.as_ref().unwrap().sites[0]
                .economy
                .enterprise_used[1]
        };
        assert!(output(&g) > 0.);
        assert_eq!(output(&unfunded), 0.);
        assert_eq!(output(&missing_material), 0.);
        assert_eq!(
            serde_json::to_vec(&g.civilizations).unwrap(),
            serde_json::to_vec(&resumed.civilizations).unwrap()
        );
    }

    #[test]
    #[ignore = "requires hardware GPU"]
    fn prepaid_capacity_gates_gpu_work_and_checkpoint_matches() {
        let mut g = world();
        install(&mut g);
        // Create a lease using the normal finite startup path, then retain a pre-month baseline.
        let h = g.civilizations.as_mut().unwrap();
        h.month = 3;
        h.prepare_enterprises();
        h.settle_enterprises();
        assert_eq!(h.enterprises.as_ref().unwrap().firms.len(), 1);
        // Lease the entire fixture workshop to isolate the production-capacity mediator.
        h.enterprises.as_mut().unwrap().firms[0].leased_units = 5.;
        let path = std::env::temp_dir().join(format!("enterprise-{}.world", std::process::id()));
        g.save(&path).unwrap();
        let mut resumed = Generator::load(g.gpu.clone(), &path).unwrap();
        let mut unfunded = Generator::load(g.gpu.clone(), &path).unwrap();
        std::fs::remove_file(&path).unwrap();
        // Transfer all firm cash back to its owner; keep the lease and its costs in the ablation.
        let h = unfunded.civilizations.as_mut().unwrap();
        let f = &mut h.enterprises.as_mut().unwrap().firms[0];
        let a = &mut h
            .society
            .as_mut()
            .unwrap()
            .household_economy
            .as_mut()
            .unwrap()
            .accounts[f.owner as usize];
        a.cash += f.cash;
        a.capital_returned += f.cash;
        f.liquidation += f.cash;
        f.cash = 0.;
        g.advance_history(1).unwrap();
        unfunded.advance_history(1).unwrap();
        let funded_h = g.civilizations.as_ref().unwrap();
        let empty_h = unfunded.civilizations.as_ref().unwrap();
        assert!(funded_h.sites[0].economy.enterprise_used[1] > 0.);
        assert_eq!(empty_h.sites[0].economy.enterprise_used[1], 0.);
        assert!(
            funded_h.sites[0].economy.workshop_types[1][2]
                > empty_h.sites[0].economy.workshop_types[1][2]
        );
        g.advance_history(5).unwrap();
        for _ in 0..6 {
            resumed.advance_history(1).unwrap();
        }
        assert_eq!(
            serde_json::to_vec(&g.civilizations).unwrap(),
            serde_json::to_vec(&resumed.civilizations).unwrap()
        );
        assert!(g
            .civilizations
            .as_ref()
            .unwrap()
            .economy_residuals()
            .iter()
            .all(|v| v.abs() < 0.001));
        g.set_enterprises(false).unwrap();
        g.advance_history(1).unwrap();
        assert!(g
            .civilizations
            .as_ref()
            .unwrap()
            .enterprises
            .as_ref()
            .unwrap()
            .firms
            .iter()
            .all(|f| f.closed.is_some()));
    }
}
