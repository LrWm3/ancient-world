//! Shared game weights for expressed relationships and known family ties.
use std::collections::BTreeMap;

/// Known ancestry supplies a potential obligation; expressed hostility can defeat it.
/// Missing parents never count as a shared ancestor. These are game behavior weights.
pub(crate) fn support_affinity(
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
