//! Ranked native/accepted-tender allocation shared by dated claim adapters.
use crate::{finance, model::*};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Default)]
pub(crate) struct CollectionGrants {
    pub(crate) native: BTreeMap<finance::ContractId, i32>,
    pub(crate) alternative: BTreeMap<finance::ContractId, (ResourceId, i32, i32)>,
}
impl CollectionGrants {
    pub(crate) fn get(&self, id: &finance::ContractId) -> Option<&i32> {
        self.native.get(id)
    }
    pub(crate) fn claim_units(&self, id: &finance::ContractId) -> i32 {
        self.native.get(id).copied().unwrap_or(0)
            + self
                .alternative
                .get(id)
                .map_or(0, |(_, paid, rate)| paid / rate)
    }
}

pub(crate) fn allocate(
    world: &World,
    state: &State,
    execution: &finance::Execution,
    protected: &BTreeMap<Account, i32>,
    requests: &[finance::CollectionRequest],
) -> Result<Option<CollectionGrants>, String> {
    if world.collection_policy == finance::CollectionPolicy::Stable {
        return Ok(None);
    }
    let currencies: BTreeSet<_> = world
        .activities
        .coin_payments
        .values()
        .map(|a| a.resource)
        .collect();
    if crate::commitments::active(world, state).any(|a| {
        world.activities.coin_payments.contains_key(&a.id)
            && currencies.contains(&a.payment.resource)
    }) {
        return Err("alternative payment routes cannot form currency chains".into());
    }
    // Preferred alternatives compete with other claims on that currency at the
    // same rank. Only their remaining native fallback runs after that allocation.
    let preferred: BTreeSet<_> = crate::commitments::active(world, state)
        .filter(|a| crate::commitments::preferred_alternative(world, state, a).is_some())
        .map(|a| finance::ContractId::Land(a.id))
        .collect();
    enum TenderPass {
        NativeGoods,
        Currency,
        PreferredNativeFallback,
    }
    let mut window = execution.clone();
    for (account, amount) in protected {
        window.protect(*account, *amount);
    }
    let mut result = CollectionGrants::default();
    let ranks: BTreeSet<_> = requests.iter().map(|r| r.rank).collect();
    for rank in ranks {
        for stage in [
            TenderPass::NativeGoods,
            TenderPass::Currency,
            TenderPass::PreferredNativeFallback,
        ] {
            let mut pass = vec![];
            let mut lots = BTreeMap::new();
            let mut alternatives = BTreeSet::new();
            for r in requests.iter().filter(|r| r.rank == rank) {
                let currency = currencies.contains(&r.claim.transfer.amount.resource);
                let alternative_first = preferred.contains(&r.contract);
                let native = match stage {
                    TenderPass::NativeGoods => !currency && !alternative_first,
                    TenderPass::Currency => currency && !alternative_first,
                    TenderPass::PreferredNativeFallback => alternative_first,
                };
                if native {
                    let mut request = r.clone();
                    if alternative_first {
                        request.claim.transfer.amount.quantity =
                            r.claim.outstanding() - result.claim_units(&r.contract);
                        request.claim.settled = 0;
                    }
                    pass.push(request);
                }
                if matches!(stage, TenderPass::Currency)
                    && let finance::ContractId::Land(id) = r.contract
                    && let Some(tender) = world.activities.coin_payments.get(&id)
                    && crate::recovery::active(world, &state.credit, r.claim.transfer.from)
                        .is_none()
                {
                    let mut alt = r.clone();
                    let remaining = r.claim.outstanding()
                        - result.native.get(&r.contract).copied().unwrap_or(0);
                    alt.claim.transfer.amount = Amount::new(
                        tender.resource,
                        remaining
                            .checked_mul(tender.coins_per_unit)
                            .ok_or("alternative claim overflow")?,
                    );
                    alt.claim.settled = 0;
                    lots.insert(r.contract, tender.coins_per_unit);
                    alternatives.insert(r.contract);
                    pass.push(alt);
                }
            }
            let grants = finance::proportional_lots(
                world,
                state.month,
                &window,
                &BTreeMap::new(),
                &pass,
                &lots,
            )?;
            for r in pass {
                let amount = grants[&r.contract];
                if amount > 0 {
                    window.exchange(
                        world,
                        &[finance::Transfer {
                            amount: Amount::new(r.claim.transfer.amount.resource, amount),
                            ..r.claim.transfer.clone()
                        }],
                    )?;
                }
                if alternatives.contains(&r.contract) {
                    result.alternative.insert(
                        r.contract,
                        (r.claim.transfer.amount.resource, amount, lots[&r.contract]),
                    );
                } else {
                    result.native.insert(r.contract, amount);
                }
            }
        }
    }
    Ok(Some(result))
}
