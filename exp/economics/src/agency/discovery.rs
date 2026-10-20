//! Candidate discovery from immutable institutional options and process recipes.
use super::*;

/// Enumeration supplies alternatives, never authority or promised production.
/// Refuse an oversized catalog rather than silently favor its first entries.
pub fn programs(w: &World, agent: AgentId) -> Result<BTreeMap<u32, Program>, String> {
    programs_with_sales(w, agent, discovery_sales::opportunities(w, agent)?, false)
}

pub fn programs_from_requests(
    w: &World,
    agent: AgentId,
    snapshot: &work_requests::Snapshot,
) -> Result<BTreeMap<u32, Program>, String> {
    snapshot.validate(w, agent, snapshot.month)?;
    programs_with_sales(
        w,
        agent,
        work_requests::sales(w, snapshot)?,
        matches!(
            w.agency[&agent].config.work_requests,
            work_requests::Policy::RespondWithWorkQuotes | work_requests::Policy::Upstream
        ),
    )
}

fn programs_with_sales(
    w: &World,
    agent: AgentId,
    sales: Vec<discovery_sales::Opportunity>,
    bundle_starts: bool,
) -> Result<BTreeMap<u32, Program>, String> {
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
    let mut processes = BTreeSet::new();
    let config = &w.agency[&agent].config;
    for objective in config
        .objectives
        .iter()
        .chain(config.preferences.values().flatten())
    {
        use objectives::Metric;
        match objective.metric {
            Metric::ProcessShortfall { definition, .. } => {
                let d = w.definition(definition);
                if d.execution == Execution::Productive {
                    processes.insert(definition);
                }
                wanted.extend(
                    d.stages
                        .iter()
                        .flat_map(|s| &s.entry_inputs)
                        .map(|a| a.resource),
                );
            }
            Metric::Reserve { resource, .. }
            | Metric::ReserveExposure { resource, .. }
            | Metric::FundingGap { resource, .. }
            | Metric::NeedDeficit(resource) => {
                wanted.insert(resource);
            }
            _ => {}
        }
    }
    wanted.extend(sales.iter().map(|o| o.resource));
    // Include intermediate technologies; reachability does not assert feasibility.
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
                        let (objectives::Metric::Reserve { resource, target }
                        | objectives::Metric::ReserveExposure { resource, target }) = o.metric
                        else {
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
    // Preserve explicit reserve-target choices. Without them, signed sale lots
    // supply bounded replenishment alternatives rather than a one-shot start.
    let explicit: BTreeSet<_> = commands
        .iter()
        .filter_map(|c| match c {
            Command::WorkTargets(orders) => orders.first().map(|o| o.definition),
            _ => None,
        })
        .collect();
    let sale_definitions: BTreeSet<_> = sales
        .iter()
        .filter(|o| !explicit.contains(&o.definition))
        .map(|o| o.definition)
        .collect();
    commands.retain(|c| !matches!(c, Command::StartProcess(id) if sale_definitions.contains(id)));
    let mut seen = BTreeSet::new();
    for o in sales {
        if !explicit.contains(&o.definition) && seen.insert((o.definition, o.resource, o.target)) {
            commands.push(Command::WorkTargets(vec![WorkOrder {
                agent,
                definition: o.definition,
                priority: 0,
                target: crate::activities::Target::Stock(Amount::new(o.resource, o.target)),
            }]));
        }
    }
    let mut portfolios = markets::discovery::portfolios(w, agent)?;
    // On equal improving forecasts, offer a bounded productive bundle before
    // work alone. Retaining existing policy still wins every equal-loss tie.
    portfolios.sort_by_key(|q| q.is_empty());
    // Compose existing work with existing signed portfolios; this adds neither
    // authority nor assumed fills. Static and request-only catalogs stay stable.
    let bundle_work = |c: &&Command| {
        matches!(c, Command::WorkTargets(_))
            || (bundle_starts && matches!(c, Command::StartProcess(_)))
    };
    let work_count = commands.iter().filter(bundle_work).count();
    let total = work_count
        .checked_mul(portfolios.len())
        .and_then(|n| n.checked_add(commands.len()))
        .and_then(|n| n.checked_add(portfolios.len()));
    if total.is_none_or(|n| n > MAX_PROGRAMS) {
        return Err("discovered program catalog exceeds bounded search".into());
    }
    let mut programs = vec![];
    for quotes in &portfolios {
        for work in commands.iter().filter(bundle_work) {
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
