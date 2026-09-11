//! Named subdivisions of installed port fleet assets, with prepaid household crew work.
//! Hull material remains in Port::assets: vessel records never duplicate that inventory.
use crate::{civilization::History, household_economy::withdraw};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Fleet {
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
impl History {
    /// Employ resident households; reserve work from the same craft budget as other services.
    pub(crate) fn prepare_vessels(&mut self) {
        let Some(mut shipping) = self.shipping.take() else {
            return;
        };
        for port in &mut shipping.ports {
            let Some(fleet) = &mut port.fleet else {
                continue;
            };
            for v in &mut fleet.vessels {
                v.funded_work = 0.;
                v.household = None;
            }
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
            let mut work_left = crate::labor::available(s, true, self.living.is_some());
            let wage = 18. * s.economy.prices[crate::economy::FOOD].max(0.01) as f64;
            for (v, &hh) in fleet.vessels.iter_mut().take(hulls).zip(&ids) {
                let work = 0.25_f32
                    .min(work_left)
                    .min((s.economy.finance[0] as f64 / wage) as f32);
                if work <= 0. {
                    break;
                }
                let paid = withdraw(&mut s.economy.finance[0], work as f64 * wage);
                // Cash uses f32 while wallets retain the exact f64 debit. Rounding
                // may pay slightly above the quote; it cannot purchase work
                // beyond the crew reservation.
                let actual = funded_work(paid, wage, work);
                v.household = Some(hh as u32);
                v.funded_work = actual;
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
        assert!(work <= 1.00001);
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
        h.prepare_discoveries();
        assert_eq!(h.sites[site].economy.enterprise_plan, [0.; 4]);
        h.reserve_cultural_work();
        let reserved = h.sites[site].economy.external[3];
        assert!(reserved > 0. && reserved <= 0.240001);
        let before = total(h);
        h.prepare_vessels();
        assert!(h.vessel_work(site as u32) < 1e-6);
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
        h.prepare_vessels();
        assert_eq!(h.shipping.as_ref().unwrap().ports[0].capacity(), 0.);
    }
}
