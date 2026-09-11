//! Paid institutional representation and remembered public commitments, as game rules.
use crate::{
    civilization::History,
    culture::{Culture, InstitutionKind},
};
use serde::{Deserialize, Serialize};
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Demand {
    Relief,
    Learning,
    Autonomy,
}
impl Demand {
    pub fn concept(self) -> &'static str {
        match self {
            Self::Relief => "gift",
            Self::Learning => "learning",
            Self::Autonomy => "home",
        }
    }
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Petition {
    pub site: u32,
    pub controller: u32,
    pub faction: u32,
    pub institution: u32,
    pub opened: u32,
    pub cause: u64,
    pub demand: Demand,
    pub requested: f64,
    pub pressure: f32,
    pub resolved: Option<u32>,
    pub outcome: Option<u64>,
    #[serde(default)]
    pub resolution_reason: Option<String>,
    pub honored: bool,
    pub paid: f64,
}
/// Credit belongs to the represented interest and the administration that answered it.
/// Older outcomes fade; only the latest local commitment counts.
pub fn credit(h: &History, politics: &crate::politics::Politics, site: u32, interest: u32) -> f32 {
    let Some(g) = &h.governance else { return 0. };
    let Some(p) = g.petitions.iter().rev().find(|p| {
        p.site == site
            && p.resolved.is_some()
            && p.controller == politics.controllers[site as usize]
            && politics.factions[p.faction as usize].interest == interest
    }) else {
        return 0.;
    };
    let age = h.month.saturating_sub(p.resolved.unwrap()) as f32;
    if p.honored {
        0.25 / (1. + age / 60.)
    } else {
        -0.15 / (1. + age / 60.)
    }
}
pub(crate) fn propose(h: &mut History, c: &mut Culture) {
    let (Some(g), Some(politics), Some(society)) = (&h.governance, &h.politics, &h.society) else {
        return;
    };
    if !g.petitions_enabled {
        return;
    }
    let mut proposals = vec![];
    for site in &h.sites {
        if site.abandoned
            || !c.work_allowed(site.id, "petition hearing")
            || c.labor_budget.get(site.id as usize).copied().unwrap_or(0.) < 0.1
            || g.petitions
                .iter()
                .rev()
                .any(|p| p.site == site.id && (p.resolved.is_none() || h.month < p.opened + 60))
        {
            continue;
        }
        let pressure =
            crate::governance::local_pressures(site.stocks.stock[3], h.social_indicators(site.id));
        let controller = h.controller(site.id);
        let mut best = None;
        for institution in c.institutions.iter().filter(|n| {
            n.site == site.id && n.operational() && h.people[n.leader as usize].died.is_none()
        }) {
            let themes = institution
                .tradition
                .and_then(|id| c.traditions.get(id as usize))
                .map(|t| t.themes);
            for faction in politics
                .factions
                .iter()
                .filter(|f| f.civilization == site.civilization && f.cohesion >= 0.25)
            {
                let represented = society
                    .households
                    .iter()
                    .filter(|hh| {
                        hh.site == site.id
                            && !society.relocation.away(hh.id)
                            && !society.relocation.lost_households.contains(&hh.id)
                            && h.people[hh.head as usize].died.is_none()
                            && institution.members.contains(&hh.head)
                            && politics.household_factions.get(hh.id as usize) == Some(&faction.id)
                    })
                    .count();
                if represented == 0 {
                    continue;
                }
                let demand = match faction.interest {
                    0 | 5 | 6 => Demand::Relief,
                    3 | 4 => Demand::Learning,
                    1 | 7 => Demand::Autonomy,
                    _ => continue,
                };
                if demand == Demand::Autonomy && !g.negotiated_autonomy {
                    continue;
                }
                let fit = match demand {
                    Demand::Relief => {
                        pressure[0]
                            + if institution.kind == InstitutionKind::Religious
                                && themes.is_some_and(|t| t.contains(&0) || t.contains(&5))
                            {
                                0.1
                            } else {
                                0.
                            }
                    }
                    Demand::Learning => {
                        if institution.kind == InstitutionKind::Scholarly
                            || institution.kind == InstitutionKind::Craft
                        {
                            0.35 * (1. - pressure[0])
                        } else {
                            0.
                        }
                    }
                    Demand::Autonomy => {
                        g.administrations[site.id as usize].unrest
                            * (1. - g.administrations[site.id as usize].autonomy)
                    }
                };
                let score = fit + 0.1 * faction.support;
                if score < 0.3 {
                    continue;
                }
                if best.as_ref().is_none_or(|(old, _, _, _)| score > *old) {
                    best = Some((score, institution.id, faction.id, demand));
                }
            }
        }
        if let Some((pressure, institution, faction, demand)) = best {
            proposals.push((site.id, controller, institution, faction, demand, pressure));
        }
    }
    for (site, controller, institution, faction, demand, pressure) in proposals {
        c.labor_budget[site as usize] -= 0.1;
        c.labor_spent += 0.1;
        crate::culture::work_requests::record_work(&mut c.work_plans, site, h.month, 0.1);
        let requested = if demand == Demand::Autonomy {
            0.
        } else {
            (h.sites[site as usize].stocks.stock[0] as f64 * 0.05).clamp(5., 100.)
        };
        h.event(
            "civic_petition",
            Some(site),
            None,
            format!(
                "{} represented {} in a {:?} petition; request {:.1}, observed pressure {:.2}",
                c.institutions[institution as usize].name,
                crate::faction_interests::name(
                    h.politics.as_ref().unwrap().factions[faction as usize].interest
                ),
                demand,
                requested,
                pressure
            ),
        );
        let ev = h.events.last_mut().unwrap();
        ev.subjects.extend([
            ("institution".into(), institution),
            ("faction".into(), faction),
        ]);
        h.governance.as_mut().unwrap().petitions.push(Petition {
            site,
            controller,
            institution,
            faction,
            demand,
            requested,
            opened: h.month,
            cause: ev.id,
            pressure,
            resolved: None,
            outcome: None,
            resolution_reason: None,
            honored: false,
            paid: 0.,
        });
    }
}
pub(crate) fn resolve(h: &mut History) {
    let Some(mut g) = h.governance.take() else {
        return;
    };
    for p in &mut g.petitions {
        if p.resolved.is_some() || h.month < p.opened + 3 {
            continue;
        }
        let valid = !h.sites[p.site as usize].abandoned
            && h.controller(p.site) == p.controller
            && h.culture
                .as_ref()
                .is_some_and(|c| c.institutions[p.institution as usize].operational());
        let governing = h.politics.as_ref().unwrap().governing[p.controller as usize];
        let interest = h.politics.as_ref().unwrap().factions[governing as usize].interest;
        let willing = governing == p.faction
            || match p.demand {
                Demand::Relief => p.pressure >= 0.6 || matches!(interest, 0 | 5 | 6),
                Demand::Learning => matches!(interest, 1 | 3 | 4),
                Demand::Autonomy => !crate::faction_interests::resists_autonomy(interest),
            };
        let cash = h.society.as_ref().unwrap().councils[p.controller as usize].treasury;
        let recipients: Vec<_> = h
            .society
            .as_ref()
            .unwrap()
            .households
            .iter()
            .filter(|hh| {
                hh.site == p.site
                    && !h.society.as_ref().unwrap().relocation.away(hh.id)
                    && !h
                        .society
                        .as_ref()
                        .unwrap()
                        .relocation
                        .lost_households
                        .contains(&hh.id)
            })
            .map(|hh| hh.id as usize)
            .collect();
        let can_deliver = match p.demand {
            Demand::Relief => {
                !recipients.is_empty() && h.society.as_ref().unwrap().household_economy.is_some()
            }
            Demand::Autonomy => g.negotiated_autonomy,
            _ => true,
        };
        let honored = valid && willing && can_deliver && cash >= p.requested;
        if !honored && valid && h.month < p.opened + 12 {
            continue;
        }
        if honored {
            let society = h.society.as_mut().unwrap();
            society.councils[p.controller as usize].treasury -= p.requested;
            match p.demand {
                Demand::Relief => {
                    let wallets = society.household_economy.as_mut().unwrap();
                    wallets
                        .accounts
                        .resize(society.households.len(), Default::default());
                    // Equal cash entitlements, not food creation. Actual purchases use the market.
                    let share = p.requested / recipients.len() as f64;
                    for id in recipients {
                        wallets.accounts[id].cash += share;
                        wallets.accounts[id].relief += share;
                    }
                }
                Demand::Learning => {
                    h.culture.as_mut().unwrap().institutions[p.institution as usize].treasury +=
                        p.requested
                }
                Demand::Autonomy => {
                    g.administrations[p.site as usize].autonomy =
                        (g.administrations[p.site as usize].autonomy + 0.15)
                            .min(0.85)
                            .max(g.administrations[p.site as usize].autonomy)
                }
            }
            p.paid = p.requested;
        }
        let reason = if honored {
            "delivered"
        } else if h.sites[p.site as usize].abandoned {
            "settlement unavailable"
        } else if h.controller(p.site) != p.controller {
            "controller changed"
        } else if !valid {
            "institution unavailable"
        } else if !can_deliver {
            "delivery channel unavailable"
        } else if !willing {
            "political opposition"
        } else {
            "insufficient council funds"
        };
        p.resolution_reason = Some(reason.into());
        p.honored = honored;
        p.resolved = Some(h.month);
        let a = &mut g.administrations[p.site as usize];
        if valid {
            a.loyalty = (a.loyalty + if honored { 0.025 } else { -0.015 }).clamp(0., 1.);
        }
        h.event(if honored {"civic_petition_honored"} else {"civic_petition_lapsed"},Some(p.site),None,
            format!("{:?} petition {} {}: {:.1} transferred; grants fund purchasing power or institutional upkeep, not automatic food or knowledge",p.demand,p.cause,if honored{"honored"}else{"closed without delivery"},p.paid));
        let ev = h.events.last_mut().unwrap();
        ev.causes.push(p.cause);
        ev.subjects.extend([
            ("institution".into(), p.institution),
            ("faction".into(), p.faction),
        ]);
        p.outcome = Some(ev.id);
        ev.detail.push_str(&format!("; resolution: {reason}"));
        let c = h.culture.as_mut().unwrap();
        let institution = &c.institutions[p.institution as usize];
        if let Some(tradition) = institution.tradition {
            c.accounts.push(crate::culture::Account{id:c.accounts.len() as u32,tradition,
                author:h.people[institution.leader as usize].died.is_none().then_some(institution.leader),
                institution:Some(institution.id),month:h.month,facts:vec![p.cause,ev.id],
                text:format!("Our congregation interprets the {:?} petition's {} as a test of its public duties; this account does not change anyone's affiliation.",p.demand,if honored{"delivery"}else{"failure"})});
        }
    }
    h.governance = Some(g);
}

pub(crate) fn validate(h: &History, petitions: &[Petition]) -> anyhow::Result<()> {
    use anyhow::ensure;
    for p in petitions {
        ensure!(
            (p.site as usize) < h.sites.len()
                && (p.controller as usize) < h.civilizations.len()
                && h.politics
                    .as_ref()
                    .is_some_and(|x| (p.faction as usize) < x.factions.len())
                && h.culture
                    .as_ref()
                    .is_some_and(|c| (p.institution as usize) < c.institutions.len())
                && p.opened <= h.month
                && p.pressure.is_finite()
                && (0. ..=1.2).contains(&p.pressure)
                && p.requested.is_finite()
                && (0. ..=100.).contains(&p.requested)
                && p.paid.is_finite()
                && p.paid >= 0.
                && p.paid <= p.requested
                && h.events
                    .get(p.cause as usize)
                    .is_some_and(|e| e.kind == "civic_petition" && e.month == p.opened)
                && p.resolved.is_some() == p.outcome.is_some()
                && p.resolved.is_none_or(|m| m >= p.opened + 3 && m <= h.month)
                && p.outcome
                    .is_none_or(|id| h.events.get(id as usize).is_some_and(|e| e.month
                        == p.resolved.unwrap()
                        && e.causes.contains(&p.cause)
                        && e.kind
                            == if p.honored {
                                "civic_petition_honored"
                            } else {
                                "civic_petition_lapsed"
                            }))
                && (if p.honored {
                    p.resolved.is_some() && p.paid == p.requested
                } else {
                    p.paid == 0.
                }),
            "invalid civic petition"
        );
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        catalog::Catalog,
        config::Config,
        culture::Institution,
        gpu::{ContextGpu, Generator},
    };
    #[test]
    #[ignore = "requires hardware GPU"]
    fn representation_delivery_memory_and_failure_are_connected() {
        for seed in [7, 17] {
            let mut g = Generator::new(
                pollster::block_on(ContextGpu::headless()).unwrap(),
                Config {
                    resolution: 64,
                    ecology_resolution: 16,
                    seed,
                    ..Default::default()
                },
                Catalog::bundled().unwrap(),
            )
            .unwrap();
            g.found_civilizations(5).unwrap();
            g.enable_society().unwrap();
            g.enable_politics().unwrap();
            g.enable_governance().unwrap();
            let h = g.civilizations.as_mut().unwrap();
            h.month = 24;
            let hh = h.society.as_ref().unwrap().households[0].clone();
            let site = hh.site;
            let controller = h.controller(site);
            let faction = h
                .politics
                .as_ref()
                .unwrap()
                .factions
                .iter()
                .find(|f| f.civilization == controller && f.interest == 6)
                .unwrap()
                .id;
            h.politics.as_mut().unwrap().household_factions[hh.id as usize] = faction;
            h.politics.as_mut().unwrap().governing[controller as usize] = faction;
            h.sites[site as usize].stocks.stock[3] = 0.8;
            let mut c = h.culture.take().unwrap();
            let institution = c.institutions.len() as u32;
            c.institutions.push(Institution {
                id: institution,
                capacity: None,
                name: "Public hearth".into(),
                kind: InstitutionKind::Religious,
                site,
                tradition: Some(c.site_faith[site as usize]),
                members: vec![hh.head],
                leader: hh.head,
                treasury: 0.,
                active: true,
                founded: 0,
                knowledge: Default::default(),
                property: vec![],
                dues: 0.,
                expenses: 0.,
            });
            c.labor_budget = vec![0.; h.sites.len()];
            propose(h, &mut c);
            assert!(
                h.governance.as_ref().unwrap().petitions.is_empty(),
                "representation takes work"
            );
            c.labor_budget[site as usize] = 0.2;
            h.governance.as_mut().unwrap().petitions_enabled = false;
            propose(h, &mut c);
            assert!(h.governance.as_ref().unwrap().petitions.is_empty());
            assert_eq!(c.labor_budget[site as usize], 0.2);
            h.governance.as_mut().unwrap().petitions_enabled = true;
            propose(h, &mut c);
            assert_eq!(h.governance.as_ref().unwrap().petitions.len(), 1);
            propose(h, &mut c);
            assert_eq!(
                h.governance.as_ref().unwrap().petitions.len(),
                1,
                "one open local petition"
            );
            let accounts = c.accounts.len();
            h.culture = Some(c);
            let total = |h: &History| {
                h.society
                    .as_ref()
                    .unwrap()
                    .councils
                    .iter()
                    .map(|c| c.treasury)
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
                    + h.culture
                        .as_ref()
                        .unwrap()
                        .institutions
                        .iter()
                        .map(|n| n.treasury)
                        .sum::<f64>()
            };
            h.society.as_mut().unwrap().councils[controller as usize].treasury = 1000.;
            // Counterfactual resolution channels share the same finite council budget.
            for demand in [Demand::Learning, Demand::Autonomy] {
                let mut branch = h.clone();
                let p = &mut branch.governance.as_mut().unwrap().petitions[0];
                p.demand = demand;
                if demand == Demand::Autonomy {
                    p.requested = 0.;
                }
                let cash = total(&branch);
                let old_autonomy =
                    branch.governance.as_ref().unwrap().administrations[site as usize].autonomy;
                let old_institution =
                    branch.culture.as_ref().unwrap().institutions[institution as usize].treasury;
                branch.month += 3;
                resolve(&mut branch);
                assert!((total(&branch) - cash).abs() < 1e-8);
                if demand == Demand::Autonomy {
                    assert!(
                        branch.governance.as_ref().unwrap().administrations[site as usize].autonomy
                            > old_autonomy
                    );
                } else {
                    assert!(
                        branch.culture.as_ref().unwrap().institutions[institution as usize]
                            .treasury
                            > old_institution
                    );
                }
            }
            let before = total(h);
            let food = h.sites[site as usize].economy.goods;
            resolve(h);
            assert!(h.governance.as_ref().unwrap().petitions[0]
                .resolved
                .is_none());
            h.month += 3;
            // Turning off future proposals must not discard a pending response.
            h.governance.as_mut().unwrap().petitions_enabled = false;
            let mut resumed: History =
                serde_json::from_value(serde_json::to_value(&*h).unwrap()).unwrap();
            resolve(h);
            resolve(&mut resumed);
            assert_eq!(
                serde_json::to_value(&*h).unwrap(),
                serde_json::to_value(resumed).unwrap(),
                "pending petitions resume identically"
            );
            assert!((total(h) - before).abs() < 1e-8);
            assert_eq!(
                h.sites[site as usize].economy.goods, food,
                "grant does not create food"
            );
            assert!(h.governance.as_ref().unwrap().petitions[0].honored);
            assert_eq!(
                h.governance.as_ref().unwrap().petitions[0]
                    .resolution_reason
                    .as_deref(),
                Some("delivered")
            );
            h.governance.as_mut().unwrap().petitions_enabled = true;
            assert_eq!(h.culture.as_ref().unwrap().accounts.len(), accounts + 1);
            let credit_now = credit(h, h.politics.as_ref().unwrap(), site, 6);
            assert!(credit_now > 0.);
            let events = h.events.len();
            resolve(h);
            assert_eq!(h.events.len(), events, "outcome only once");
            h.month += 120;
            assert!(credit(h, h.politics.as_ref().unwrap(), site, 6) < credit_now);
            let mut c = h.culture.take().unwrap();
            c.labor_budget[site as usize] = 0.2;
            propose(h, &mut c);
            h.culture = Some(c);
            assert_eq!(h.governance.as_ref().unwrap().petitions.len(), 2);
            h.society.as_mut().unwrap().councils[controller as usize].treasury = 0.;
            h.month += 12;
            resolve(h);
            assert!(!h.governance.as_ref().unwrap().petitions[1].honored);
            assert!(credit(h, h.politics.as_ref().unwrap(), site, 6) < 0.);
            validate(h, &h.governance.as_ref().unwrap().petitions).unwrap();
            h.validate_event_links().unwrap();
            let encoded = serde_json::to_value(h.governance.as_ref().unwrap()).unwrap();
            let restored: crate::governance::Governance =
                serde_json::from_value(encoded.clone()).unwrap();
            assert_eq!(serde_json::to_value(restored).unwrap(), encoded);
            let mut old = encoded;
            old.as_object_mut().unwrap().remove("petitions");
            old.as_object_mut().unwrap().remove("petitions_enabled");
            let old: crate::governance::Governance = serde_json::from_value(old).unwrap();
            assert!(old.petitions.is_empty());
            assert!(old.petitions_enabled);
            let other = h
                .sites
                .iter()
                .find(|s| h.controller(s.id) != controller)
                .unwrap()
                .id;
            let parties = [
                controller.min(h.controller(other)),
                controller.max(h.controller(other)),
            ];
            let trust = |h: &History| {
                h.governance
                    .as_ref()
                    .unwrap()
                    .relations
                    .iter()
                    .find(|r| r.parties == parties)
                    .unwrap()
                    .trust
            };
            let contacts = |h: &History| {
                h.governance
                    .as_ref()
                    .unwrap()
                    .relations
                    .iter()
                    .find(|r| r.parties == parties)
                    .unwrap()
                    .trade_contacts
            };
            let mut baseline = h.clone();
            let mut delivered = h.clone();
            let mut lost = h.clone();
            delivered.event(
                "arrival",
                Some(site),
                Some(other),
                "Witnessed food delivery fixture".into(),
            );
            lost.event(
                "appeal_relief_lost",
                Some(site),
                Some(other),
                "Lost shipment fixture".into(),
            );
            baseline.governance_month();
            delivered.governance_month();
            lost.governance_month();
            assert!((trust(&delivered) - trust(&baseline) - 1.).abs() < 1e-5);
            assert_eq!(trust(&lost), trust(&baseline));
            assert_eq!(
                contacts(&delivered),
                contacts(&baseline),
                "aid is not commercial contact"
            );
            delivered.governance_month();
            assert!((trust(&delivered) - trust(&baseline) - 1.).abs() < 1e-5);
        }
    }
}

#[cfg(test)]
mod causal_tests;
