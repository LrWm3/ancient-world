//! Political authority is resolved after estate succession, never conveyed by property.
use crate::{civilization::History, participation::Presence, politics::Politics};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

const ELIGIBLE_AGE_MONTHS: i64 = 216;
const ABSENCE_GRACE_MONTHS: u32 = 6;
const INTERNAL_CHALLENGE_MARGIN: f32 = 0.25;
const PERSONAL_HERITAGE_WEIGHT: f32 = 0.15;
const FACTION_HERITAGE_WEIGHT: f32 = 0.20;
const LOYALTY_WEIGHT: f32 = 0.5;
const COUNCIL_QUALITY_WEIGHT: f32 = 0.25;
const SAME_FACTION_BALLOT_WEIGHT: f32 = 1.0;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum Rule {
    /// Recorded adult children first; no invented relatives or transfer of an estate.
    #[default]
    Hereditary,
    GoverningFaction,
    CouncilElection,
}
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct Mandate {
    pub last_scores: Vec<(u32, f32)>,
    pub selection_month: Option<u32>,
    pub rule: Rule,
    pub absent_since: Option<u32>,
    pub vacant: bool,
    pub last_review: Option<u32>,
    pub last_annual_review: Option<u32>,
}
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Leadership {
    pub mandates: BTreeMap<u32, Mandate>,
}
impl Leadership {
    pub(crate) fn new(count: usize) -> Self {
        Self {
            mandates: (0..count as u32)
                .map(|c| {
                    (
                        c,
                        Mandate {
                            rule: Rule::GoverningFaction,
                            ..Default::default()
                        },
                    )
                })
                .collect(),
        }
    }
    pub fn rule(&self, civilization: u32) -> Rule {
        self.mandates
            .get(&civilization)
            .map_or(Rule::Hereditary, |m| m.rule)
    }
    pub(crate) fn validate(&self, h: &History) -> anyhow::Result<()> {
        for (&c, m) in &self.mandates {
            anyhow::ensure!(
                (c as usize) < h.civilizations.len()
                    && m.last_scores
                        .iter()
                        .all(|(id, score)| (*id as usize) < h.people.len()
                            && score.is_finite()
                            && *score >= 0.)
                    && [
                        m.absent_since,
                        m.last_review,
                        m.last_annual_review,
                        m.selection_month
                    ]
                    .into_iter()
                    .flatten()
                    .all(|t| t <= h.month),
                "invalid leadership boundary"
            );
        }
        Ok(())
    }
}

/// Recognition is local and saturating; current eligible member households supply affiliation.
/// One expedition record is counted once even if several members returned together.
pub(crate) fn faction_heritage(h: &History, p: &Politics, observer: u32, faction: u32) -> f32 {
    let (Some(c), Some(s)) = (&h.culture, &h.society) else {
        return 0.;
    };
    FACTION_HERITAGE_WEIGHT
        * crate::heritage_renown::score(c, observer, h.month, |r| {
            r.people.iter().any(|&person| {
                h.people
                    .get(person as usize)
                    .is_some_and(|v| v.died.is_none())
                    && s.households.iter().any(|f| {
                        p.household_factions.get(f.id as usize) == Some(&faction)
                            && h.political_household_eligible(f)
                            && (f.head == person
                                || p.kin
                                    .iter()
                                    .any(|k| k.person == person && k.household == f.id))
                    })
            })
        })
}
impl History {
    pub fn leadership_report(&self) -> serde_json::Value {
        serde_json::json!({"month":self.month,"leadership":self.politics.as_ref().map(|p| &p.leadership),"rules":"New political baselines use governing-faction selection; council ballots currently give each eligible household one vote. Traits and witnessed heritage influence candidates; personal service-accountability is not yet scored."})
    }
    fn leadership_site(&self, person: u32, civilization: u32) -> Option<u32> {
        let p = self.people.get(person as usize)?;
        if p.died.is_some()
            || p.civilization != civilization
            || i64::from(self.month) - i64::from(p.born) < ELIGIBLE_AGE_MONTHS
        {
            return None;
        }
        match self.person_presence(person).1 {
            Presence::Resident(site)
                if !self.sites[site as usize].abandoned
                    && self.controller(site) == civilization =>
            {
                Some(site)
            }
            _ => None,
        }
    }
    fn leadership_score(&self, person: u32, site: u32) -> f32 {
        self.culture.as_ref().map_or(0., |c| {
            c.agents.get(person as usize).map_or(0., |a| {
                a.traits[0] + LOYALTY_WEIGHT * a.traits[4] + a.skills[0]
            }) + PERSONAL_HERITAGE_WEIGHT
                * crate::heritage_renown::score(c, site, self.month, |r| r.people.contains(&person))
        })
    }
    pub(crate) fn review_leadership(&mut self, annual: bool) {
        let Some(p) = self.politics.as_mut() else {
            return;
        };
        let mut leadership = std::mem::take(&mut p.leadership);
        for civ in 0..self.civilizations.len() as u32 {
            let old = self.civilizations[civ as usize].leader;
            let present = self.leadership_site(old, civ);
            let dead = self.people[old as usize].died.is_some();
            let p = self.politics.as_ref().unwrap();
            let m = leadership.mandates.entry(civ).or_default();
            if annual && m.last_annual_review == Some(self.month)
                || !annual && m.last_review == Some(self.month)
            {
                continue;
            }
            m.last_review = Some(self.month);
            if annual {
                m.last_annual_review = Some(self.month);
            }
            if present.is_some() {
                m.absent_since = None;
                m.vacant = false;
            } else {
                m.absent_since.get_or_insert(self.month);
            }
            let absent = m
                .absent_since
                .is_some_and(|t| self.month.saturating_sub(t) >= ABSENCE_GRACE_MONTHS);
            if !dead && !absent && (present.is_none() || !annual || m.rule == Rule::Hereditary) {
                continue;
            }
            let rule = m.rule;
            let governing = p.governing[civ as usize];
            let candidates: Vec<_> = self
                .society
                .as_ref()
                .unwrap()
                .households
                .iter()
                .filter(|f| {
                    self.political_household_eligible(f)
                        && (rule == Rule::CouncilElection
                            || p.household_factions[f.id as usize] == governing)
                })
                .filter_map(|f| self.leadership_site(f.head, civ).map(|site| (f.head, site)))
                .collect();
            let dynastic = if rule == Rule::Hereditary && dead {
                p.kin
                    .iter()
                    .filter(|k| k.parents.contains(&Some(old)))
                    .filter_map(|k| self.leadership_site(k.person, civ).map(|s| (k.person, s)))
                    .min_by_key(|&(id, _)| (self.people[id as usize].born, id))
            } else {
                None
            };
            let affiliation = |person| {
                self.society
                    .as_ref()
                    .unwrap()
                    .households
                    .iter()
                    .find(|f| f.head == person)
                    .map(|f| p.household_factions[f.id as usize])
            };
            let mut ballots = BTreeMap::<u32, f32>::new();
            if rule == Rule::CouncilElection {
                for voter in self
                    .society
                    .as_ref()
                    .unwrap()
                    .households
                    .iter()
                    .filter(|f| {
                        self.political_household_eligible(f)
                            && self.leadership_site(f.head, civ).is_some()
                    })
                {
                    let preference = |id, site| {
                        COUNCIL_QUALITY_WEIGHT * self.leadership_score(id, site)
                            + if affiliation(id) == Some(p.household_factions[voter.id as usize]) {
                                SAME_FACTION_BALLOT_WEIGHT
                            } else {
                                0.
                            }
                    };
                    if let Some(&(id, _)) = candidates.iter().max_by(|&&(a, sa), &&(b, sb)| {
                        preference(a, sa)
                            .total_cmp(&preference(b, sb))
                            .then_with(|| b.cmp(&a))
                    }) {
                        *ballots.entry(id).or_default() += 1.;
                    }
                }
            }
            let score = |id, site| {
                if rule == Rule::CouncilElection {
                    ballots.get(&id).copied().unwrap_or(0.)
                } else {
                    self.leadership_score(id, site)
                }
            };
            let changed_faction =
                rule == Rule::GoverningFaction && affiliation(old) != Some(governing);
            m.selection_month = Some(self.month);
            m.last_scores = candidates
                .iter()
                .map(|&(id, site)| (id, score(id, site)))
                .collect();
            let chosen = dynastic.or_else(|| {
                candidates.iter().copied().max_by(|&(a, sa), &(b, sb)| {
                    score(a, sa)
                        .total_cmp(&score(b, sb))
                        .then_with(|| b.cmp(&a))
                })
            });
            if let Some((next, site)) = chosen {
                if next == old {
                    continue;
                }
                if !dead
                    && !absent
                    && !changed_faction
                    && score(next, site) <= score(old, present.unwrap()) + INTERNAL_CHALLENGE_MARGIN
                {
                    continue;
                }
                self.civilizations[civ as usize].leader = next;
                m.vacant = false;
                m.absent_since = None;
                let reason = if dead {
                    "death"
                } else if absent {
                    "prolonged absence"
                } else {
                    "internal selection"
                };
                self.event("political_succession",Some(site),None,format!("{:?}: {} replaced {} after {}; governing faction {} retained; estates unchanged{}",rule,self.people[next as usize].name,self.people[old as usize].name,reason,governing,if rule == Rule::Hereditary && dynastic.is_none() { if dead { "; non-dynastic caretaker: no eligible present child" } else { "; caretaker during prolonged absence" } } else { "" }));
                self.events.last_mut().unwrap().subjects.extend([
                    ("person".into(), old),
                    ("person".into(), next),
                    ("faction".into(), governing),
                ]);
                if dynastic.is_none() {
                    let causes: Vec<_> = self
                        .culture
                        .as_ref()
                        .into_iter()
                        .flat_map(|c| &c.heritage_renown)
                        .filter(|r| r.people.contains(&next) && r.weight(site, self.month) > 0.)
                        .map(|r| r.event)
                        .collect();
                    let event = self.events.last_mut().unwrap();
                    event.causes.extend(causes);
                    event.causes.sort_unstable();
                    event.causes.dedup();
                }
            } else {
                if !m.vacant {
                    self.event("political_vacancy",None,None,format!("Civilization {civ}: no eligible present successor under {rule:?}; estate ownership is unaffected"));
                }
                m.vacant = true;
            }
        }
        self.politics.as_mut().unwrap().leadership = leadership;
    }
}
impl crate::gpu::Generator {
    pub fn set_succession_rule(&mut self, civilization: u32, rule: Rule) -> anyhow::Result<()> {
        self.validate_living_boundary()?;
        anyhow::ensure!(
            self.progress.stage == crate::gpu::Stage::Boundary,
            "succession policy requires completed boundary"
        );
        let h = self
            .civilizations
            .as_mut()
            .ok_or_else(|| anyhow::anyhow!("found civilizations first"))?;
        anyhow::ensure!(
            (civilization as usize) < h.civilizations.len(),
            "invalid civilization"
        );
        let p = h
            .politics
            .as_mut()
            .ok_or_else(|| anyhow::anyhow!("enable politics first"))?;
        let m = p.leadership.mandates.entry(civilization).or_default();
        if m.rule != rule {
            m.rule = rule;
            h.event("succession_policy",None,None,format!("Civilization {civilization} adopted {rule:?}; property succession remains independent"));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore = "requires hardware GPU"]
    fn estates_and_dynastic_or_faction_authority_resolve_independently() {
        let g = crate::continuity_fixture::world();
        let mut h = g.civilizations.unwrap();
        h.month = 1;
        let civ = 0;
        let old = h.civilizations[civ].leader;
        let heads: Vec<_> = h
            .society
            .as_ref()
            .unwrap()
            .households
            .iter()
            .filter(|f| f.head != old && h.leadership_site(f.head, civ as u32).is_some())
            .map(|f| (f.id, f.head))
            .collect();
        assert!(heads.len() >= 2);
        let (child_hh, child) = heads[0];
        let (member_hh, member) = heads[1];
        let governing = h.politics.as_ref().unwrap().governing[civ];
        h.people[old as usize].born = h.month as i32 - 600;
        h.people[child as usize].born = h.month as i32 - 300;
        let p = h.politics.as_mut().unwrap();
        for f in &h.society.as_ref().unwrap().households {
            if h.people[f.head as usize].civilization == civ as u32 {
                p.household_factions[f.id as usize] = governing + 1;
            }
        }
        p.household_factions[member_hh as usize] = governing;
        let k = p.kin.iter_mut().find(|k| k.person == child).unwrap();
        assert_eq!(k.household, child_hh);
        k.parents = [Some(old), None];
        h.people[old as usize].died = Some(h.month);
        let mut dynastic = h.clone();
        dynastic
            .politics
            .as_mut()
            .unwrap()
            .leadership
            .mandates
            .get_mut(&0)
            .unwrap()
            .rule = Rule::Hereditary;
        h.social_month().unwrap();
        dynastic.social_month().unwrap();
        assert_eq!(h.civilizations[0].leader, member);
        assert_eq!(dynastic.civilizations[0].leader, child);
        assert_eq!(
            serde_json::to_value(&h.society).unwrap(),
            serde_json::to_value(&dynastic.society).unwrap(),
            "changing the constitution cannot change estate settlement"
        );
        assert_eq!(h.politics.as_ref().unwrap().governing[0], governing);
        let before = serde_json::to_value(&h).unwrap();
        h.review_leadership(false);
        assert_eq!(
            before,
            serde_json::to_value(&h).unwrap(),
            "same monthly boundary is idempotent"
        );
        let mut resumed: History =
            serde_json::from_slice(&serde_json::to_vec(&h).unwrap()).unwrap();
        h.month += 1;
        resumed.month += 1;
        h.review_leadership(false);
        resumed.review_leadership(false);
        assert_eq!(
            serde_json::to_value(&h).unwrap(),
            serde_json::to_value(&resumed).unwrap()
        );
        let mut old_archive = serde_json::to_value(&h).unwrap();
        old_archive["politics"]
            .as_object_mut()
            .unwrap()
            .remove("leadership");
        let legacy: History = serde_json::from_value(old_archive).unwrap();
        assert_eq!(
            legacy.politics.unwrap().leadership.rule(0),
            Rule::Hereditary
        );
    }

    #[test]
    #[ignore = "requires hardware GPU"]
    fn absence_has_grace_and_council_ballots_do_not_transfer_property() {
        let g = crate::continuity_fixture::world();
        let mut h = g.civilizations.unwrap();
        let old = h.civilizations[0].leader;
        let home = h
            .society
            .as_ref()
            .unwrap()
            .households
            .iter()
            .find(|f| f.head == old)
            .unwrap()
            .site;
        // An occupied home removes political availability without inventing a traveler.
        h.politics.as_mut().unwrap().controllers[home as usize] = 1;
        let start = h.month;
        h.review_leadership(true); // Annual review during grace must not unwrap missing presence.
        assert_eq!(h.civilizations[0].leader, old);
        for i in 1..6 {
            h.month = start + i;
            h.review_leadership(false);
            assert_eq!(h.civilizations[0].leader, old);
        }
        h.month = start + 6;
        h.review_leadership(false);
        assert!(
            h.civilizations[0].leader != old
                || h.politics.as_ref().unwrap().leadership.mandates[&0].vacant
        );
        // Restored presence clears a vacancy and the absence clock.
        h.politics.as_mut().unwrap().controllers[home as usize] = 0;
        h.month += 1;
        h.review_leadership(false);
        assert!(!h.politics.as_ref().unwrap().leadership.mandates[&0].vacant);
        h.politics
            .as_mut()
            .unwrap()
            .leadership
            .mandates
            .get_mut(&0)
            .unwrap()
            .rule = Rule::CouncilElection;
        let estates = serde_json::to_value(&h.society).unwrap();
        h.review_leadership(true);
        let scores = &h.politics.as_ref().unwrap().leadership.mandates[&0].last_scores;
        let votes: f32 = scores.iter().map(|(_, v)| v).sum();
        let electorate = h
            .society
            .as_ref()
            .unwrap()
            .households
            .iter()
            .filter(|f| h.political_household_eligible(f) && h.leadership_site(f.head, 0).is_some())
            .count();
        assert_eq!(votes, electorate as f32);
        assert_eq!(serde_json::to_value(&h.society).unwrap(), estates);
    }
    #[test]
    #[ignore = "requires hardware GPU"]
    fn internal_challenge_and_local_heritage_have_causal_effects() {
        let g = crate::continuity_fixture::world();
        let mut h = g.civilizations.unwrap();
        let old = h.civilizations[0].leader;
        let (hh, challenger, site) = h
            .society
            .as_ref()
            .unwrap()
            .households
            .iter()
            .filter(|f| f.head != old && h.leadership_site(f.head, 0).is_some())
            .map(|f| (f.id, f.head, f.site))
            .next()
            .unwrap();
        let governing = h.politics.as_ref().unwrap().governing[0];
        h.politics.as_mut().unwrap().household_factions[hh as usize] = governing;
        let c = h.culture.as_mut().unwrap();
        for a in &mut c.agents {
            a.traits = [0.; 6];
            a.skills = [0.; 4];
        }
        c.agents[challenger as usize].traits[0] = 1.;
        c.agents[challenger as usize].skills[0] = 1.;
        let estates = serde_json::to_value(&h.society).unwrap();
        let mut negative = h.clone();
        negative.culture.as_mut().unwrap().agents[challenger as usize].traits[0] = 0.;
        negative.culture.as_mut().unwrap().agents[challenger as usize].skills[0] = 0.;
        h.review_leadership(true);
        negative.review_leadership(true);
        assert_eq!(h.civilizations[0].leader, challenger);
        assert_eq!(negative.civilizations[0].leader, old);
        assert_eq!(h.politics.as_ref().unwrap().governing[0], governing);
        assert_eq!(estates, serde_json::to_value(&h.society).unwrap());
        h.culture
            .as_mut()
            .unwrap()
            .heritage_renown
            .push(crate::heritage_renown::Recognition {
                artifact: 0,
                expedition: 0,
                event: 0,
                month: h.month,
                origin: site,
                civilization: 0,
                tradition: 0,
                institution: None,
                people: vec![challenger],
                survival: 1.,
                witnesses: vec![],
            });
        assert_eq!(
            faction_heritage(&h, h.politics.as_ref().unwrap(), site, governing),
            0.
        );
        h.culture
            .as_mut()
            .unwrap()
            .heritage_renown
            .last_mut()
            .unwrap()
            .witnesses
            .push((site, h.month));
        let known = faction_heritage(&h, h.politics.as_ref().unwrap(), site, governing);
        assert!(known > 0. && known <= 0.2);
        assert_eq!(
            faction_heritage(&h, h.politics.as_ref().unwrap(), site, governing + 1),
            0.
        );
        let remote = (site + 1) % h.sites.len() as u32;
        assert_eq!(
            faction_heritage(&h, h.politics.as_ref().unwrap(), remote, governing),
            0.
        );
        h.month += 120;
        assert!(faction_heritage(&h, h.politics.as_ref().unwrap(), site, governing) < known);
        h.people[challenger as usize].died = Some(h.month);
        assert_eq!(
            faction_heritage(&h, h.politics.as_ref().unwrap(), site, governing),
            0.
        );
    }
}
