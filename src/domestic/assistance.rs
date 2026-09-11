//! Local voluntary assistance after family care, before other monthly work.
use super::*;

impl History {
    pub(super) fn match_neighbor_care(&self, domestic: &Domestic, rows: &mut [CareRow]) {
        let Some(culture) = &self.culture else {
            return;
        };
        let parents: BTreeMap<_, _> = self
            .politics
            .as_ref()
            .map(|p| p.kin.iter().map(|k| (k.person, k.parents)).collect())
            .unwrap_or_default();
        // Only households with no dependent-care demand supply volunteers in v1.
        // Each helper offers to one household, retaining the unique-carer contract.
        let occupied: BTreeSet<_> = rows.iter().map(|r| r.unit).collect();
        let mut helpers: Vec<_> = domestic
            .units
            .iter()
            .filter(|u| u.ended.is_none() && !occupied.contains(&u.id))
            .flat_map(|u| u.members.iter().map(move |p| (*p, u.home)))
            .collect();
        helpers.sort_unstable();
        let mut remaining: BTreeMap<_, _> = self
            .sites
            .iter()
            .map(|s| {
                let family: f64 = rows
                    .iter()
                    .filter(|r| r.site == s.id)
                    .map(|r| r.granted)
                    .sum();
                (
                    s.id,
                    (crate::labor::available(s, self.society.is_some(), self.living.is_some())
                        as f64
                        - family)
                        .max(0.),
                )
            })
            .collect();
        for (person, site) in helpers {
            let cap = capacity(self, person, site);
            if cap <= 0. {
                continue;
            }
            let Some(agent) = culture
                .agents
                .get(person as usize)
                .filter(|a| a.person == person)
            else {
                continue;
            };
            let generosity = agent.traits[1].clamp(0., 1.);
            let mut best = None;
            for (index, row) in rows.iter().enumerate() {
                if row.site != site || row.need - row.granted <= 1e-6 {
                    continue;
                }
                let affinity = domestic.units[row.unit as usize]
                    .members
                    .iter()
                    .filter(|p| self.person_presence(**p).1 == Presence::Resident(site))
                    .map(|p| {
                        support_affinity(
                            person,
                            *p,
                            agent.relations.get(p).copied(),
                            &parents,
                            domestic.kin_help,
                        )
                    })
                    .fold(0_f32, f32::max)
                    .clamp(0., 1.);
                if affinity <= 0.2 || generosity <= 0. {
                    continue;
                }
                let score = affinity as f64 * (row.need - row.granted) / row.need.max(1e-12);
                if best.is_none_or(|(_, old_score, old_unit, _)| {
                    score > old_score || (score == old_score && row.unit < old_unit)
                }) {
                    best = Some((index, score, row.unit, affinity));
                }
            }
            let Some((index, _, _, affinity)) = best else {
                continue;
            };
            let row = &mut rows[index];
            let budget = remaining.get_mut(&site).unwrap();
            let offered = (0.1 * generosity * affinity).min(cap) as f64;
            let work = offered.min(*budget).min((row.need - row.granted).max(0.)) as f32;
            if work <= 1e-6 {
                continue;
            }
            row.carers.push((person, work));
            row.granted = row.carers.iter().map(|(_, w)| *w as f64).sum();
            *budget = (*budget - work as f64).max(0.);
        }
    }
}

/// Known ancestry supplies a potential obligation; expressed hostility can defeat it.
/// Missing parents never count as a shared ancestor. These are game behavior weights.
fn support_affinity(
    helper: u32,
    recipient: u32,
    relationship: Option<f32>,
    parents: &BTreeMap<u32, [Option<u32>; 2]>,
    kin_enabled: bool,
) -> f32 {
    let a = parents.get(&helper).copied().unwrap_or([None; 2]);
    let b = parents.get(&recipient).copied().unwrap_or([None; 2]);
    let kin = if !kin_enabled || helper == recipient {
        0.
    } else if a.contains(&Some(recipient)) || b.contains(&Some(helper)) {
        0.75
    } else if a.iter().flatten().any(|p| b.contains(&Some(*p))) {
        0.5
    } else {
        0.
    };
    let relation = relationship.unwrap_or(0.).clamp(-1., 1.);
    relation.max(kin * (1. + relation.min(0.))).clamp(0., 1.)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn ancestry_support_requires_known_kin_and_respects_estrangement() {
        let parents = BTreeMap::from([(2, [Some(1), None]), (3, [Some(1), None])]);
        assert_eq!(support_affinity(2, 1, None, &parents, true), 0.75);
        assert_eq!(support_affinity(1, 2, None, &parents, true), 0.75);
        assert_eq!(support_affinity(2, 3, None, &parents, true), 0.5);
        assert_eq!(support_affinity(4, 5, None, &parents, true), 0.);
        assert_eq!(support_affinity(2, 1, Some(-1.), &parents, true), 0.);
        assert_eq!(support_affinity(2, 1, None, &parents, false), 0.);
        assert_eq!(support_affinity(2, 1, Some(0.9), &parents, true), 0.9);
    }
}
