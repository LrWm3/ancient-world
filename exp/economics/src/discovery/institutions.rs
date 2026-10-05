use super::*;

const REVIEW_MONTHS: u32 = 1;
const MINIMUM_TENURE_MONTHS: u32 = 1;

fn controller_config(
    objectives: Vec<agency::objectives::Objective>,
    horizon: u32,
) -> agency::Config {
    agency::Config {
        horizon,
        review_every: REVIEW_MONTHS,
        minimum_tenure: MINIMUM_TENURE_MONTHS,
        emergency: None,
        objectives,
        preferences: BTreeMap::new(),
        programs: BTreeMap::new(),
    }
}

pub(super) fn govern(w: &mut World, s: &State, c: &Config) -> Result<(), String> {
    let Some(rule) = &c.state else {
        return Ok(());
    };
    if w.state_governance.is_some() {
        return Ok(());
    }
    let state = w.transaction_policy.as_ref().unwrap().authority;
    let citizens: Vec<_> = people(w, s)
        .into_iter()
        .filter(|a| {
            s.memberships
                .contains_key(&(*a, state, crate::membership::CITIZEN))
        })
        .collect();
    let Some(&founder) = citizens.first() else {
        return Ok(());
    };
    let initial_policy = *rule
        .constitution
        .policies
        .keys()
        .next()
        .ok_or("empty state constitution")?;
    w.state_governance = Some(crate::state_governance::Governance {
        formation: None,
        state,
        formed: s.month,
        constitution: rule.constitution.clone(),
        charter: crate::state_governance::Charter {
            founder,
            term_months: rule.term_months,
            election: Default::default(),
            initial_policy,
        },
        ballots: vec![],
        changes: vec![],
    });
    let mut config = controller_config(rule.objectives.clone(), c.horizon);
    // A constitution delegates policy to the selected governor; no fabricated votes.
    for person in citizens {
        config.preferences.insert(person, rule.objectives.clone());
    }
    w.agency
        .insert(state, agency::Controller::discovering(config));
    record(
        w,
        s,
        format!("installed public governance; founding governor {founder}"),
        BTreeMap::new(),
        true,
    );
    Ok(())
}

pub(super) fn households(w: &mut World, s: &State, c: &Config) -> Result<(), String> {
    let Some(rule) = &c.household else {
        return Ok(());
    };
    let candidates: Vec<_> = people(w, s)
        .into_iter()
        .filter(|a| {
            crate::households::parent(w, s, *a).is_none()
                && opportunities::permits(w, s, *a, Action::FoundHousehold)
        })
        .collect();
    if candidates.len() < 2 {
        return Ok(());
    }
    // Decentralized bilateral proposal: each person must be no worse off under
    // their own ordered need objectives, and at least one must strictly improve.
    // No global best grouping or counterparty production promise is assumed.
    for (i, &first) in candidates.iter().enumerate() {
        if crate::households::parent(w, s, first).is_some() {
            continue;
        }
        for &second in &candidates[i + 1..] {
            if crate::households::parent(w, s, second).is_some() {
                continue;
            }
            let baseline = forecast(w, s, c.horizon)?;
            let members = [first, second];
            let objectives: BTreeMap<_, _> = members.iter().map(|&a| (a, needs(w, a))).collect();
            let mut choices = vec![];
            for &policy in &rule.constitution.permitted_policies {
                let mut candidate = w.clone();
                let agent = next_id(candidate.agents.iter().map(|a| a.id))?;
                let id = next_id(candidate.households.iter().map(|h| h.id))?;
                let mut charter = rule.charter.clone();
                charter.leader = first;
                charter.initial_policy = policy;
                let agreement = crate::households::Agreement {
                    id,
                    agent,
                    adults: members.to_vec(),
                    formed: s.month,
                    governance: h::Governance {
                        constitution: rule.constitution.clone(),
                        charter,
                        changes: vec![],
                        ballots: vec![],
                    },
                    dwelling_process: None,
                    admission: None,
                    membership: vec![],
                    asset_sales: vec![],
                    equipment_retirements: vec![],
                    support: vec![],
                };
                if crate::households::form(&mut candidate, s, agreement).is_err() {
                    continue;
                }
                let projected = match forecast(&candidate, s, c.horizon) {
                    Ok(v) => v,
                    Err(_) => continue,
                };
                let comparisons: BTreeMap<_, _> = members
                    .iter()
                    .map(|&a| {
                        Ok((
                            a,
                            (
                                loss(&baseline, a, &objectives[&a])?,
                                loss(&projected, a, &objectives[&a])?,
                            ),
                        ))
                    })
                    .collect::<Result<_, String>>()?;
                let accepted = comparisons.values().all(|(before, after)| after <= before)
                    && comparisons.values().any(|(before, after)| after < before);
                record(
                    w,
                    s,
                    format!("household proposal {first}+{second} policy {policy:?}"),
                    comparisons.clone(),
                    accepted,
                );
                if accepted {
                    choices.push((
                        comparisons
                            .values()
                            .flat_map(|(_, after)| after.clone())
                            .collect::<Vec<_>>(),
                        policy,
                        candidate,
                        agent,
                    ));
                }
            }
            choices.sort_by_key(|(loss, policy, _, _)| (loss.clone(), *policy));
            if let Some((_, _, mut candidate, agent)) = choices.into_iter().next() {
                // Keep every rejected comparison, not just the chosen branch's history.
                candidate.discovery = w.discovery.clone();
                let objectives = members.iter().flat_map(|a| objectives[a].clone()).collect();
                let mut config = controller_config(objectives, c.horizon);
                for member in members {
                    config.preferences.insert(member, config.objectives.clone());
                }
                w.clone_from(&candidate);
                w.agency
                    .insert(agent, agency::Controller::discovering(config));
                record(
                    w,
                    s,
                    format!("formed household {agent}: {first}+{second}"),
                    BTreeMap::new(),
                    true,
                );
                break;
            }
        }
    }
    Ok(())
}
