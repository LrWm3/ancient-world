use economics_compute_smoke::{
    compute::Backend, model::*, planning, scenario::*, search::*, simulation::Simulation,
};

fn sim() -> Simulation {
    let (w, s) = named("opportunity-farming").unwrap();
    Simulation::new(w, s, Backend::Reference).unwrap()
}
fn acquire(s: &mut Simulation) {
    while s.state.phase != Phase::Acquire {
        s.step().unwrap();
    }
}

#[test]
fn search_is_read_only_budgeted_and_excludes_future_scripted_observations() {
    let mut s = sim();
    s.world.capacity_overrides.insert((99, PERSON), 0);
    s.world.scheduled_starts.push(ScheduledStart {
        month: 99,
        agent: PERSON,
        definition: GROW,
    });
    acquire(&mut s);
    let before = s.state.clone();
    let world = s.world.clone();
    let context = SearchContext::new(&s.world, &s.state);
    assert!(context.world().capacity_overrides.is_empty());
    assert!(
        context
            .world()
            .scheduled_starts
            .iter()
            .all(|x| x.month == s.state.month)
    );
    let full = NeedDirectedOpportunitySearch
        .search(&context, SearchBudget::default())
        .unwrap();
    assert!(!full.budget_exhausted);
    assert!(full.candidates.iter().any(|p| matches!(
        p.steps.as_slice(),
        [
            PlanStep::AcceptMembership { .. },
            PlanStep::AcceptLand { .. }
        ]
    )));
    let partial = NeedDirectedOpportunitySearch
        .search(&context, SearchBudget { max_candidates: 2 })
        .unwrap();
    assert!(partial.budget_exhausted);
    assert_eq!(partial.candidates, full.candidates[..2]);
    let exact = NeedDirectedOpportunitySearch
        .search(
            &context,
            SearchBudget {
                max_candidates: full.candidates.len(),
            },
        )
        .unwrap();
    assert_eq!(exact, full);
    let empty = NeedDirectedOpportunitySearch
        .search(&context, SearchBudget { max_candidates: 0 })
        .unwrap();
    assert!(empty.budget_exhausted && empty.candidates.is_empty());
    assert_eq!(s.state, before);
    assert_eq!(s.world, world);
}

#[test]
fn per_agent_configuration_swaps_search_without_changing_settlement() {
    let mut ordinary = sim();
    let mut restricted = sim();
    restricted.world.agent_search.insert(
        PERSON,
        SearchConfig {
            strategy: SearchStrategy::ExistingCommitmentsOnly,
            ..Default::default()
        },
    );
    for s in [&mut ordinary, &mut restricted] {
        s.run_months(1).unwrap();
    }
    assert_eq!(ordinary.state.memberships.len(), 1);
    assert!(restricted.state.memberships.is_empty());
    assert!(restricted.state.accepted_agreements.is_empty());
    assert!(
        !restricted
            .state
            .processes
            .values()
            .any(|p| p.definition == GROW)
    );
    let d = restricted
        .ledger
        .iter()
        .find_map(|b| b.decision.as_ref())
        .unwrap();
    assert_eq!(d.search_name, "ExistingCommitmentsOnly");
    assert_eq!(d.candidates_generated, 1);
    assert!(d.alternatives[d.selected].plan.steps.is_empty());
    // The named strategy continues an already accepted crop with the same resolver.
    ordinary.world.agent_search = restricted.world.agent_search.clone();
    let crop = ordinary
        .state
        .processes
        .values()
        .find(|p| p.definition == GROW)
        .unwrap()
        .id;
    ordinary.run_months(1).unwrap();
    assert_eq!(ordinary.state.processes[&crop].status, Status::Active);
    assert_eq!(ordinary.state.processes[&crop].elapsed, 1);
}

struct Fixed(Vec<CandidatePlan>);
impl OpportunitySearch for Fixed {
    fn name(&self) -> &'static str {
        "TestProposalSearch"
    }
    fn search(&self, _: &SearchContext, _: SearchBudget) -> Result<SearchResult, String> {
        Ok(SearchResult {
            candidates: self.0.clone(),
            budget_exhausted: false,
        })
    }
}

#[test]
fn injected_strategy_uses_same_evaluator_and_invalid_prerequisite_order_is_rejected() {
    let mut s = sim();
    acquire(&mut s);
    let mut original = Batch::empty(&s.state);
    planning::choose_with_search(
        &s,
        &mut original,
        &NeedDirectedOpportunitySearch,
        SearchBudget::default(),
    )
    .unwrap();
    let d = original.decision.as_ref().unwrap();
    let selected = d.alternatives[d.selected].clone();
    assert!(selected.plan.membership_offer().is_some());
    let before = s.state.clone();
    let mut batch = Batch::empty(&s.state);
    planning::choose_with_search(
        &s,
        &mut batch,
        &Fixed(vec![selected.plan.clone()]),
        SearchBudget::default(),
    )
    .unwrap();
    let d = batch.decision.as_ref().unwrap();
    assert_eq!(d.search_name, "TestProposalSearch");
    assert_eq!(d.alternatives[0], selected);
    assert_eq!(batch.production_plan, original.production_plan);
    assert_eq!(batch.accept_membership, original.accept_membership);
    assert_eq!(batch.accept_access, original.accept_access);
    assert_eq!(s.state, before);
    let mut invalid = selected.plan.clone();
    invalid.steps.reverse();
    let mut empty = Batch::empty(&s.state);
    let untouched = empty.clone();
    assert!(
        planning::choose_with_search(
            &s,
            &mut empty,
            &Fixed(vec![invalid]),
            SearchBudget::default()
        )
        .is_err()
    );
    assert_eq!(empty, untouched);
    assert_eq!(s.state, before);
    assert!(
        planning::choose_with_search(
            &s,
            &mut empty,
            &Fixed(vec![selected.plan.clone(), selected.plan]),
            SearchBudget { max_candidates: 1 }
        )
        .is_err()
    );
    assert_eq!(empty, untouched);
}

#[test]
fn search_budget_and_configuration_survive_midmonth_checkpoint() {
    let mut s = sim();
    s.world.agent_search.insert(
        PERSON,
        SearchConfig {
            budget: SearchBudget { max_candidates: 2 },
            ..Default::default()
        },
    );
    acquire(&mut s);
    s.step().unwrap();
    let decision = s.ledger.last().unwrap().decision.as_ref().unwrap();
    assert_eq!(decision.search_name, "NeedDirectedOpportunitySearch");
    assert!(decision.search_budget_exhausted);
    assert_eq!(decision.candidates_generated, 2);
    let mut checkpoint = s.clone();
    s.run_months(3).unwrap();
    for _ in 0..3 {
        checkpoint.run_months(1).unwrap();
    }
    assert_eq!(s.state, checkpoint.state);
    assert_eq!(s.ledger, checkpoint.ledger);
    assert_eq!(s.world.agent_search, checkpoint.world.agent_search);
}

#[test]
fn legacy_joint_planner_rejects_mixed_configuration_explicitly() {
    let (mut w, s) = named("four-person-exchange").unwrap();
    w.agent_search.insert(
        PERSON,
        SearchConfig {
            strategy: SearchStrategy::ExistingCommitmentsOnly,
            ..Default::default()
        },
    );
    assert!(
        configuration(&w, &s)
            .unwrap_err()
            .contains("joint forecast")
    );
    w.agent_search.clear();
    w.agent_search.insert(
        PERSON,
        SearchConfig {
            budget: SearchBudget { max_candidates: 0 },
            ..Default::default()
        },
    );
    assert!(configuration(&w, &s).is_err());
}
