//! A bounded mixed-organization acceptance fixture, not a new execution driver.
use crate::{
    agency::integration,
    compute::Backend,
    financial_reporting::Audit,
    minting::{COIN, ISSUER, MINT, NUTRITION},
    model::*,
    simulation::Simulation,
};

pub const RUN_MONTHS: u32 = integration::RUN_MONTHS;

pub fn scenario() -> Result<(World, State), String> {
    integration::commitment_scenario()
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Evidence {
    pub months: u32,
    pub people_food_deficit: i32,
    pub completed_mints: usize,
    pub household_delivered: i32,
    pub land_dues_paid: i32,
    pub issuance: i128,
    pub reconciled_agents: usize,
}

pub struct Run {
    pub sim: Simulation,
    pub audit: Audit,
    opening_coins: i128,
}

fn coins(s: &State) -> i128 {
    s.balances
        .iter()
        .filter(|((_, r), _)| *r == COIN)
        .map(|(_, q)| i128::from(*q))
        .sum()
}

impl Run {
    pub fn new(w: World, s: State, backend: Backend) -> Result<Self, String> {
        let audit = integration::audit(&w, &s)?;
        let opening_coins = coins(&s);
        Ok(Self {
            sim: Simulation::new(w, s, backend)?,
            audit,
            opening_coins,
        })
    }

    pub fn through(&mut self, month: u32) -> Result<(), String> {
        while self.sim.state.month <= month {
            self.audit.step(&mut self.sim)?;
        }
        Ok(())
    }

    /// Reconcile actual completed boundaries; labels and forecasts are not evidence.
    pub fn evidence(&self) -> Result<Evidence, String> {
        let sim = &self.sim;
        if sim.state.phase != Phase::Open || sim.state.month <= 1 {
            return Err("acceptance evidence requires a completed month".into());
        }
        let months = sim.state.month - 1;
        for agent in &sim.world.agents {
            let statement = self.audit.book().statements(agent.id, 1, months)?;
            if statement.assets != statement.liabilities + statement.equity
                || statement.closing_cash != i128::from(sim.state.balance(agent.id, COIN))
            {
                return Err(format!("unreconciled agent {}", agent.id));
            }
        }
        let issuance = self
            .audit
            .book()
            .statements(ISSUER, 1, months)?
            .issuance_change;
        if coins(&sim.state) - self.opening_coins != issuance {
            return Err("coin supply does not reconcile to issuance".into());
        }
        Ok(Evidence {
            months,
            people_food_deficit: sim.reports.iter().map(|r| r.deficit(NUTRITION)).sum(),
            completed_mints: sim
                .state
                .processes
                .values()
                .filter(|p| p.definition == MINT && p.status == Status::Completed)
                .count(),
            household_delivered: sim
                .state
                .exchange
                .forwards
                .get(&20)
                .map_or(0, |f| f.delivered),
            land_dues_paid: sim.state.obligations.get(&(77, 13)).map_or(0, |c| c.paid),
            issuance,
            reconciled_agents: sim.world.agents.len(),
        })
    }
}
