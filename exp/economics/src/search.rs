//! Read-only opportunity search. Strategies propose plans; evaluation and settlement
//! own feasibility, consequences and state changes.
use crate::model::*;
use std::collections::BTreeSet;

const DEFAULT_CANDIDATE_LIMIT: usize = 4096;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SearchBudget {
    pub max_candidates: usize,
}
impl Default for SearchBudget {
    fn default() -> Self {
        Self {
            max_candidates: DEFAULT_CANDIDATE_LIMIT,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SearchStrategy {
    #[default]
    NeedDirectedOpportunitySearch,
    ExistingCommitmentsOnly,
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct SearchConfig {
    pub strategy: SearchStrategy,
    pub budget: SearchBudget,
}

/// Sanitized snapshot: known catalogs and current observations, without future
/// capacity shocks or future scripted starts. No reference to live mutable state.
pub struct SearchContext {
    world: World,
    state: State,
}
impl SearchContext {
    pub fn new(world: &World, state: &State) -> Self {
        let mut world = world.clone();
        world.capacity_overrides.clear();
        world.scheduled_starts.retain(|s| s.month == state.month);
        Self {
            world,
            state: state.clone(),
        }
    }
    pub fn world(&self) -> &World {
        &self.world
    }
    pub fn state(&self) -> &State {
        &self.state
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PlanStep {
    AcceptMembership { offer: u32, agent: AgentId },
    AcceptLand { offer: u32, agent: AgentId },
    BuyEquipment { offer: u32, agent: AgentId },
    SellStock { bid: u32, agent: AgentId },
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct WorkProposal {
    pub priority: Priority,
    pub defer_new: bool,
    pub preferred_process: Option<DefinitionId>,
}
/// Ordered prerequisite acceptances followed by a bounded monthly work policy.
/// Work is expanded to dated reservations by the shared evaluator/resolver.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CandidatePlan {
    pub steps: Vec<PlanStep>,
    pub work: WorkProposal,
    pub explanation: String,
}
impl CandidatePlan {
    pub fn membership_offer(&self) -> Option<(u32, AgentId)> {
        self.steps.iter().find_map(|s| match s {
            PlanStep::AcceptMembership { offer, agent } => Some((*offer, *agent)),
            _ => None,
        })
    }
    pub fn access_offer(&self) -> Option<u32> {
        self.steps.iter().find_map(|s| match s {
            PlanStep::AcceptLand { offer, .. } => Some(*offer),
            _ => None,
        })
    }
    pub fn equipment_offer(&self) -> Option<u32> {
        self.steps.iter().find_map(|s| match s {
            PlanStep::BuyEquipment { offer, .. } => Some(*offer),
            _ => None,
        })
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SearchResult {
    pub candidates: Vec<CandidatePlan>,
    /// True only if further candidates were omitted by the supplied limit.
    pub budget_exhausted: bool,
}

pub trait OpportunitySearch {
    fn name(&self) -> &'static str;
    fn search(&self, context: &SearchContext, budget: SearchBudget)
    -> Result<SearchResult, String>;
}

pub struct NeedDirectedOpportunitySearch;
pub struct ExistingCommitmentsOnly;
impl OpportunitySearch for ExistingCommitmentsOnly {
    fn name(&self) -> &'static str {
        "ExistingCommitmentsOnly"
    }
    fn search(
        &self,
        _context: &SearchContext,
        budget: SearchBudget,
    ) -> Result<SearchResult, String> {
        Ok(SearchResult {
            candidates: if budget.max_candidates == 0 {
                vec![]
            } else {
                vec![CandidatePlan {
                steps:vec![], work:WorkProposal {priority:Priority::ContinuingFirst,defer_new:true,preferred_process:None},
                explanation:"Continue existing commitments; propose no new acquisitions or productive starts".into(),
            }]
            },
            budget_exhausted: budget.max_candidates == 0,
        })
    }
}

/// Legacy forecast groups jointly share resources. Their active participants must
/// currently select the same strategy/budget; fail explicitly rather than silently
/// choosing whichever agent appears first. Single-person configuration is per agent.
pub fn configuration(world: &World, state: &State) -> Result<SearchConfig, String> {
    for (&agent, config) in &world.agent_search {
        if !world.agents.iter().any(|a| a.id == agent) {
            return Err("unknown search-configured agent".into());
        }
        if config.budget.max_candidates == 0
            || config.budget.max_candidates > DEFAULT_CANDIDATE_LIMIT
        {
            return Err("search candidate budget must be 1..=4096".into());
        }
    }
    let mut selected = None;
    for participant in &world.participants {
        if state.terminal.contains_key(&participant.agent) {
            continue;
        }
        let config = world
            .agent_search
            .get(&participant.agent)
            .copied()
            .unwrap_or_default();
        if config.budget.max_candidates == 0
            || config.budget.max_candidates > DEFAULT_CANDIDATE_LIMIT
        {
            return Err("search candidate budget must be 1..=4096".into());
        }
        if selected.is_some_and(|s| s != config) {
            return Err("joint forecast participants must share search configuration".into());
        }
        selected = Some(config);
    }
    Ok(selected.unwrap_or_default())
}

pub(crate) fn substitute_producers(world: &World) -> BTreeSet<DefinitionId> {
    world
        .participants
        .iter()
        .flat_map(|p| &p.needs)
        .filter(|n| n.quantity > 0)
        .filter(|n| crate::substitution::recipes(world, n.resource).len() > 1)
        .flat_map(|n| crate::substitution::recipes(world, n.resource))
        .flat_map(|c| {
            world
                .definitions
                .iter()
                .filter(move |d| {
                    d.enabled
                        && d.execution == Execution::Productive
                        && d.outputs
                            .iter()
                            .any(|a| a.resource == c.stages[0].entry_inputs[0].resource)
                })
                .map(|d| d.id)
        })
        .collect()
}

impl OpportunitySearch for NeedDirectedOpportunitySearch {
    fn name(&self) -> &'static str {
        "NeedDirectedOpportunitySearch"
    }
    fn search(
        &self,
        context: &SearchContext,
        budget: SearchBudget,
    ) -> Result<SearchResult, String> {
        let world = context.world();
        let state = context.state();
        let requirements: BTreeSet<_> = world
            .participants
            .iter()
            .flat_map(|p| &p.needs)
            .filter(|n| n.quantity > 0)
            .map(|n| n.resource)
            .collect();
        let mut policies = vec![Priority::ContinuingFirst];
        policies.extend(requirements.into_iter().map(Priority::NeedFirstFor));
        let mut offers = vec![Vec::<PlanStep>::new()];
        if state.phase == Phase::Acquire {
            let mut actors: Vec<_> = world
                .participants
                .iter()
                .map(|p| p.agent)
                .filter(|id| !state.terminal.contains_key(id))
                .collect();
            actors.sort_unstable();
            let mut equipment: Vec<_> = world.offers.iter().collect();
            equipment.sort_by_key(|o| o.id);
            for offer in equipment {
                for &buyer in &actors {
                    if crate::equipment::transaction(
                        world,
                        state,
                        crate::equipment::Trade {
                            offer: offer.id,
                            buyer,
                        },
                    )
                    .is_ok()
                    {
                        offers.push(vec![PlanStep::BuyEquipment {
                            offer: offer.id,
                            agent: buyer,
                        }]);
                    }
                }
            }
            let relevant: BTreeSet<_> = actors
                .iter()
                .flat_map(|a| crate::opportunities::relevant_access(world, state, *a))
                .collect();
            let mut access: Vec<_> = world
                .access_offers
                .iter()
                .filter(|a| world.transaction_policy.is_none() || relevant.contains(&a.id))
                .collect();
            access.sort_by_key(|o| o.id);
            let snapshot = crate::simulation::Simulation::new(
                world.clone(),
                state.clone(),
                crate::compute::Backend::Reference,
            )?;
            let memberships: Vec<_> = actors
                .iter()
                .flat_map(|&agent| {
                    crate::offers::discover(world, state, agent)
                        .into_iter()
                        .filter_map(move |offer| {
                            if let crate::offers::Id::Membership(id) = offer.id {
                                Some((id, agent))
                            } else {
                                None
                            }
                        })
                })
                .filter(|&(id, agent)| {
                    crate::offers::feasible(
                        &snapshot,
                        &[crate::offers::Request::new(
                            crate::offers::Id::Membership(id),
                            agent,
                        )],
                    )
                    .is_ok()
                })
                .collect();
            for &membership in &memberships {
                offers.push(vec![PlanStep::AcceptMembership {
                    offer: membership.0,
                    agent: membership.1,
                }]);
            }
            for offer in access {
                for &applicant in actors
                    .iter()
                    .filter(|&&a| a == offer.debtor || world.open_access_offers.contains(&offer.id))
                {
                    for membership in std::iter::once(None).chain(
                        memberships
                            .iter()
                            .copied()
                            .filter(|(_, a)| *a == applicant)
                            .map(Some),
                    ) {
                        let mut bundle = Vec::new();
                        if let Some((id, agent)) = membership {
                            bundle.push(crate::offers::Request::new(
                                crate::offers::Id::Membership(id),
                                agent,
                            ));
                        }
                        bundle.push(crate::offers::Request::new(
                            crate::offers::Id::Land(offer.id),
                            applicant,
                        ));
                        if crate::offers::feasible(&snapshot, &bundle).is_ok() {
                            let mut steps = Vec::new();
                            if let Some((offer, agent)) = membership {
                                steps.push(PlanStep::AcceptMembership { offer, agent });
                            }
                            steps.push(PlanStep::AcceptLand {
                                offer: offer.id,
                                agent: applicant,
                            });
                            offers.push(steps);
                        }
                    }
                }
            }
            let mut bids: Vec<_> = world.bids.iter().collect();
            bids.sort_by_key(|b| b.id);
            for bid in bids {
                let sellers: Vec<_> = actors
                    .iter()
                    .copied()
                    .filter(|&seller| {
                        crate::currency::transaction(
                            world,
                            state,
                            crate::currency::StockTrade {
                                bid: bid.id,
                                seller,
                            },
                        )
                        .is_ok()
                    })
                    .collect();
                // All bounded subsets share the same opening treasury budget and space.
                // One lot per seller; masks provide explicit stable-ID tie breaking.
                for mask in 1..(1usize << sellers.len()) {
                    let selected: Vec<PlanStep> = sellers
                        .iter()
                        .enumerate()
                        .filter(|(i, _)| mask & (1 << i) != 0)
                        .map(|(_, id)| PlanStep::SellStock {
                            bid: bid.id,
                            agent: *id,
                        })
                        .collect();
                    offers.push(selected);
                }
            }
        }

        let mut candidates = Vec::new();
        for steps in offers {
            for &priority in &policies {
                let mut choices = vec![(false, None), (true, None)];
                choices.extend(
                    substitute_producers(world)
                        .into_iter()
                        .map(|id| (false, Some(id))),
                );
                for (defer_new, preferred_process) in choices {
                    if candidates.len() == budget.max_candidates {
                        return Ok(SearchResult {
                            candidates,
                            budget_exhausted: true,
                        });
                    }
                    let explanation = format!(
                        "Need-directed prerequisites {:?}; work {:?}, defer new={}, preferred={:?}",
                        steps, priority, defer_new, preferred_process
                    );
                    candidates.push(CandidatePlan {
                        steps: steps.clone(),
                        work: WorkProposal {
                            priority,
                            defer_new,
                            preferred_process,
                        },
                        explanation,
                    });
                }
            }
        }
        Ok(SearchResult {
            candidates,
            budget_exhausted: false,
        })
    }
}
