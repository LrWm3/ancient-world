//! Legitimacy, administrative payments and diplomacy. Monetary stocks remain in councils/sites.
use crate::{civilization::History, gpu::Generator};
use anyhow::{ensure, Result};
use serde::{Deserialize, Serialize};

// Read the last completed social observation; do not age its memory again here.
// Maxima avoid counting the same hunger/displacement through multiple proxies.
pub(crate) fn local_pressures(
    shortage: f32,
    social: Option<&crate::social_state::SocialCell>,
) -> [f32; 2] {
    let Some(c) = social else {
        return [shortage.clamp(0., 1.), 0.];
    };
    [
        shortage
            .max(c.pressure[0])
            .max(c.household_stress[2])
            .clamp(0., 1.),
        c.pressure[2].max(c.housing[2]).clamp(0., 1.),
    ]
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Administration {
    pub controller: u32,
    pub autonomy: f32,
    pub loyalty: f32,
    pub unrest: f32,
    pub unpaid_months: u32,
    pub crisis_months: u32,
    pub wages_paid: f64,
    pub cause: Option<u64>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Relation {
    pub parties: [u32; 2],
    pub trust: f32,
    pub trade_contacts: u32,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Treaty {
    pub id: u32,
    pub parties: [u32; 2],
    pub signed: u32,
    pub expires: u32,
    pub expired: Option<u32>,
    pub cause: u64,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Governance {
    #[serde(default)]
    pub petitions: Vec<crate::civic_petitions::Petition>,
    /// Older histories retain scenario-only autonomy unless explicitly enabled.
    #[serde(default)]
    pub negotiated_autonomy: bool,
    pub version: u32,
    pub started: u32,
    pub administrations: Vec<Administration>,
    pub relations: Vec<Relation>,
    pub treaties: Vec<Treaty>,
    pub event_cursor: usize,
}
fn pair(a: u32, b: u32) -> [u32; 2] {
    if a < b {
        [a, b]
    } else {
        [b, a]
    }
}
/// Snapshot commitments before paying any town. Round down so allocated f32
/// payments cannot exceed either the entitlement or a council's f64 treasury.
fn payroll_plan(demands: &[(usize, f32)], treasuries: &[f64]) -> Vec<f32> {
    let mut totals = vec![0.; treasuries.len()];
    for &(controller, required) in demands {
        totals[controller] += required as f64;
    }
    demands
        .iter()
        .map(|&(controller, required)| {
            let fraction = if totals[controller] > 0. {
                (treasuries[controller] / totals[controller]).clamp(0., 1.)
            } else {
                1.
            };
            let entitlement = required as f64 * fraction;
            let mut paid = entitlement as f32;
            if paid as f64 > entitlement {
                paid = f32::from_bits(paid.to_bits().saturating_sub(1));
            }
            paid
        })
        .collect()
}
impl Governance {
    pub fn protected(&self, a: u32, b: u32, month: u32) -> bool {
        self.treaties
            .iter()
            .any(|t| t.parties == pair(a, b) && t.signed <= month && month < t.expires)
    }
    pub fn validate(&self, h: &History) -> Result<()> {
        crate::civic_petitions::validate(h, &self.petitions)?;
        ensure!(
            h.politics.is_some()
                && self.version == 1
                && self.started <= h.month
                && self.event_cursor <= h.events.len(),
            "invalid governance baseline"
        );
        ensure!(
            self.administrations.len() == h.sites.len(),
            "missing site administration"
        );
        for (i, a) in self.administrations.iter().enumerate() {
            ensure!(
                a.controller == h.controller(i as u32)
                    && [a.autonomy, a.loyalty, a.unrest]
                        .iter()
                        .all(|v| (0. ..=1.).contains(v))
                    && a.wages_paid.is_finite()
                    && a.wages_paid >= 0.
                    && a.cause.is_none_or(|c| (c as usize) < h.events.len()),
                "invalid administration"
            );
        }
        let expected = h.civilizations.len() * (h.civilizations.len() - 1) / 2;
        ensure!(
            self.relations.len() == expected
                && self
                    .relations
                    .windows(2)
                    .all(|r| r[0].parties < r[1].parties)
                && self.relations.iter().all(|r| r.parties[0] < r.parties[1]
                    && (r.parties[1] as usize) < h.civilizations.len()
                    && (0. ..=100.).contains(&r.trust)),
            "invalid diplomatic relations"
        );
        for (i, t) in self.treaties.iter().enumerate() {
            ensure!(
                t.id == i as u32
                    && t.parties[0] < t.parties[1]
                    && (t.parties[1] as usize) < h.civilizations.len()
                    && t.signed >= self.started
                    && t.signed <= h.month
                    && t.expires > t.signed
                    && t.expires - t.signed <= 600
                    && t.expired.is_none_or(|m| m >= t.expires && m <= h.month)
                    && (t.cause as usize) < h.events.len(),
                "invalid treaty"
            );
            ensure!(
                !self.treaties[..i].iter().any(|o| o.parties == t.parties
                    && t.signed < o.expires
                    && o.signed < t.expires),
                "overlapping treaties"
            );
        }
        Ok(())
    }
}
impl History {
    pub(crate) fn prepare_governance(&mut self) {
        let Some(mut g) = self.governance.take() else {
            return;
        };
        while g.administrations.len() < self.sites.len() {
            let i = g.administrations.len();
            let controller = self.controller(i as u32);
            g.administrations.push(Administration {
                controller,
                autonomy: 0.25,
                loyalty: if controller == self.sites[i].civilization {
                    0.8
                } else {
                    0.35
                },
                unrest: 0.,
                unpaid_months: 0,
                crisis_months: 0,
                wages_paid: 0.,
                cause: None,
            });
        }
        // A capture is synchronized both after battle and at the final month boundary.
        for (i, a) in g.administrations.iter_mut().enumerate() {
            let controller = self.controller(i as u32);
            if a.controller != controller {
                a.controller = controller;
                a.loyalty = 0.25;
                a.unrest = 0.3;
                a.crisis_months = 0;
                a.unpaid_months = 0;
                a.cause = self
                    .events
                    .iter()
                    .rev()
                    .find(|e| e.site == Some(i as u32) && e.kind == "peace")
                    .map(|e| e.id);
            }
        }
        self.governance = Some(g);
    }
    pub(crate) fn governance_month(&mut self) {
        let office_capacity: Vec<_> = (0..self.sites.len())
            .map(|s| self.office_capacity(s as u32))
            .collect();
        let occupation: Vec<_> = (0..self.sites.len())
            .map(|s| self.occupation_strength(s as u32))
            .collect();
        self.prepare_governance();
        let Some(mut g) = self.governance.take() else {
            return;
        };
        let end = self.events.len();
        for e in &self.events[g.event_cursor..end] {
            if e.kind == "war_declared" {
                if let Some(w) = self
                    .politics
                    .as_ref()
                    .unwrap()
                    .wars
                    .iter()
                    .find(|w| w.cause == e.id)
                {
                    if let Some(r) = g
                        .relations
                        .iter_mut()
                        .find(|r| r.parties == pair(w.attacker, w.defender))
                    {
                        r.trust = (r.trust - 35.).max(0.);
                    }
                }
            } else if e.kind == "market_arrival" || e.kind == "arrival" {
                if let (Some(a), Some(b)) = (e.site, e.other) {
                    if let Some(r) = g
                        .relations
                        .iter_mut()
                        .find(|r| r.parties == pair(self.controller(a), self.controller(b)))
                    {
                        r.trust = (r.trust + if e.kind == "arrival" { 1. } else { 0.2 }).min(100.);
                        if e.kind == "market_arrival" {
                            r.trade_contacts = r.trade_contacts.saturating_add(1);
                        }
                    }
                }
            }
        }
        g.event_cursor = end;
        for t in &mut g.treaties {
            if t.expired.is_none() && self.month >= t.expires {
                t.expired = Some(self.month);
                self.event(
                    "treaty_expired",
                    None,
                    None,
                    format!("Non-aggression treaty {} expired", t.id),
                );
                self.events.last_mut().unwrap().causes.push(t.cause);
            }
        }
        let demands: Vec<_> = self
            .sites
            .iter()
            .zip(&g.administrations)
            .map(|(s, a)| {
                let required = if s.abandoned {
                    0.
                } else {
                    s.stocks.stock[0]
                        * if a.controller != s.civilization {
                            0.04
                        } else {
                            0.01
                        }
                        * (1. - a.autonomy * 0.5)
                };
                (a.controller as usize, required)
            })
            .collect();
        let treasuries: Vec<_> = self
            .society
            .as_ref()
            .unwrap()
            .councils
            .iter()
            .map(|c| c.treasury)
            .collect();
        let payments = payroll_plan(&demands, &treasuries);
        for i in 0..self.sites.len() {
            if self.sites[i].abandoned {
                continue;
            }
            let [hunger, disruption] = local_pressures(
                self.sites[i].stocks.stock[3],
                self.social_indicators(i as u32),
            );
            let a = &mut g.administrations[i];
            let foreign = a.controller != self.sites[i].civilization;
            let required = demands[i].1;
            let council = &mut self.society.as_mut().unwrap().councils[a.controller as usize];
            // Round down to the available f32 payment rather than overdrawing a f64 treasury.
            let mut paid = payments[i].min(council.treasury as f32);
            if paid as f64 > council.treasury {
                paid = f32::from_bits(paid.to_bits().saturating_sub(1));
            }
            council.treasury -= paid as f64;
            self.sites[i].economy.finance[0] += paid;
            a.wages_paid += paid as f64;
            let funded = if required > 0. { paid / required } else { 1. };
            a.unpaid_months = if funded < 0.9 {
                a.unpaid_months.saturating_add(1)
            } else {
                0
            };
            let tax = council.tax_rate * (1. - a.autonomy * 0.75) * office_capacity[i];
            a.loyalty = (a.loyalty + 0.008 * funded * office_capacity[i] + 0.006 * a.autonomy
                - 0.004 * (1. - office_capacity[i])
                - 0.002 * occupation[i]
                - 0.012 * (1. - funded)
                - 0.02 * hunger
                - 0.008 * disruption
                - tax * 0.08)
                .clamp(0., 1.);
            a.unrest = (a.unrest
                + 0.003 * occupation[i]
                + hunger * 0.03
                + disruption * 0.012
                + (1. - funded) * 0.012
                + tax * 0.1
                - 0.012 * a.loyalty
                - 0.012 * a.autonomy)
                .clamp(0., 1.);
            let previous_crisis = a.crisis_months;
            a.crisis_months = if foreign && a.autonomy < 0.75 && a.loyalty < 0.25 && a.unrest > 0.65
            {
                a.crisis_months + 1
            } else {
                0
            };
            if a.crisis_months == 1 {
                self.event(
                    "governance_crisis",
                    Some(i as u32),
                    None,
                    format!("Low local legitimacy threatens territorial control: payroll {:.0}%, effective tax {:.1}%, hunger pressure {:.0}%, crowding/disruption {:.0}%, loyalty {:.0}%, unrest {:.0}%", funded * 100., tax * 100., hunger * 100., disruption * 100., a.loyalty * 100., a.unrest * 100.),
                );
                if let Some(c) = a.cause {
                    self.events.last_mut().unwrap().causes.push(c);
                }
                a.cause = Some(self.events.last().unwrap().id);
            }
            if previous_crisis > 0 && a.crisis_months == 0 {
                self.event("governance_recovery", Some(i as u32), None,
                    format!("Administrative crisis interrupted after {previous_crisis} months: autonomy {:.0}%, loyalty {:.0}%, unrest {:.0}%; hunger pressure {:.0}%, crowding/disruption {:.0}%", a.autonomy * 100., a.loyalty * 100., a.unrest * 100., hunger * 100., disruption * 100.));
                if let Some(cause) = a.cause {
                    self.events.last_mut().unwrap().causes.push(cause);
                }
                a.cause = Some(self.events.last().unwrap().id);
            }
            if g.negotiated_autonomy && self.month % 3 == 0 && (3..12).contains(&a.crisis_months) {
                let politics = self.politics.as_ref().unwrap();
                let faction =
                    &politics.factions[politics.governing[a.controller as usize] as usize];
                // Centralizing interests hold out on devolved revenue unless payroll fails.
                if !crate::faction_interests::resists_autonomy(faction.interest)
                    || a.unpaid_months >= 6
                {
                    let previous = a.autonomy;
                    a.autonomy = 0.75;
                    self.event("autonomy_negotiated", Some(i as u32), None,
                        format!("{} council granted 75% local autonomy after {} crisis months (previous {:.0}%, unpaid {} months); future tax collection falls from {:.0}% to 44% of the standard rate", crate::faction_interests::name(faction.interest), a.crisis_months, previous * 100., a.unpaid_months, (1. - previous * 0.75) * 100.));
                    if let Some(cause) = a.cause {
                        self.events.last_mut().unwrap().causes.push(cause);
                    }
                    a.cause = Some(self.events.last().unwrap().id);
                }
            }
            if a.crisis_months >= 12 + (occupation[i] * 3.).ceil() as u32 {
                let previous = a.controller;
                let restored = self.sites[i].civilization;
                self.politics.as_mut().unwrap().controllers[i] = restored;
                a.controller = restored;
                a.loyalty = 0.5;
                a.unrest = 0.3;
                a.crisis_months = 0;
                a.unpaid_months = 0;
                self.event("secession",Some(i as u32),None,format!("{} ceased recognizing {} after sustained governance crisis; local administration restored",self.sites[i].name,self.civilizations[previous as usize].name));
                if let Some(c) = a.cause {
                    self.events.last_mut().unwrap().causes.push(c);
                }
                a.cause = Some(self.events.last().unwrap().id);
            }
        }
        self.governance = Some(g);
        crate::civic_petitions::resolve(self);
    }
    pub(crate) fn governance_year(&mut self) {
        let Some(g) = &self.governance else {
            return;
        };
        let offers: Vec<_> = g
            .relations
            .iter()
            .filter(|r| r.trust >= 30. && r.trade_contacts >= 12)
            .map(|r| r.parties)
            .collect();
        for [a, b] in offers {
            let _ = self.sign_treaty(a, b, 120);
        }
    }
    pub(crate) fn sign_treaty(&mut self, a: u32, b: u32, months: u32) -> Result<u32> {
        ensure!(
            a != b
                && (a as usize) < self.civilizations.len()
                && (b as usize) < self.civilizations.len()
                && (12..=600).contains(&months),
            "invalid treaty parties or duration"
        );
        let g = self
            .governance
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("enable governance first"))?;
        ensure!(
            !g.protected(a, b, self.month),
            "existing non-aggression treaty"
        );
        ensure!(
            !self
                .politics
                .as_ref()
                .unwrap()
                .wars
                .iter()
                .any(|w| w.ended.is_none() && pair(w.attacker, w.defender) == pair(a, b)),
            "cannot sign a non-aggression pact during an active war"
        );
        ensure!(
            self.society
                .as_ref()
                .unwrap()
                .routes
                .iter()
                .any(|r| r.open
                    && pair(self.controller(r.from), self.controller(r.to)) == pair(a, b))
                || self.trade_distances().is_some_and(|roads| {
                    let n = self.sites.len();
                    self.sea_quotes(&roads).iter().enumerate().any(|(i, q)| {
                        q.is_some()
                            && pair(
                                self.controller((i / n) as u32),
                                self.controller((i % n) as u32),
                            ) == pair(a, b)
                    })
                }),
            "no open diplomatic contact route"
        );
        let id = g.treaties.len() as u32;
        let contact = self
            .events
            .iter()
            .rev()
            .find(|e| {
                e.kind == "market_arrival"
                    && e.site.zip(e.other).is_some_and(|(x, y)| {
                        pair(self.controller(x), self.controller(y)) == pair(a, b)
                    })
            })
            .map(|e| e.id);
        self.event(
            "treaty_signed",
            None,
            None,
            format!(
                "{} and {} agreed to {} months of non-aggression (treaty {id})",
                self.civilizations[a as usize].name, self.civilizations[b as usize].name, months
            ),
        );
        if let Some(c) = contact {
            self.events.last_mut().unwrap().causes.push(c);
        }
        let cause = self.events.last().unwrap().id;
        self.governance.as_mut().unwrap().treaties.push(Treaty {
            id,
            parties: pair(a, b),
            signed: self.month,
            expires: self.month + months,
            expired: None,
            cause,
        });
        Ok(id)
    }
}
impl Generator {
    pub fn enable_governance(&mut self) -> Result<()> {
        let cells = self.snapshot()?;
        let mut h = self
            .civilizations
            .clone()
            .ok_or_else(|| anyhow::anyhow!("no history"))?;
        ensure!(
            h.politics.is_some() && h.governance.is_none(),
            "governance requires politics without an existing governance baseline"
        );
        let mut relations = vec![];
        for a in 0..h.civilizations.len() as u32 {
            for b in a + 1..h.civilizations.len() as u32 {
                relations.push(Relation {
                    parties: [a, b],
                    trust: 10.,
                    trade_contacts: 0,
                });
            }
        }
        h.governance = Some(Governance {
            petitions: vec![],
            negotiated_autonomy: true,
            version: 1,
            started: h.month,
            administrations: vec![],
            relations,
            treaties: vec![],
            event_cursor: h.events.len(),
        });
        h.prepare_governance();
        h.event(
            "governance_baseline",
            None,
            None,
            "Local legitimacy, paid administration and diplomacy established".into(),
        );
        h.validate(&cells)?;
        self.civilizations = Some(h);
        Ok(())
    }
    pub fn set_negotiated_autonomy(&mut self, enabled: bool) -> Result<()> {
        self.validate_living_boundary()?;
        ensure!(
            self.progress.stage == crate::gpu::Stage::Boundary,
            "governance policy requires a completed boundary"
        );
        let cells = self.snapshot()?;
        let mut h = self
            .civilizations
            .clone()
            .ok_or_else(|| anyhow::anyhow!("no history"))?;
        let g = h
            .governance
            .as_mut()
            .ok_or_else(|| anyhow::anyhow!("enable governance first"))?;
        if g.negotiated_autonomy == enabled {
            return Ok(());
        }
        g.negotiated_autonomy = enabled;
        h.event(
            "autonomy_negotiation_policy",
            None,
            None,
            format!(
                "Council negotiation of local autonomy {}",
                if enabled { "enabled" } else { "disabled" }
            ),
        );
        h.validate(&cells)?;
        self.civilizations = Some(h);
        Ok(())
    }
    pub fn set_autonomy(&mut self, site: u32, autonomy: f32) -> Result<()> {
        self.validate_living_boundary()?;
        ensure!(
            self.progress.stage == crate::gpu::Stage::Boundary,
            "autonomy requires a completed boundary"
        );
        ensure!(
            (0. ..=1.).contains(&autonomy),
            "autonomy must be between zero and one"
        );
        let cells = self.snapshot()?;
        let mut h = self
            .civilizations
            .clone()
            .ok_or_else(|| anyhow::anyhow!("no history"))?;
        let a = h
            .governance
            .as_mut()
            .and_then(|g| g.administrations.get_mut(site as usize))
            .ok_or_else(|| anyhow::anyhow!("unknown administration"))?;
        let previous = a.cause;
        a.autonomy = autonomy;
        h.event(
            "autonomy_policy",
            Some(site),
            None,
            format!("Local autonomy set to {:.0}%", autonomy * 100.),
        );
        if let Some(c) = previous {
            h.events.last_mut().unwrap().causes.push(c);
        }
        h.governance.as_mut().unwrap().administrations[site as usize].cause =
            Some(h.events.last().unwrap().id);
        h.sync_offices();
        h.validate(&cells)?;
        self.civilizations = Some(h);
        Ok(())
    }
    pub fn sign_nonaggression(&mut self, a: u32, b: u32, months: u32) -> Result<u32> {
        let cells = self.snapshot()?;
        let mut h = self
            .civilizations
            .clone()
            .ok_or_else(|| anyhow::anyhow!("no history"))?;
        let id = h.sign_treaty(a, b, months)?;
        h.validate(&cells)?;
        self.civilizations = Some(h);
        Ok(id)
    }
}

#[cfg(test)]
mod payroll_tests {
    use super::*;
    #[test]
    fn local_pressure_uses_memory_without_double_counting() {
        use crate::social_state::SocialCell;
        let mut c = SocialCell::default();
        assert_eq!(local_pressures(0.3, None), [0.3, 0.]);
        assert_eq!(local_pressures(0.3, Some(&c)), [0.3, 0.]);
        c.pressure = [0.5, 1., 0.4, 1.];
        c.household_stress[2] = 0.8;
        c.housing[2] = 0.7;
        assert_eq!(local_pressures(0.3, Some(&c)), [0.8, 0.7]);
        assert_eq!(local_pressures(1., Some(&c)), [1., 0.7]);
        c.pressure[1] = 0.;
        c.pressure[3] = 0.;
        assert_eq!(local_pressures(0.3, Some(&c)), [0.8, 0.7]);
    }

    #[test]
    #[ignore = "requires hardware GPU"]
    fn remembered_household_hardship_changes_control_and_recovery() {
        let mut g = Generator::new(
            pollster::block_on(crate::gpu::ContextGpu::headless()).unwrap(),
            crate::config::Config {
                resolution: 32,
                ecology_resolution: 16,
                ..Default::default()
            },
            crate::catalog::Catalog::bundled().unwrap(),
        )
        .unwrap();
        g.found_civilizations(5).unwrap();
        g.enable_society().unwrap();
        g.enable_politics().unwrap();
        g.enable_governance().unwrap();
        let h = g.civilizations.as_mut().unwrap();
        h.governance.as_mut().unwrap().negotiated_autonomy = false;
        h.politics.as_mut().unwrap().controllers[1] = 0;
        h.prepare_governance();
        for site in &mut h.sites {
            site.stocks.stock[0] = 100.;
            site.stocks.stock[3] = 0.;
        }
        for council in &mut h.society.as_mut().unwrap().councils {
            council.treasury = 100_000.;
            council.tax_rate = 0.;
        }
        for cell in &mut h
            .society
            .as_mut()
            .unwrap()
            .indicators
            .as_mut()
            .unwrap()
            .sites
        {
            *cell = Default::default();
        }
        let a = &mut h.governance.as_mut().unwrap().administrations[1];
        a.autonomy = 0.;
        a.loyalty = 0.3;
        a.unrest = 0.6;
        let baseline = h.clone();
        let c = &mut h
            .society
            .as_mut()
            .unwrap()
            .indicators
            .as_mut()
            .unwrap()
            .sites[1];
        c.household_stress[2] = 1.;
        c.housing[2] = 1.;
        let mut secure = baseline.clone();
        let mut resumed: History =
            serde_json::from_value(serde_json::to_value(&*h).unwrap()).unwrap();
        for _ in 0..4 {
            h.month += 1;
            h.governance_month();
            secure.month += 1;
            secure.governance_month();
            resumed.month += 1;
            resumed.governance_month();
        }
        assert_eq!(
            serde_json::to_value(&*h).unwrap(),
            serde_json::to_value(&resumed).unwrap()
        );
        let crisis = h
            .events
            .iter()
            .find(|e| e.kind == "governance_crisis" && e.site == Some(1))
            .unwrap();
        assert!(crisis.detail.contains("payroll 100%"));
        assert!(crisis.detail.contains("hunger pressure 100%"));
        let crisis_id = crisis.id;
        let mut autonomy = h.clone();
        autonomy.governance.as_mut().unwrap().administrations[1].autonomy = 1.;
        autonomy.month += 1;
        autonomy.governance_month();
        let recovery = autonomy
            .events
            .iter()
            .find(|e| e.kind == "governance_recovery" && e.site == Some(1))
            .unwrap();
        assert!(recovery.causes.contains(&crisis_id));
        let mut negotiated = h.clone();
        negotiated.governance.as_mut().unwrap().negotiated_autonomy = true;
        negotiated.politics.as_mut().unwrap().governing[0] = 0;
        let mut negotiated_resumed: History =
            serde_json::from_value(serde_json::to_value(&negotiated).unwrap()).unwrap();
        let mut hardline = negotiated.clone();
        hardline.politics.as_mut().unwrap().governing[0] = 2;
        let mut old = serde_json::to_value(negotiated.governance.as_ref().unwrap()).unwrap();
        old.as_object_mut().unwrap().remove("negotiated_autonomy");
        assert!(
            !serde_json::from_value::<Governance>(old)
                .unwrap()
                .negotiated_autonomy
        );
        let mut relieved = h.clone();
        // Controlled pressure ablation: interventions reach governance only through observations.
        *relieved
            .society
            .as_mut()
            .unwrap()
            .indicators
            .as_mut()
            .unwrap()
            .sites
            .get_mut(1)
            .unwrap() = Default::default();
        for _ in 0..26 {
            for case in [
                &mut *h,
                &mut secure,
                &mut autonomy,
                &mut relieved,
                &mut negotiated,
                &mut negotiated_resumed,
                &mut hardline,
            ] {
                case.month += 1;
                case.governance_month();
            }
        }
        assert_eq!(h.controller(1), h.sites[1].civilization);
        assert_eq!(secure.controller(1), 0);
        assert_eq!(autonomy.controller(1), 0);
        assert_eq!(relieved.controller(1), 0);
        assert_eq!(negotiated.controller(1), 0);
        assert_eq!(hardline.controller(1), hardline.sites[1].civilization);
        assert_eq!(
            negotiated.governance.as_ref().unwrap().administrations[1].autonomy,
            0.75
        );
        let concession = negotiated
            .events
            .iter()
            .find(|e| e.kind == "autonomy_negotiated" && e.site == Some(1))
            .unwrap();
        assert!(concession.causes.contains(&crisis_id));
        assert!(negotiated
            .events
            .iter()
            .any(|e| e.kind == "governance_recovery" && e.causes.contains(&concession.id)));
        assert_eq!(
            serde_json::to_value(&negotiated).unwrap(),
            serde_json::to_value(&negotiated_resumed).unwrap()
        );
        assert!(relieved
            .events
            .iter()
            .any(|e| e.kind == "governance_recovery" && e.site == Some(1)));
        let cash = |h: &History| {
            h.sites
                .iter()
                .map(|s| s.economy.finance[0] as f64)
                .sum::<f64>()
                + h.society
                    .as_ref()
                    .unwrap()
                    .councils
                    .iter()
                    .map(|c| c.treasury)
                    .sum::<f64>()
        };
        for case in [&*h, &secure, &autonomy, &relieved, &negotiated, &hardline] {
            assert!((cash(case) - cash(&baseline)).abs() < 0.02);
            assert_eq!(case.sites[1].stocks.stock, baseline.sites[1].stocks.stock);
        }
    }

    #[test]
    fn scarce_payroll_is_proportional_and_order_independent() {
        let demands = [(0, 10.), (0, 30.), (1, 20.), (0, 0.)];
        let wages = payroll_plan(&demands, &[20., 100.]);
        assert_eq!(wages, vec![5., 15., 20., 0.]);
        let reverse: Vec<_> = demands.into_iter().rev().collect();
        assert_eq!(
            payroll_plan(&reverse, &[20., 100.])
                .into_iter()
                .rev()
                .collect::<Vec<_>>(),
            wages
        );
        assert_eq!(payroll_plan(&demands, &[0., 0.]), vec![0.; 4]);
        assert_eq!(
            payroll_plan(&demands, &[100., 100.]),
            vec![10., 30., 20., 0.]
        );
        for budget in [0.001, 0.1, 1.23456789, 39.99999999] {
            let p = payroll_plan(&demands, &[budget, 0.]);
            assert!(p[0] as f64 + p[1] as f64 <= budget);
            assert!((p[0] / 10. - p[1] / 30.).abs() < 1e-7);
        }
    }
    #[test]
    #[ignore = "requires hardware GPU"]
    fn shared_treasury_shortfalls_reach_both_administrations() {
        let mut g = Generator::new(
            pollster::block_on(crate::gpu::ContextGpu::headless()).unwrap(),
            crate::config::Config {
                resolution: 32,
                ecology_resolution: 16,
                ..Default::default()
            },
            crate::catalog::Catalog::bundled().unwrap(),
        )
        .unwrap();
        g.found_civilizations(5).unwrap();
        g.enable_society().unwrap();
        g.enable_politics().unwrap();
        g.enable_governance().unwrap();
        let h = g.civilizations.as_mut().unwrap();
        h.governance.as_mut().unwrap().negotiated_autonomy = false;
        h.politics.as_mut().unwrap().controllers[1] = 0;
        h.prepare_governance();
        for s in &mut h.sites {
            s.stocks.stock[0] = 100.;
            s.stocks.stock[3] = 0.;
        }
        for a in &mut h.governance.as_mut().unwrap().administrations {
            a.autonomy = 0.;
            a.loyalty = 0.5;
            a.unrest = 0.2;
        }
        // One domestic town needs 1 money; its occupied neighbor needs 4.
        h.society.as_mut().unwrap().councils[0].treasury = 2.5;
        let before = h.clone();
        let cash = |h: &History| {
            h.sites
                .iter()
                .map(|s| s.economy.finance[0] as f64)
                .sum::<f64>()
                + h.society
                    .as_ref()
                    .unwrap()
                    .councils
                    .iter()
                    .map(|c| c.treasury)
                    .sum::<f64>()
        };
        let total = cash(h);
        h.governance_month();
        let admins = &h.governance.as_ref().unwrap().administrations;
        assert_eq!(
            admins[0].wages_paid
                - before.governance.as_ref().unwrap().administrations[0].wages_paid,
            0.5
        );
        assert_eq!(
            admins[1].wages_paid
                - before.governance.as_ref().unwrap().administrations[1].wages_paid,
            2.
        );
        assert_eq!(admins[0].unpaid_months, 1);
        assert_eq!(admins[1].unpaid_months, 1);
        assert!((cash(h) - total).abs() < 0.001);
        let mut funded = before.clone();
        funded.society.as_mut().unwrap().councils[0].treasury = 5.;
        funded.governance_month();
        for (i, admin) in admins.iter().enumerate().take(2) {
            let full = &funded.governance.as_ref().unwrap().administrations[i];
            assert_eq!(full.unpaid_months, 0);
            assert!(full.loyalty > admin.loyalty);
            assert!(full.unrest < admin.unrest);
        }
        let mut resumed: History =
            serde_json::from_value(serde_json::to_value(&*h).unwrap()).unwrap();
        h.month += 1;
        resumed.month += 1;
        h.governance_month();
        resumed.governance_month();
        assert_eq!(
            serde_json::to_value(&*h).unwrap(),
            serde_json::to_value(resumed).unwrap()
        );
    }
}
