use economics_compute_smoke::{
    acceptance::{self, Run, TOWN, WORKSHOP, continuation as c, publication},
    agency::Command,
    compute::Backend,
    minting::{METAL, ORE},
};
#[test]
fn stocked_asks_publish_next_open_before_independent_buyer_action() {
    let (w, s) = publication::scenario(c::Case::Baseline).unwrap();
    let run = |backend| {
        let mut run = Run::new(w.clone(), s.clone(), backend).unwrap();
        run.through(1).unwrap();
        for id in [WORKSHOP, TOWN] {
            let d = &run.sim.world.agency[&id].history[0];
            assert!(d.accepted.is_some(), "{id}: {d:?}");
            assert_eq!(d.reason, "publish stocked sale offers on equal projection");
            assert_eq!(d.effective_month, 2);
            assert!(matches!(
                d.accepted.as_ref().unwrap().commands.as_slice(),
                [Command::PeerStockQuotes(_)]
            ));
        }
        assert!(
            !acceptance::accepted_deals(&run.sim)
                .any(|d| d.seller == WORKSHOP && d.market == METAL)
        );
        assert!(
            run.sim
                .world
                .minting
                .as_ref()
                .unwrap()
                .order_policy
                .as_ref()
                .unwrap()
                .quotes
                .iter()
                .all(|q| q.agent != WORKSHOP)
        );
        assert_eq!(
            run.sim.state.balance(WORKSHOP, METAL),
            s.balance(WORKSHOP, METAL)
        );
        assert_eq!(run.sim.state.balance(TOWN, ORE), s.balance(TOWN, ORE));
        run.through(5).unwrap();
        assert!(
            acceptance::accepted_deals(&run.sim)
                .any(|d| d.seller == WORKSHOP && d.market == METAL && d.month >= 2)
        );
        assert!(run.evidence().unwrap().completed_mints > 0);
        run
    };
    let r = run(Backend::Reference);
    let cpu = run(Backend::CubeCpu);
    assert_eq!(r.sim.world, cpu.sim.world);
    assert_eq!(r.sim.state, cpu.sim.state);
    assert_eq!(r.sim.ledger, cpu.sim.ledger);
    assert_eq!(r.audit, cpu.audit);
}
