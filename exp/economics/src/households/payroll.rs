//! Optional assistance at the existing before-Close allocation boundary.
//! Earned claims remain the member's debt; no future wages are funded here.
use super::*;

pub(super) fn requests(world: &World, state: &State) -> Result<Vec<Request>, String> {
    let mut result = vec![];
    for h in &world.households {
        if !h.governance.charter.support_member_wages || !market::active(world, state, h.agent) {
            continue;
        }
        for member in members(h, state) {
            for (resource, owed) in crate::employment::claims(state, member)? {
                let quantity =
                    i32::try_from((owed - i128::from(state.balance(member, resource))).max(0))
                        .map_err(|_| "household wage support overflow")?;
                if quantity == 0 {
                    continue;
                }
                let rank = state
                    .employment
                    .earned
                    .iter()
                    .filter(|(_, e)| {
                        e.claim.transfer.from == member
                            && e.claim.transfer.amount.resource == resource
                            && e.claim.outstanding() > 0
                    })
                    .filter_map(|((id, _), _)| world.employment.iter().find(|t| t.id == *id))
                    .map(|t| t.rank)
                    .min()
                    .ok_or("missing wage claim rank")?;
                result.push(Request {
                    household: h.agent,
                    member,
                    resource,
                    quantity,
                    minimum: 1,
                    individual_benefit: PAYMENT_BENEFIT,
                    collective_benefit: PAYMENT_BENEFIT,
                    sequence: result.len() as u64,
                    purpose: Purpose::WageSupport {
                        rank,
                        policy: h.governance.charter.debt_support,
                    },
                });
            }
        }
    }
    Ok(result)
}
