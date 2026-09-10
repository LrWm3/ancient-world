//! Small, naming-only lexical conventions, not a population language model.
use super::{hash, key_hash, Language, Source};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Lexeme {
    pub form: String,
    pub adopted: u32,
    pub source: Option<Source>,
    pub borrowed_from: Option<u32>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WordUse {
    pub concept: String,
    pub word: Lexeme,
}
impl Language {
    /// Already pronounced forms: never sound-shift an eponym or loan a second time.
    pub fn lexical_choice(&self, concept: &str, key: &str, slot: usize) -> Lexeme {
        let Some(options) = self.lexicon.get(concept).filter(|v| !v.is_empty()) else {
            return Lexeme {
                form: self.word(concept),
                adopted: 0,
                source: None,
                borrowed_from: None,
            };
        };
        let total = (1u32 << options.len()) - 1;
        let mut choice = hash(self.seed ^ key_hash(key) ^ (slot as u32).wrapping_mul(7919)) % total;
        for (i, word) in options.iter().enumerate() {
            let weight = 1 << i;
            if choice < weight {
                return word.clone();
            }
            choice -= weight;
        }
        unreachable!()
    }
    /// Keep the original until three newer distinct options have displaced it.
    /// Repeated exposure to an identical form does not refresh its age or multiply it.
    pub fn adopt_word(&mut self, concept: &str, word: Lexeme) -> bool {
        if concept == "of"
            || !self.roots.contains_key(concept)
            || word.form.chars().count() < 3
            || word.form.chars().count() > 24
            || !word.form.chars().all(|c| c.is_alphabetic())
        {
            return false;
        }
        let original = self.word(concept);
        let options = self.lexicon.entry(concept.into()).or_insert_with(|| {
            vec![Lexeme {
                form: original,
                adopted: 0,
                source: None,
                borrowed_from: None,
            }]
        });
        if options.last().is_some_and(|v| v.adopted > word.adopted)
            || options.iter().any(|v| v.form == word.form)
        {
            return false;
        }
        options.push(word);
        if options.len() > 3 {
            options.remove(0);
        }
        true
    }
}

impl crate::civilization::History {
    pub(crate) fn evolve_lexicons(&mut self) {
        if self.month == 0 || self.month % 12 != 0 {
            return;
        }
        // Completed snapshot: a borrowed word cannot hop through multiple languages this year.
        let snapshot: BTreeMap<_, _> = self
            .civilizations
            .iter()
            .filter_map(|c| c.language.as_ref().map(|l| (c.id, l.lexicon.clone())))
            .collect();
        let mut contacts = BTreeSet::new();
        if let Some(society) = &self.society {
            for route in society.routes.iter().filter(|r| r.passable()) {
                let Some(a) = self.sites.get(route.from as usize) else {
                    continue;
                };
                let Some(b) = self.sites.get(route.to as usize) else {
                    continue;
                };
                if !a.abandoned && !b.abandoned && a.civilization != b.civilization {
                    contacts.insert((a.civilization, b.civilization));
                    contacts.insert((b.civilization, a.civilization));
                }
            }
        }
        for ci in 0..self.civilizations.len() {
            let civ = self.civilizations[ci].id;
            let Some(l) = self.civilizations[ci].language.as_mut() else {
                continue;
            };
            if l.lexicon_month >= self.month {
                continue;
            }
            l.lexicon_month = self.month;
            l.contact_years
                .retain(|other, _| contacts.contains(&(civ, *other)));
            for &(_, b) in contacts.iter().filter(|(a, _)| *a == civ) {
                let years = l.contact_years.entry(b).or_default();
                *years = years.saturating_add(1);
            }
            let q = hash(l.seed ^ self.month ^ 0x6c657869);
            // At most one adoption per civilization/year, including contact loans.
            let mut candidates: Vec<(String, Lexeme, Option<u32>)> = vec![];
            if q % 5 == 0 {
                for (&other, &years) in &l.contact_years {
                    if years < 3 {
                        continue;
                    }
                    if let Some(words) = snapshot.get(&other) {
                        for (concept, options) in words {
                            if let Some(word) = options.last().filter(|w| w.source.is_some()) {
                                let mut loan = word.clone();
                                loan.adopted = self.month;
                                loan.borrowed_from = Some(other);
                                candidates.push((concept.clone(), loan, None));
                            }
                        }
                    }
                }
            } else if q % 7 == 0 {
                // Production and durable institutions supply associations, not arbitrary virtues
                // inferred from a person's name. Thresholds are explicit game conventions.
                let mut add = |concept: &str, source: Source, site: Option<u32>| {
                    let form = l
                        .source_stem(&source.name, hash(q ^ source.id))
                        .chars()
                        .take(16)
                        .collect();
                    candidates.push((
                        concept.into(),
                        Lexeme {
                            form,
                            adopted: self.month,
                            source: Some(source),
                            borrowed_from: None,
                        },
                        site,
                    ));
                };
                for site in self.sites.iter().filter(|s| {
                    s.civilization == civ
                        && !s.abandoned
                        && self.month.saturating_sub(s.founded) >= 60
                }) {
                    for (good, concept) in [
                        (0, "timber"),
                        (2, "metal"),
                        (5, "clay"),
                        (8, "grain"),
                        (18, "cloth"),
                        (28, "fisher"),
                    ] {
                        if site.economy.made[good] >= 1000. {
                            add(
                                concept,
                                Source {
                                    kind: "site".into(),
                                    id: site.id,
                                    name: site.name.clone(),
                                },
                                Some(site.id),
                            );
                        }
                    }
                }
                if let Some(culture) = &self.culture {
                    for n in culture.institutions.iter().filter(|n| {
                        n.active
                            && n.expenses >= 50.
                            && self.month.saturating_sub(n.founded) >= 60
                            && self
                                .sites
                                .get(n.site as usize)
                                .is_some_and(|s| s.civilization == civ && !s.abandoned)
                    }) {
                        use crate::culture::InstitutionKind::*;
                        let concept = match n.kind {
                            Religious => "sanctuary",
                            Merchant => "market",
                            Craft => "craft",
                            Scholarly => "learning",
                        };
                        add(
                            concept,
                            Source {
                                kind: "institution".into(),
                                id: n.id,
                                name: n.name.clone(),
                            },
                            Some(n.site),
                        );
                    }
                    for p in culture
                        .patrons
                        .iter()
                        .filter(|p| p.civilization == civ && p.departed.is_some())
                    {
                        add(
                            "guide",
                            Source {
                                kind: "patron".into(),
                                id: p.id,
                                name: p.name.clone(),
                            },
                            Some(p.site),
                        );
                    }
                    for a in culture.agents.iter().filter(|a| a.actions >= 5) {
                        let Some(p) = self
                            .people
                            .get(a.person as usize)
                            .filter(|p| p.civilization == civ)
                        else {
                            continue;
                        };
                        let concept = match a.occupation.as_str() {
                            "farmer" => "grower",
                            "navigator" => "journey",
                            "craftworker" => "craft",
                            "teacher" => "learning",
                            _ => continue,
                        };
                        add(
                            concept,
                            Source {
                                kind: "person".into(),
                                id: p.id,
                                name: p.name.clone(),
                            },
                            None,
                        );
                    }
                    for a in culture.artifacts.iter().filter(|a| {
                        !a.destroyed
                            && !a.lost
                            && a.topic.is_some()
                            && a.site
                                .and_then(|id| self.sites.get(id as usize))
                                .is_some_and(|s| s.civilization == civ && !s.abandoned)
                    }) {
                        add(
                            "book",
                            Source {
                                kind: "artifact".into(),
                                id: a.id,
                                name: a.name.clone(),
                            },
                            a.site,
                        );
                    }
                }
            }
            candidates.retain(|(concept, word, _)| {
                l.lexicon
                    .get(concept)
                    .is_none_or(|v| !v.iter().any(|w| w.form == word.form))
                    && word.form != l.word(concept)
            });
            if candidates.is_empty() {
                continue;
            }
            let (concept, word, site) =
                candidates.swap_remove(hash(q ^ 11) as usize % candidates.len());
            if !l.adopt_word(&concept, word.clone()) {
                continue;
            }
            let source = word.source.as_ref().unwrap();
            let detail = format!(
                "{} adopted '{}' as an option for {} from {}{}; existing names retained",
                l.name,
                word.form,
                concept,
                source.name,
                word.borrowed_from
                    .map(|c| format!(" via civilization {c} after sustained route contact"))
                    .unwrap_or_default()
            );
            self.event("lexicon_adoption", site, None, detail);
            let event = self.events.last_mut().unwrap();
            event.subjects.push(("civilization".into(), civ));
            event.subjects.push((source.kind.clone(), source.id));
            if let Some(other) = word.borrowed_from {
                event.subjects.push(("civilization".into(), other));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn word(name: &str, month: u32) -> Lexeme {
        Lexeme {
            form: name.into(),
            adopted: month,
            source: Some(Source {
                kind: "person".into(),
                id: month,
                name: name.into(),
            }),
            borrowed_from: None,
        }
    }
    #[test]
    fn synonyms_age_weight_and_preserve_old_coinages() {
        let mut l = Language::new(17, 0);
        let old = l.coin("fixture:old", &["metal"], None);
        assert!(l.adopt_word("metal", word("avela", 12)));
        assert!(l.adopt_word("metal", word("morin", 24)));
        assert!(!l.adopt_word("metal", word("morin", 36)));
        let mut counts = BTreeMap::<String, u32>::new();
        for i in 0..7000 {
            *counts
                .entry(l.lexical_choice("metal", &format!("sample:{i}"), 0).form)
                .or_default() += 1;
        }
        assert!(counts["morin"] > counts["avela"]);
        assert!(counts["avela"] > counts[&l.word("metal")]);
        assert!(l.adopt_word("metal", word("selara", 36)));
        assert_eq!(
            l.lexicon["metal"]
                .iter()
                .map(|w| w.form.as_str())
                .collect::<Vec<_>>(),
            vec!["avela", "morin", "selara"]
        );
        assert_eq!(l.coin("fixture:old", &["metal"], None), old);
        assert_eq!(
            l.names["fixture:old"].words[0]
                .word
                .source
                .as_ref()
                .map(|s| s.id),
            None
        );
        assert!(l.adopt_word("metal", word("tesara", 48)));
        assert!(!l.lexicon["metal"].iter().any(|w| w.form == "avela"));
        let mut loaded: Language =
            serde_json::from_str(&serde_json::to_string(&l).unwrap()).unwrap();
        for i in 0..100 {
            assert_eq!(
                l.coin(&format!("fixture:{i}"), &["metal"], None),
                loaded.coin(&format!("fixture:{i}"), &["metal"], None)
            );
        }
        assert!(loaded.valid());
        eprintln!("lexical weight sample: {counts:?}");
    }
    #[test]
    #[ignore = "requires a GPU"]
    fn route_contact_borrows_without_same_year_cascades_or_renaming() {
        let mut g = crate::gpu::Generator::new(
            pollster::block_on(crate::gpu::ContextGpu::headless()).unwrap(),
            crate::config::Config {
                resolution: 32,
                ecology_resolution: 16,
                ..Default::default()
            },
            crate::catalog::Catalog::bundled().unwrap(),
        )
        .unwrap();
        g.found_civilizations(3).unwrap();
        g.enable_society().unwrap();
        let mut open = g.civilizations.as_ref().unwrap().clone();
        // A declared two-edge contact fixture; route geometry is outside this lexical test.
        let sites: Vec<u32> = (0..3)
            .map(|c| open.sites.iter().find(|s| s.civilization == c).unwrap().id)
            .collect();
        open.society.as_mut().unwrap().routes = (0..2)
            .map(|i| crate::society::Route {
                id: i as u32,
                from: sites[i],
                to: sites[i + 1],
                cells: vec![],
                cost_km: 10.,
                open: true,
                flood_months: 0,
                road_bricks: 0.,
                upkeep: None,
            })
            .collect();
        open.civilizations[0]
            .language
            .as_mut()
            .unwrap()
            .adopt_word("metal", word("avelara", 0));
        let original_labels: Vec<_> = open.sites.iter().map(|s| s.name.clone()).collect();
        let mut closed = open.clone();
        for r in &mut closed.society.as_mut().unwrap().routes {
            r.open = false;
        }
        let mut learned = false;
        for year in 1..=100 {
            open.month = year * 12;
            closed.month = year * 12;
            let before = open.civilizations[1]
                .language
                .as_ref()
                .unwrap()
                .lexicon
                .get("metal")
                .is_some_and(|v| v.iter().any(|w| w.form == "avelara"));
            open.evolve_lexicons();
            closed.evolve_lexicons();
            let after = open.civilizations[1]
                .language
                .as_ref()
                .unwrap()
                .lexicon
                .get("metal")
                .is_some_and(|v| v.iter().any(|w| w.form == "avelara"));
            if after && !before {
                assert!(year >= 3);
                assert!(!open.civilizations[2]
                    .language
                    .as_ref()
                    .unwrap()
                    .lexicon
                    .get("metal")
                    .is_some_and(|v| v.iter().any(|w| w.form == "avelara")));
                learned = true;
                break;
            }
        }
        assert!(learned);
        assert!(!closed.civilizations[1]
            .language
            .as_ref()
            .unwrap()
            .lexicon
            .get("metal")
            .is_some_and(|v| v.iter().any(|w| w.form == "avelara")));
        assert_eq!(
            open.sites
                .iter()
                .map(|s| s.name.clone())
                .collect::<Vec<_>>(),
            original_labels
        );
        let before = serde_json::to_value(&open).unwrap();
        open.evolve_lexicons();
        assert_eq!(
            before,
            serde_json::to_value(&open).unwrap(),
            "same-boundary replay"
        );
        let mut resumed: crate::civilization::History = serde_json::from_value(before).unwrap();
        for _ in 0..20 {
            open.month += 12;
            resumed.month += 12;
            open.evolve_lexicons();
            resumed.evolve_lexicons();
        }
        assert_eq!(
            serde_json::to_value(&open).unwrap(),
            serde_json::to_value(&resumed).unwrap()
        );
    }
}
