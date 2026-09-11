//! Named subdivisions of installed port fleet assets, with prepaid household crew work.
//! Hull material remains in Port::assets: vessel records never duplicate that inventory.
use crate::{civilization::History, household_economy::withdraw};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Fleet {
    #[serde(default)]
    pub requested_work: f32,
    pub vessels: Vec<Vessel>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Vessel {
    pub id: u32,
    pub name: String,
    pub commissioned: u32,
    pub household: Option<u32>,
    pub funded_work: f32,
    pub wages_paid: f64,
}
/// Completed travel intervals, persisted independently of the projected arrival date.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct VoyageClock {
    pub month: u32,
    pub remaining: f32,
}
impl VoyageClock {
    fn advance(&mut self, month: u32, staffing: f32) {
        // A funding observation pays for one interval, never an arbitrary time jump.
        if month == self.month.saturating_add(1) {
            self.remaining = (self.remaining - staffing.clamp(0., 1.)).max(0.);
            // f32 payroll transfers may leave a few millionths of an interval unpaid.
            if self.remaining < 1e-4 {
                self.remaining = 0.;
            }
        }
        self.month = self.month.max(month);
    }
}
fn funded_work(paid: f64, wage: f64, reserved: f32) -> f32 {
    ((paid / wage) as f32).min(reserved)
}

impl Fleet {
    pub fn capacity(&self) -> f32 {
        self.vessels
            .iter()
            .map(|v| 250. * (v.funded_work / 0.25).clamp(0., 1.))
            .sum()
    }
    pub fn work(&self) -> f32 {
        self.vessels.iter().map(|v| v.funded_work).sum()
    }
}
// Cargo already in transit reserves both endpoints, exactly as sea_capacity does.
// Closed lanes still hold their cargo reservation; flooded ports cannot fund work.
fn committed_port_loads(
    shipping: &crate::shipping::Shipping,
    cargo: &[crate::economy::Cargo],
) -> Vec<f32> {
    let mut loads = vec![0.; shipping.ports.len()];
    for c in cargo {
        if let Some(lane) = c.sea_lane.and_then(|id| shipping.lanes.get(id as usize)) {
            for &port in &lane.ports {
                loads[port as usize] += c.kg;
            }
        }
    }
    loads
}
impl History {
    /// Opening reads the preceding month's funded crews before reservations reset.
    pub(crate) fn advance_cargo_voyages(&mut self) {
        let Some(shipping) = &self.shipping else {
            return;
        };
        let loads = committed_port_loads(shipping, &self.cargo);
        for cargo in &mut self.cargo {
            let Some(lane) = cargo
                .sea_lane
                .and_then(|id| shipping.lanes.get(id as usize))
            else {
                continue;
            };
            let staffing = lane.ports.iter().fold(1_f32, |fraction, &id| {
                let port = &shipping.ports[id as usize];
                // Archives/configurations without the vessel subsystem retain scheduled travel.
                let funded = port.fleet.as_ref().map_or(1., |fleet| {
                    (fleet.capacity() / loads[id as usize].max(0.001)).clamp(0., 1.)
                });
                fraction.min(funded)
            });
            let clock = cargo.voyage_clock.get_or_insert(VoyageClock {
                month: self.month.saturating_sub(1),
                remaining: cargo.arrives.saturating_sub(self.month.saturating_sub(1)) as f32,
            });
            clock.advance(self.month, staffing);
            if clock.remaining > 0. {
                cargo.arrives = self.month.saturating_add(clock.remaining.ceil() as u32);
            }
        }
    }
    /// Clear completed reservations once, before this month's service claims.
    pub(crate) fn begin_service_reservations(&mut self) {
        for site in &mut self.sites {
            site.economy.external[3] = 0.;
            site.economy.enterprise_plan = [0.; 4];
        }
        if let Some(e) = self
            .society
            .as_mut()
            .and_then(|s| s.household_economy.as_mut())
        {
            for account in &mut e.accounts {
                account.employer_income = 0.;
            }
        }
        if let Some(shipping) = &mut self.shipping {
            for port in &mut shipping.ports {
                if let Some(fleet) = &mut port.fleet {
                    fleet.requested_work = 0.;
                    for vessel in &mut fleet.vessels {
                        vessel.funded_work = 0.;
                        vessel.household = None;
                    }
                }
            }
        }
    }
    pub(crate) fn prepare_committed_vessels(&mut self) {
        self.reserve_vessels(true);
    }
    pub(crate) fn prepare_vessels(&mut self) {
        self.reserve_vessels(false);
    }
    fn reserve_vessels(&mut self, committed_only: bool) {
        let Some(mut shipping) = self.shipping.take() else {
            return;
        };
        let loads = committed_port_loads(&shipping, &self.cargo);
        for (port_index, port) in shipping.ports.iter_mut().enumerate() {
            let Some(fleet) = &mut port.fleet else {
                continue;
            };
            let site = port.site as usize;
            if self.sites[site].abandoned || port.commissioned.is_none() || port.flood_months > 0 {
                continue;
            }
            let hulls = (port.assets[0] / 50.)
                .min(port.assets[1] / 2.5)
                .floor()
                .clamp(0., 4.) as usize;
            // Existing identities remain laid up when backing material or workers are missing.
            while fleet.vessels.len() < hulls {
                let id = fleet.vessels.len() as u32;
                let name = self.civilizations[self.sites[site].civilization as usize]
                    .naming(self.seed)
                    .coin(
                        &format!("vessel:{}:{id}", port.site),
                        &["water", "journey"],
                        None,
                    );
                fleet.vessels.push(Vessel {
                    id,
                    name: name.clone(),
                    commissioned: self.month,
                    household: None,
                    funded_work: 0.,
                    wages_paid: 0.,
                });
                self.event("vessel_commissioned",Some(port.site),None,
                    format!("{name} entered the port fleet; 50 kg timber and 2.5 kg equipment already held in port assets back its hull"));
            }
            let Some(society) = &mut self.society else {
                continue;
            };
            let ids: Vec<_> = society
                .households
                .iter()
                .filter(|hh| {
                    hh.site == port.site
                        && !society.relocation.away(hh.id)
                        && !society.relocation.lost_households.contains(&hh.id)
                })
                .map(|hh| hh.id as usize)
                .collect();
            let Some(wallets) = &mut society.household_economy else {
                continue;
            };
            wallets
                .accounts
                .resize(society.households.len(), Default::default());
            let s = &mut self.sites[site];
            let target = (loads[port_index] / 1000. + if committed_only { 0. } else { 0.1 })
                .min(hulls as f32 * 0.25);
            fleet.requested_work = target;
            let mut work_left = crate::labor::available(s, true, self.living.is_some())
                .min((target - fleet.work()).max(0.));
            let wage = 18. * s.economy.prices[crate::economy::FOOD].max(0.01) as f64;
            for (v, &hh) in fleet.vessels.iter_mut().take(hulls).zip(&ids) {
                let work = (0.25 - v.funded_work)
                    .max(0.)
                    .min(work_left)
                    .min((s.economy.finance[0] as f64 / wage) as f32);
                if work <= 0. {
                    continue;
                }
                let paid = withdraw(&mut s.economy.finance[0], work as f64 * wage);
                // Cash uses f32 while wallets retain the exact f64 debit. Rounding
                // may pay slightly above the quote; it cannot purchase work
                // beyond the crew reservation.
                let actual = funded_work(paid, wage, work);
                v.household = Some(hh as u32);
                v.funded_work += actual;
                v.wages_paid += paid;
                wallets.accounts[hh].cash += paid;
                wallets.accounts[hh].wages += paid;
                wallets.accounts[hh].employer_income += paid;
                s.economy.external[3] += actual;
                work_left -= actual;
            }
        }
        self.shipping = Some(shipping);
    }
    pub(crate) fn vessel_work(&self, site: u32) -> f32 {
        self.shipping.as_ref().map_or(0., |s| {
            s.ports
                .iter()
                .filter(|p| p.site == site)
                .filter_map(|p| p.fleet.as_ref())
                .map(Fleet::work)
                .sum()
        })
    }
    pub(crate) fn release_vessel_work(&mut self) {
        for i in 0..self.sites.len() {
            self.sites[i].economy.external[3] =
                (self.sites[i].economy.external[3] - self.vessel_work(i as u32)).max(0.);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn travel_uses_completed_intervals_and_survives_resume() {
        let mut clock = VoyageClock {
            month: 11,
            remaining: 2.,
        };
        clock.advance(12, 0.);
        assert_eq!(
            clock.remaining, 2.,
            "no crews, including across year boundary"
        );
        clock.advance(12, 1.);
        assert_eq!(
            clock.remaining, 2.,
            "same-month hiring cannot advance completed interval twice"
        );
        clock.advance(13, 0.5);
        assert_eq!(clock.remaining, 1.5);
        let mut restored: VoyageClock =
            serde_json::from_str(&serde_json::to_string(&clock).unwrap()).unwrap();
        for month in [14, 15] {
            clock.advance(month, 0.75);
            restored.advance(month, 0.75);
        }
        assert_eq!(clock.remaining, 0.);
        assert_eq!(restored.remaining, clock.remaining);
        let mut skipped = VoyageClock {
            month: 1,
            remaining: 4.,
        };
        skipped.advance(4, 1.);
        assert_eq!(
            skipped.remaining, 4.,
            "one funding observation cannot fund skipped months"
        );
    }
    #[test]
    fn cargo_claims_use_actual_sea_endpoints() {
        let shipping = crate::shipping::Shipping {
            version: 1,
            started: 0,
            surveyed_sites: 3,
            ports: (0..3)
                .map(|site| crate::shipping::Port {
                    site,
                    fleet: None,
                    work: None,
                    access: vec![],
                    water_cell: 0,
                    access_km: 0.,
                    assets: [0.; 3],
                    commissioned: None,
                    flood_months: 0,
                })
                .collect(),
            lanes: vec![
                crate::shipping::SeaLane {
                    ports: [0, 1],
                    cells: vec![],
                    km: 1.,
                    open: true,
                    flood_months: 0,
                },
                crate::shipping::SeaLane {
                    ports: [1, 2],
                    cells: vec![],
                    km: 1.,
                    open: false,
                    flood_months: 0,
                },
            ],
        };
        let cargo = |kg, sea_lane| crate::economy::Cargo {
            voyage_clock: None,
            freight_stops: vec![],
            from: 0,
            to: 2,
            good: 0,
            kg,
            paid: 0.,
            arrives: 4,
            sea_lane,
            weather_delay_months: 0,
        };
        let mut loads = vec![cargo(50., Some(0)), cargo(70., Some(1)), cargo(999., None)];
        assert_eq!(
            committed_port_loads(&shipping, &loads),
            vec![50., 120., 70.]
        );
        loads.reverse();
        assert_eq!(
            committed_port_loads(&shipping, &loads),
            vec![50., 120., 70.]
        );
        // Arrival removal releases the old commitment; inland cargo creates none.
        loads.retain(|c| c.sea_lane.is_none());
        assert_eq!(committed_port_loads(&shipping, &loads), vec![0.; 3]);
    }
    #[test]
    fn rounded_wages_cannot_expand_reserved_labor() {
        let mut cash = 1_000_000f32;
        let before = cash;
        let wage = 0.18;
        let reserved = 0.25;
        let paid = withdraw(&mut cash, reserved as f64 * wage);
        assert!(paid / wage > reserved as f64);
        assert_eq!(funded_work(paid, wage, reserved), reserved);
        assert_eq!(cash as f64 + paid, before as f64);
        assert_eq!(funded_work(0., wage, reserved), 0.);
        assert_eq!(funded_work(0.009, wage, reserved), 0.05);
    }

    #[test]
    #[ignore = "requires hardware GPU"]
    fn crew_pay_conserves_money_and_reserves_finite_work() {
        use crate::{
            catalog::Catalog,
            config::Config,
            gpu::{ContextGpu, Generator},
        };
        let mut g = Generator::new(
            pollster::block_on(ContextGpu::headless()).unwrap(),
            Config {
                resolution: 64,
                ecology_resolution: 64,
                seed: 7,
                ..Default::default()
            },
            Catalog::bundled().unwrap(),
        )
        .unwrap();
        g.run_epochs(1).unwrap();
        g.found_civilizations(5).unwrap();
        g.enable_society().unwrap();
        g.enable_shipping().unwrap();
        let h = g.civilizations.as_mut().unwrap();
        let site = h.shipping.as_ref().unwrap().ports[0].site as usize;
        h.shipping.as_mut().unwrap().ports[0].assets = [200., 10., 100.];
        h.shipping.as_mut().unwrap().ports[0].commissioned = Some(h.month);
        h.sites[site].economy.finance[0] = 10000.;
        let total = |h: &History| {
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
        };
        let before = total(h);
        h.prepare_vessels();
        assert!((total(h) - before).abs() < 1e-8);
        let fleet = h.shipping.as_ref().unwrap().ports[0]
            .fleet
            .as_ref()
            .unwrap();
        assert!(!fleet.vessels.is_empty());
        assert!(fleet.capacity() > 0.);
        let work = fleet.work();
        assert!(work <= 0.100001, "idle ports hire only bounded standby");
        assert!(h.sites[site].economy.external[3] >= work);
        assert_eq!(
            h.shipping.as_ref().unwrap().ports[0].assets,
            [200., 10., 100.]
        );
        let restored: Fleet = serde_json::from_str(&serde_json::to_string(fleet).unwrap()).unwrap();
        assert_eq!(restored.capacity(), fleet.capacity());
        h.release_vessel_work();
        assert!(h.sites[site].economy.external[3] < 1e-5);
        // Only 0.24 worker-months remain with two sick adults. Quarterly
        // cultural work must respect that limit and leave no duplicate crew labor.
        h.month = 3;
        h.sites[site].demography.ages[1] = 2.;
        h.sites[site].demography.health[0] = 0.5;
        h.sites[site].economy.enterprise_plan = [99.; 4]; // completed old plans
        h.begin_service_reservations();
        h.prepare_discoveries();
        assert_eq!(h.sites[site].economy.enterprise_plan, [0.; 4]);
        h.reserve_cultural_work();
        let reserved = h.sites[site].economy.external[3];
        assert!(reserved <= 0.240001);
        let before = total(h);
        h.prepare_vessels();
        assert!(h.vessel_work(site as u32) + reserved <= 0.240001);
        assert!((total(h) - before).abs() < 1e-8);
        h.release_vessel_work();
        h.release_cultural_work();
        // Prior service and paid enterprise plans leave only 0.10 for crews.
        h.sites[site].economy.external[3] = 0.1;
        h.sites[site].economy.enterprise_plan = [0.04, 0., 0., 0.];
        h.prepare_vessels();
        let crew = h.vessel_work(site as u32);
        assert!(crew > 0. && crew <= 0.100001);
        assert!(h.sites[site].economy.external[3] + 0.04 <= 0.240001);
        assert!((total(h) - before).abs() < 1e-8);
        h.release_vessel_work();
        h.sites[site].economy.finance[0] = 0.;
        h.begin_service_reservations();
        h.prepare_vessels();
        assert_eq!(h.shipping.as_ref().unwrap().ports[0].capacity(), 0.);

        // Synthetic already-dispatched load: reserve both lane endpoints, and
        // protect paid crews before quarterly discretionary activity.
        assert!(h.shipping.as_ref().unwrap().ports.len() >= 2);
        h.shipping
            .as_mut()
            .unwrap()
            .lanes
            .push(crate::shipping::SeaLane {
                ports: [0, 1],
                cells: vec![],
                km: 1.,
                open: true,
                flood_months: 0,
            });
        let lane = h.shipping.as_ref().unwrap().lanes.len() as u32 - 1;
        h.cargo.push(crate::economy::Cargo {
            voyage_clock: None,
            freight_stops: vec![],
            from: site as u32,
            to: h.shipping.as_ref().unwrap().ports[1].site,
            good: crate::economy::FOOD as u32,
            kg: 800.,
            paid: 0.,
            arrives: h.month + 2,
            sea_lane: Some(lane),
            weather_delay_months: 0,
        });
        h.sites[site].economy.finance[0] = 10000.;
        h.sites[site].demography.ages[1] = 8.;
        h.sites[site].demography.health[0] = 0.;
        h.begin_service_reservations();
        let before = total(h);
        h.prepare_committed_vessels();
        assert!((h.vessel_work(site as u32) - 0.8).abs() < 1e-5);
        h.reserve_cultural_work();
        assert!(
            h.culture.as_ref().unwrap().labor_budget[site] <= 0.48001,
            "culture {} external {} crew {} adults {}",
            h.culture.as_ref().unwrap().labor_budget[site],
            h.sites[site].economy.external[3],
            h.vessel_work(site as u32),
            h.sites[site].demography.ages[1]
        );
        h.prepare_enterprises();
        let crew_income: f64 = h
            .society
            .as_ref()
            .unwrap()
            .household_economy
            .as_ref()
            .unwrap()
            .accounts
            .iter()
            .map(|a| a.employer_income)
            .sum();
        assert!(
            crew_income > 0.,
            "enterprise preparation must retain earlier crew payroll"
        );
        h.prepare_vessels();
        assert!(h.vessel_work(site as u32) >= 0.79999);
        assert!(h.sites[site].economy.external[3] <= 1.280001);
        assert!((total(h) - before).abs() < 1e-8);
        let paid = total(h);
        let work = h.vessel_work(site as u32);
        h.prepare_vessels();
        assert!((h.vessel_work(site as u32) - work).abs() < 1e-6);
        assert!((total(h) - paid).abs() < 1e-8);

        // Exercise actual delivery, with both ports sharing the same cargo footprint.
        h.living = None;
        h.month = 11;
        h.cargo[0].voyage_clock = Some(VoyageClock {
            month: 11,
            remaining: 1.,
        });
        h.cargo[0].arrives = 12;
        let destination = h.cargo[0].to as usize;
        let food_before = h.sites[destination].stocks.stock[1];
        let set_crews = |h: &mut History, work: f32| {
            for port in &mut h.shipping.as_mut().unwrap().ports[..2] {
                port.fleet = Some(Fleet {
                    requested_work: work,
                    vessels: (0..4)
                        .map(|id| Vessel {
                            id,
                            name: format!("fixture {id}"),
                            commissioned: 0,
                            household: None,
                            funded_work: work / 4.,
                            wages_paid: 0.,
                        })
                        .collect(),
                });
            }
        };
        set_crews(h, 0.);
        h.month = 12;
        h.market_arrivals();
        assert_eq!(h.cargo.len(), 1);
        assert_eq!(h.sites[destination].stocks.stock[1], food_before);
        set_crews(h, 0.4); // half the 800 kg shipment can advance per interval
        h.market_arrivals(); // repeated opening cannot consume newly funded work
        assert_eq!(h.cargo[0].voyage_clock.as_ref().unwrap().remaining, 1.);
        h.month = 13;
        h.market_arrivals();
        assert!((h.cargo[0].voyage_clock.as_ref().unwrap().remaining - 0.5).abs() < 1e-6);
        h.cargo = serde_json::from_str(&serde_json::to_string(&h.cargo).unwrap()).unwrap();
        h.month = 14;
        h.market_arrivals();
        assert!(h.cargo.is_empty());
        assert!((h.sites[destination].stocks.stock[1] - food_before - 800.).abs() < 0.1);
    }
}
