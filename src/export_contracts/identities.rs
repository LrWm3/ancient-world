//! Persistent source identities survive removal of inactive market evidence.
use crate::civilization::History;
use anyhow::{ensure, Result};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Identity {
    pub id: u64,
    pub buyer: u32,
    pub seller: u32,
    pub good: u32,
    /// Identity assignment date, not a fabricated historical order date.
    pub assigned_month: u32,
    pub legacy_baseline: bool,
    #[serde(default)]
    pub retired_month: Option<u32>,
}
impl History {
    pub(crate) fn create_export_identity(
        &mut self,
        buyer: u32,
        seller: u32,
        good: u32,
        legacy_baseline: bool,
    ) -> u64 {
        let id = self.export_identities.len() as u64;
        self.export_identities.push(Identity {
            id,
            buyer,
            seller,
            good,
            assigned_month: self.month,
            legacy_baseline,
            retired_month: None,
        });
        id
    }
    /// Old archives retain their goods and escrow. Only missing identities are
    /// assigned, in stable existing order, before evidence may expire this month.
    pub fn ensure_export_contract_identities(&mut self) -> Result<()> {
        self.validate_export_identities()?;
        for index in 0..self.export_contracts.len() {
            let contract = &self.export_contracts[index];
            if contract.id.is_none() {
                let id = self.create_export_identity(
                    contract.buyer,
                    contract.seller,
                    contract.good,
                    true,
                );
                self.export_contracts[index].id = Some(id);
            }
        }
        Ok(())
    }
    pub fn validate_export_identities(&self) -> Result<()> {
        for (index, identity) in self.export_identities.iter().enumerate() {
            ensure!(
                identity.id == index as u64
                    && identity.assigned_month <= self.month
                    && identity
                        .retired_month
                        .is_none_or(|m| m >= identity.assigned_month && m <= self.month)
                    && identity.buyer != identity.seller
                    && (identity.buyer as usize) < self.sites.len()
                    && (identity.seller as usize) < self.sites.len()
                    && self
                        .economy_catalog
                        .as_ref()
                        .is_some_and(|c| (identity.good as usize) < c.goods.len()),
                "invalid export source identity"
            );
        }
        let mut active = BTreeSet::new();
        for contract in &self.export_contracts {
            if let Some(id) = contract.id {
                let identity = self.export_identities.get(id as usize);
                ensure!(
                    active.insert(id)
                        && identity.is_some_and(|i| i.id == id
                            && i.retired_month.is_none()
                            && i.buyer == contract.buyer
                            && i.seller == contract.seller
                            && i.good == contract.good),
                    "invalid active export source reference"
                );
            }
        }
        Ok(())
    }
}
