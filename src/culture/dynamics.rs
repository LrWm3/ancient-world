//! Bounded social evidence, not universal historical conversion probabilities.
//! Decisions read one affiliation snapshot; commits cannot propagate within a year.
use super::*;

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct ReligiousDynamics {
    pub observed: Option<u32>,
    pub persuasion: Vec<Persuasion>,
    pub reforms: Vec<Reform>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Persuasion {
    pub household: u32,
    pub tradition: u32,
    pub strength: f32,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Reform {
    pub site: u32,
    pub tradition: u32,
    pub theme: u32,
    pub years: u32,
}
impl ReligiousDynamics {
    pub fn validate(&self, h: &History, traditions: usize) -> Result<()> {
        ensure!(
            self.observed.is_none_or(|m| m <= h.month),
            "invalid religious observation clock"
        );
        ensure!(
            self.observed.is_some() || (self.persuasion.is_empty() && self.reforms.is_empty()),
            "religious evidence without an observation boundary"
        );
        let households = h.society.as_ref().map_or(0, |s| s.households.len());
        let mut keys = BTreeSet::new();
        for p in &self.persuasion {
            ensure!(
                (p.household as usize) < households
                    && (p.tradition as usize) < traditions
                    && p.strength.is_finite()
                    && (0. ..=1.).contains(&p.strength)
                    && keys.insert((p.household, p.tradition)),
                "invalid religious persuasion"
            );
        }
        let mut keys = BTreeSet::new();
        for r in &self.reforms {
            ensure!(
                (r.site as usize) < h.sites.len()
                    && (r.tradition as usize) < traditions
                    && r.theme < 8
                    && (1..=100).contains(&r.years)
                    && keys.insert((r.site, r.tradition)),
                "invalid religious reform"
            );
        }
        Ok(())
    }
}
// Benefits accumulate only while net social advantage persists. Forgotten reports
// decay; tolerance is stable per household, not a new conversion lottery each year.
fn persuasion_step(old: f32, advantage: f32) -> f32 {
    (old * 0.75 + advantage.max(0.) * 0.35)
        .min(old + 0.25)
        .clamp(0., 1.)
}
fn similarity(a: &[u32; 4], b: &[u32; 4]) -> f32 {
    a.iter().filter(|t| b.contains(t)).count() as f32 / 4.
}
fn preferences(traits: [f32; 6], pressure: [f32; 4]) -> [f32; 8] {
    let [ambition, generosity, piety, curiosity, loyalty, caution] = traits;
    let [hunger, disease, disruption, inequality] = pressure;
    [
        generosity + 0.8 * hunger,
        caution + 0.5 * hunger,
        ambition + 0.4 * disruption,
        caution + 0.4 * disease,
        curiosity + 0.5 * disease,
        generosity + 0.8 * inequality,
        piety + 0.3 * loyalty,
        ambition + 0.6 * disruption,
    ]
}
fn fit(themes: &[u32; 4], values: &[f32; 8]) -> f32 {
    themes.iter().map(|&t| values[t as usize]).sum::<f32>() / 4.
}
#[derive(Clone)]
struct Resident {
    id: u32,
    site: u32,
    head: u32,
    parent: Option<u32>,
    faith: u32,
    traits: [f32; 6],
    values: [f32; 8],
}
type Adoption = (f32, u32, u32, f32, f32, f32, f32, f32, Option<u64>);

impl Culture {
    fn religious_pressure(&self, h: &History, site: u32) -> [f32; 4] {
        h.society
            .as_ref()
            .and_then(|s| s.indicators.as_ref())
            .and_then(|s| s.sites.get(site as usize))
            .map(|s| {
                [
                    s.pressure[0].max(s.household_stress[2]),
                    s.pressure[1],
                    s.pressure[2].max(s.housing[2]),
                    s.pressure[3],
                ]
            })
            .unwrap_or([
                h.sites[site as usize].stocks.stock[3].clamp(0., 1.),
                0.,
                0.,
                0.,
            ])
    }
    fn religious_service(&self, site: u32, faith: u32) -> f32 {
        self.institutions
            .iter()
            .filter(|n| {
                n.site == site
                    && n.tradition == Some(faith)
                    && n.kind == InstitutionKind::Religious
                    && n.operational()
            })
            .map(|n| n.capacity.as_ref().map_or(0.5, |c| c.readiness))
            .fold(0., f32::max)
    }
    pub(super) fn advance_religious_dynamics(&mut self, h: &mut History, routes: &[(u32, u32)]) {
        if self
            .religious_dynamics
            .observed
            .is_some_and(|m| h.month < m.saturating_add(12))
        {
            return;
        }
        self.religious_dynamics.observed = Some(h.month);
        let Some(society) = &h.society else { return };
        let mut residents: Vec<_> = society
            .households
            .iter()
            .filter(|hh| {
                !society.relocation.away(hh.id)
                    && !h.sites[hh.site as usize].abandoned
                    && h.people[hh.head as usize].died.is_none()
            })
            .map(|hh| {
                let traits = self.agents[hh.head as usize].traits;
                let mut pressure = self.religious_pressure(h, hh.site);
                if let Some(account) = society
                    .household_economy
                    .as_ref()
                    .and_then(|e| e.accounts.get(hh.id as usize))
                    .filter(|a| a.food_site == Some(hh.site))
                {
                    pressure[0] = pressure[0].max(account.hunger as f32).clamp(0., 1.);
                }
                Resident {
                    id: hh.id,
                    site: hh.site,
                    head: hh.head,
                    parent: hh.parent,
                    faith: self.household_faith[hh.id as usize],
                    traits,
                    values: preferences(traits, pressure),
                }
            })
            .collect();
        residents.sort_by_key(|r| r.id);
        let links: BTreeSet<_> = routes.iter().copied().collect();
        let old: BTreeMap<_, _> = self
            .religious_dynamics
            .persuasion
            .iter()
            .map(|p| ((p.household, p.tradition), p.strength))
            .collect();
        let mut next = Vec::new();
        let mut conversions = Vec::new();
        for hh in &residents {
            let own = &self.traditions[hh.faith as usize];
            let mut signals = BTreeMap::<u32, (f32, f32, f32, u32)>::new();
            let mut total = 0.;
            for other in &residents {
                if hh.id == other.id {
                    continue;
                }
                let local = hh.site == other.site;
                if !local && !links.contains(&(other.site, hh.site)) {
                    continue;
                }
                let kin = hh.parent == Some(other.id)
                    || other.parent == Some(hh.id)
                    || (hh.parent.is_some() && hh.parent == other.parent);
                let relation = self.agents[hh.head as usize]
                    .relations
                    .get(&other.head)
                    .copied()
                    .unwrap_or(0.)
                    .clamp(-1., 1.);
                let distance = if local {
                    1.
                } else {
                    let km = society
                        .routes
                        .iter()
                        .filter(|r| {
                            r.passable()
                                && ((r.from == other.site && r.to == hh.site)
                                    || (r.to == other.site && r.from == hh.site))
                        })
                        .map(|r| r.cost_km)
                        .reduce(f32::min)
                        .unwrap_or(1500.);
                    0.35 / (1. + km / 500.)
                };
                let weight = distance * (1. + 2. * f32::from(kin) + relation).max(0.);
                let entry = signals
                    .entry(other.faith)
                    .or_insert((0., 0., 0., other.head));
                entry.0 += weight;
                entry.1 = entry.1.max(f32::from(kin));
                entry.2 = entry.2.max(relation);
                if other.head < entry.3 {
                    entry.3 = other.head;
                }
                total += weight;
            }
            let own_support = signals.get(&hh.faith).map_or(0., |s| s.0) / total.max(1e-6);
            let mut best: Option<Adoption> = None;
            for (&faith, &(support, kin, trust, witness)) in &signals {
                if faith == hh.faith || support <= 0. {
                    continue;
                }
                let target = &self.traditions[faith as usize];
                let prevalence = support / total.max(1e-6);
                let service = self.religious_service(hh.site, faith)
                    - self.religious_service(hh.site, hh.faith);
                // Only completed, recent, locally received relief is evidence of service.
                let relief = self
                    .religious_relief
                    .missions
                    .iter()
                    .filter(|m| {
                        m.recipient == hh.site
                            && m.delivered_kg.is_some_and(|kg| kg > 0.)
                            && m.outcome_event
                                .and_then(|id| h.events.get(id as usize))
                                .is_some_and(|e| e.month > h.month.saturating_sub(36))
                    })
                    .filter(|m| self.institutions[m.institution as usize].tradition == Some(faith))
                    .max_by_key(|m| m.outcome_event);
                let compatibility = similarity(&own.themes, &target.themes);
                let heritage =
                    crate::heritage_renown::score(self, hh.site, h.month, |r| r.tradition == faith)
                        - crate::heritage_renown::score(self, hh.site, h.month, |r| {
                            r.tradition == hh.faith
                        });
                let advantage = 0.15 * heritage
                    + 0.55 * (prevalence - own_support)
                    + 0.25 * kin
                    + 0.25 * trust
                    + 0.25 * service
                    + 0.3 * (fit(&target.themes, &hh.values) - fit(&own.themes, &hh.values))
                    + 0.18 * f32::from(relief.is_some())
                    - 0.15 * (1. - compatibility)
                    - 0.15 * hh.traits[4];
                let strength = persuasion_step(*old.get(&(hh.id, faith)).unwrap_or(&0.), advantage);
                if strength > 0.001 {
                    next.push(Persuasion {
                        household: hh.id,
                        tradition: faith,
                        strength,
                    });
                }
                let threshold = 0.28 + 0.12 * hh.traits[2] + 0.10 * unit(h.seed, hh.id, 0, 2401);
                if strength > threshold
                    && advantage > 0.
                    && best.as_ref().is_none_or(|(score, ..)| strength > *score)
                {
                    best = Some((
                        strength,
                        faith,
                        witness,
                        prevalence,
                        kin,
                        trust,
                        service,
                        compatibility,
                        relief.and_then(|m| m.outcome_event),
                    ));
                }
            }
            // Disconnected memories decay too, but cannot trigger an adoption.
            for (&(id, faith), &strength) in old.range((hh.id, 0)..=(hh.id, u32::MAX)) {
                if faith != hh.faith && !signals.contains_key(&faith) && strength * 0.75 > 0.001 {
                    next.push(Persuasion {
                        household: id,
                        tradition: faith,
                        strength: strength * 0.75,
                    });
                }
            }
            if let Some(result) = best {
                conversions.push((hh.clone(), result));
            }
        }
        // Reforms also use pre-conversion affiliation, and may not recruit converts
        // committed in this same annual boundary.
        let converted: BTreeSet<_> = conversions.iter().map(|(hh, _)| hh.id).collect();
        self.religious_dynamics.persuasion = next
            .into_iter()
            .filter(|p| !converted.contains(&p.household))
            .collect();
        for (
            hh,
            (strength, faith, witness, prevalence, kin, trust, service, compatibility, cause),
        ) in conversions
        {
            self.household_faith[hh.id as usize] = faith;
            let ev=self.log(h,"faith_adopted",hh.site,Some(hh.head),Some(faith),None,cause,
                format!("Household {} adopted after sustained social evidence: conviction {:.2}, contact share {:.2}, kin {:.0}, relationship {:.2}, local service advantage {:.2}, doctrine overlap {:.2}; prior tradition {}",hh.id,strength,prevalence,kin,trust,service,compatibility,hh.faith));
            h.events[ev as usize].subjects.extend([
                ("household".into(), hh.id),
                ("person".into(), witness),
                ("tradition".into(), hh.faith),
            ]);
        }
        self.religious_reforms(h, &residents, &links, &converted);
    }
    fn religious_reforms(
        &mut self,
        h: &mut History,
        residents: &[Resident],
        links: &BTreeSet<(u32, u32)>,
        converted: &BTreeSet<u32>,
    ) {
        let prior: BTreeMap<_, _> = self
            .religious_dynamics
            .reforms
            .iter()
            .map(|r| ((r.site, r.tradition), (r.theme, r.years)))
            .collect();
        let mut next = Vec::new();
        let groups: BTreeSet<_> = residents.iter().map(|hh| (hh.site, hh.faith)).collect();
        let mut groups: Vec<_> = groups.into_iter().collect();
        // When several congregations can reform the same parent, prioritize unmet
        // local pressure rather than whichever settlement occupies the first slot.
        groups.sort_by(|a, b| {
            let urgency = |(site, faith)| {
                self.religious_pressure(h, site)
                    .into_iter()
                    .fold(0_f32, f32::max)
                    * (1. - 0.65 * self.religious_service(site, faith))
            };
            urgency(*b).total_cmp(&urgency(*a)).then_with(|| a.cmp(b))
        });
        let mut changed = BTreeSet::new();
        let mut dissent = vec![0_f32; self.traditions.len()];
        for (site, faith) in groups {
            if changed.contains(&faith) {
                continue;
            }
            let congregation: Vec<_> = residents
                .iter()
                .filter(|hh| hh.site == site && hh.faith == faith && !converted.contains(&hh.id))
                .collect();
            if congregation.len() < 3 {
                continue;
            }
            let tradition = self.traditions[faith as usize].clone();
            let pressure = self.religious_pressure(h, site);
            let hardship = pressure.into_iter().fold(0_f32, f32::max);
            let service = self.religious_service(site, faith);
            // Propose a concrete absent value and replace the least supported existing
            // value; neither catalogue order nor arithmetic on IDs selects doctrine.
            let mean: Vec<_> = (0..8)
                .map(|t| {
                    congregation.iter().map(|hh| hh.values[t]).sum::<f32>()
                        / congregation.len() as f32
                })
                .collect();
            let Some(theme) = (0..8u32)
                .filter(|t| !tradition.themes.contains(t))
                .max_by(|a, b| {
                    mean[*a as usize]
                        .total_cmp(&mean[*b as usize])
                        .then_with(|| b.cmp(a))
                })
            else {
                continue;
            };
            let slot = (0..4)
                .min_by(|&a, &b| {
                    mean[tradition.themes[a] as usize]
                        .total_cmp(&mean[tradition.themes[b] as usize])
                })
                .unwrap();
            let old_theme = tradition.themes[slot];
            let supporters: Vec<_> = congregation
                .iter()
                .copied()
                .filter(|hh| hh.values[theme as usize] - hh.values[old_theme as usize] > 0.35)
                .collect();
            let support = supporters.len() as f32 / congregation.len() as f32;
            let grievance = hardship * (1. - 0.65 * service) * support;
            dissent[faith as usize] = dissent[faith as usize].max(grievance);
            if supporters.len() < 3 || support < 0.4 {
                continue;
            }
            // A leader needs actual local adherents and positive relationship support.
            let leader = *supporters
                .iter()
                .max_by(|a, b| {
                    let score = |r: &Resident| {
                        r.traits[2] * 0.3
                            + r.traits[0] * 0.2
                            + r.traits[3] * 0.2
                            + supporters
                                .iter()
                                .map(|hh| {
                                    self.agents[hh.head as usize]
                                        .relations
                                        .get(&r.head)
                                        .copied()
                                        .unwrap_or(0.)
                                        .max(0.)
                                })
                                .sum::<f32>()
                                / supporters.len() as f32
                                * 0.3
                    };
                    score(a)
                        .total_cmp(&score(b))
                        .then_with(|| b.head.cmp(&a.head))
                })
                .unwrap();
            let sources: Vec<_> = residents
                .iter()
                .filter(|r| {
                    r.faith != faith
                        && self.traditions[r.faith as usize].themes.contains(&theme)
                        && (r.site == site || links.contains(&(r.site, site)))
                })
                .collect();
            let institution = self
                .institutions
                .iter()
                .filter(|n| {
                    n.site == site
                        && n.tradition == Some(faith)
                        && n.kind == InstitutionKind::Religious
                        && n.operational()
                        && supporters.iter().any(|r| r.head == n.leader)
                })
                .min_by_key(|n| n.id)
                .map(|n| (n.id, n.leader));
            let broad_support = supporters.len() as f32
                / residents.iter().filter(|r| r.faith == faith).count().max(1) as f32;
            let syncretic = !sources.is_empty()
                && institution.is_some()
                && broad_support >= 0.5
                && sources.iter().any(|r| {
                    similarity(&tradition.themes, &self.traditions[r.faith as usize].themes) >= 0.5
                });
            let split = grievance > 0.25 && leader.traits[2] > 0.4 && service < 0.75;
            if !syncretic && !split {
                continue;
            }
            let years = prior
                .get(&(site, faith))
                .filter(|(t, _)| *t == theme)
                .map_or(1, |(_, y)| (y + 1).min(100));
            next.push(Reform {
                site,
                tradition: faith,
                theme,
                years,
            });
            if years < 5 {
                continue;
            }
            if syncretic {
                let (institution, interpreter) = institution.unwrap();
                let source = sources
                    .iter()
                    .filter(|r| {
                        similarity(&tradition.themes, &self.traditions[r.faith as usize].themes)
                            >= 0.5
                    })
                    .min_by_key(|r| r.id)
                    .unwrap();
                self.traditions[faith as usize].themes[slot] = theme;
                let ev=self.log(h,"religious_syncretism",site,Some(interpreter),Some(faith),None,None,
                    format!("After {years} annual observations, {:.0}% of this tradition's resident households supported {} replacing {}; active congregation {} interpreted a currently contacted practice of tradition {}",broad_support*100.,THEMES[theme as usize],THEMES[old_theme as usize],institution,source.faith));
                h.events[ev as usize].subjects.extend([
                    ("institution".into(), institution),
                    ("person".into(), source.head),
                    ("tradition".into(), source.faith),
                ]);
                self.account(
                    h,
                    faith,
                    Some(interpreter),
                    vec![ev],
                    format!(
                        "Our neighbors' {} accords with our present obligations.",
                        THEMES[theme as usize]
                    ),
                );
            } else if self.traditions.len() < 256
                && h.month.saturating_sub(tradition.founded) >= 120
            {
                let id = self.traditions.len() as u32;
                let mut child = tradition.clone();
                child.id = id;
                child.parent = Some(faith);
                child.founded = h.month;
                child.name = h.civilizations[h.sites[site as usize].civilization as usize]
                    .naming(h.seed)
                    .coin(
                        &format!("tradition:{id}"),
                        &["covenant"],
                        Some(crate::naming::Source {
                            kind: "person".into(),
                            id: leader.head,
                            name: h.people[leader.head as usize].name.clone(),
                        }),
                    );
                child.leader = leader.head;
                child.sacred_site = site;
                child.themes[slot] = theme;
                child.dissent = 0.;
                self.traditions.push(child);
                for hh in &supporters {
                    self.household_faith[hh.id as usize] = id;
                }
                let cause = h
                    .events
                    .iter()
                    .rev()
                    .find(|e| {
                        e.site == Some(site)
                            && e.month > h.month.saturating_sub(60)
                            && matches!(
                                e.kind.as_str(),
                                "food_crisis"
                                    | "social_crisis"
                                    | "institution_impaired"
                                    | "household_food_crisis"
                            )
                    })
                    .map(|e| e.id);
                let ev=self.log(h,"religious_schism",site,Some(leader.head),Some(id),None,cause,
                    format!("{} of {} resident households supported {} over {} for {years} years: hardship {:.2}, institutional readiness {:.2}, unmet grievance {:.2}; non-supporters retain tradition {faith}",supporters.len(),congregation.len(),THEMES[theme as usize],THEMES[old_theme as usize],hardship,service,grievance));
                h.events[ev as usize]
                    .subjects
                    .push(("tradition".into(), faith));
                self.account(
                    h,
                    id,
                    Some(leader.head),
                    vec![ev],
                    format!(
                        "We interpret our shared patron's charge as a duty of {}.",
                        THEMES[theme as usize]
                    ),
                );
            } else {
                continue;
            }
            changed.insert(faith);
        }
        self.religious_dynamics.reforms = next
            .into_iter()
            .filter(|r| !changed.contains(&r.tradition))
            .collect();
        for (t, value) in self.traditions.iter_mut().zip(dissent) {
            t.dissent = value;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn persuasion_requires_persistent_advantage_and_decays() {
        let mut strong = 0.;
        let mut weak = 0.;
        for _ in 0..8 {
            strong = persuasion_step(strong, 0.5);
            weak = persuasion_step(weak, 0.05);
        }
        assert!(strong > 0.5 && weak < 0.1);
        assert!(persuasion_step(strong, -1.) < strong);
        assert_eq!(persuasion_step(0., 0.), 0.);
        assert_eq!(persuasion_step(0., -1.), 0.);
    }
    #[test]
    fn values_respond_to_material_concerns_and_preserve_doctrine_identity() {
        let ordinary = preferences([0.5; 6], [0.; 4]);
        let hunger = preferences([0.5; 6], [1., 0., 0., 0.]);
        assert!(hunger[0] > ordinary[0] && hunger[1] > ordinary[1]);
        assert_eq!(hunger[4], ordinary[4]);
        assert_eq!(similarity(&[0, 1, 2, 3], &[3, 2, 1, 0]), 1.);
        assert_eq!(similarity(&[0, 1, 2, 3], &[4, 5, 6, 7]), 0.);
    }
}

#[cfg(test)]
mod integration_tests {
    use super::*;
    fn world(seed: u32) -> (History, Culture) {
        let mut g = Generator::new(
            pollster::block_on(crate::gpu::ContextGpu::headless()).unwrap(),
            crate::config::Config {
                resolution: 64,
                ecology_resolution: 32,
                seed,
                ecology_years_per_epoch: 1,
                ..Default::default()
            },
            crate::catalog::Catalog::bundled().unwrap(),
        )
        .unwrap();
        g.run_epochs(1).unwrap();
        g.found_civilizations(8).unwrap();
        g.enable_society().unwrap();
        let mut h = g.civilizations.take().unwrap();
        let mut c = h.culture.take().unwrap();
        c.sync(&h);
        (h, c)
    }
    fn advance(h: &mut History, c: &mut Culture, links: &[(u32, u32)], years: u32) {
        for _ in 0..years {
            h.month += 12;
            c.advance_religious_dynamics(h, links);
        }
    }
    #[test]
    #[ignore = "requires hardware GPU"]
    fn contact_controls_conversion_and_resume_is_order_independent() {
        for seed in [17, 81, 256] {
            let (mut h, mut c) = world(seed);
            // A single remote household, surrounded by an accessible contrasting
            // community only in the open-route arm. Matched doctrine isolates contact.
            let target = h.society.as_ref().unwrap().households.last().unwrap().id;
            for hh in &mut h.society.as_mut().unwrap().households {
                hh.site = if hh.id == target { 1 } else { 0 };
            }
            c.household_faith.fill(0);
            c.household_faith[target as usize] = 1;
            c.traditions[1].themes = c.traditions[0].themes;
            // Kinship and a positive personal relationship increase the immediate
            // mediator under the same contact, before affiliations can diverge.
            let mut plain_h = h.clone();
            let mut plain = c.clone();
            let mut kin_h = h.clone();
            let mut kin = c.clone();
            let source = kin_h.society.as_ref().unwrap().households[0].clone();
            let target_head = kin_h.society.as_ref().unwrap().households[target as usize].head;
            kin_h.society.as_mut().unwrap().households[target as usize].parent = Some(source.id);
            kin.agents[target_head as usize]
                .relations
                .insert(source.head, 1.);
            advance(&mut plain_h, &mut plain, &[(0, 1), (1, 0)], 1);
            advance(&mut kin_h, &mut kin, &[(0, 1), (1, 0)], 1);
            let memory = |c: &Culture| {
                c.religious_dynamics
                    .persuasion
                    .iter()
                    .find(|p| p.household == target && p.tradition == 0)
                    .unwrap()
                    .strength
            };
            assert!(memory(&kin) > memory(&plain));
            let mut closed_h = h.clone();
            let mut closed = c.clone();
            advance(&mut closed_h, &mut closed, &[], 12);
            assert_eq!(closed.household_faith[target as usize], 1);
            let mut reordered_h = h.clone();
            reordered_h.society.as_mut().unwrap().households.reverse();
            let mut reordered = c.clone();
            let links = [(0, 1), (1, 0)];
            let stocks = serde_json::to_value(&h.sites).unwrap();
            advance(&mut h, &mut c, &links, 3);
            let mut resumed: Culture =
                serde_json::from_slice(&serde_json::to_vec(&c).unwrap()).unwrap();
            let mut resumed_h = h.clone();
            advance(&mut h, &mut c, &links, 9);
            advance(&mut resumed_h, &mut resumed, &links, 9);
            advance(&mut reordered_h, &mut reordered, &links, 12);
            assert_eq!(c.household_faith[target as usize], 0, "seed {seed}");
            assert_eq!(
                serde_json::to_value(&c).unwrap(),
                serde_json::to_value(&resumed).unwrap()
            );
            assert_eq!(
                serde_json::to_value(&c).unwrap(),
                serde_json::to_value(&reordered).unwrap()
            );
            assert_eq!(
                serde_json::to_value(&h.events).unwrap(),
                serde_json::to_value(&reordered_h.events).unwrap()
            );
            let before = serde_json::to_value(&c).unwrap();
            c.advance_religious_dynamics(&mut h, &links);
            assert_eq!(before, serde_json::to_value(&c).unwrap());
            assert_eq!(stocks, serde_json::to_value(&h.sites).unwrap());
            c.religious_dynamics
                .validate(&h, c.traditions.len())
                .unwrap();
            println!("seed {seed}: isolated household retains faith; contacted household adopts; reorder/resume match");
        }
    }
    #[test]
    #[ignore = "requires hardware GPU"]
    fn syncretism_requires_current_contact_support_and_operational_interpreter() {
        let (mut h, mut c) = world(81);
        h.month = 240;
        let congregation: Vec<_> = h
            .society
            .as_ref()
            .unwrap()
            .households
            .iter()
            .filter(|hh| hh.site == 0)
            .map(|hh| (hh.id, hh.head))
            .collect();
        assert!(congregation.len() >= 3);
        c.household_faith.fill(1);
        c.site_faith.fill(1);
        c.site_faith[0] = 0;
        c.traditions[0].themes = [2, 3, 6, 7];
        c.traditions[1].themes = [0, 3, 6, 7];
        h.society.as_mut().unwrap().indicators = None;
        for site in &mut h.sites {
            site.stocks.stock[3] = 0.;
        }
        for &(id, head) in &congregation {
            c.household_faith[id as usize] = 0;
            c.agents[head as usize].traits = [0.05, 1., 0.7, 0.1, 0.9, 0.05];
        }
        c.institutions.clear();
        c.institutions.push(Institution {
            capacity: Some(crate::institution_capacity::Capacity::new(h.month)),
            id: 0,
            name: "Supported interpreters".into(),
            kind: InstitutionKind::Religious,
            site: 0,
            tradition: Some(0),
            members: congregation.iter().map(|(_, p)| *p).collect(),
            leader: congregation[0].1,
            treasury: 0.,
            active: true,
            founded: 0,
            knowledge: Default::default(),
            property: vec![],
            dues: 0.,
            expenses: 0.,
        });
        c.institutions[0].capacity.as_mut().unwrap().readiness = 1.;
        c.contact.insert("0:1".into(), 100); // stale global contact is insufficient
        let mut closed_h = h.clone();
        let mut closed = c.clone();
        let mut impaired_h = h.clone();
        let mut impaired = c.clone();
        impaired.institutions[0].active = false;
        let count = c.traditions.len();
        advance(&mut h, &mut c, &[(1, 0), (0, 1)], 5);
        advance(&mut closed_h, &mut closed, &[], 5);
        advance(&mut impaired_h, &mut impaired, &[(1, 0), (0, 1)], 5);
        assert!(c.traditions[0].themes.contains(&0));
        assert!(!closed.traditions[0].themes.contains(&0));
        assert!(!impaired.traditions[0].themes.contains(&0));
        assert_eq!(c.traditions.len(), count);
        assert!(h
            .events
            .iter()
            .any(|e| e.kind == "religious_syncretism"
                && e.subjects.contains(&("institution".into(), 0))));
    }
    #[test]
    #[ignore = "requires hardware GPU"]
    fn supported_reforms_retain_dissenters_and_service_prevents_split() {
        let (mut h, mut c) = world(17);
        h.month = 120;
        let site = 0;
        let households: Vec<_> = h
            .society
            .as_ref()
            .unwrap()
            .households
            .iter()
            .filter(|hh| hh.site == site)
            .map(|hh| (hh.id, hh.head))
            .collect();
        assert!(households.len() >= 4);
        c.household_faith.fill(0);
        c.site_faith.fill(0);
        c.traditions[0].themes = [2, 3, 6, 7];
        h.society.as_mut().unwrap().indicators = None;
        for s in &mut h.sites {
            s.stocks.stock[3] = 0.;
        }
        h.sites[site as usize].stocks.stock[3] = 1.;
        for &(id, head) in &households {
            c.agents[head as usize].traits = [0.05, 1., 0.7, 0.1, 0.05, 0.05];
            // One household values restraint more than hospitality.
            if id == households[0].0 {
                c.agents[head as usize].traits = [1., 0., 1., 0., 1., 1.];
            }
        }
        c.institutions.clear();
        let count = c.traditions.len();
        let mut stable_h = h.clone();
        let mut stable = c.clone();
        stable.institutions.push(Institution {
            capacity: Some(crate::institution_capacity::Capacity::new(h.month)),
            id: 0,
            name: "Maintained congregation".into(),
            kind: InstitutionKind::Religious,
            site,
            tradition: Some(0),
            members: households.iter().map(|(_, p)| *p).collect(),
            leader: households[1].1,
            treasury: 0.,
            active: true,
            founded: 0,
            knowledge: Default::default(),
            property: vec![],
            dues: 0.,
            expenses: 0.,
        });
        stable.institutions[0].capacity.as_mut().unwrap().readiness = 1.;
        advance(&mut h, &mut c, &[], 4);
        assert_eq!(c.traditions.len(), count);
        advance(&mut h, &mut c, &[], 1);
        assert_eq!(c.traditions.len(), count + 1);
        advance(&mut stable_h, &mut stable, &[], 8);
        assert_eq!(stable.traditions.len(), count);
        let child = c.traditions.last().unwrap();
        assert_eq!(child.parent, Some(0));
        assert_eq!(child.patron, c.traditions[0].patron);
        assert!(child.themes.contains(&0));
        assert_eq!(c.household_faith[households[0].0 as usize], 0);
        assert!(households[1..]
            .iter()
            .all(|(id, _)| c.household_faith[*id as usize] == child.id));
        assert!(c
            .accounts
            .iter()
            .any(|a| a.tradition == child.id && a.author == Some(child.leader)));
        let mut legacy = serde_json::to_value(&c).unwrap();
        legacy.as_object_mut().unwrap().remove("religious_dynamics");
        let legacy: Culture = serde_json::from_value(legacy).unwrap();
        assert!(legacy.religious_dynamics.observed.is_none());
        let mut bad = c.religious_dynamics.clone();
        bad.persuasion.push(Persuasion {
            household: u32::MAX,
            tradition: 0,
            strength: 1.,
        });
        assert!(bad.validate(&h, c.traditions.len()).is_err());
    }
}
