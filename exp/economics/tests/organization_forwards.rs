use economics_compute_smoke::{
    acceptance::{self, continuation as c, publication},
    agency::{forwards::Options, organization},
    minting::ORE,
    model::*,
};

fn authorize(w: &mut World, s: &State, agent: AgentId, option: Options) -> Result<(), String> {
    let controller = w.agency.remove(&agent).unwrap();
    let mut agreement = controller.operating_agreement().unwrap().clone();
    agreement.forwards.insert(ORE, option);
    let signatures = agreement.members.clone();
    organization::install_discovering_initial(w, s, agreement, &signatures, controller.config)
}

#[test]
fn forward_authority_is_separate_and_bounded() {
    let (w, s) = publication::scenario(c::Case::Baseline).unwrap();
    assert!(
        w.agency[&acceptance::WORKSHOP]
            .operating_agreement()
            .unwrap()
            .forwards
            .is_empty()
    );
    for (price, months, valid) in [(2, 2, true), (0, 2, false), (2, 1, false), (2, 13, false)] {
        let mut candidate = w.clone();
        assert_eq!(
            authorize(
                &mut candidate,
                &s,
                acceptance::WORKSHOP,
                Options {
                    prepayment: price,
                    delivery_months: months
                }
            )
            .is_ok(),
            valid
        );
    }
}
