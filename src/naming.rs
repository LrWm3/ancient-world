//! Naming-only fictional daughter languages. No social or economic effects.
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

const ROOTS: &[(&str, &str)] = &[
    ("metal", "metalom"),
    ("bright", "alika"),
    ("steadfast", "tarem"),
    ("kind", "amita"),
    ("wise", "senika"),
    ("bold", "fortis"),
    ("patient", "melom"),
    ("dawn", "aurai"),
    ("star", "aster"),
    ("reed", "kalamos"),
    ("oak", "daru"),
    ("stone", "petra"),
    ("water", "akwa"),
    ("fire", "fokis"),
    ("wind", "anemos"),
    ("silver", "argenta"),
    ("gold", "aurum"),
    ("iron", "ferom"),
    ("clay", "keramos"),
    ("timber", "silwa"),
    ("grain", "grana"),
    ("field", "agros"),
    ("grove", "nemora"),
    ("hill", "kolis"),
    ("shore", "litor"),
    ("island", "insula"),
    ("haven", "portam"),
    ("home", "domus"),
    ("league", "foidera"),
    ("people", "genta"),
    ("guide", "dukis"),
    ("memory", "memora"),
    ("covenant", "paktom"),
    ("return", "redita"),
    ("sanctuary", "sakra"),
    ("market", "merkato"),
    ("craft", "tektom"),
    ("learning", "lektis"),
    ("house", "familia"),
    ("journey", "itara"),
    ("book", "libera"),
    ("vessel", "vasom"),
    ("gift", "donum"),
    ("keeper", "tutor"),
    ("bridge", "ponta"),
    ("harvest", "messis"),
    ("shelter", "tegom"),
    ("oath", "juram"),
    ("healing", "medika"),
];
const VIRTUES: &[&str] = &[
    "bright",
    "steadfast",
    "kind",
    "wise",
    "bold",
    "patient",
    "keeper",
    "healing",
];
const EMBLEMS: &[&str] = &[
    "dawn", "star", "reed", "oak", "stone", "water", "fire", "wind", "silver", "gold", "grove",
    "hill",
];
fn hash(mut x: u32) -> u32 {
    x = (x ^ (x >> 16)).wrapping_mul(0x7feb352d);
    x = (x ^ (x >> 15)).wrapping_mul(0x846ca68b);
    x ^ (x >> 16)
}
fn key_hash(s: &str) -> u32 {
    s.bytes()
        .fold(2166136261u32, |h, b| (h ^ b as u32).wrapping_mul(16777619))
}
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub enum Landmark {
    Shore,
    Water,
    Hill,
    Field,
}
impl Landmark {
    pub fn meaning(self) -> &'static str {
        match self {
            Self::Shore => "shore",
            Self::Water => "water",
            Self::Hill => "hill",
            Self::Field => "field",
        }
    }
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Source {
    pub kind: String,
    pub id: u32,
    pub name: String,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NameRecord {
    pub name: String,
    pub meanings: Vec<String>,
    pub source: Option<Source>,
    pub form: String,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Language {
    pub name: String,
    pub seed: u32,
    pub palatalization: u8,
    pub lenition: u8,
    pub lose_final_m: bool,
    pub lose_final_s: bool,
    pub initial_f_to_h: bool,
    pub kt: String,
    pub ai: String,
    pub oi: String,
    pub vowels: [char; 5],
    pub preferred_form: u8,
    /// Archived vocabulary: future source edits do not change an existing language.
    pub roots: BTreeMap<String, String>,
    pub names: BTreeMap<String, NameRecord>,
    #[serde(skip)]
    used: BTreeSet<(String, String)>,
}
impl Language {
    pub fn new(world: u32, civilization: u32) -> Self {
        let seed = hash(world ^ civilization.wrapping_mul(7919) ^ 0x6e616d65);
        let q = |salt| hash(seed ^ salt);
        let mut l = Self {
            name: String::new(),
            seed,
            palatalization: (q(1) % 3) as u8,
            lenition: (q(2) % 3) as u8,
            lose_final_m: q(3) % 4 != 0,
            lose_final_s: q(4) % 2 == 0,
            initial_f_to_h: q(5) % 3 == 0,
            kt: ["kt", "it", "ch"][q(6) as usize % 3].into(),
            ai: ["ai", "e", "ae"][q(7) as usize % 3].into(),
            oi: ["oi", "u", "e"][q(8) as usize % 3].into(),
            vowels: [
                ['a', 'e', 'i', 'o', 'u'],
                ['e', 'i', 'i', 'u', 'u'],
                ['a', 'e', 'e', 'o', 'o'],
                ['o', 'e', 'i', 'u', 'a'],
            ][q(9) as usize % 4],
            preferred_form: (q(10) % 3) as u8,
            roots: ROOTS.iter().map(|&(a, b)| (a.into(), b.into())).collect(),
            names: BTreeMap::new(),
            used: BTreeSet::new(),
        };
        l.name = title(&format!(
            "{}{}",
            l.word(EMBLEMS[q(11) as usize % EMBLEMS.len()]),
            l.word(VIRTUES[q(12) as usize % VIRTUES.len()])
        ));
        l
    }
    /// Ordered sound changes over a fictional proto-lexicon, not a Latin translator.
    pub fn evolve(&self, proto: &str) -> String {
        let early = proto
            .to_ascii_lowercase()
            .replace("kt", &self.kt)
            .replace("ai", &self.ai)
            .replace("oi", &self.oi);
        let chars: Vec<char> = early.chars().collect();
        let vowel = |c: char| "aeiou".contains(c);
        let mut out = String::new();
        for (i, &c) in chars.iter().enumerate() {
            let front = chars.get(i + 1).is_some_and(|c| matches!(c, 'e' | 'i'));
            let between =
                i > 0 && vowel(chars[i - 1]) && chars.get(i + 1).is_some_and(|&c| vowel(c));
            let replacement = if front && self.palatalization > 0 && matches!(c, 'k' | 'g') {
                match (self.palatalization, c) {
                    (1, 'k') => "ch",
                    (1, _) => "j",
                    (_, 'k') => "ts",
                    _ => "dz",
                }
            } else if between && self.lenition > 0 {
                match (self.lenition, c) {
                    (1, 'p') => "b",
                    (1, 't') => "d",
                    (1, 'k') => "g",
                    (2, 'p') => "f",
                    (2, 't') => "th",
                    (2, 'k') => "h",
                    _ => "",
                }
            } else {
                ""
            };
            if replacement.is_empty() {
                out.push(c);
            } else {
                out.push_str(replacement);
            }
        }
        if self.initial_f_to_h && out.starts_with('f') {
            out.replace_range(..1, "h");
        }
        if (self.lose_final_m && out.ends_with('m')) || (self.lose_final_s && out.ends_with('s')) {
            out.pop();
        }
        out.chars()
            .map(|c| "aeiou".find(c).map_or(c, |i| self.vowels[i]))
            .collect::<String>()
            .replace("kw", "qu")
    }
    pub fn valid(&self) -> bool {
        self.palatalization <= 2
            && self.lenition <= 2
            && self.preferred_form <= 2
            && self.vowels.iter().all(|v| "aeiou".contains(*v))
            && !self.name.is_empty()
            && !self.roots.is_empty()
            && self
                .roots
                .values()
                .all(|s| !s.is_empty() && s.bytes().all(|b| b.is_ascii_lowercase()))
            && [&self.kt, &self.ai, &self.oi]
                .iter()
                .all(|s| !s.is_empty() && s.bytes().all(|b| b.is_ascii_lowercase()))
            && self
                .names
                .values()
                .all(|r| !r.name.is_empty() && !r.meanings.is_empty())
    }
    pub fn word(&self, meaning: &str) -> String {
        self.evolve(
            self.roots
                .get(meaning)
                .map(String::as_str)
                .unwrap_or(meaning),
        )
    }
    pub fn personal(&mut self, id: u32) -> String {
        self.person_in("person", id)
    }
    /// Expedition crew are sparse crew records, not duplicate historical Person IDs.
    pub fn person_in(&mut self, category: &str, id: u32) -> String {
        let q = hash(self.seed ^ id.wrapping_mul(31337) ^ key_hash(category));
        self.coin(
            &format!("{category}:{id}"),
            &[
                VIRTUES[q as usize % VIRTUES.len()],
                EMBLEMS[(q >> 8) as usize % EMBLEMS.len()],
                EMBLEMS[(q >> 16) as usize % EMBLEMS.len()],
            ],
            None,
        )
    }
    pub fn coin(&mut self, key: &str, meanings: &[&str], source: Option<Source>) -> String {
        if let Some(record) = self.names.get(key) {
            return record.name.clone();
        }
        let q = hash(self.seed ^ key_hash(key));
        let mut words: Vec<String> = meanings.iter().map(|m| self.word(m)).collect();
        if let Some(s) = &source {
            // Proper names retain their existing sound, rather than undergoing the shifts twice.
            let stem: String = s
                .name
                .split_whitespace()
                .next()
                .unwrap_or("Anon")
                .chars()
                .filter(|c| c.is_alphabetic())
                .collect();
            words.insert(0, stem.to_lowercase());
        }
        let mut form = if q % 4 == 0 {
            (q / 4 % 3) as u8
        } else {
            self.preferred_form
        };
        if form == 1 && words.iter().map(|s| s.chars().count()).sum::<usize>() > 20 {
            form = 0;
        }
        let text = match form {
            0 => words.iter().map(|s| title(s)).collect::<Vec<_>>().join(" "),
            1 => title(&words.join("")),
            _ => {
                let mut blend = String::new();
                for (i, word) in words.iter().enumerate() {
                    let chars: Vec<_> = word.chars().collect();
                    let piece = if i + 1 == words.len() {
                        &chars[chars.len() / 2..]
                    } else {
                        &chars[..chars.len().div_ceil(2)]
                    };
                    blend.extend(piece);
                }
                title(&blend)
            }
        };
        let mut name = text.clone();
        // Uniqueness within entity category. Labels may legitimately repeat across categories.
        let category = key.split(':').next().unwrap_or(key);
        if self.used.len() != self.names.len() {
            self.used = self
                .names
                .iter()
                .map(|(k, v)| (k.split(':').next().unwrap_or(k).into(), v.name.clone()))
                .collect();
        }
        let taken = |candidate: &str| self.used.contains(&(category.into(), candidate.into()));
        if taken(&name) {
            name = format!(
                "{} {}",
                text,
                title(&self.word(EMBLEMS[(q >> 12) as usize % EMBLEMS.len()]))
            );
            if taken(&name) {
                name = format!("{} {}", name, key.split(':').next_back().unwrap_or(key));
            }
        }
        self.used.insert((category.into(), name.clone()));
        self.names.insert(
            key.into(),
            NameRecord {
                name: name.clone(),
                meanings: meanings.iter().map(|s| (*s).into()).collect(),
                source,
                form: ["separate words", "compound", "blend"][form as usize].into(),
            },
        );
        name
    }
}
fn title(s: &str) -> String {
    let mut c = s.chars();
    c.next().map_or(String::new(), |first| {
        first.to_uppercase().collect::<String>() + c.as_str()
    })
}
impl crate::civilization::Civilization {
    pub fn naming(&mut self, seed: u32) -> &mut Language {
        self.language
            .get_or_insert_with(|| Language::new(seed, self.id))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn ordered_sound_changes_have_known_results() {
        let mut l = Language::new(17, 0);
        l.palatalization = 1;
        l.lenition = 1;
        l.lose_final_m = true;
        l.lose_final_s = true;
        l.initial_f_to_h = true;
        l.kt = "it".into();
        l.ai = "e".into();
        l.oi = "u".into();
        l.vowels = ['a', 'e', 'i', 'o', 'u'];
        assert_eq!(l.evolve("faketum"), "hachedu");
        assert_eq!(l.evolve("paktom"), "pedo");
        assert_eq!(l.evolve("apakas"), "abaga");
        assert_eq!(l.evolve("akwa"), "aqua");
        l.lenition = 2;
        assert_eq!(l.evolve("apatas"), "afatha");
        l.vowels = ['e', 'i', 'a', 'u', 'o'];
        // Shifts are simultaneous, not recursively reapplied.
        assert_eq!(l.evolve("aeiou"), "eiauo");
    }
    #[test]
    fn identities_glosses_collisions_and_archives_are_stable() {
        let mut l = Language::new(81, 2);
        for i in 0..2000 {
            l.personal(i);
        }
        let unique: BTreeSet<_> = l.names.values().map(|r| &r.name).collect();
        assert_eq!(unique.len(), 2000);
        let source = Source {
            kind: "person".into(),
            id: 5,
            name: l.personal(5),
        };
        let n = l.coin("institution:3", &["market"], Some(source));
        assert_eq!(l.names["institution:3"].source.as_ref().unwrap().id, 5);
        assert_eq!(l.names["institution:3"].meanings, ["market"]);
        assert_eq!(n, l.coin("institution:3", &["fire"], None));
        let mut loaded: Language =
            serde_json::from_str(&serde_json::to_string(&l).unwrap()).unwrap();
        for i in 2000..2100 {
            assert_eq!(l.personal(i), loaded.personal(i));
        }
        assert!(loaded.valid());
        loaded.preferred_form = 99;
        assert!(!loaded.valid());
        let old: crate::civilization::Civilization =
            serde_json::from_str(r#"{"id":0,"name":"Old League","leader":0}"#).unwrap();
        assert!(old.language.is_none());
        assert_eq!(old.name, "Old League");
    }
    #[test]
    fn cultures_differ_without_changing_the_underlying_meanings() {
        let mut names = BTreeSet::new();
        for i in 0..16 {
            let mut l = Language::new(17, i);
            let n = l.coin("site:1", &["shore", "haven"], None);
            eprintln!(
                "{}: {} = shore + haven; {} = memory",
                l.name,
                n,
                l.word("memory")
            );
            names.insert(n);
            assert_eq!(l.names["site:1"].meanings, ["shore", "haven"]);
        }
        assert!(names.len() >= 12);
    }
    #[test]
    #[ignore = "requires a GPU"]
    fn named_history_keeps_context_and_checkpoint_continuity() {
        for seed in [17, 81, 256] {
            let mut g = crate::gpu::Generator::new(
                pollster::block_on(crate::gpu::ContextGpu::headless()).unwrap(),
                crate::config::Config {
                    seed,
                    resolution: 32,
                    ecology_resolution: 16,
                    ..Default::default()
                },
                crate::catalog::Catalog::bundled().unwrap(),
            )
            .unwrap();
            g.run_epochs(1).unwrap();
            g.found_civilizations(5).unwrap();
            g.enable_society().unwrap();
            g.advance_history(120).unwrap();
            let h = g.civilizations.as_ref().unwrap();
            for site in &h.sites {
                let candidate = h.candidates.iter().find(|c| c.cell == site.cell).unwrap();
                let record = &h.civilizations[site.civilization as usize]
                    .language
                    .as_ref()
                    .unwrap()
                    .names[&format!("site:{}", site.id)];
                assert_eq!(
                    record.meanings[0],
                    candidate
                        .naming_landmark
                        .map(Landmark::meaning)
                        .unwrap_or("island")
                );
            }
            for c in &h.civilizations {
                let l = c.language.as_ref().unwrap();
                assert!(l.valid());
                assert!(l
                    .names
                    .values()
                    .any(|r| r.source.as_ref().is_some_and(|s| s.kind == "patron")));
                let examples: Vec<_> = l
                    .names
                    .iter()
                    .filter(|(k, _)| k.starts_with("institution:") || k.starts_with("site:"))
                    .take(3)
                    .map(|(_, r)| format!("{} [{}]", r.name, r.meanings.join("+")))
                    .collect();
                eprintln!(
                    "seed {seed} · {} · {} · {} records: {}",
                    c.name,
                    l.name,
                    l.names.len(),
                    examples.join("; ")
                );
            }
            assert!(h.economy_residuals().iter().all(|x| x.abs() < 1e-4));
            if seed == 17 {
                let p = std::env::temp_dir()
                    .join(format!("ancient-names-{}.world", std::process::id()));
                g.save(&p).unwrap();
                let mut resumed = crate::gpu::Generator::load(g.gpu.clone(), &p).unwrap();
                std::fs::remove_file(p).unwrap();
                g.advance_history(12).unwrap();
                for _ in 0..12 {
                    resumed.advance_history(1).unwrap();
                }
                assert_eq!(
                    serde_json::to_value(&g.civilizations).unwrap(),
                    serde_json::to_value(&resumed.civilizations).unwrap()
                );
            }
        }
    }
}
