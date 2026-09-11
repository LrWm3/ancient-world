//! Shared CPU reservation ceiling, in effective worker-months.
//! Match workforce modifiers in society.wgsl::workers and ecological_production.
use crate::civilization::Site;

fn service_capacity(
    population: f32,
    adults: f32,
    illness: f32,
    recovery: f32,
    society: bool,
    living: bool,
) -> f32 {
    let workers = if society {
        adults.max(0.) * 0.8 * (1. - 0.5 * illness.clamp(0., 0.5))
    } else {
        population.max(0.) * 0.5
    };
    let recovery = if living { recovery.clamp(0., 1.) } else { 0. };
    workers * (1. - 0.4 * recovery) * 0.2
}

fn remaining(capacity: f32, external: f32, enterprise: [f32; 4]) -> f32 {
    (capacity - external.max(0.) - enterprise.iter().sum::<f32>()).max(0.)
}

/// Existing reservations have priority. This is a ceiling, not a promise that
/// materials, orders, cash or the GPU craft allocation will permit all work.
pub(crate) fn available(site: &Site, society: bool, living: bool) -> f32 {
    if site.abandoned {
        return 0.;
    }
    remaining(
        service_capacity(
            site.stocks.stock[0],
            site.demography.ages[1],
            site.demography.health[0],
            site.economy.soil[3],
            society,
            living,
        ),
        site.economy.external[3],
        site.economy.enterprise_plan,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn workforce_flags_match_production_contract() {
        assert_eq!(service_capacity(200., 100., 0., 0., true, true), 16.);
        assert_eq!(service_capacity(200., 100., 0.5, 0., true, true), 12.);
        assert!((service_capacity(200., 100., 0.5, 1., true, true) - 7.2).abs() < 1e-6);
        assert_eq!(service_capacity(200., 100., 0.5, 1., true, false), 12.);
        assert_eq!(service_capacity(200., 100., 0.5, 1., false, false), 20.);
        assert_eq!(service_capacity(200., 0., 0., 0., true, true), 0.);
    }
    #[test]
    fn earlier_reservations_reduce_later_grants() {
        let capacity = service_capacity(200., 100., 0., 0., true, true);
        assert_eq!(remaining(capacity, 2.5, [3., 4., 0., 0.]), 6.5);
        assert_eq!(remaining(capacity, 20., [0.; 4]), 0.);
        assert_eq!(remaining(capacity, 2.5, [8.; 4]), 0.);
        // Current illness closes capacity even if last month's craft pool was larger.
        let sick = service_capacity(200., 100., 0.5, 0., true, true);
        assert_eq!(remaining(sick, 2.5, [3., 4., 0., 0.]), 2.5);
    }
}
