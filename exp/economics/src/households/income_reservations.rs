//! Exact contribution storage reservations for town trades and paid wages.
//! Credit advances and unpaid wage claims are excluded.
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
    /// Financing and other non-income transfers affect room without contributions.
    pub fn reserve_unpooled(&mut self, world: &World, effects: &[Effect]) -> Result<(), String> {
        if !crate::storage::fits(world, &self.used, effects) {
            return Err("non-income transfer exceeds reserved household storage".into());
        }
        crate::storage::apply(world, &mut self.used, effects);
        Ok(())
    }
    /// Greatest fitting divisible payment. Internal household employment is
    /// excluded: payment frees space at the employer's household while raw and
    /// pooled usage grow at the worker's distinct household. Fractional carry is
    /// included in every preview; a fitting raw transfer alone is insufficient.
    pub fn payment_limit(
        &self,
        world: &World,
        execution: &crate::finance::Execution,
        claim: &crate::finance::Obligation,
    ) -> Result<i32, String> {
        self.bounded_payment(world, execution, claim, true)
    }
    pub fn unpooled_payment_limit(
        &self,
        world: &World,
        execution: &crate::finance::Execution,
        claim: &crate::finance::Obligation,
    ) -> Result<i32, String> {
        self.bounded_payment(world, execution, claim, false)
    }
    fn bounded_payment(
        &self,
        world: &World,
        execution: &crate::finance::Execution,
        claim: &crate::finance::Obligation,
        income: bool,
    ) -> Result<i32, String> {
        let key = (claim.transfer.from, claim.transfer.amount.resource);
        let (mut low, mut high) = (
            0,
            claim
                .outstanding()
                .min(execution.available.get(&key).copied().unwrap_or(0))
                .max(0),
        );
        while low < high {
            let middle = low + ((i64::from(high) - i64::from(low) + 1) / 2) as i32;
            let effects = claim.payment(middle)?;
            let reserved_fits = if income {
                self.preview(world, &effects)?.is_some()
            } else {
                crate::storage::fits(world, &self.used, &effects)
            };
            if crate::storage::fits(world, &execution.stored, &effects) && reserved_fits {
                low = middle;
            } else {
                high = middle - 1;
            }
        }
        Ok(low)
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
