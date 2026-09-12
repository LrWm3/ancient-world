//! Naming-only fictional daughter languages. No social or economic effects.
mod evolution;
pub use evolution::{Lexeme, WordUse};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

const ROOTS: &[(&str, &str)] = &[
    ("islet", "insela"),
    ("canopy", "komara"),
    ("daring", "valora"),
    ("humble", "humila"),
    ("serene", "placida"),
    ("ardent", "ardera"),
    ("nimble", "levika"),
    ("solemn", "gravena"),
    ("playful", "lusira"),
    ("prudent", "kavena"),
    ("hopeful", "sperina"),
    ("unyielding", "firama"),
    ("tender", "dulena"),
    ("secretive", "keluma"),
    ("echo", "ekora"),
    ("silence", "silenta"),
    ("dream", "somira"),
    ("wonder", "mirava"),
    ("riddle", "enigma"),
    ("horizon", "oriza"),
    ("comet", "kometa"),
    ("eclipse", "eklira"),
    ("halo", "aurela"),
    ("lightning", "fulgura"),
    ("rainbow", "irida"),
    ("monsoon", "mavora"),
    ("breeze", "zefera"),
    ("dew", "rosina"),
    ("hail", "grandina"),
    ("sleet", "nivera"),
    ("glacier", "glakara"),
    ("gorge", "ravena"),
    ("ravine", "karuna"),
    ("terrace", "gradara"),
    ("estuary", "estuara"),
    ("delta", "deltana"),
    ("shoal", "vadina"),
    ("reef", "korala"),
    ("lagoon", "taluna"),
    ("springwater", "fontara"),
    ("cascade", "kaskara"),
    ("rapids", "vortina"),
    ("inlet", "sinara"),
    ("dune", "dunara"),
    ("basalt", "basalara"),
    ("obsidian", "obsira"),
    ("quartz", "krista"),
    ("jade", "jadira"),
    ("mica", "mikara"),
    ("garnet", "graneta"),
    ("cypress", "kuparisa"),
    ("juniper", "junira"),
    ("yew", "taxara"),
    ("alder", "alnera"),
    ("hazel", "korila"),
    ("heather", "erika"),
    ("lily", "lilena"),
    ("lotus", "lotara"),
    ("iris", "irisa"),
    ("sedge", "kareka"),
    ("lichen", "likena"),
    ("coral", "koralina"),
    ("ibis", "ibira"),
    ("kestrel", "kestara"),
    ("gull", "larina"),
    ("tern", "stera"),
    ("kingfisher", "alkena"),
    ("lynx", "linka"),
    ("marten", "martena"),
    ("ibex", "ibara"),
    ("bison", "bisara"),
    ("seal", "fokana"),
    ("eel", "angula"),
    ("sturgeon", "akipra"),
    ("beetle", "skaraba"),
    ("cicada", "zikara"),
    ("spider", "aranea"),
    ("silk", "serika"),
    ("needle", "akula"),
    ("spindle", "fusara"),
    ("loom", "telara"),
    ("chisel", "skalpa"),
    ("anvil", "inkuda"),
    ("plough", "aratra"),
    ("basket", "kanistra"),
    ("cup", "kalika"),
    ("mirror", "spekula"),
    ("flute", "tibira"),
    ("lyre", "lirena"),
    ("mask", "persona"),
    ("tapestry", "tapeta"),
    ("threshold", "limena"),
    ("courtyard", "atrina"),
    ("arcade", "arkada"),
    ("commons", "komuna"),
    ("council", "konsila"),
    ("kinship", "parena"),
    ("hospitality", "hospira"),
    ("lullaby", "nannara"),
    ("lament", "elega"),
    ("parable", "fabula"),
    ("verse", "versena"),
    ("respite", "pausa"),
    ("beacon", "farena"),
    ("harbor", "limarae"),
    ("watcher", "skopira"),
    ("wanderer", "errana"),
    ("companion", "komera"),
    ("herald", "keruka"),
    ("mediator", "mesara"),
    ("awakening", "evela"),
    ("farewell", "valeta"),
    ("sorrow", "dolara"),
    ("joy", "gaudia"),
    ("harmony", "konkora"),
    ("of", "na"),
    ("war", "belora"),
    ("campaign", "strateia"),
    ("claim", "vindika"),
    ("frontier", "limara"),
    ("hearth", "fokara"),
    ("cloth", "pannara"),
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
    ("gentle", "lemira"),
    ("merry", "hilara"),
    ("just", "equita"),
    ("loyal", "fidela"),
    ("curious", "quesira"),
    ("free", "liberi"),
    ("swift", "velora"),
    ("calm", "serena"),
    ("generous", "donara"),
    ("watchful", "vigila"),
    ("resilient", "tenaka"),
    ("honest", "verita"),
    ("moon", "seluna"),
    ("sun", "solara"),
    ("rain", "pluvia"),
    ("snow", "nivara"),
    ("mist", "nebula"),
    ("cloud", "nubera"),
    ("thunder", "tonara"),
    ("frost", "gelida"),
    ("spring", "verna"),
    ("summer", "estara"),
    ("autumn", "frugara"),
    ("winter", "hiberna"),
    ("dusk", "vespara"),
    ("night", "nokta"),
    ("ember", "brasa"),
    ("ash", "cinera"),
    ("flame", "flama"),
    ("tide", "undara"),
    ("wave", "ondina"),
    ("foam", "spuma"),
    ("current", "fluvena"),
    ("river", "rivana"),
    ("brook", "riloma"),
    ("marsh", "paluda"),
    ("fen", "ulmera"),
    ("pool", "laguna"),
    ("lake", "lakora"),
    ("sea", "marina"),
    ("cliff", "rupina"),
    ("valley", "valena"),
    ("ridge", "dorsana"),
    ("peak", "akrena"),
    ("cave", "speluna"),
    ("hollow", "konava"),
    ("meadow", "pratena"),
    ("heath", "bruyera"),
    ("willow", "salika"),
    ("pine", "pinara"),
    ("birch", "betula"),
    ("cedar", "kedara"),
    ("fern", "filika"),
    ("moss", "musena"),
    ("blossom", "florena"),
    ("seed", "semina"),
    ("root", "radika"),
    ("leaf", "folia"),
    ("thorn", "spina"),
    ("vine", "vitara"),
    ("flax", "linara"),
    ("barley", "hordea"),
    ("millet", "milika"),
    ("wheat", "tritika"),
    ("orchard", "pomara"),
    ("apple", "malina"),
    ("pear", "pirena"),
    ("olive", "oleva"),
    ("fig", "fikara"),
    ("heron", "ardea"),
    ("crane", "gruva"),
    ("raven", "korva"),
    ("swallow", "hiruna"),
    ("lark", "aloda"),
    ("wren", "troda"),
    ("owl", "ulula"),
    ("eagle", "akuila"),
    ("hawk", "falena"),
    ("fox", "vulpa"),
    ("wolf", "lupena"),
    ("deer", "cerva"),
    ("hare", "lepora"),
    ("otter", "lutra"),
    ("badger", "melora"),
    ("bear", "ursena"),
    ("bee", "apina"),
    ("moth", "noktila"),
    ("salmon", "salara"),
    ("trout", "trutta"),
    ("carp", "karpa"),
    ("shell", "konka"),
    ("pearl", "perula"),
    ("copper", "kupara"),
    ("tin", "stanara"),
    ("bronze", "bronta"),
    ("salt", "salena"),
    ("flint", "sileka"),
    ("chalk", "kreta"),
    ("amber", "sukina"),
    ("weaver", "texira"),
    ("smith", "ferrika"),
    ("potter", "keramita"),
    ("grower", "agrena"),
    ("shepherd", "pastora"),
    ("fisher", "piskara"),
    ("sailor", "navita"),
    ("mason", "lapida"),
    ("healer", "medena"),
    ("scribe", "skriba"),
    ("teacher", "docera"),
    ("merchant", "merkana"),
    ("miller", "molina"),
    ("baker", "panera"),
    ("dyer", "tinkara"),
    ("carpenter", "lignara"),
    ("messenger", "nuntia"),
    ("watch", "vigara"),
    ("pilgrim", "peregra"),
    ("witness", "testara"),
    ("ancestor", "avena"),
    ("promise", "sponda"),
    ("mercy", "venia"),
    ("vigil", "vigilia"),
    ("feast", "festara"),
    ("song", "kantara"),
    ("dance", "saltera"),
    ("drum", "timpana"),
    ("bell", "kampana"),
    ("lantern", "lukerna"),
    ("ribbon", "tenia"),
    ("crown", "korona"),
    ("mantle", "palium"),
    ("banner", "veksila"),
    ("circle", "orbina"),
    ("assembly", "komita"),
    ("fellowship", "sodala"),
    ("tower", "turena"),
    ("gate", "valva"),
    ("court", "kuria"),
    ("hall", "aula"),
    ("path", "semeta"),
    ("crossing", "vadara"),
    ("refuge", "refugia"),
    ("renewal", "novara"),
    ("victory", "viktara"),
    ("peace", "pakira"),
    ("endurance", "durana"),
    ("remembrance", "memorina"),
];
const VIRTUES: &[&str] = &[
    "daring",
    "humble",
    "serene",
    "ardent",
    "nimble",
    "solemn",
    "playful",
    "prudent",
    "hopeful",
    "unyielding",
    "tender",
    "secretive",
    "bright",
    "steadfast",
    "kind",
    "wise",
    "bold",
    "patient",
    "keeper",
    "healing",
    "gentle",
    "merry",
    "just",
    "loyal",
    "curious",
    "free",
    "swift",
    "calm",
    "generous",
    "watchful",
    "resilient",
    "honest",
];
const EMBLEMS: &[&str] = &[
    "echo",
    "silence",
    "dream",
    "wonder",
    "riddle",
    "horizon",
    "comet",
    "eclipse",
    "halo",
    "lightning",
    "rainbow",
    "monsoon",
    "breeze",
    "dew",
    "hail",
    "sleet",
    "glacier",
    "gorge",
    "ravine",
    "terrace",
    "estuary",
    "delta",
    "shoal",
    "reef",
    "lagoon",
    "springwater",
    "cascade",
    "rapids",
    "inlet",
    "dune",
    "basalt",
    "obsidian",
    "quartz",
    "jade",
    "mica",
    "garnet",
    "cypress",
    "juniper",
    "yew",
    "alder",
    "hazel",
    "heather",
    "lily",
    "lotus",
    "iris",
    "sedge",
    "lichen",
    "coral",
    "ibis",
    "kestrel",
    "gull",
    "tern",
    "kingfisher",
    "lynx",
    "marten",
    "ibex",
    "bison",
    "seal",
    "eel",
    "sturgeon",
    "beetle",
    "cicada",
    "spider",
    "silk",
    "needle",
    "spindle",
    "loom",
    "chisel",
    "anvil",
    "plough",
    "basket",
    "cup",
    "mirror",
    "flute",
    "lyre",
    "mask",
    "tapestry",
    "threshold",
    "courtyard",
    "dawn",
    "star",
    "reed",
    "oak",
    "stone",
    "water",
    "fire",
    "wind",
    "silver",
    "gold",
    "grove",
    "hill",
    "moon",
    "rain",
    "snow",
    "mist",
    "thunder",
    "dusk",
    "ember",
    "tide",
    "river",
    "cliff",
    "valley",
    "willow",
    "pine",
    "birch",
    "fern",
    "blossom",
    "thorn",
    "heron",
    "raven",
    "lark",
    "owl",
    "fox",
    "deer",
    "otter",
    "bee",
    "pearl",
    "amber",
    "flint",
    "song",
    "lantern",
    "spring",
    "sun",
    "cloud",
    "frost",
    "summer",
    "autumn",
    "winter",
    "night",
    "ash",
    "flame",
    "wave",
    "foam",
    "current",
    "brook",
    "marsh",
    "fen",
    "pool",
    "lake",
    "sea",
    "ridge",
    "peak",
    "cave",
    "hollow",
    "meadow",
    "heath",
    "cedar",
    "moss",
    "seed",
    "root",
    "leaf",
    "vine",
    "orchard",
    "apple",
    "pear",
    "olive",
    "fig",
    "crane",
    "swallow",
    "wren",
    "eagle",
    "hawk",
    "wolf",
    "hare",
    "badger",
    "bear",
    "moth",
    "salmon",
    "trout",
    "carp",
    "shell",
    "bell",
    "ribbon",
    "mantle",
    "banner",
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
/// Only associations available at naming time, never a scan of future/global names.
#[derive(Clone, Debug, Default)]
pub struct PersonalContext {
    pub references: Vec<(Source, u32)>,
    pub concepts: Vec<String>,
}
impl PersonalContext {
    pub fn local(
        site: &crate::civilization::Site,
        culture: Option<&crate::culture::Culture>,
    ) -> Self {
        let mut c = Self::default();
        c.references.push((
            Source {
                kind: "site".into(),
                id: site.id,
                name: site.name.clone(),
            },
            3,
        ));
        for (g, material, occupation) in [
            (0, "timber", "carpenter"),
            (2, "metal", "smith"),
            (5, "clay", "mason"),
            (7, "clay", "potter"),
            (8, "wheat", "grower"),
            (9, "barley", "grower"),
            (10, "millet", "grower"),
            (18, "cloth", "weaver"),
            (28, "water", "fisher"),
        ] {
            if site.economy.made[g] > 1. {
                c.concepts.push(material.into());
                c.concepts.push(occupation.into());
            }
        }
        if let Some(culture) = culture {
            if let Some(t) = culture
                .site_faith
                .get(site.id as usize)
                .and_then(|id| culture.traditions.get(*id as usize))
            {
                c.references.push((
                    Source {
                        kind: "tradition".into(),
                        id: t.id,
                        name: t.name.clone(),
                    },
                    3,
                ));
                if let Some(p) = t.patron.and_then(|id| culture.patrons.get(id as usize)) {
                    c.references.push((
                        Source {
                            kind: "patron".into(),
                            id: p.id,
                            name: p.name.clone(),
                        },
                        2,
                    ));
                }
            }
            // Bound the candidate list; choose established local organizations, not distant fame.
            for n in culture
                .institutions
                .iter()
                .filter(|n| n.active && n.site == site.id)
                .take(4)
            {
                c.references.push((
                    Source {
                        kind: "institution".into(),
                        id: n.id,
                        name: n.name.clone(),
                    },
                    1,
                ));
            }
        }
        c
    }
    pub fn with_person(mut self, person: &crate::civilization::Person) -> Self {
        self.references.push((
            Source {
                kind: "person".into(),
                id: person.id,
                name: person.name.clone(),
            },
            6,
        ));
        self
    }
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NameRecord {
    #[serde(default)]
    pub words: Vec<WordUse>,
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
    #[serde(default)]
    pub lexicon: BTreeMap<String, Vec<Lexeme>>,
    #[serde(default)]
    pub lexicon_month: u32,
    #[serde(default)]
    pub contact_years: BTreeMap<u32, u32>,
    #[serde(default)]
    pub trade_kg: BTreeMap<u32, f64>,
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
            preferred_form: (q(10) % 5) as u8,
            roots: ROOTS.iter().map(|&(a, b)| (a.into(), b.into())).collect(),
            names: BTreeMap::new(),
            lexicon: BTreeMap::new(),
            lexicon_month: 0,
            contact_years: BTreeMap::new(),
            trade_kg: BTreeMap::new(),
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
            && self.preferred_form <= 4
            && self.vowels.iter().all(|v| "aeiou".contains(*v))
            && !self.name.is_empty()
            && !self.roots.is_empty()
            && self.trade_kg.values().all(|v| v.is_finite() && *v >= 0.)
            && self.lexicon.iter().all(|(concept, options)| {
                self.roots.contains_key(concept)
                    && !options.is_empty()
                    && options.len() <= 3
                    && options
                        .iter()
                        .all(|w| !w.form.is_empty() && w.form.chars().all(|c| c.is_alphabetic()))
                    && options.windows(2).all(|w| w[0].adopted <= w[1].adopted)
                    && options
                        .iter()
                        .map(|w| &w.form)
                        .collect::<BTreeSet<_>>()
                        .len()
                        == options.len()
            })
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
        self.person_with(category, id, &PersonalContext::default())
    }
    pub fn person_with(&mut self, category: &str, id: u32, context: &PersonalContext) -> String {
        let key = format!("{category}:{id}");
        if let Some(r) = self.names.get(&key) {
            return r.name.clone();
        }
        let q = hash(self.seed ^ id.wrapping_mul(31337) ^ key_hash(category));
        let virtue = VIRTUES[q as usize % VIRTUES.len()];
        let emblem = EMBLEMS[hash(q ^ 1) as usize % EMBLEMS.len()];
        let other = EMBLEMS[hash(q ^ 2) as usize % EMBLEMS.len()];
        // Weighted conventions: 30% aspiration, 20% nature, 20% local livelihood,
        // 30% commemoration. A third of the aspiration share uses celebration imagery.
        // Missing context falls back to an aspirational name.
        let mut source = None;
        let (meanings, style) = match hash(q ^ 3) % 10 {
            3..=4 => (vec![emblem, other], "nature pairing"),
            5..=6 if !context.concepts.is_empty() => (
                vec![
                    emblem,
                    context.concepts[hash(q ^ 4) as usize % context.concepts.len()].as_str(),
                ],
                "local livelihood",
            ),
            7..=9 if !context.references.is_empty() => {
                let total: u32 = context.references.iter().map(|(_, w)| *w).sum();
                let mut pick = hash(q ^ 5) % total.max(1);
                for (s, w) in &context.references {
                    if pick < *w {
                        source = Some(s.clone());
                        break;
                    }
                    pick -= *w;
                }
                (vec![virtue, emblem], "commemorative")
            }
            0 => (
                vec![
                    [
                        "feast",
                        "song",
                        "promise",
                        "kinship",
                        "awakening",
                        "joy",
                        "harmony",
                        "hospitality",
                    ][hash(q ^ 6) as usize % 8],
                    other,
                ],
                "celebratory",
            ),
            _ => (vec![virtue, emblem], "aspirational"),
        };
        let name = self.coin(&key, &meanings, source);
        let record = self.names.get_mut(&key).unwrap();
        record.form = format!("{style}; {}", record.form);
        name
    }
    /// A community's symbolic name for its guide, not a claim about anatomy or powers.
    /// Habitat contributes to some names; most draw from the wider shared lexicon.
    pub fn patron(&mut self, id: u32, habitat: &str, intelligent: bool) -> String {
        let key = format!("patron:{id}");
        if let Some(r) = self.names.get(&key) {
            return r.name.clone();
        }
        let q = hash(self.seed ^ key_hash(&key) ^ 0x70617472);
        let emblem = EMBLEMS[q as usize % EMBLEMS.len()];
        let other = EMBLEMS[hash(q ^ 1) as usize % EMBLEMS.len()];
        let quality = VIRTUES[hash(q ^ 2) as usize % VIRTUES.len()];
        let habitat_words: &[&str] = match habitat {
            "lake" => &["lake", "current", "reed", "islet", "mist", "pearl"],
            "wetland" => &["sedge", "lotus", "heron", "marsh", "pool", "willow"],
            "forest" => &["grove", "lichen", "cedar", "fern", "root", "canopy"],
            "volcanic" => &["obsidian", "ember", "basalt", "ash", "garnet", "flame"],
            "coast" => &["tide", "coral", "reef", "gull", "horizon", "foam"],
            _ => EMBLEMS,
        };
        let local = habitat_words[hash(q ^ 3) as usize % habitat_words.len()];
        let roles: &[&str] = if intelligent {
            &[
                "guide", "herald", "witness", "mediator", "wanderer", "keeper",
            ]
        } else {
            &[
                "guide",
                "watcher",
                "companion",
                "wanderer",
                "shelter",
                "beacon",
            ]
        };
        let role = roles[hash(q ^ 4) as usize % roles.len()];
        let (meanings, style) = match hash(q ^ 5) % 10 {
            0..=2 => (vec![local, quality], "homeland imagery"),
            3..=4 => (vec![emblem, role], "guide epithet"),
            5..=6 => (vec![quality, emblem], "character epithet"),
            7..=8 => (vec![emblem, other], "symbolic pairing"),
            _ => (vec![other], "single emblem"),
        };
        let name = self.coin(&key, &meanings, None);
        let record = self.names.get_mut(&key).unwrap();
        record.form = format!("{style}; {}", record.form);
        name
    }
    fn source_stem(&self, name: &str, q: u32) -> String {
        let particle = self.word("of");
        let tokens: Vec<String> = name
            .split_whitespace()
            .map(|s| {
                s.chars()
                    .filter(|c| c.is_alphabetic())
                    .collect::<String>()
                    .to_lowercase()
            })
            .filter(|s| {
                s.chars().count() >= 3
                    && s != &particle
                    && !matches!(s.as_str(), "of" | "the" | "and")
            })
            .collect();
        tokens
            .get((q as usize / 7) % tokens.len().max(1))
            .cloned()
            .unwrap_or_else(|| self.word("memory"))
    }
    /// Alternative naming heads, not assertions about the entity's legal status.
    pub fn descriptor<'a>(&self, key: &str, base: &'a str) -> &'a str {
        let options: &[&str] = match base {
            "home" => &[
                "home",
                "home",
                "hearth",
                "shelter",
                "refuge",
                "haven",
                "threshold",
                "commons",
            ],
            "league" => &[
                "league",
                "league",
                "people",
                "assembly",
                "fellowship",
                "council",
                "covenant",
                "kinship",
            ],
            "sanctuary" => &[
                "sanctuary",
                "sanctuary",
                "vigil",
                "covenant",
                "circle",
                "beacon",
                "harmony",
                "shelter",
            ],
            "market" => &[
                "market",
                "market",
                "crossing",
                "fellowship",
                "house",
                "commons",
                "courtyard",
                "hospitality",
            ],
            "learning" => &[
                "learning", "learning", "memory", "witness", "circle", "riddle", "parable",
                "wonder",
            ],
            "craft" => &[
                "craft",
                "craft",
                "hall",
                "house",
                "fellowship",
                "courtyard",
                "assembly",
                "kinship",
            ],
            "memory" => &[
                "memory",
                "memory",
                "remembrance",
                "song",
                "oath",
                "echo",
                "verse",
                "lament",
            ],
            "gift" => &["gift", "gift", "promise", "joy", "remembrance", "wonder"],
            "book" => &["book", "book", "verse", "parable", "witness", "memory"],
            "journey" => &[
                "journey", "journey", "path", "crossing", "horizon", "return",
            ],
            _ => return base,
        };
        options[hash(self.seed ^ key_hash(key) ^ 0x68656164) as usize % options.len()]
    }
    /// An attacker's commemorative label, not a claim that all sides use this name.
    pub fn war(&mut self, id: u32, target: Source, origin: Source, leader: Source) -> String {
        let key = format!("war:{id}");
        let q = hash(self.seed ^ key_hash(&key) ^ 0x776172);
        let (meanings, source): (&[&str], Source) = match q % 6 {
            0 | 1 => (&["war"], target),
            2 => (&["claim", "war"], target),
            3 => (&["frontier", "campaign"], target),
            4 => (&["campaign"], origin),
            _ => (&["claim", "campaign"], leader),
        };
        self.coin(&key, meanings, Some(source))
    }
    pub fn coin(&mut self, key: &str, meanings: &[&str], source: Option<Source>) -> String {
        if let Some(record) = self.names.get(key) {
            return record.name.clone();
        }
        // Extend old vocabularies without replacing archived roots or old labels.
        for &(meaning, root) in ROOTS {
            self.roots
                .entry(meaning.into())
                .or_insert_with(|| root.into());
        }
        let q = hash(self.seed ^ key_hash(key));
        let meanings: Vec<&str> = meanings
            .iter()
            .map(|m| {
                if key.starts_with("person:") || key.starts_with("crew:") {
                    *m
                } else {
                    self.descriptor(key, m)
                }
            })
            .collect();
        let mut chosen: Vec<WordUse> = meanings
            .iter()
            .enumerate()
            .map(|(slot, m)| WordUse {
                concept: (*m).into(),
                word: self.lexical_choice(m, key, slot),
            })
            .collect();
        let mut words: Vec<String> = chosen.iter().map(|w| w.word.form.clone()).collect();
        if let Some(s) = &source {
            // Proper names retain their existing sound, rather than undergoing the shifts twice.
            let stem = self.source_stem(&s.name, q);
            words.insert(
                (q as usize / 11) % (words.len() + 1),
                stem.chars().take(12).collect::<String>().to_lowercase(),
            );
        }
        // Each language favors an order, but permits reversals and rotations.
        if q % 5 == 0 {
            words.reverse();
        } else if q % 5 == 1 && words.len() > 1 {
            words.rotate_left(1);
        }
        let mut form = if q % 4 == 0 {
            (q / 4 % 5) as u8
        } else {
            self.preferred_form
        };
        if form == 1 && words.iter().map(|s| s.chars().count()).sum::<usize>() > 20 {
            form = 0;
        }
        let text = match form {
            0 => words.iter().map(|s| title(s)).collect::<Vec<_>>().join(" "),
            1 => title(&words.join("")),
            3 => words
                .iter()
                .map(|w| title(&w.chars().take(4).collect::<String>()))
                .collect::<Vec<_>>()
                .join(" "),
            4 => words
                .iter()
                .map(|w| title(w))
                .collect::<Vec<_>>()
                .join(&format!(" {} ", self.word("of"))),
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
        let mut gloss: Vec<String> = meanings.iter().map(|s| (*s).into()).collect();
        let mut attempt = 0u32;
        let mut byname_words = vec![];
        while taken(&name) {
            let salt = hash(q.wrapping_add(attempt));
            if attempt < 16 {
                let a = EMBLEMS[salt as usize % EMBLEMS.len()];
                let b = VIRTUES[hash(salt) as usize % VIRTUES.len()];
                let left = self.lexical_choice(a, key, 100 + attempt as usize * 2);
                let right = self.lexical_choice(b, key, 101 + attempt as usize * 2);
                name = format!("{text} {}", title(&format!("{}{}", left.form, right.form)));
                byname_words = vec![
                    WordUse {
                        concept: a.into(),
                        word: left,
                    },
                    WordUse {
                        concept: b.into(),
                        word: right,
                    },
                ];
                gloss = meanings
                    .iter()
                    .map(|s| (*s).into())
                    .chain([a.into(), b.into()])
                    .collect();
            } else {
                byname_words.clear();
                // A phonotactic family byname, not an exposed entity number.
                let mut value = salt;
                let syllables = [
                    "ba", "de", "fi", "go", "hu", "ka", "le", "mi", "no", "pu", "ra", "se", "ti",
                    "vo", "wa", "zu",
                ];
                let mut byname = String::new();
                for _ in 0..8 {
                    byname.push_str(syllables[(value & 15) as usize]);
                    value >>= 4;
                }
                name = format!("{text} {}", title(&self.evolve(&byname)));
                gloss = meanings
                    .iter()
                    .map(|s| (*s).into())
                    .chain(["family byname (untranslated)".into()])
                    .collect();
            }
            attempt = attempt.wrapping_add(1);
        }
        chosen.extend(byname_words);
        self.used.insert((category.into(), name.clone()));
        self.names.insert(
            key.into(),
            NameRecord {
                words: chosen,
                name: name.clone(),
                meanings: gloss,
                source,
                form: [
                    "separate words",
                    "compound",
                    "blend",
                    "clipped words",
                    "linked words",
                ][form as usize]
                    .into(),
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
    fn broad_patron_draw_is_traceable_and_preserves_archived_names() {
        let habitats = ["lake", "wetland", "forest", "volcanic", "coast"];
        let mut concepts = BTreeSet::new();
        let mut styles = BTreeSet::new();
        for seed in [17, 81, 256, 409, 1024] {
            let mut l = Language::new(seed, 0);
            assert_eq!(l.roots.len(), ROOTS.len(), "root concepts must be unique");
            for concept in VIRTUES.iter().chain(EMBLEMS) {
                assert!(l.roots.contains_key(*concept), "missing {concept}");
            }
            for id in 0..200 {
                let name = l.patron(id, habitats[id as usize % habitats.len()], id % 3 == 0);
                assert!(!name.chars().any(|c| c.is_ascii_digit()));
                assert!(name.len() < 100);
                let record = &l.names[&format!("patron:{id}")];
                assert!(record.meanings.iter().all(|m| l.roots.contains_key(m)));
                concepts.extend(record.meanings.iter().cloned());
                styles.insert(record.form.split(';').next().unwrap().to_string());
                if id < 3 {
                    eprintln!("seed {seed}: {name} — {}", record.meanings.join(" + "));
                }
                assert_eq!(
                    l.patron(id, "forest", false),
                    name,
                    "saved labels never change"
                );
            }
            let mut restored: Language =
                serde_json::from_str(&serde_json::to_string(&l).unwrap()).unwrap();
            for id in 200..220 {
                assert_eq!(
                    l.patron(id, "coast", true),
                    restored.patron(id, "coast", true)
                );
            }
            let legacy = l.coin("patron:999", &["guide", "journey"], None);
            l.roots.remove("lullaby");
            assert_eq!(l.patron(999, "volcanic", false), legacy);
            l.patron(1000, "forest", false);
            assert!(
                l.roots.contains_key("lullaby"),
                "new coinages extend old vocabularies"
            );
        }
        assert_eq!(styles.len(), 5);
        assert!(
            concepts.len() > 120,
            "only {} concepts reached",
            concepts.len()
        );
        eprintln!(
            "patron draw: {} concepts, {} conventions; {} roots",
            concepts.len(),
            styles.len(),
            ROOTS.len()
        );
    }
    #[test]
    fn wars_keep_context_and_names_across_vocabulary_changes() {
        let site = |id, name: &str| Source {
            kind: "site".into(),
            id,
            name: name.into(),
        };
        let leader = Source {
            kind: "person".into(),
            id: 5,
            name: "Morina".into(),
        };
        let mut forms = BTreeSet::new();
        for seed in 0..64 {
            let mut l = Language::new(seed, 0);
            let name = l.war(0, site(1, "Avelara"), site(0, "Talora"), leader.clone());
            let record = &l.names["war:0"];
            assert!(record
                .meanings
                .iter()
                .any(|m| m == "war" || m == "campaign"));
            assert!(record.source.is_some());
            assert!(!name.chars().any(|c| c.is_ascii_digit()));
            forms.insert(name.clone());
            l.roots.insert("war".into(), "changed".into());
            assert_eq!(
                name,
                l.war(0, site(1, "Renamed"), site(0, "Elsewhere"), leader.clone())
            );
        }
        assert!(forms.len() > 20);
    }
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
        assert_eq!(
            l.names["institution:3"].meanings,
            [l.descriptor("institution:3", "market")]
        );
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
    fn references_skip_particles_and_keep_proper_name_sounds() {
        let l = Language::new(81, 2);
        for q in 0..100 {
            assert_eq!(
                l.source_stem(&format!("of the Avela {}", l.word("of")), q),
                "avela"
            );
        }
    }
    #[test]
    fn weighted_context_names_are_varied_traceable_and_nonnumeric() {
        let mut l = Language::new(256, 1);
        assert!(ROOTS.len() >= 180);
        assert_eq!(l.roots.len(), ROOTS.len(), "duplicate vocabulary meaning");
        let context = PersonalContext {
            concepts: vec!["smith".into(), "metal".into(), "weaver".into()],
            references: ["person", "site", "tradition", "patron", "institution"]
                .into_iter()
                .enumerate()
                .map(|(i, kind)| {
                    (
                        Source {
                            kind: kind.into(),
                            id: i as u32,
                            name: [
                                "Avela Morin",
                                "Reed Haven",
                                "Returning Lantern",
                                "Sena Guide",
                                "Clay Circle",
                            ][i]
                                .into(),
                        },
                        2,
                    )
                })
                .collect(),
        };
        let mut styles = BTreeMap::<String, usize>::new();
        let mut sources = BTreeSet::new();
        let mut forms = BTreeSet::new();
        for id in 0..10000 {
            let n = l.person_with("person", id, &context);
            assert!(!n.chars().any(|c| c.is_ascii_digit()));
            assert!(n.len() < 100, "unbounded name: {n}");
            let r = &l.names[&format!("person:{id}")];
            let (style, form) = r.form.split_once("; ").unwrap();
            *styles.entry(style.into()).or_default() += 1;
            forms.insert(form.to_string());
            if let Some(s) = &r.source {
                sources.insert(s.kind.clone());
            }
            if id < 16 {
                eprintln!("{n} — {} ({})", r.meanings.join(" + "), r.form);
            }
        }
        assert_eq!(sources.len(), 5);
        assert_eq!(forms.len(), 5);
        assert!((800..1200).contains(&styles["celebratory"]));
        assert!((1700..2300).contains(&styles["aspirational"]));
        assert!((2500..3500).contains(&styles["commemorative"]));
        assert!((1500..2500).contains(&styles["local livelihood"]));
        assert_eq!(
            l.names
                .values()
                .map(|r| &r.name)
                .collect::<BTreeSet<_>>()
                .len(),
            10000
        );
        eprintln!("personal conventions: {styles:?}; roots: {}", ROOTS.len());
        let mut resumed: Language =
            serde_json::from_str(&serde_json::to_string(&l).unwrap()).unwrap();
        for id in 10000..10100 {
            assert_eq!(
                l.person_with("person", id, &context),
                resumed.person_with("person", id, &context)
            );
        }
        // Stress repeated base names; no identity number may leak into a label.
        for id in 0..3000 {
            let n = l.coin(&format!("fixture:{id}"), &["bright"], None);
            assert!(!n.chars().any(|c| c.is_ascii_digit()));
        }
        assert_eq!(
            l.names
                .iter()
                .filter(|(k, _)| k.starts_with("fixture:"))
                .map(|(_, r)| &r.name)
                .collect::<BTreeSet<_>>()
                .len(),
            3000
        );
        // Force phonetic collapse independently of vocabulary size, so this still
        // exercises the fallback when the available byname pool expands.
        let mut collapsed = Language::new(17, 0);
        for root in collapsed.roots.values_mut() {
            *root = "a".into();
        }
        for id in 0..32 {
            let n = collapsed.coin(&format!("fixture:{id}"), &["bright"], None);
            assert!(!n.chars().any(|c| c.is_ascii_digit()));
        }
        assert!(collapsed
            .names
            .values()
            .any(|r| r.meanings.iter().any(|m| m.contains("untranslated"))));
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
                    .iter()
                    .any(|(key, r)| key.starts_with("person:") && r.source.is_some()));
                for (key, record) in &l.names {
                    if key.starts_with("person:") {
                        assert!(!record.name.chars().any(|c| c.is_ascii_digit()));
                        if let Some(source) = &record.source {
                            let exists = match source.kind.as_str() {
                                "site" => h
                                    .sites
                                    .iter()
                                    .any(|s| s.id == source.id && s.name == source.name),
                                "person" => h
                                    .people
                                    .iter()
                                    .any(|p| p.id == source.id && p.name == source.name),
                                "tradition" => h
                                    .culture
                                    .as_ref()
                                    .unwrap()
                                    .traditions
                                    .iter()
                                    .any(|t| t.id == source.id),
                                "patron" => h
                                    .culture
                                    .as_ref()
                                    .unwrap()
                                    .patrons
                                    .iter()
                                    .any(|p| p.id == source.id),
                                "institution" => h
                                    .culture
                                    .as_ref()
                                    .unwrap()
                                    .institutions
                                    .iter()
                                    .any(|n| n.id == source.id),
                                _ => false,
                            };
                            assert!(
                                exists,
                                "unresolved name source {}:{}",
                                source.kind, source.id
                            );
                        }
                    }
                }
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
                    "seed {seed} · {} · {} · {} records · {} evolved concepts: {}",
                    c.name,
                    l.name,
                    l.names.len(),
                    l.lexicon.len(),
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
