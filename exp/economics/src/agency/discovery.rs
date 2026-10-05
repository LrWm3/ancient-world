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
    commands.extend(processes.into_iter().map(Command::StartProcess));
    if commands.len() > MAX_PROGRAMS {
        return Err("discovered program catalog exceeds bounded search".into());
    }
    Ok(commands
        .into_iter()
        .enumerate()
        .map(|(i, command)| {
            (
                i as u32,
                Program {
                    name: format!("discovered {command:?}"),
                    commands: vec![command],
                },
            )
        })
        .collect())
}
