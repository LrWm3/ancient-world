//! Game political interests and pressure-dependent movements, not historical categories.
pub const COUNT: usize = 9;
pub const NAMES: [&str; COUNT] = [
    "Growers",
    "Merchants",
    "Retainers",
    "Artisans",
    "Scholars",
    "Congregations",
    "Bread leagues",
    "Revivalists",
    "Warbands",
];
pub const TAX: [f32; COUNT] = [0.02, 0.03, 0.06, 0.035, 0.04, 0.025, 0.01, 0.055, 0.08];
pub fn name(interest: u32) -> &'static str {
    NAMES
        .get(interest as usize)
        .copied()
        .unwrap_or("Unknown interest")
}
pub fn resists_autonomy(interest: u32) -> bool {
    matches!(interest, 2 | 7 | 8)
}
/// Shared local conditions: hunger, inequality, disruption, war, craft, trade, piety, curiosity.
pub fn appeal(k: usize, x: [f32; 8]) -> f32 {
    let [hunger, inequality, disruption, war, craft, trade, piety, curiosity] = x;
    match k {
        0 => 0.8 + 1.5 * hunger,
        1 => 0.5 + 2. * trade,
        2 => 0.5 + 1.5 * war,
        3 => 0.5 + 2. * craft + 0.5 * inequality,
        4 => 0.2 + 1.5 * curiosity * (1. - hunger),
        5 => 0.3 + 1.5 * piety,
        6 => 0.1 + 5. * hunger + 2. * inequality,
        7 => 0.1 + 4. * piety * (disruption + hunger).min(1.),
        8 => 0.1 + 5. * war + disruption,
        _ => 0.,
    }
}
/// Crisis coalitions lose their organizing purpose as pressure fades. Charismatic
/// movements and warbands also depend on leadership continuity. No random dissolution.
pub fn cohesion(k: usize, previous: f32, pressure: f32, leader_changed: bool) -> f32 {
    if k < 6 {
        return 1.;
    }
    let change = 0.35 * pressure - 0.18 * (1. - pressure);
    (previous + change - if leader_changed && k >= 7 { 0.4 } else { 0. }).clamp(0.05, 1.)
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn crisis_movements_rise_then_fragment_without_dice() {
        let calm = [0.; 8];
        let hungry = [1., 0.5, 0., 0., 0., 0., 0., 0.];
        assert!(appeal(6, hungry) > appeal(0, hungry));
        assert!(appeal(6, calm) < appeal(0, calm));
        for k in 6..COUNT {
            let mut c = 0.3;
            for _ in 0..3 {
                c = cohesion(k, c, 1., false);
            }
            assert!(c > 0.95);
            for _ in 0..5 {
                c = cohesion(k, c, 0., false);
            }
            assert!(c < 0.25);
        }
        assert!(cohesion(7, 0.8, 0.5, true) < cohesion(7, 0.8, 0.5, false));
        assert_eq!(cohesion(3, 1., 0., true), 1.);
        assert!(
            appeal(8, [0., 0., 0., 1., 0., 0., 0., 0.])
                > appeal(2, [0., 0., 0., 1., 0., 0., 0., 0.])
        );
    }
}
