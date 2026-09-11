//! Small, naming-only lexical conventions, not a population language model.
use super::{hash, key_hash, Language, Source};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Lexeme {
    #[serde(default)]
    pub original_form: Option<String>,
    #[serde(default)]
    pub adapted: bool,
    #[serde(default)]
    pub basis: String,
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
    fn pronunciation(&self, mut word: Lexeme, salt: u32) -> Lexeme {
        let original = word.form.clone();
        let adapted = self.evolve(&original);
        word.adapted = hash(self.seed ^ salt ^ 0x736f756e) % 2 == 0
            && adapted != original
            && adapted.chars().count() >= 3
            && adapted.chars().count() <= 24;
        if word.adapted {
            word.form = adapted;
        }
        word.original_form = Some(original);
        word
    }
    fn association(&self, activity: &str, source: &Source, salt: u32) -> (String, &'static str) {
        if hash(salt ^ self.seed ^ 0x6574796d) % 4 == 0 {
            if let Some(record) = self.names.get(&format!("{}:{}", source.kind, source.id)) {
                let meanings: BTreeSet<_> = record
                    .meanings
                    .iter()
                    .filter(|m| m.as_str() != "of" && self.roots.contains_key(*m))
                    .collect();
                if !meanings.is_empty() {
                    return (
                        (*meanings
                            .iter()
                            .nth(hash(salt) as usize % meanings.len())
                            .unwrap())
                        .clone(),
                        "name etymology",
                    );
                }
            }
        }
        (activity.into(), "activity")
    }
    /// Pronunciation is settled at adoption; subsequent name use keeps that archived form.
    pub fn lexical_choice(&self, concept: &str, key: &str, slot: usize) -> Lexeme {
        let Some(options) = self.lexicon.get(concept).filter(|v| !v.is_empty()) else {
            return Lexeme {
                original_form: None,
                adapted: false,
                basis: String::new(),
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
                original_form: None,
                adapted: false,
                basis: String::new(),
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

// Deliberate milestones, rather than routine monthly accounting or lexical feedback.
fn significant(kind: &str) -> bool {
    matches!(
        kind,
        "founded"
            | "patron_arrival"
            | "patron_departure"
            | "abandoned"
            | "settlement_reoccupied"
            | "food_crisis"
            | "governance_crisis"
            | "governance_recovery"
            | "secession"
            | "war_declared"
            | "peace"
            | "raid_outcome"
            | "occupation_started"
            | "occupation_ended"
            | "succession"
            | "treaty_signed"
            | "faction_fragmentation"
            | "institution_founded"
            | "artifact_created"
            | "religious_schism"
            | "religious_syncretism"
            | "expedition_return"
            | "heritage_fragment_received"
            | "civic_petition_honored"
            | "port_opened"
            | "meeting_place_completed"
            | "regional_mine_activated"
    )
}
fn annual_slots(seed: u32, month: u32, events: usize) -> usize {
    1 + hash(seed ^ month ^ 0x62756467) as usize % events.max(1)
}

impl crate::civilization::History {
    pub(crate) fn observe_lexical_trade(&mut self, from: u32, to: u32, kg: f32) {
        if !kg.is_finite() || kg <= 0. {
            return;
        }
        let a = self.sites[from as usize].civilization;
        let b = self.sites[to as usize].civilization;
        if a == b {
            return;
        }
        for (observer, other) in [(a, b), (b, a)] {
            if let Some(l) = self.civilizations[observer as usize].language.as_mut() {
                *l.trade_kg.entry(other).or_default() += kg as f64;
            }
        }
    }
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
        // Completed receipts also establish contact, even when a route closes later.
        for c in &self.civilizations {
            if let Some(l) = &c.language {
                for (&other, &kg) in &l.trade_kg {
                    if kg > 0. {
                        contacts.insert((c.id, other));
                    }
                }
            }
        }
        for ci in 0..self.civilizations.len() {
            let civ = self.civilizations[ci].id;
            let significant_events = self
                .events
                .iter()
                .filter(|e| {
                    e.month > self.month.saturating_sub(12)
                        && e.month <= self.month
                        && significant(&e.kind)
                        && (e.subjects.contains(&("civilization".into(), civ))
                            || [e.site, e.other].into_iter().flatten().any(|id| {
                                self.sites
                                    .get(id as usize)
                                    .is_some_and(|s| s.civilization == civ)
                            }))
                })
                .count();
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
            let slots = annual_slots(l.seed, self.month, significant_events);
            let mut pending = Vec::new();
            let mut adopted = BTreeSet::new();
            let trade = std::mem::take(&mut l.trade_kg);
            let trade_bonus = trade.values().copied().sum::<f64>().sqrt().min(30.) as u32;
            for slot in 0..slots {
                let q =
                    hash(l.seed ^ self.month ^ 0x6c657869 ^ (slot as u32).wrapping_mul(0x9e3779b9));
                let mut candidates: Vec<(String, Lexeme, Option<u32>)> = vec![];
                if q % 5 == 0 || hash(q ^ 0x74726164) % 100 < trade_bonus {
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
                                    loan.basis = "borrowed".into();
                                    loan = l.pronunciation(loan, q ^ other ^ key_hash(concept));
                                    let tickets = 1
                                        + (trade.get(&other).copied().unwrap_or(0.) / 100.)
                                            .sqrt()
                                            .min(8.)
                                            as usize;
                                    for _ in 0..tickets {
                                        candidates.push((concept.clone(), loan.clone(), None));
                                    }
                                }
                            }
                        }
                    }
                } else {
                    // Production and durable institutions supply associations, not arbitrary virtues
                    // inferred from a person's name. Thresholds are explicit game conventions.
                    let mut add = |concept: &str, source: Source, site: Option<u32>| {
                        let form = l
                            .source_stem(&source.name, hash(q ^ source.id))
                            .chars()
                            .take(16)
                            .collect();
                        let (concept, basis) =
                            l.association(concept, &source, q ^ source.id ^ key_hash(concept));
                        candidates.push((
                            concept,
                            Lexeme {
                                original_form: None,
                                adapted: false,
                                basis: basis.into(),
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
                    if let (Some(g), Some(c)) = (&self.governance, &self.culture) {
                        for petition in g.petitions.iter().filter(|p| {
                            p.honored
                                && self.sites[p.site as usize].civilization == civ
                                && p.resolved
                                    .is_some_and(|m| self.month.saturating_sub(m) <= 120)
                        }) {
                            let n = &c.institutions[petition.institution as usize];
                            add(
                                petition.demand.concept(),
                                Source {
                                    kind: "institution".into(),
                                    id: n.id,
                                    name: n.name.clone(),
                                },
                                Some(petition.site),
                            );
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
                let (concept, mut word, site) =
                    candidates.swap_remove(hash(q ^ 11) as usize % candidates.len());
                if word.borrowed_from.is_none() {
                    word = l.pronunciation(word, q ^ key_hash(&concept));
                }
                if !adopted.insert((concept.clone(), word.form.clone()))
                    || !l.adopt_word(&concept, word.clone())
                {
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
                        .map(|c| format!(" via civilization {c} after sustained contact"))
                        .unwrap_or_default()
                );
                let detail = format!(
                    "{detail}; basis {}; pronunciation {}",
                    word.basis,
                    if word.adapted {
                        "locally adapted"
                    } else {
                        "retained"
                    }
                );
                pending.push((site, detail, source.clone(), word.borrowed_from, concept));
            }
            for (site, detail, source, borrowed_from, concept) in pending {
                self.event("lexicon_adoption", site, None, detail);
                let event = self.events.last_mut().unwrap();
                event.subjects.push(("civilization".into(), civ));
                event.subjects.push((source.kind.clone(), source.id));
                if source.kind == "institution" {
                    if let Some(cause) = self.governance.as_ref().and_then(|g| {
                        g.petitions
                            .iter()
                            .rev()
                            .find(|p| {
                                p.institution == source.id
                                    && p.honored
                                    && p.demand.concept() == concept
                                    && p.resolved
                                        .is_some_and(|m| self.month.saturating_sub(m) <= 120)
                            })
                            .and_then(|p| p.outcome)
                    }) {
                        event.causes.push(cause);
                    }
                }
                if let Some(other) = borrowed_from {
                    event.subjects.push(("civilization".into(), other));
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn word(name: &str, month: u32) -> Lexeme {
        Lexeme {
            original_form: None,
            adapted: false,
            basis: String::new(),
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
    fn event_budget_is_bounded_and_varies() {
        let mut seen = BTreeSet::new();
        for seed in 0..1000 {
            assert_eq!(annual_slots(seed, 12, 0), 1);
            assert_eq!(annual_slots(seed, 12, 1), 1);
            let slots = annual_slots(seed, 12, 8);
            assert!((1..=8).contains(&slots));
            seen.insert(slots);
        }
        assert_eq!(seen.len(), 8);
        assert!(significant("religious_schism"));
        assert!(!significant("lexicon_adoption"));
        assert!(!significant("market_arrival"));
        assert!(!significant("harvest"));
    }
    #[test]
    fn pronunciation_and_name_etymology_are_seeded_alternatives() {
        let mut l = Language::new(17, 0);
        l.vowels = ['e', 'i', 'a', 'u', 'o'];
        let name = l.coin("site:4", &["moon", "water"], None);
        let source = Source {
            kind: "site".into(),
            id: 4,
            name,
        };
        let mut associations = BTreeMap::<String, usize>::new();
        let mut retained = 0;
        let mut adapted = 0;
        for salt in 0..1000 {
            let result = l.pronunciation(word("morina", 12), salt);
            assert_eq!(result.original_form.as_deref(), Some("morina"));
            if result.adapted {
                adapted += 1;
                assert_eq!(result.form, l.evolve("morina"));
            } else {
                retained += 1;
                assert_eq!(result.form, "morina");
            }
            let (concept, _) = l.association("metal", &source, salt);
            *associations.entry(concept).or_default() += 1;
        }
        assert!((400..600).contains(&retained) && (400..600).contains(&adapted));
        assert!((650..850).contains(&associations["metal"]));
        assert!(associations["moon"] > 50 && associations["water"] > 50);
        eprintln!(
            "pronunciation retained/adapted {retained}/{adapted}; associations {associations:?}"
        );
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
    fn delivered_trade_is_counted_once_and_volume_changes_borrowing() {
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
        let mut h = g.civilizations.as_ref().unwrap().clone();
        for s in &mut h.sites {
            s.economy.policy[3] = 0.;
        }
        h.sites[0].economy.goods[3] -= 12.;
        h.cargo.push(crate::economy::Cargo {
            voyage_clock: None,
            freight_stops: vec![],
            from: 0,
            to: 1,
            good: 3,
            kg: 12.,
            paid: 0.,
            arrives: 2,
            sea_lane: None,
            weather_delay_months: 0,
        });
        h.month = 1;
        h.market_month(g.config.radius_km);
        assert!(h.civilizations[1]
            .language
            .as_ref()
            .unwrap()
            .trade_kg
            .is_empty());
        h.month = 2;
        h.market_month(g.config.radius_km);
        assert_eq!(
            h.civilizations[1].language.as_ref().unwrap().trade_kg[&0],
            12.
        );
        h.market_month(g.config.radius_km);
        assert_eq!(
            h.civilizations[1].language.as_ref().unwrap().trade_kg[&0],
            12.
        );
        assert!(h.economy_residuals().iter().all(|r| r.abs() < 0.001));
        h.civilizations[0]
            .language
            .as_mut()
            .unwrap()
            .adopt_word("metal", word("morina", 0));
        let mut counts = [0; 2];
        for trial in 0..200 {
            for (case, kg) in [10., 10000.].into_iter().enumerate() {
                let mut run = h.clone();
                for c in &mut run.civilizations {
                    c.language.as_mut().unwrap().trade_kg.clear();
                }
                run.month = (trial + 4) * 12;
                run.civilizations[1]
                    .language
                    .as_mut()
                    .unwrap()
                    .contact_years
                    .insert(0, 3);
                run.observe_lexical_trade(0, 1, kg);
                run.evolve_lexicons();
                if run.civilizations[1]
                    .language
                    .as_ref()
                    .unwrap()
                    .lexicon
                    .get("metal")
                    .is_some_and(|words| words.iter().any(|w| w.borrowed_from == Some(0)))
                {
                    counts[case] += 1;
                }
            }
        }
        // Many milestones allow multiple distinct adoptions, not just repeated yearly checks.
        for concept in ["moon", "water", "grain", "book", "craft", "guide"] {
            h.civilizations[0]
                .language
                .as_mut()
                .unwrap()
                .adopt_word(concept, word("avelara", 0));
        }
        let mut maximum = 0;
        for trial in 0..40 {
            let mut run = h.clone();
            run.month = (trial + 4) * 12;
            run.civilizations[1]
                .language
                .as_mut()
                .unwrap()
                .contact_years
                .insert(0, 3);
            run.observe_lexical_trade(0, 1, 10000.);
            for _ in 0..12 {
                run.event("food_crisis", Some(1), None, "Budget fixture".into());
            }
            let start = run.events.len();
            run.evolve_lexicons();
            let n = run.events[start..]
                .iter()
                .filter(|e| {
                    e.kind == "lexicon_adoption" && e.subjects.contains(&("civilization".into(), 1))
                })
                .count();
            assert!(n <= 12);
            maximum = maximum.max(n);
            let saved = serde_json::to_value(&run).unwrap();
            run.evolve_lexicons();
            assert_eq!(saved, serde_json::to_value(&run).unwrap());
        }
        assert!(maximum > 1, "eventful years must permit multiple adoptions");
        eprintln!("maximum adoptions with twelve milestones: {maximum}");
        assert!(counts[1] > counts[0], "{counts:?}");
        eprintln!(
            "low/high delivered-volume borrowing in 200 matched annual opportunities: {counts:?}"
        );
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
                .is_some_and(|v| {
                    v.iter()
                        .any(|w| w.source.as_ref().is_some_and(|s| s.name == "avelara"))
                });
            open.evolve_lexicons();
            closed.evolve_lexicons();
            let after = open.civilizations[1]
                .language
                .as_ref()
                .unwrap()
                .lexicon
                .get("metal")
                .is_some_and(|v| {
                    v.iter()
                        .any(|w| w.source.as_ref().is_some_and(|s| s.name == "avelara"))
                });
            if after && !before {
                assert!(year >= 3);
                assert!(!open.civilizations[2]
                    .language
                    .as_ref()
                    .unwrap()
                    .lexicon
                    .get("metal")
                    .is_some_and(|v| v
                        .iter()
                        .any(|w| w.source.as_ref().is_some_and(|s| s.name == "avelara"))));
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
            .is_some_and(|v| v
                .iter()
                .any(|w| w.source.as_ref().is_some_and(|s| s.name == "avelara"))));
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
