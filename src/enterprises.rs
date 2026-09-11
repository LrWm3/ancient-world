//! Household-owned toll manufacturers leasing a subset of communal workshop equipment.
//! Cash is separate; materials, products and embodied equipment stay in canonical town stocks.
use crate::{
    civilization::History,
    household_economy::{deposit, withdraw, HouseholdAccount},
};
use anyhow::{ensure, Result};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

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
        let vacancy = if expected > 1e-6 {
            (1. - hired / expected).clamp(0., 1.)
        } else {
            0.
        };
        self.shortage = 0.75 * self.shortage + 0.25 * vacancy;
        let utilization = if hired > 1e-6 {
            (completed / hired).clamp(0., 1.)
        } else {
            0.
        };
        let payroll = hired * reference * self.multiplier;
        // Raises need demonstrated productive work, payment and a cash buffer.
        // The independent 1.25x service quote limits the affordable wage.
        let profitable_ceiling = (1.25 * utilization * 0.9).clamp(0.6, 1.125);
        let next = if self.shortage > 0.15
            && utilization >= 0.8
            && paid_invoice >= payroll
            && cash >= 3. * payroll
        {
            (self.multiplier * (1. + 0.08 * self.shortage))
                .min(profitable_ceiling)
                .max(self.multiplier)
        } else if utilization < 0.5 || paid_invoice < payroll {
            self.multiplier * 0.97
        } else {
            self.multiplier
        };
        self.pending = Some(next.clamp(0.6, 1.125));
    }
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Firm {
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
    pub enabled: bool,
    pub firms: Vec<Firm>,
}
impl Default for Enterprises {
    fn default() -> Self {
        Self {
            enabled: true,
            firms: vec![],
        }
    }
}
impl Enterprises {
    pub fn validate(&self, h: &History) -> Result<()> {
        let mut occupied = BTreeSet::new();
        for (i, f) in self.firms.iter().enumerate() {
            if let Some(p) = &f.wage_policy {
                ensure!(
                    p.multiplier.is_finite()
                        && (0.6..=1.125).contains(&p.multiplier)
                        && p.pending
                            .is_none_or(|v| v.is_finite() && (0.6..=1.125).contains(&v))
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
                        && staff.granted <= staff.expected + 1e-5
                        && (staff.granted - f.last_funded_work).abs() < 1e-4,
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
                            < 1e-5,
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
            ensure!(
                (f.cash - f.capital - f.revenue + f.wages + f.rent + f.dividends + f.liquidation)
                    .abs()
                    < 1e-7 * (1. + f.capital + f.revenue),
                "enterprise cash ledger does not reconcile"
            );
            ensure!(
                f.completed_work <= f.paid_work + 1e-4 * (1. + f.paid_work)
                    && f.last_completed_work <= f.last_funded_work + 1e-4
                    && (f.closed.is_none() || f.cash == 0.),
                "unfunded enterprise work or stranded liquidation"
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
                    (a.capital_invested - claim[0]).abs() <= 1e-7 * (1. + claim[0])
                        && (a.capital_returned - claim[1]).abs() <= 1e-7 * (1. + claim[1]),
                    "household equity does not match owned businesses"
                );
            }
        }
        Ok(())
    }
}
fn close(f: &mut Firm, owner: &mut HouseholdAccount, month: u32, reason: &str) {
    owner.cash += f.cash;
    owner.capital_returned += f.cash;
    f.liquidation += f.cash;
    f.cash = 0.;
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
    expected_work * 1.25 > paid_shift + units * 0.08
}

// Equity investment is risk capital, not earned profit. Losses must be recovered
// before dividends resume; the operating reserve is an additional independent cap.
fn distributable_profit(f: &Firm) -> f64 {
    let earned = (f.revenue - f.wages - f.rent - f.dividends).max(0.);
    let surplus = (f.cash - 3. * f.last_funded_work * f.wage_rate).max(0.) * 0.05;
    earned.min(surplus)
}
impl History {
    pub fn enterprise_summary(&self) -> serde_json::Value {
        let firms = self
            .enterprises
            .as_ref()
            .map(|e| e.firms.as_slice())
            .unwrap_or(&[]);
        serde_json::json!({"enabled":self.enterprises.as_ref().is_some_and(|e|e.enabled),
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
                close(f, &mut e.accounts[f.owner as usize], self.month, reason);
                notices.push(("enterprise_closed",f.site,format!("Workshop operator {} closed: {}; remaining cash returned to household {} and equipment lease released",f.id,reason,f.owner)));
            }
        }
        // Founders supply actual wallet capital. No city grant, loan or new workshop is implied.
        if enterprises.enabled && self.month > 0 && self.month % 3 == 0 {
            for (site, ids) in residents.iter().enumerate() {
                let town = &self.sites[site];
                if town.abandoned || town.economy.workshop_types[0][3] < 0.5 {
                    continue;
                }
                let rate = 18. * town.economy.prices[crate::economy::FOOD].max(0.01) as f64;
                for family in 0..4 {
                    if enterprises.firms.iter().any(|f| {
                        f.site as usize == site
                            && f.family as usize == family
                            && (f.closed.is_none()
                                || f.closed.is_some_and(|m| self.month.saturating_sub(m) < 6))
                    }) {
                        continue;
                    }
                    let units = (town.economy.workshop_types[family][0] as f64 * 0.5).min(1.);
                    if units < 0.02 || town.economy.workshop_types[family][2] < 0.02 {
                        continue;
                    }
                    // Use only locally observed work and the current service quote.
                    // Do not repeatedly finance a shift whose forecast fees cannot cover
                    // wages and rent even before uncertainty or customer nonpayment.
                    let observed = town.economy.workshop_types[family][2] as f64;
                    let share = units / (town.economy.workshop_types[family][0] as f64).max(0.001);
                    let expected = (observed * share).min(units * 4.);
                    let shift = (expected * 1.1).max(0.05).min(units * 4.);
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
                    let capital = e.accounts[owner].cash * 0.25;
                    if capital < 3. * rate * (shift + units * 0.08) {
                        continue;
                    }
                    e.accounts[owner].cash -= capital;
                    e.accounts[owner].capital_invested += capital;
                    let id = enterprises.firms.len() as u32;
                    enterprises.firms.push(Firm {
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
            let reference = 18. * town.economy.prices[crate::economy::FOOD].max(0.01) as f64;
            if refine {
                let policy = f.wage_policy.get_or_insert_with(Default::default);
                if let Some(next) = policy.pending.take() {
                    policy.multiplier = next;
                }
                f.wage_rate = reference * policy.multiplier;
                f.service_rate = Some(reference * 1.25);
            } else {
                f.wage_rate = reference;
                f.service_rate = None;
            }
            let units = f
                .leased_units
                .min(town.economy.workshop_types[family][0] as f64);
            let rent_request = (units * 0.08 * reference).min(f.cash);
            let rent = deposit(&mut town.economy.finance[0], rent_request);
            f.cash -= rent;
            f.rent += rent;
            let lease_share = units / (town.economy.workshop_types[family][0] as f64).max(0.001);
            let desired = (units * 4.)
                .min((town.economy.workshop_types[family][2] as f64 * lease_share * 1.1).max(0.05))
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
                let eligible: Vec<_> = offers[site]
                    .iter()
                    .filter(|o| residents[site].contains(&(o.household as usize)))
                    .map(|o| o.at_wage(f.wage_rate, f.family))
                    .collect();
                self.participation.as_mut().map(|p| {
                    crate::workshop_resolution::resolve(
                        p,
                        boundary,
                        grants[site][f.family as usize],
                        &eligible,
                    )
                })
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
            f.distressed_months = if work < desired * 0.25 || units < 0.001 {
                f.distressed_months + 1
            } else {
                0
            };
            town.economy.enterprise_plan[family] = work as f32;
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
                    (payroll * weights[j] / total.max(1e-12)).min(remaining)
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
        let mut invoices = vec![0.; self.sites.len()];
        for f in enterprises.firms.iter_mut().filter(|f| f.closed.is_none()) {
            let work =
                self.sites[f.site as usize].economy.enterprise_used[f.family as usize] as f64;
            f.last_completed_work = work;
            f.completed_work += work;
            invoices[f.site as usize] += work * f.service_rate.unwrap_or(f.wage_rate * 1.25);
        }
        // Gather per-town invoices first. A common affordability factor prevents first-operator priority.
        let mut funds = vec![0.; self.sites.len()];
        for (site, invoice) in invoices.iter().enumerate() {
            let pool = &mut self.sites[site].economy.finance[0];
            funds[site] = withdraw(pool, invoice.min(*pool as f64 * 0.2));
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
            let invoice = f.last_completed_work * f.service_rate.unwrap_or(f.wage_rate * 1.25);
            let received = if invoices[site] <= 0. {
                0.
            } else if last[site] == Some(i) {
                remaining[site]
            } else {
                (funds[site] * invoice / invoices[site]).min(remaining[site])
            };
            remaining[site] -= received;
            f.cash += received;
            f.revenue += received;
            f.written_off += (invoice - received).max(0.);
            let surplus = distributable_profit(f);
            f.cash -= surplus;
            f.dividends += surplus;
            e.accounts[f.owner as usize].cash += surplus;
            e.accounts[f.owner as usize].dividends += surplus;
            f.idle_months = if f.last_completed_work < f.last_funded_work.max(0.01) * 0.15 {
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
                        service / 1.25,
                        received,
                    );
                }
            }
            let reason = if f.distressed_months >= 3 {
                Some("working capital exhausted")
            } else if f.idle_months >= 6 {
                Some("six months without sufficient orders or inputs")
            } else {
                None
            };
            if let Some(reason) = reason {
                close(f, &mut e.accounts[f.owner as usize], self.month, reason);
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
