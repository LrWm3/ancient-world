//! Local economic succession, separate from household-head and political offices.
use crate::{civilization::History, credit::Account, participation::Presence};
use anyhow::{ensure, Result};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

const ESTATE_WAIT_MONTHS: u32 = 12;
const ESTATE_FOOD_RESERVE_MONTHS: f64 = 3.;

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Inheritance {
    pub enabled: bool,
    pub processed_month: Option<u32>,
    pub receipts: Vec<Receipt>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Receipt {
    pub month: u32,
    pub site: u32,
    pub estate: u32,
    pub beneficiary: u32,
    pub heir: u32,
    pub cash: f64,
    /// Retained for any age-band dependents still represented by this wallet.
    #[serde(default)]
    pub protected_food_cash: f64,
    pub share: f64,
}

/// Stop at living descendants; a living child's children do not compete with it.
/// The local policy divides equally among the resulting recorded descendants.
fn heirs(root: u32, children: &BTreeMap<u32, Vec<u32>>, living: &BTreeSet<u32>) -> BTreeSet<u32> {
    let mut seen = BTreeSet::from([root]);
    let mut queue = children.get(&root).cloned().unwrap_or_default();
    let mut result = BTreeSet::new();
    while let Some(person) = queue.pop() {
        if !seen.insert(person) {
            continue;
        }
        if living.contains(&person) {
            result.insert(person);
        } else {
            queue.extend(children.get(&person).into_iter().flatten().copied());
        }
    }
    result
}

pub(super) fn validate(economy: &super::HouseholdEconomy, h: &History) -> Result<()> {
    ensure!(
        economy
            .inheritance
            .processed_month
            .is_none_or(|m| m <= h.month),
        "invalid inheritance clock"
    );
    let mut paid = vec![0.; economy.accounts.len()];
    let mut received = paid.clone();
    let mut seen = BTreeSet::new();
    let mut case_shares = BTreeMap::<(u32, u32), f64>::new();
    for r in &economy.inheritance.receipts {
        ensure!(
            r.month <= economy.inheritance.processed_month.unwrap_or(0)
                && (r.site as usize) < h.sites.len()
                && (r.heir as usize) < h.people.len()
                && (r.estate as usize) < paid.len()
                && (r.beneficiary as usize) < paid.len()
                && r.estate != r.beneficiary
                && r.cash.is_finite()
                && r.cash >= 0.
                && r.protected_food_cash.is_finite()
                && r.protected_food_cash >= 0.
                && r.share.is_finite()
                && (0. ..=1. + super::BALANCE_TOLERANCE).contains(&r.share)
                && seen.insert((r.month, r.estate, r.heir)),
            "invalid inheritance receipt"
        );
        let shares = case_shares.entry((r.month, r.estate)).or_default();
        *shares += r.share;
        ensure!(
            *shares <= 1. + super::BALANCE_TOLERANCE,
            "inheritance overallocates ownership"
        );
        paid[r.estate as usize] += r.cash;
        received[r.beneficiary as usize] += r.cash;
    }
    for (id, a) in economy.accounts.iter().enumerate() {
        ensure!(
            (a.inheritance_paid - paid[id]).abs() <= super::BALANCE_TOLERANCE * (1. + paid[id])
                && (a.inheritance_received - received[id]).abs()
                    <= super::BALANCE_TOLERANCE * (1. + received[id]),
            "inheritance receipts do not reconcile"
        );
    }
    Ok(())
}

impl History {
    /// Before retail, once per month. Gather claims before committing transfers.
    /// All recorded heirs must be local and eligible; no remote wealth teleportation.
    pub(crate) fn inherit_household_estates(&mut self) {
        let Some(society) = &self.society else {
            return;
        };
        let Some(economy) = &society.household_economy else {
            return;
        };
        if !economy.inheritance.enabled || economy.inheritance.processed_month == Some(self.month) {
            return;
        }
        let Some(politics) = &self.politics else {
            return;
        };
        let living: BTreeSet<_> = self
            .people
            .iter()
            .filter(|p| p.died.is_none())
            .map(|p| p.id)
            .collect();
        let presences = self.person_presences();
        let mut children: BTreeMap<u32, Vec<u32>> = BTreeMap::new();
        for kin in &politics.kin {
            for parent in kin.parents.iter().flatten() {
                children.entry(*parent).or_default().push(kin.person);
            }
        }
        let occupied: BTreeSet<_> = self
            .people
            .iter()
            .filter(|p| living.contains(&p.id))
            .filter_map(|p| presences.get(p.id as usize).and_then(|(hh, _)| *hh))
            .collect();
        let mut plans = vec![];
        for estate in &society.households {
            if estate
                .vacant_since
                .is_none_or(|m| self.month.saturating_sub(m) < ESTATE_WAIT_MONTHS)
                || self.people[estate.head as usize].died.is_none()
                || occupied.contains(&estate.id)
                || society.relocation.away(estate.id)
                || self.sites[estate.site as usize].abandoned
                || self.credit.account_has_debt(Account::Household(estate.id))
            {
                continue;
            }
            let candidates = heirs(estate.head, &children, &living);
            if candidates.is_empty() {
                continue;
            }
            let beneficiaries: Option<Vec<_>> = candidates
                .iter()
                .map(|&heir| {
                    let (Some(beneficiary), Presence::Resident(site)) =
                        presences.get(heir as usize)?
                    else {
                        return None;
                    };
                    let target = society.households.get(*beneficiary as usize)?;
                    if *beneficiary == estate.id
                        || *site != estate.site
                        || target.site != estate.site
                        || target.vacant_since.is_some()
                        || society.relocation.away(*beneficiary)
                        || economy.accounts.get(*beneficiary as usize).is_none()
                    {
                        return None;
                    }
                    Some((heir, *beneficiary))
                })
                .collect();
            let Some(beneficiaries) = beneficiaries else {
                continue;
            };
            let Some(source) = economy.accounts.get(estate.id as usize) else {
                continue;
            };
            // Sparse identities do not prove that anonymous dependents vanished.
            // Protect three months of the last completed food need at today's
            // local quote before transferring cash. Ownership succession alone
            // must not sweep subsistence relief out of a retained account.
            let reserve = source.need
                * ESTATE_FOOD_RESERVE_MONTHS
                * f64::from(
                    self.sites[estate.site as usize].economy.prices[crate::economy::FOOD]
                        .max(super::MIN_FOOD_PRICE),
                );
            let cash = (source.cash - reserve).max(0.);
            if cash <= 0. && estate.share <= 0. {
                continue;
            }
            let count = beneficiaries.len() as f64;
            let mut remaining_cash = cash;
            let mut remaining_share = estate.share;
            for (index, (heir, beneficiary)) in beneficiaries.into_iter().enumerate() {
                // Stable person order, with the last heir receiving the rounding
                // remainder. Multiple heirs can belong to one receiving wallet.
                let last = index + 1 == count as usize;
                let part_cash = if last { remaining_cash } else { cash / count };
                let part_share = if last {
                    remaining_share
                } else {
                    estate.share / count
                };
                remaining_cash -= part_cash;
                remaining_share -= part_share;
                plans.push(Receipt {
                    month: self.month,
                    site: estate.site,
                    estate: estate.id,
                    beneficiary,
                    heir,
                    cash: part_cash,
                    protected_food_cash: reserve.min(source.cash),
                    share: part_share,
                });
            }
        }
        let society = self.society.as_mut().unwrap();
        let economy = society.household_economy.as_mut().unwrap();
        economy.inheritance.processed_month = Some(self.month);
        for receipt in &plans {
            let source = &mut economy.accounts[receipt.estate as usize];
            source.cash -= receipt.cash;
            source.inheritance_paid += receipt.cash;
            let target = &mut economy.accounts[receipt.beneficiary as usize];
            target.cash += receipt.cash;
            target.inheritance_received += receipt.cash;
            society.households[receipt.estate as usize].share -= receipt.share;
            society.households[receipt.beneficiary as usize].share += receipt.share;
        }
        economy.inheritance.receipts.extend(plans.iter().cloned());
        for receipt in plans {
            self.event("household_estate_inherited", Some(self.society.as_ref().unwrap().households[receipt.estate as usize].site), None,
                format!("Household {} inherited {:.2} cash and {:.2}% ownership from vacant household {} through recorded descendant {}; household identities retained",
                    receipt.beneficiary, receipt.cash, receipt.share * 100., receipt.estate, receipt.heir));
            self.events.last_mut().unwrap().subjects.extend([
                ("household".into(), receipt.estate),
                ("household".into(), receipt.beneficiary),
                ("person".into(), receipt.heir),
            ]);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn nearest_living_branches_and_cycles() {
        let children = BTreeMap::from([(0, vec![1, 2]), (1, vec![3]), (2, vec![4]), (4, vec![0])]);
        assert_eq!(
            heirs(0, &children, &BTreeSet::from([1, 3, 4])),
            BTreeSet::from([1, 4])
        );
        assert_eq!(
            heirs(0, &children, &BTreeSet::from([3])),
            BTreeSet::from([3])
        );
        assert!(heirs(0, &children, &BTreeSet::new()).is_empty());
    }
    #[test]
    #[ignore = "requires hardware GPU"]
    fn local_estate_inheritance_conserves_cash_shares_and_protects_members() {
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
                ..Default::default()
            },
            Catalog::bundled().unwrap(),
        )
        .unwrap();
        g.found_civilizations(5).unwrap();
        g.enable_society().unwrap();
        g.enable_politics().unwrap();
        let h = g.civilizations.as_mut().unwrap();
        h.prepare_household_retail();
        h.month = 24;
        let ids: Vec<_> = h
            .society
            .as_ref()
            .unwrap()
            .households
            .iter()
            .filter(|f| f.site == 0)
            .map(|f| f.id)
            .collect();
        let (estate, target) = (ids[0], ids[1]);
        let root = h.society.as_ref().unwrap().households[estate as usize].head;
        let heir = h.society.as_ref().unwrap().households[target as usize].head;
        let presence = h.person_presences();
        for p in &mut h.people {
            if presence[p.id as usize].0 == Some(estate) {
                p.died = Some(12);
            }
        }
        let society = h.society.as_mut().unwrap();
        society.households[estate as usize].vacant_since = Some(12);
        let e = society.household_economy.as_mut().unwrap();
        e.inheritance.enabled = true;
        let funding = super::super::withdraw(&mut h.sites[0].economy.finance[0], 100.);
        e.accounts[estate as usize].cash += funding;
        e.accounts[estate as usize].dividends += funding;
        e.accounts[estate as usize].need = 1.;
        e.accounts[estate as usize].common_food = 0.;
        e.accounts[estate as usize].purchased_food = 0.;
        h.sites[0].economy.prices[crate::economy::FOOD] = 1.;
        for kin in &mut h.politics.as_mut().unwrap().kin {
            kin.parents = [None; 2];
        }
        h.politics
            .as_mut()
            .unwrap()
            .kin
            .iter_mut()
            .find(|k| k.person == heir)
            .unwrap()
            .parents = [Some(root), None];
        let mut protected = h.clone();
        protected.people[root as usize].died = None;
        protected.inherit_household_estates();
        assert!(protected
            .society
            .as_ref()
            .unwrap()
            .household_economy
            .as_ref()
            .unwrap()
            .inheritance
            .receipts
            .is_empty());
        let before = h.money_residual();
        let before_target = h.household_account(target).unwrap().cash;
        let source_cash = h.household_account(estate).unwrap().cash;
        let reserve = h.household_account(estate).unwrap().need
            * ESTATE_FOOD_RESERVE_MONTHS
            * f64::from(
                h.sites[0].economy.prices[crate::economy::FOOD].max(super::super::MIN_FOOD_PRICE),
            );
        let transferred = (source_cash - reserve).max(0.);
        assert!(transferred > 0. && reserve > 0.);
        let shares = h.society.as_ref().unwrap().households[estate as usize].share
            + h.society.as_ref().unwrap().households[target as usize].share;
        // Two local branches divide the same estate rather than freezing it.
        let mut divided = h.clone();
        let other_target = ids[2];
        let other_heir = divided.society.as_ref().unwrap().households[other_target as usize].head;
        divided
            .politics
            .as_mut()
            .unwrap()
            .kin
            .iter_mut()
            .find(|k| k.person == other_heir)
            .unwrap()
            .parents = [Some(root), None];
        let other_cash = divided.household_account(other_target).unwrap().cash;
        let mut remote = divided.clone();
        remote.society.as_mut().unwrap().households[other_target as usize].site = 1;
        remote.inherit_household_estates();
        assert_eq!(remote.household_account(estate).unwrap().cash, source_cash);
        assert!(remote
            .society
            .as_ref()
            .unwrap()
            .household_economy
            .as_ref()
            .unwrap()
            .inheritance
            .receipts
            .is_empty());
        divided.inherit_household_estates();
        assert!(
            (divided.household_account(target).unwrap().cash - before_target - transferred / 2.)
                .abs()
                < 1e-8
        );
        assert!(
            (divided.household_account(other_target).unwrap().cash - other_cash - transferred / 2.)
                .abs()
                < 1e-8
        );
        assert!((divided.money_residual() - before).abs() < 1e-6);
        let economy = divided
            .society
            .as_ref()
            .unwrap()
            .household_economy
            .as_ref()
            .unwrap();
        assert_eq!(economy.inheritance.receipts.len(), 2);
        economy.validate(&divided).unwrap();
        let boundary = serde_json::to_value(&divided).unwrap();
        divided.inherit_household_estates();
        assert_eq!(boundary, serde_json::to_value(&divided).unwrap());
        let mut resumed: History = serde_json::from_value(boundary).unwrap();
        for state in [&mut divided, &mut resumed] {
            state.month += 1;
            state.inherit_household_estates();
        }
        assert_eq!(
            serde_json::to_value(&divided).unwrap(),
            serde_json::to_value(&resumed).unwrap()
        );
        h.inherit_household_estates();
        assert_eq!(
            h.household_account(estate).unwrap().cash,
            source_cash - transferred
        );
        assert_eq!(
            h.household_account(target).unwrap().cash,
            before_target + transferred
        );
        assert_eq!(
            h.society.as_ref().unwrap().households[target as usize].share,
            shares
        );
        assert_eq!(
            h.society.as_ref().unwrap().households[estate as usize].share,
            0.
        );
        assert!((h.money_residual() - before).abs() < 1e-6);
        h.society
            .as_ref()
            .unwrap()
            .household_economy
            .as_ref()
            .unwrap()
            .validate(h)
            .unwrap();
        let boundary = serde_json::to_value(&*h).unwrap();
        h.inherit_household_estates();
        assert_eq!(boundary, serde_json::to_value(&*h).unwrap());
        let mut resumed: History = serde_json::from_value(boundary).unwrap();
        resumed.month += 1;
        h.month += 1;
        resumed.inherit_household_estates();
        h.inherit_household_estates();
        assert_eq!(
            serde_json::to_value(&*h).unwrap(),
            serde_json::to_value(&resumed).unwrap()
        );
        resumed
            .society
            .as_mut()
            .unwrap()
            .household_economy
            .as_mut()
            .unwrap()
            .inheritance
            .receipts[0]
            .cash += 1.;
        assert!(resumed
            .society
            .as_ref()
            .unwrap()
            .household_economy
            .as_ref()
            .unwrap()
            .validate(&resumed)
            .is_err());
    }
}
