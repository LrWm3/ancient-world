//! Local family cash sharing: opening-wallet proposals, recipient caps, once-only commit.
use super::{HouseholdEconomy, RetailPlan};
use crate::{civilization::History, participation::Presence};
use anyhow::{ensure, Result};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct FamilySupportPolicy {
    /// Monthly share of cash above the donor's full private food budget.
    pub surplus_share: f64,
    /// Recipient food entitlement to target, including common access.
    pub food_target: f64,
}
impl Default for FamilySupportPolicy {
    fn default() -> Self {
        Self {
            surplus_share: 0.25,
            food_target: 0.9,
        }
    }
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FamilyGift {
    pub donor: u32,
    pub recipient: u32,
    pub site: u32,
    pub amount: f64,
}
#[derive(Clone, Copy, Debug)]
pub(super) struct Link {
    donor: usize,
    recipient: usize,
    strength: f64,
}

pub(super) fn links(h: &History) -> Vec<Link> {
    if h
        .society
        .as_ref()
        .and_then(|s| s.household_economy.as_ref())
        .is_none_or(|e| e.family_support.is_none())
    {
        return vec![];
    }
    let (Some(pool), Some(culture), Some(politics)) = (&h.participation, &h.culture, &h.politics)
    else {
        return vec![];
    };
    let parents: BTreeMap<_, _> = politics.kin.iter().map(|k| (k.person, k.parents)).collect();
    let mut children: BTreeMap<u32, BTreeSet<u32>> = BTreeMap::new();
    for (&person, ancestors) in &parents {
        for parent in ancestors.iter().flatten() {
            children.entry(*parent).or_default().insert(person);
        }
    }
    let resident = |person: u32| {
        let r = pool.residents.get(&person)?;
        if h.people.get(person as usize)?.died.is_some() {
            return None;
        }
        let Presence::Resident(site) = r.presence else {
            return None;
        };
        Some((r.household? as usize, site))
    };
    let mut pairs: BTreeMap<(usize, usize), f64> = BTreeMap::new();
    for (&person, r) in &pool.residents {
        let Some((donor, site)) = resident(person) else {
            continue;
        };
        if h.month as i64 - i64::from(h.people[person as usize].born) < 216 {
            continue;
        }
        let Some(agent) = culture
            .agents
            .get(r.person as usize)
            .filter(|a| a.person == person)
        else {
            continue;
        };
        let generosity = agent.traits[1].clamp(0., 1.);
        if generosity <= 0. {
            continue;
        }
        let mut kin = children.get(&person).cloned().unwrap_or_default();
        for parent in parents.get(&person).into_iter().flatten().flatten() {
            kin.insert(*parent);
            if let Some(siblings) = children.get(parent) {
                kin.extend(siblings);
            }
        }
        for relative in kin {
            let Some((recipient, destination)) = resident(relative) else {
                continue;
            };
            if donor == recipient || destination != site {
                continue;
            }
            let affinity = crate::kin_support::support_affinity(
                person,
                relative,
                agent.relations.get(&relative).copied(),
                &parents,
                true,
            );
            if affinity <= 0.2 {
                continue;
            }
            let strength = f64::from(generosity * affinity);
            pairs
                .entry((donor, recipient))
                .and_modify(|v| *v = v.max(strength))
                .or_insert(strength);
        }
    }
    pairs
        .into_iter()
        .map(|((donor, recipient), strength)| Link {
            donor,
            recipient,
            strength,
        })
        .collect()
}
#[derive(Clone, Copy)]
struct Budget {
    site: u32,
    surplus: f64,
    deficit: f64,
}

// Gather all offers against opening cash. A recipient cap cannot increase an offer;
// transfers received here are never reoffered in the same month.
fn allocate(budgets: &[Option<Budget>], links: &[Link], share: f64) -> Vec<FamilyGift> {
    let mut ordered = links.to_vec();
    ordered.sort_by_key(|l| (l.donor, l.recipient));
    let mut weighted = vec![0.; budgets.len()];
    let mut strength = vec![0_f64; budgets.len()];
    for l in &ordered {
        if let (Some(a), Some(b)) = (budgets[l.donor], budgets[l.recipient]) {
            if a.site == b.site && b.deficit > 0. {
                weighted[l.donor] += l.strength * b.deficit;
                strength[l.donor] = strength[l.donor].max(l.strength);
            }
        }
    }
    let mut proposed = vec![];
    let mut incoming = vec![0.; budgets.len()];
    for l in ordered {
        let (Some(a), Some(b)) = (budgets[l.donor], budgets[l.recipient]) else {
            continue;
        };
        if a.site != b.site || weighted[l.donor] <= 0. {
            continue;
        }
        let amount =
            a.surplus * share * strength[l.donor] * l.strength * b.deficit / weighted[l.donor];
        if amount > 0. {
            incoming[l.recipient] += amount;
            proposed.push(FamilyGift {
                donor: l.donor as u32,
                recipient: l.recipient as u32,
                site: a.site,
                amount,
            });
        }
    }
    for gift in &mut proposed {
        let id = gift.recipient as usize;
        gift.amount *= (budgets[id].unwrap().deficit / incoming[id]).min(1.);
    }
    proposed
}

pub(super) fn settle(e: &mut HouseholdEconomy, plans: &[RetailPlan], links: &[Link], month: u32) {
    let Some(policy) = e.family_support else {
        return;
    };
    if e.family_support_month == Some(month) {
        return;
    }
    let mut budgets = vec![None; e.accounts.len()];
    for p in plans {
        let common = p.free / p.need.max(1e-12);
        for &id in &p.ids {
            let a = &e.accounts[id];
            let reserve = (1. - common).max(0.) * a.need * p.price;
            let target = (policy.food_target - common).max(0.) * a.need * p.price;
            budgets[id] = Some(Budget {
                site: p.site as u32,
                surplus: (a.cash - reserve).max(0.),
                deficit: (target - a.cash).max(0.),
            });
        }
    }
    let floors: Vec<_> = budgets
        .iter()
        .zip(&e.accounts)
        .map(|(b, a)| b.map_or(a.cash, |b| a.cash - b.surplus))
        .collect();
    let mut gifts = allocate(&budgets, links, policy.surplus_share);
    for gift in &mut gifts {
        let donor = &mut e.accounts[gift.donor as usize];
        // Protect the actual wallet floor against accumulated f64 rounding.
        gift.amount = gift
            .amount
            .min((donor.cash - floors[gift.donor as usize]).max(0.));
        donor.cash -= gift.amount;
        donor.family_sent += gift.amount;
        let recipient = &mut e.accounts[gift.recipient as usize];
        recipient.cash += gift.amount;
        recipient.family_received += gift.amount;
    }
    gifts.retain(|g| g.amount > 0.);
    e.family_gifts = gifts;
    e.family_support_month = Some(month);
}
pub(super) fn validate(e: &HouseholdEconomy, h: &History) -> Result<()> {
    ensure!(
        e.family_support
            .is_none_or(|p| [p.surplus_share, p.food_target]
                .iter()
                .all(|v| v.is_finite() && (0. ..=1.).contains(v))),
        "invalid family support policy"
    );
    ensure!(
        e.family_support_month
            .is_none_or(|m| m >= e.started && m <= h.month),
        "invalid family support clock"
    );
    let mut seen = BTreeSet::new();
    for gift in &e.family_gifts {
        ensure!(
            e.family_support_month.is_some()
                && gift.donor != gift.recipient
                && (gift.donor as usize) < e.accounts.len()
                && (gift.recipient as usize) < e.accounts.len()
                && (gift.site as usize) < h.sites.len()
                && gift.amount.is_finite()
                && gift.amount > 0.
                && seen.insert((gift.donor, gift.recipient)),
            "invalid family gift receipt"
        );
    }
    let sent = e.accounts.iter().map(|a| a.family_sent).sum::<f64>();
    let received = e.accounts.iter().map(|a| a.family_received).sum::<f64>();
    ensure!(
        (sent - received).abs() <= 1e-8 * (1. + sent),
        "family transfers do not reconcile"
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn simultaneous_gifts_respect_donors_recipients_order_and_no_relay() {
        let budgets = vec![
            Some(Budget {
                site: 0,
                surplus: 100.,
                deficit: 0.,
            }),
            Some(Budget {
                site: 0,
                surplus: 100.,
                deficit: 0.,
            }),
            Some(Budget {
                site: 0,
                surplus: 0.,
                deficit: 10.,
            }),
            Some(Budget {
                site: 0,
                surplus: 0.,
                deficit: 20.,
            }),
        ];
        let mut links = vec![
            Link {
                donor: 0,
                recipient: 2,
                strength: 1.,
            },
            Link {
                donor: 1,
                recipient: 2,
                strength: 1.,
            },
            Link {
                donor: 2,
                recipient: 3,
                strength: 1.,
            },
        ];
        let gifts = allocate(&budgets, &links, 0.5);
        assert_eq!(gifts.len(), 2);
        assert_eq!(gifts.iter().map(|g| g.amount).sum::<f64>(), 10.);
        assert!(gifts.iter().all(|g| g.amount == 5. && g.recipient == 2));
        links.reverse();
        assert_eq!(
            serde_json::to_value(&gifts).unwrap(),
            serde_json::to_value(allocate(&budgets, &links, 0.5)).unwrap()
        );
        let mut scarce = budgets.clone();
        scarce[0].as_mut().unwrap().surplus = 1.;
        scarce[1].as_mut().unwrap().site = 1;
        let gifts = allocate(&scarce, &links, 0.5);
        assert_eq!(gifts.len(), 1);
        assert_eq!(gifts[0].amount, 0.5);
        assert!(allocate(&budgets, &links, 0.).is_empty());
    }
    #[test]
    fn gifts_protect_full_food_budget_and_do_not_repeat() {
        let mut e = HouseholdEconomy::new(0);
        e.family_support = Some(FamilySupportPolicy {
            surplus_share: 1.,
            food_target: 0.9,
        });
        e.accounts = vec![Default::default(); 2];
        for a in &mut e.accounts {
            a.need = 100.;
        }
        e.accounts[0].cash = 120.;
        e.accounts[0].wages = 120.;
        let plans = vec![RetailPlan {
            site: 0,
            ids: vec![0, 1],
            demand: vec![],
            needs: vec![100., 100.],
            need: 200.,
            free: 100.,
            price: 2.,
        }];
        let links = vec![Link {
            donor: 0,
            recipient: 1,
            strength: 1.,
        }];
        settle(&mut e, &plans, &links, 1);
        assert_eq!(e.accounts[0].cash, 100.);
        assert_eq!(e.accounts[1].cash, 20.);
        assert_eq!(e.accounts[0].family_sent, e.accounts[1].family_received);
        let saved = serde_json::to_vec(&e).unwrap();
        settle(&mut e, &plans, &links, 1);
        assert_eq!(serde_json::to_vec(&e).unwrap(), saved);
        let mut resumed: HouseholdEconomy = serde_json::from_slice(&saved).unwrap();
        settle(&mut e, &plans, &links, 2);
        settle(&mut resumed, &plans, &links, 2);
        assert_eq!(
            serde_json::to_vec(&e).unwrap(),
            serde_json::to_vec(&resumed).unwrap()
        );
        assert!(e.family_gifts.is_empty());
        let mut old = serde_json::to_value(&e).unwrap();
        old.as_object_mut().unwrap().remove("family_support");
        let old: HouseholdEconomy = serde_json::from_value(old).unwrap();
        assert!(old.family_support.is_none());
    }
    #[test]
    #[ignore = "requires hardware GPU"]
    fn local_kin_gifts_raise_access_without_creating_cash_or_food() {
        use crate::{
            catalog::Catalog,
            config::Config,
            gpu::{ContextGpu, Generator},
        };
        let mut g = Generator::new(
            pollster::block_on(ContextGpu::headless()).unwrap(),
            Config {
                resolution: 32,
                ecology_resolution: 16,
                ecology_years_per_epoch: 1,
                seed: 17,
                ..Default::default()
            },
            Catalog::bundled().unwrap(),
        )
        .unwrap();
        g.found_civilizations(5).unwrap();
        g.enable_society().unwrap();
        g.enable_politics().unwrap();
        g.civilizations
            .as_mut()
            .unwrap()
            .set_individual_participation(true)
            .unwrap();
        // Legacy named births require 36 months after the first annual marriages.
        g.advance_history(60).unwrap();
        let mut base = g.civilizations.as_ref().unwrap().clone();
        let ids = base
            .society
            .as_ref()
            .unwrap()
            .households
            .iter()
            .filter(|hh| hh.site == 0)
            .map(|hh| hh.id as usize)
            .collect::<Vec<_>>();
        // Use an actual recorded birth and its resident parent in another ownership household.
        let (donor, recipient, parent) = base
            .politics
            .as_ref()
            .unwrap()
            .kin
            .iter()
            .find_map(|k| {
                if !ids.contains(&(k.household as usize)) {
                    return None;
                }
                let (Some(child_house), Presence::Resident(0)) = base.person_presence(k.person)
                else {
                    return None;
                };
                k.parents.iter().flatten().find_map(|&parent| {
                    let (Some(hh), Presence::Resident(0)) = base.person_presence(parent) else {
                        return None;
                    };
                    (hh != child_house).then_some((hh as usize, child_house as usize, parent))
                })
            })
            .expect("fixture needs a real cross-household parent/child link");
        let recipients: Vec<_> = base
            .participation
            .as_ref()
            .unwrap()
            .residents
            .values()
            .filter(|r| r.household == Some(recipient as u32))
            .map(|r| r.person)
            .collect();
        for a in &mut base.culture.as_mut().unwrap().agents {
            a.traits[1] = if a.person == parent { 1. } else { 0. };
            a.relations.clear();
        }
        let funds = super::super::withdraw(&mut base.sites[0].economy.finance[0], 2000.);
        let e = base
            .society
            .as_mut()
            .unwrap()
            .household_economy
            .as_mut()
            .unwrap();
        e.family_support = Some(FamilySupportPolicy::default());
        e.founding_access = None;
        e.common_share = 0.5;
        e.payroll_share = 0.;
        e.dividend_share = 0.;
        e.relief_share = 0.;
        e.accounts[donor].cash += funds;
        e.accounts[donor].wages += funds;
        // Return other fixture wallets to town cash using the real account ledger.
        for &id in ids.iter().filter(|&&id| id != donor) {
            let a = &mut e.accounts[id];
            let returned = super::super::deposit(&mut base.sites[0].economy.finance[0], a.cash);
            a.cash -= returned;
            a.estate_returned += returned;
        }
        assert_eq!(links(&base).len(), 1);
        let mut control = base.clone();
        control
            .society
            .as_mut()
            .unwrap()
            .household_economy
            .as_mut()
            .unwrap()
            .family_support = None;
        let mut hostile = base.clone();
        for &person in &recipients {
            hostile.culture.as_mut().unwrap().agents[parent as usize]
                .relations
                .insert(person, -1.);
        }
        assert!(links(&hostile).is_empty());
        let mut absent = base.clone();
        absent
            .participation
            .as_mut()
            .unwrap()
            .residents
            .get_mut(&parent)
            .unwrap()
            .presence = Presence::Unknown;
        assert!(links(&absent).is_empty());
        let mut distant = base.clone();
        for person in &recipients {
            distant
                .participation
                .as_mut()
                .unwrap()
                .residents
                .get_mut(person)
                .unwrap()
                .presence = Presence::Resident(1);
        }
        assert!(links(&distant).is_empty());
        let cash = base.economy_residuals()[4];
        let goods = base.sites[0].economy.goods;
        control.prepare_household_retail();
        hostile.prepare_household_retail();
        base.prepare_household_retail();
        assert!(
            base.sites[0].demography.household_food[0]
                > control.sites[0].demography.household_food[0]
        );
        assert_eq!(
            hostile.sites[0].demography.household_food[0],
            control.sites[0].demography.household_food[0]
        );
        let e = base
            .society
            .as_ref()
            .unwrap()
            .household_economy
            .as_ref()
            .unwrap();
        assert_eq!(e.family_gifts.len(), 1);
        assert!(e.accounts[recipient].family_received > 0.);
        assert!(
            e.accounts[donor].cash
                >= e.accounts[donor].need
                    * 0.5
                    * f64::from(base.sites[0].economy.prices[crate::economy::FOOD].max(0.01))
                    - 1e-8
        );
        assert_eq!(base.sites[0].economy.goods, goods);
        assert!((base.economy_residuals()[4] - cash).abs() < 1e-7);
        e.validate(&base).unwrap();
        // Continue the full monthly pipeline from the funded fixture, then save at Close.
        g.civilizations = Some(base);
        g.advance_history(2).unwrap();
        let file =
            std::env::temp_dir().join(format!("family-support-{}.world", std::process::id()));
        g.save(&file).unwrap();
        let mut resumed = Generator::load(g.gpu.clone(), &file).unwrap();
        std::fs::remove_file(file).unwrap();
        g.advance_history(3).unwrap();
        for _ in 0..3 {
            resumed.advance_history(1).unwrap();
        }
        assert_eq!(
            serde_json::to_vec(&g.civilizations).unwrap(),
            serde_json::to_vec(&resumed.civilizations).unwrap()
        );
    }
}
