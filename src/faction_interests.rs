//! Game political interests and pressure-dependent movements, not historical categories.
pub const COUNT: usize = 9;
const HOUSEHOLD_FOOD_PRESSURE_WEIGHT: f32 = 0.75;
const TOWN_FOOD_PRESSURE_WEIGHT: f32 = 0.25;
const BASE_APPEAL: [f32; COUNT] = [0.8, 0.5, 0.5, 0.5, 0.2, 0.3, 0.1, 0.1, 0.1];
const GROWER_HUNGER_WEIGHT: f32 = 1.5;
const MERCHANT_TRADE_WEIGHT: f32 = 2.;
const RETAINER_WAR_WEIGHT: f32 = 1.5;
const ARTISAN_CRAFT_WEIGHT: f32 = 2.;
const ARTISAN_INEQUALITY_WEIGHT: f32 = 0.5;
const SCHOLAR_CURIOSITY_WEIGHT: f32 = 1.5;
const CONGREGATION_PIETY_WEIGHT: f32 = 1.5;
const BREAD_LEAGUE_HUNGER_WEIGHT: f32 = 5.;
const BREAD_LEAGUE_INEQUALITY_WEIGHT: f32 = 2.;
const REVIVALIST_PIETY_WEIGHT: f32 = 4.;
const WARBAND_WAR_WEIGHT: f32 = 5.;
const COHESION_PRESSURE_GAIN: f32 = 0.35;
const COHESION_CALM_LOSS: f32 = 0.18;
const COHESION_LEADER_LOSS: f32 = 0.4;
const MIN_COHESION: f32 = 0.05;

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
/// Completed household access dominates current appeal; remembered town hardship
/// retains a smaller solidarity effect. Missing retail observations use the town proxy.
pub fn food_pressure(town: f32, household: Option<f32>) -> f32 {
    household
        .map_or(town, |h| {
            HOUSEHOLD_FOOD_PRESSURE_WEIGHT * h + TOWN_FOOD_PRESSURE_WEIGHT * town
        })
        .clamp(0., 1.)
}
/// Shared local conditions: hunger, inequality, disruption, war, craft, trade, piety, curiosity.
pub fn appeal(k: usize, x: [f32; 8]) -> f32 {
    let [hunger, inequality, disruption, war, craft, trade, piety, curiosity] = x;
    match k {
        0 => BASE_APPEAL[0] + GROWER_HUNGER_WEIGHT * hunger,
        1 => BASE_APPEAL[1] + MERCHANT_TRADE_WEIGHT * trade,
        2 => BASE_APPEAL[2] + RETAINER_WAR_WEIGHT * war,
        3 => BASE_APPEAL[3] + ARTISAN_CRAFT_WEIGHT * craft + ARTISAN_INEQUALITY_WEIGHT * inequality,
        4 => BASE_APPEAL[4] + SCHOLAR_CURIOSITY_WEIGHT * curiosity * (1. - hunger),
        5 => BASE_APPEAL[5] + CONGREGATION_PIETY_WEIGHT * piety,
        6 => {
            BASE_APPEAL[6]
                + BREAD_LEAGUE_HUNGER_WEIGHT * hunger
                + BREAD_LEAGUE_INEQUALITY_WEIGHT * inequality
        }
        7 => BASE_APPEAL[7] + REVIVALIST_PIETY_WEIGHT * piety * (disruption + hunger).min(1.),
        8 => BASE_APPEAL[8] + WARBAND_WAR_WEIGHT * war + disruption,
        _ => 0.,
    }
}
/// Crisis coalitions lose their organizing purpose as pressure fades. Charismatic
/// movements and warbands also depend on leadership continuity. No random dissolution.
pub fn cohesion(k: usize, previous: f32, pressure: f32, leader_changed: bool) -> f32 {
    if k < 6 {
        return 1.;
    }
    let change = COHESION_PRESSURE_GAIN * pressure - COHESION_CALM_LOSS * (1. - pressure);
    (previous + change
        - if leader_changed && k >= 7 {
            COHESION_LEADER_LOSS
        } else {
            0.
        })
    .clamp(MIN_COHESION, 1.)
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn household_access_changes_politics_without_changing_production() {
        let conditions = |h| [food_pressure(0., Some(h)), 0., 0., 0., 0., 0., 0., 0.];
        assert!(appeal(6, conditions(1.)) > appeal(0, conditions(1.)));
        assert!(appeal(6, conditions(0.)) < appeal(0, conditions(0.)));
        assert_eq!(food_pressure(0.8, None), 0.8);
        assert!(food_pressure(1., Some(0.)) < food_pressure(1., Some(1.)));
    }
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
