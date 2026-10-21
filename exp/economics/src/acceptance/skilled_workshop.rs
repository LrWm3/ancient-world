//! Personal learning while an organization retains its productive assets and books.
use super::{Run, SMITH, WORKSHOP, WORKSHOP_EMPLOYMENT, maintained_workshop as maintained};
use crate::{
    compute::Backend,
    equipment::{PracticeRule, Technique},
    minting::{HOURS, REFINE},
    model::*,
};
pub const COMPETENCY: u32 = 140;
pub const TECHNIQUE: u32 = 140;
pub const QUALIFYING_COMPLETIONS: u32 = 2;
const NOVICE_HOURS: i32 = 2;
const QUALIFIED_HOURS: i32 = 1;
const PERSONAL_MONTHLY_HOURS: i32 = 3;

/// Same endowments as maintained-workshop, with an explicit labor calibration:
/// novice refining costs two hours; three personal hours can also cover town work.
/// Disabling the benefit changes only the technique's required hours.
pub fn run(benefit: bool, backend: Backend) -> Result<Run, String> {
    let mut run = maintained::run(maintained::Case::Discovered, backend)?;
    let w = &mut run.sim.world;
    w.definitions
        .iter_mut()
        .find(|d| d.id == REFINE)
        .unwrap()
        .stages[0]
        .monthly_services = vec![Amount::new(HOURS, NOVICE_HOURS)];
    w.participants
        .iter_mut()
        .find(|p| p.agent == SMITH)
        .unwrap()
        .capacity
        .quantity = PERSONAL_MONTHLY_HOURS;
    w.employment
        .iter_mut()
        .find(|t| t.id == WORKSHOP_EMPLOYMENT)
        .unwrap()
        .capacity
        .quantity = NOVICE_HOURS;
    w.agency
        .get_mut(&WORKSHOP)
        .unwrap()
        .config
        .hiring
        .as_mut()
        .unwrap()
        .payroll
        .quantity = NOVICE_HOURS;
    for d in [REFINE, maintained::REPAIR] {
        w.activities.performers.insert((d, 0), HOURS);
    }
    w.practice_rules.push(PracticeRule {
        definition: REFINE,
        stage: 0,
        competency: COMPETENCY,
        points: 1,
    });
    w.techniques.push(Technique {
        id: TECHNIQUE,
        definition: REFINE,
        stage: 0,
        equipment_kind: None,
        competency: Some((COMPETENCY, QUALIFYING_COMPLETIONS)),
        output_multiplier: 1,
        wear: 0,
        services: vec![Amount::new(
            HOURS,
            if benefit {
                QUALIFIED_HOURS
            } else {
                NOVICE_HOURS
            },
        )],
    });
    crate::settlement::validate_world(w, &run.sim.state)?;
    Ok(run)
}
