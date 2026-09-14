//! Live experimental policies shared by the registry, CLI aliases and library.
use super::{System, Systems};
use crate::{
    civilization::History,
    gpu::{Generator, Stage},
};
use anyhow::{ensure, Context, Result};

impl History {
    /// Actual archived state; registry defaults are not evidence of activation.
    pub fn registered_policy_enabled(&self, system: System) -> Option<bool> {
        Some(match system {
            System::CouncilCredit => self.credit.council_policy.enabled,
            System::InstitutionCreditLenders => self.credit.council_policy.institution_lenders,
            System::InstitutionCreditOperatingReserve => {
                self.credit.council_policy.institution_reserve
                    == crate::credit::councils::InstitutionReserve::AnnualOperatingCosts
            }
            System::CommercialCredit => self.credit.commercial_policy.enabled,
            System::ServiceOrderCredit => self.credit.commercial_policy.service_orders,
            System::ServiceOrderProcurement => self
                .enterprises
                .as_ref()
                .is_some_and(|e| e.procurement.enabled),
            System::ContractWorkshopStaffing => self
                .enterprises
                .as_ref()
                .is_some_and(|e| e.procurement.contract_staffing),
            System::DemandWorkshopStaffing => self
                .enterprises
                .as_ref()
                .is_some_and(|e| e.procurement.demand_staffing),
            System::HouseholdEstateInheritance => self
                .society
                .as_ref()
                .and_then(|s| s.household_economy.as_ref())
                .is_some_and(|e| e.inheritance.enabled),
            System::HouseholdClothing => self
                .society
                .as_ref()
                .and_then(|s| s.household_economy.as_ref())
                .is_some_and(|e| e.clothing_enabled),
            System::HouseholdWealthTax => self
                .society
                .as_ref()
                .and_then(|s| s.household_economy.as_ref())
                .is_some_and(|e| e.wealth_tax.enabled),
            System::CouncilWelfareReserves => self
                .society
                .as_ref()
                .and_then(|s| s.household_economy.as_ref())
                .is_some_and(|e| {
                    e.council_allocation
                        == crate::household_economy::council_allocation::Policy::NeedsFirst
                }),
            System::NeedsBasedFood => self
                .society
                .as_ref()
                .and_then(|s| s.household_economy.as_ref())
                .is_some_and(|e| e.needs_based_food),
            System::GradualNutrition => self
                .society
                .as_ref()
                .and_then(|s| s.household_economy.as_ref())
                .is_some_and(|e| e.gradual_nutrition),
            System::InstitutionWorkingCore => self
                .culture
                .as_ref()
                .is_some_and(|c| c.institution_working_core),
            System::InstitutionOperatingFunding => self.culture.as_ref().is_some_and(|c| {
                c.institution_funding == crate::institution_funding::Policy::Operating
            }),
            System::MunicipalFoodRelief => self
                .society
                .as_ref()
                .and_then(|s| s.household_economy.as_ref())
                .is_some_and(|e| e.municipal_relief.enabled),
            System::FoodSolidarity => self
                .society
                .as_ref()
                .and_then(|s| s.household_economy.as_ref())
                .is_some_and(|e| e.solidarity.enabled),
            System::DemographicAudit => self.demographic_audit.is_some(),
            System::StagedHarbors => self.shipping.as_ref().is_some_and(|s| s.staged_harbors),
            System::PracticalResearch => {
                self.culture.as_ref().is_some_and(|c| c.practical_research)
            }
            System::HouseholdEstateReclamation => self
                .society
                .as_ref()
                .and_then(|s| s.household_economy.as_ref())
                .is_some_and(|e| e.reclamation.enabled),
            System::AbandonedStockRecovery => {
                self.society.as_ref().is_some_and(|s| s.stock_recovery)
            }
            System::NamedOfficeService => {
                self.offices.as_ref().is_some_and(|o| o.service.is_some())
            }
            System::ExportDefaultRecovery => self.credit.export_recovery.policy.enabled,
            System::SharedIssuance => self.credit.issuance.enabled,
            System::DeliveryPaidExports => {
                self.export_payment_timing == crate::export_contracts::payments::Timing::Delivery
            }
            _ => return None,
        })
    }
    fn set_registered_policy(&mut self, system: System, enabled: bool) -> Result<()> {
        match system {
            System::CouncilCredit => {
                self.credit.council_policy.enabled = enabled;
            }
            System::InstitutionCreditLenders => {
                self.credit.council_policy.institution_lenders = enabled;
            }
            System::InstitutionCreditOperatingReserve => {
                self.credit.council_policy.institution_reserve = if enabled {
                    crate::credit::councils::InstitutionReserve::AnnualOperatingCosts
                } else {
                    crate::credit::councils::InstitutionReserve::CouncilFloor
                };
            }
            System::CommercialCredit => {
                self.credit.commercial_policy.enabled = enabled;
            }
            System::ServiceOrderCredit => {
                self.credit.commercial_policy.service_orders = enabled;
            }
            System::ServiceOrderProcurement => {
                self.enterprises
                    .as_mut()
                    .context("service procurement requires enterprises")?
                    .procurement
                    .enabled = enabled;
            }
            System::ContractWorkshopStaffing => {
                self.enterprises
                    .as_mut()
                    .context("contract workshop staffing requires enterprises")?
                    .procurement
                    .contract_staffing = enabled;
            }
            System::DemandWorkshopStaffing => {
                self.enterprises
                    .as_mut()
                    .context("demand workshop staffing requires enterprises")?
                    .procurement
                    .demand_staffing = enabled;
            }
            System::HouseholdEstateInheritance => {
                self.society
                    .as_mut()
                    .and_then(|s| s.household_economy.as_mut())
                    .context("estate inheritance requires household accounts")?
                    .inheritance
                    .enabled = enabled;
            }
            System::HouseholdClothing => {
                self.society
                    .as_mut()
                    .and_then(|s| s.household_economy.as_mut())
                    .context("clothing requires household accounts")?
                    .clothing_enabled = enabled;
            }
            System::HouseholdWealthTax => {
                self.society
                    .as_mut()
                    .and_then(|s| s.household_economy.as_mut())
                    .context("wealth tax requires household accounts")?
                    .wealth_tax
                    .enabled = enabled;
            }
            System::CouncilWelfareReserves => {
                self.society
                    .as_mut()
                    .and_then(|s| s.household_economy.as_mut())
                    .context("welfare requires household accounts")?
                    .council_allocation = if enabled {
                    crate::household_economy::council_allocation::Policy::NeedsFirst
                } else {
                    crate::household_economy::council_allocation::Policy::Existing
                };
            }
            System::NeedsBasedFood => {
                self.society
                    .as_mut()
                    .and_then(|s| s.household_economy.as_mut())
                    .context("food policy requires household accounts")?
                    .needs_based_food = enabled;
            }
            System::GradualNutrition => {
                ensure!(
                    !enabled || !self.individual_demography_enabled(),
                    "gradual nutrition currently requires aggregate demography"
                );
                self.society
                    .as_mut()
                    .and_then(|s| s.household_economy.as_mut())
                    .context("food policy requires household accounts")?
                    .gradual_nutrition = enabled;
            }
            System::InstitutionWorkingCore => {
                self.culture
                    .as_mut()
                    .context("institution working core requires culture")?
                    .institution_working_core = enabled;
            }
            System::InstitutionOperatingFunding => {
                self.culture
                    .as_mut()
                    .context("institution funding requires culture")?
                    .institution_funding = if enabled {
                    crate::institution_funding::Policy::Operating
                } else {
                    crate::institution_funding::Policy::Legacy
                };
            }
            System::MunicipalFoodRelief => {
                self.society
                    .as_mut()
                    .and_then(|s| s.household_economy.as_mut())
                    .context("municipal relief requires household accounts")?
                    .municipal_relief
                    .enabled = enabled;
            }
            System::FoodSolidarity => {
                self.society
                    .as_mut()
                    .and_then(|s| s.household_economy.as_mut())
                    .context("solidarity requires household accounts")?
                    .solidarity
                    .enabled = enabled;
            }
            System::DemographicAudit => {
                let h = self;
                anyhow::ensure!(
            !enabled
                || (h.society.is_some()
                    && h.resolution.is_none()
                    && !h.individual_demography_enabled()),
            "demographic audit requires GPU aggregate demography without resolution overrides"
        );
                if enabled {
                    h.demographic_audit.get_or_insert_with(Default::default);
                } else {
                    h.demographic_audit = None;
                }
            }
            System::StagedHarbors => {
                self.shipping
                    .as_mut()
                    .context("staged harbors require shipping")?
                    .staged_harbors = enabled;
            }
            System::PracticalResearch => {
                self.culture
                    .as_mut()
                    .context("research requires culture")?
                    .practical_research = enabled;
            }
            System::HouseholdEstateReclamation => {
                self.society
                    .as_mut()
                    .and_then(|s| s.household_economy.as_mut())
                    .context("estate reclamation requires household accounts")?
                    .reclamation
                    .enabled = enabled;
            }
            System::AbandonedStockRecovery => {
                self.society
                    .as_mut()
                    .context("stock recovery requires society")?
                    .stock_recovery = enabled;
            }
            System::NamedOfficeService => {
                self.set_office_service(enabled)?;
            }
            System::ExportDefaultRecovery => {
                self.credit.export_recovery.policy.enabled = enabled;
            }
            System::SharedIssuance => {
                self.configure_shared_issuance(enabled)?;
            }
            System::DeliveryPaidExports => {
                self.export_payment_timing = if enabled {
                    crate::export_contracts::payments::Timing::Delivery
                } else {
                    crate::export_contracts::payments::Timing::Dispatch
                };
            }
            _ => anyhow::bail!("{} is not a live registered policy", system.label()),
        }
        Ok(())
    }
}
impl Generator {
    /// Change only explicitly requested experimental policies. Preserve every
    /// other saved switch and all outstanding contracts. Stage on a copy so a
    /// failed prerequisite or pending-work check cannot partially apply a batch.
    pub fn apply_registered_policies(&mut self, options: &Systems) -> Result<()> {
        options.validate()?;
        self.validate_living_boundary()?;
        ensure!(
            self.progress.stage == Stage::Boundary,
            "policy changes require a completed boundary"
        );
        let mut history = self
            .civilizations
            .as_ref()
            .context("policies require history")?
            .clone();
        for (&system, &enabled) in &options.overrides {
            if !system.is_registered_policy() {
                continue;
            }
            if history.registered_policy_enabled(system) == Some(enabled) {
                continue;
            }
            // Policy prerequisites refer to real baselines, not hypothetical
            // default-on startup choices. Missing parents are never auto-created.
            if enabled {
                for parent in system.requires() {
                    let present = match parent {
                        System::Society => history.society.is_some(),
                        System::Shipping => history.shipping.is_some(),
                        System::Offices => history.offices.is_some(),
                        System::Enterprises => history.enterprises.is_some(),
                        System::Workshops => history
                            .economy_catalog
                            .as_ref()
                            .is_some_and(|c| c.production.workshops),
                        System::ExportContracts => history
                            .economy_catalog
                            .as_ref()
                            .is_some_and(|c| c.production.export_contracts),
                        _ => false,
                    };
                    ensure!(present, "{} requires {}", system.label(), parent.label());
                }
            }
            history.set_registered_policy(system, enabled)?;
        }
        self.civilizations = Some(history);
        self.refresh_registered_policies();
        Ok(())
    }
    /// Import policy state from old saves, which predate registry entries.
    /// Does not enable anything, reset a ledger or authorize issuance.
    pub(crate) fn refresh_registered_policies(&mut self) {
        if let Some(history) = &self.civilizations {
            for &system in System::REGISTERED_POLICIES {
                self.config
                    .systems
                    .overrides
                    .insert(system, history.registered_policy_enabled(system).unwrap());
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    #[ignore = "requires a GPU; registry policy application and archive continuation"]
    fn registered_policies_apply_preserve_and_resume() {
        let gpu = pollster::block_on(crate::gpu::ContextGpu::headless()).unwrap();
        let mut g = Generator::new(
            gpu.clone(),
            crate::config::Config {
                resolution: 32,
                ecology_resolution: 32,
                ..Default::default()
            },
            crate::catalog::Catalog::bundled().unwrap(),
        )
        .unwrap();
        g.found_civilizations(3).unwrap();
        // A loaded-policy intervention must not silently found missing councils.
        let before = serde_json::to_value(&g.civilizations).unwrap();
        let missing = Systems {
            overrides: [(System::CouncilCredit, true)].into(),
        };
        assert!(g.apply_registered_policies(&missing).is_err());
        assert_eq!(serde_json::to_value(&g.civilizations).unwrap(), before);
        g.apply_systems(&Systems::default()).unwrap();
        for &system in System::REGISTERED_POLICIES {
            assert_eq!(
                g.civilizations
                    .as_ref()
                    .unwrap()
                    .registered_policy_enabled(system),
                Some(false)
            );
            for enabled in [true, false] {
                g.apply_registered_policies(&Systems {
                    overrides: [(system, enabled)].into(),
                })
                .unwrap();
                assert_eq!(
                    g.civilizations
                        .as_ref()
                        .unwrap()
                        .registered_policy_enabled(system),
                    Some(enabled),
                    "{}",
                    system.label()
                );
                assert_eq!(g.config.systems.enabled(system), enabled);
            }
        }
        // Keep independent policies when another explicit intervention is applied.
        g.apply_registered_policies(&Systems {
            overrides: [(System::HouseholdWealthTax, true)].into(),
        })
        .unwrap();
        g.apply_registered_policies(&Systems {
            overrides: [(System::PracticalResearch, true)].into(),
        })
        .unwrap();
        assert!(g.config.systems.enabled(System::HouseholdWealthTax));
        // Disabling/re-enabling issuance must preserve its finite authorization.
        let schedule =
            serde_json::to_value(&g.civilizations.as_ref().unwrap().credit.issuance.schedule)
                .unwrap();
        g.apply_registered_policies(&Systems {
            overrides: [(System::SharedIssuance, true)].into(),
        })
        .unwrap();
        assert_eq!(
            serde_json::to_value(&g.civilizations.as_ref().unwrap().credit.issuance.schedule)
                .unwrap(),
            schedule
        );
        g.apply_registered_policies(&Systems {
            overrides: [(System::SharedIssuance, false)].into(),
        })
        .unwrap();
        // Old archives have policy fields but no registry entries. Import those
        // fields without changing history; then compare continued monthly state.
        g.advance_history(1).unwrap();
        g.config
            .systems
            .overrides
            .retain(|s, _| !s.is_registered_policy());
        let path =
            std::env::temp_dir().join(format!("registered-policies-{}.world", std::process::id()));
        g.save(&path).unwrap();
        let mut resumed = Generator::load(gpu, &path).unwrap();
        std::fs::remove_file(path).unwrap();
        assert_eq!(
            serde_json::to_value(&g.civilizations).unwrap(),
            serde_json::to_value(&resumed.civilizations).unwrap()
        );
        assert!(resumed.config.systems.enabled(System::HouseholdWealthTax));
        assert!(resumed.config.systems.enabled(System::PracticalResearch));
        g.refresh_registered_policies();
        g.advance_history(2).unwrap();
        resumed.advance_history(1).unwrap();
        resumed.advance_history(1).unwrap();
        assert_eq!(
            serde_json::to_value(&g.civilizations).unwrap(),
            serde_json::to_value(&resumed.civilizations).unwrap()
        );
        assert_eq!(g.config.systems, resumed.config.systems);
        // A failing later member of a policy batch must not commit earlier edits.
        g.civilizations.as_mut().unwrap().culture = None;
        let before = serde_json::to_value(&g.civilizations).unwrap();
        let config_before = g.config.systems.clone();
        assert!(g
            .apply_registered_policies(&Systems {
                overrides: [
                    (System::CouncilCredit, true),
                    (System::PracticalResearch, true)
                ]
                .into()
            })
            .is_err());
        assert_eq!(serde_json::to_value(&g.civilizations).unwrap(), before);
        assert_eq!(g.config.systems, config_before);
    }
}
