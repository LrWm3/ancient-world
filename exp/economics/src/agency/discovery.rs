//! Candidate discovery from immutable institutional options and process recipes.
use super::*;

/// Enumeration supplies alternatives, never authority or promised production.
/// Refuse an oversized catalog rather than silently favor its first entries.
pub fn programs(w: &World, agent: AgentId) -> Result<BTreeMap<u32, Program>, String> {
    let mut commands = vec![];
    if let Some(g) = w.state_governance.as_ref().filter(|g| g.state == agent) {
        commands.extend(
            g.constitution
                .policies
                .keys()
                .map(|p| Command::StatePolicy(*p)),
        );
    }
    if let Some(h) = w.households.iter().find(|h| h.agent == agent) {
        commands.extend(
            h.governance
                .constitution
                .permitted_policies
                .iter()
                .map(|p| Command::HouseholdPolicy(*p)),
        );
    }
    let mut wanted = BTreeSet::new();
    for objective in &w.agency[&agent].config.objectives {
        use objectives::Metric;
        match objective.metric {
            Metric::Reserve { resource, .. }
            | Metric::FundingGap { resource, .. }
            | Metric::NeedDeficit(resource) => {
                wanted.insert(resource);
            }
            _ => {}
        }
    }
    // Include intermediate technologies; reachability does not assert feasibility.
    let mut processes = BTreeSet::new();
    loop {
        let previous = (wanted.len(), processes.len());
        for d in &w.definitions {
            if d.outputs.iter().any(|a| wanted.contains(&a.resource)) {
                if d.execution == Execution::Productive {
                    processes.insert(d.id);
                }
                wanted.extend(
                    d.stages
                        .iter()
                        .flat_map(|s| &s.entry_inputs)
                        .map(|a| a.resource),
                );
            }
        }
        if previous == (wanted.len(), processes.len()) {
            break;
        }
    }
    commands.extend(
        processes
            .into_iter()
            .filter(|id| {
                w.agency[&agent]
                    .operating_agreement()
                    .is_none_or(|a| a.processes.contains(id))
            })
            .map(|id| {
                let target = w.agency[&agent].operating_agreement().and_then(|_| {
                    w.agency[&agent].config.objectives.iter().find_map(|o| {
                        if o.scope != objectives::Scope::Organization {
                            return None;
                        }
                        let objectives::Metric::Reserve { resource, target } = o.metric else {
                            return None;
                        };
                        (target > 0
                            && w.definitions.iter().any(|d| {
                                d.id == id && d.outputs.iter().any(|a| a.resource == resource)
                            }))
                        .then_some((resource, target))
                    })
                });
                if let Some((resource, target)) = target {
                    Command::WorkTargets(vec![WorkOrder {
                        agent,
                        definition: id,
                        priority: 0,
                        target: crate::activities::Target::Stock(Amount::new(resource, target)),
                    }])
                } else {
                    Command::StartProcess(id)
                }
            }),
    );
    let mut portfolios = markets::discovery::portfolios(w, agent)?;
    // On equal improving forecasts, offer a bounded productive bundle before
    // work alone. Retaining existing policy still wins every equal-loss tie.
    portfolios.sort_by_key(|q| q.is_empty());
    let work_count = commands
        .iter()
        .filter(|c| matches!(c, Command::WorkTargets(_)))
        .count();
    let total = work_count
        .checked_mul(portfolios.len())
        .and_then(|n| n.checked_add(commands.len()))
        .and_then(|n| n.checked_add(portfolios.len()));
    if total.is_none_or(|n| n > MAX_PROGRAMS) {
        return Err("discovered program catalog exceeds bounded search".into());
    }
    let mut programs = vec![];
    for quotes in &portfolios {
        for work in commands
            .iter()
            .filter(|c| matches!(c, Command::WorkTargets(_)))
        {
            programs.push(vec![work.clone(), Command::PeerStockQuotes(quotes.clone())]);
        }
    }
    programs.extend(commands.into_iter().map(|c| vec![c]));
    programs.extend(
        portfolios
            .into_iter()
            .map(|q| vec![Command::PeerStockQuotes(q)]),
    );
    if programs.len() > MAX_PROGRAMS {
        return Err("discovered program catalog exceeds bounded search".into());
    }
    Ok(programs
        .into_iter()
        .enumerate()
        .map(|(i, commands)| {
            (
                i as u32,
                Program {
                    name: if commands.len() == 1 {
                        format!("discovered {:?}", commands[0])
                    } else {
                        format!("discovered {commands:?}")
                    },
                    commands,
                },
            )
        })
        .collect())
}
