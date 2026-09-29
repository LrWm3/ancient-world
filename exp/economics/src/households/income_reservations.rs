//! Exact contribution storage reservations for town trades. Credit is excluded.
use super::*;

#[derive(Clone, Debug)]
pub(crate) struct Reservations {
    used: BTreeMap<AgentId, i128>,
    parents: BTreeMap<AgentId, AgentId>,
    remainders: Remainders,
}
impl Reservations {
    pub fn new(world: &World, state: &State, used: BTreeMap<AgentId, i128>) -> Self {
        Self {
            used,
            parents: world
                .agents
                .iter()
                .filter_map(|a| parent(world, state, a.id).map(|h| (a.id, h)))
                .collect(),
            remainders: state.household_remainders.clone(),
        }
    }
    /// Reserve raw exchange plus its incremental pooled share. Fractional carry
    /// is relationship-specific and accumulates across the entire monthly book.
    /// Incoming shares do not enter spendable opening budgets.
    pub fn preview(&self, world: &World, effects: &[Effect]) -> Result<Option<Self>, String> {
        let mut next = self.clone();
        let mut combined = effects.to_vec();
        for ((member, resource), quantity) in
            incomes_for(world, effects, |a| self.parents.contains_key(&a))?
        {
            let household = self.parents[&member];
            let carry = next
                .remainders
                .entry((household, member, resource))
                .or_default();
            let numerator = i64::from(quantity) + i64::from(*carry);
            let share = (numerator / i64::from(POOL_DIVISOR)) as i32;
            *carry = (numerator % i64::from(POOL_DIVISOR)) as i32;
            combined.extend(transfer(member, household, resource, share));
        }
        if !crate::storage::fits(world, &next.used, &combined) {
            return Ok(None);
        }
        crate::storage::apply(world, &mut next.used, &combined);
        Ok(Some(next))
    }
}
