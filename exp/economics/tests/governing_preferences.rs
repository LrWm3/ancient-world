use economics_compute_smoke::{
    acceptance::{SMITH, WORKSHOP, continuation::Case, financing},
    agency::{
        self,
        objectives::{self, Metric, Objective, Scope},
    },
    governance::Leadership,
    minting::{COIN, REFINE, WORKER},
};
#[test]
fn active_office_selects_personal_goals_then_falls_back_without_a_vote() {
    let (mut w, mut s) = financing::scenario(Case::Baseline).unwrap();
    let old = w.agency.remove(&WORKSHOP).unwrap();
    let mut agreement = old.operating_agreement().unwrap().clone();
    agreement.members.insert(WORKER);
    agreement.leadership = Leadership::Rotating;
    agreement.term_months = 1;
    let fallback = vec![Objective {
        scope: Scope::Organization,
        metric: Metric::Reserve {
            resource: COIN,
            target: 10,
        },
    }];
    let personal = vec![Objective {
        scope: Scope::Members,
        metric: Metric::ProcessShortfall {
            definition: REFINE,
            target: 1,
        },
    }];
    let mut config = old.config;
    config.objectives = fallback.clone();
    config.preferences = [(SMITH, personal.clone())].into();
    agency::organization::install_discovering_initial(
        &mut w,
        &s,
        agreement.clone(),
        &agreement.members,
        config,
    )
    .unwrap();
    assert_eq!(agency::governor(&w, &s, WORKSHOP), Some(SMITH));
    assert_eq!(
        objectives::governing(&w, &s, WORKSHOP),
        Some(personal.as_slice())
    );
    let frozen = objectives::freeze(
        &w,
        &s,
        WORKSHOP,
        objectives::governing(&w, &s, WORKSHOP).unwrap(),
    );
    assert_eq!(frozen[0].scope, Scope::Agents([SMITH, WORKER].into()));
    s.month = 2;
    assert_eq!(agency::governor(&w, &s, WORKSHOP), Some(WORKER));
    assert_eq!(
        objectives::governing(&w, &s, WORKSHOP),
        Some(fallback.as_slice())
    );
    assert_eq!(frozen[0].scope, Scope::Agents([SMITH, WORKER].into()));
    assert!(objectives::governing(&w, &s, WORKER).is_none());
}
