//! Application startup choices. Low-level baseline APIs remain available for diagnostic fixtures.
use crate::gpu::Generator;
use anyhow::{ensure, Result};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(
    Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, clap::ValueEnum,
)]
#[serde(rename_all = "kebab-case")]
pub enum System {
    OccupationalPayroll,
    NegotiatedAutonomy,
    CivicPetitions,
    AutomaticExpeditions,
    Society,
    Politics,
    Governance,
    Offices,
    Shipping,
    Expeditions,
    Discoveries,
    LivingWorld,
    SharedResources,
    MineralProcessing,
    AlloyProcessing,
    EnvironmentalReturns,
    SocialIndicators,
    Enterprises,
    DiversifiedFarming,
    AdaptivePrices,
    NetworkTrade,
    Fisheries,
    AdaptiveFishing,
    PrimitiveFishingGear,
    FishingOpportunityCost,
    Production,
    AdaptiveLabor,
    FoodSecurityLabor,
    FoodSecurityMaintenance,
    ReplacementToolJobs,
    ToolmakingExpertise,
    Workshops,
    PersistentStorage,
    PersistentHousing,
    Waterworks,
    WaterworksRepairPriority,
    SpecializedWorkshops,
    ExportContracts,
    SupplierProfitability,
}
impl System {
    pub const ALL: &'static [Self] = &[
        Self::OccupationalPayroll,
        Self::NegotiatedAutonomy,
        Self::CivicPetitions,
        Self::AutomaticExpeditions,
        Self::Society,
        Self::Politics,
        Self::Governance,
        Self::Offices,
        Self::Shipping,
        Self::Expeditions,
        Self::Discoveries,
        Self::LivingWorld,
        Self::SharedResources,
        Self::MineralProcessing,
        Self::AlloyProcessing,
        Self::EnvironmentalReturns,
        Self::SocialIndicators,
        Self::Enterprises,
        Self::DiversifiedFarming,
        Self::AdaptivePrices,
        Self::NetworkTrade,
        Self::Fisheries,
        Self::AdaptiveFishing,
        Self::PrimitiveFishingGear,
        Self::FishingOpportunityCost,
        Self::Production,
        Self::AdaptiveLabor,
        Self::FoodSecurityLabor,
        Self::FoodSecurityMaintenance,
        Self::ReplacementToolJobs,
        Self::ToolmakingExpertise,
        Self::Workshops,
        Self::PersistentStorage,
        Self::PersistentHousing,
        Self::Waterworks,
        Self::WaterworksRepairPriority,
        Self::SpecializedWorkshops,
        Self::ExportContracts,
        Self::SupplierProfitability,
    ];
    pub fn label(self) -> String {
        serde_json::to_value(self)
            .unwrap()
            .as_str()
            .unwrap()
            .to_owned()
    }
    fn requires(self) -> &'static [Self] {
        match self {
            Self::OccupationalPayroll => &[Self::Society],
            Self::CivicPetitions => &[Self::Governance],
            Self::NegotiatedAutonomy => &[Self::Governance],
            Self::AutomaticExpeditions => &[Self::Expeditions],
            Self::Politics => &[Self::Society],
            Self::Governance => &[Self::Politics],
            Self::Offices => &[Self::Governance],
            Self::Shipping => &[Self::Society],
            Self::Expeditions => &[Self::Shipping, Self::Governance],
            Self::Discoveries => &[Self::Expeditions],
            Self::MineralProcessing => &[Self::SharedResources],
            Self::AlloyProcessing => &[Self::MineralProcessing],
            Self::EnvironmentalReturns => &[Self::LivingWorld],
            Self::SocialIndicators => &[Self::Society],
            Self::Enterprises => &[Self::Society],
            Self::Fisheries => &[Self::DiversifiedFarming],
            Self::AdaptiveFishing => &[Self::Fisheries],
            Self::PrimitiveFishingGear => &[Self::AdaptiveFishing],
            Self::FishingOpportunityCost => &[Self::AdaptiveFishing],
            Self::SpecializedWorkshops => &[Self::Workshops],
            Self::SupplierProfitability => &[Self::ExportContracts],
            Self::AdaptiveLabor => &[Self::Production],
            Self::FoodSecurityLabor => &[Self::AdaptiveLabor],
            Self::FoodSecurityMaintenance => &[Self::Production],
            Self::ReplacementToolJobs => &[Self::Production],
            Self::ToolmakingExpertise => &[Self::Production],
            Self::Workshops => &[Self::Production],
            Self::PersistentStorage => &[Self::Production],
            Self::PersistentHousing => &[Self::Production],
            Self::Waterworks => &[Self::Production],
            Self::WaterworksRepairPriority => &[Self::Waterworks],
            Self::ExportContracts => &[Self::Production],
            _ => &[],
        }
    }
}
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Systems {
    /// Missing entries default on. Explicitly disabled prerequisites suppress descendants.
    pub overrides: BTreeMap<System, bool>,
}
impl Systems {
    pub fn enabled(&self, s: System) -> bool {
        self.overrides.get(&s).copied().unwrap_or(true)
            && s.requires().iter().all(|p| self.enabled(*p))
    }
    /// Interactive selection enables prerequisites or clears contradictory child overrides.
    pub fn select(&mut self, s: System, enabled: bool) {
        if enabled {
            for &parent in s.requires() {
                self.select(parent, true);
            }
        }
        self.overrides.insert(s, enabled);
        if !enabled {
            let snapshot = self.clone();
            self.overrides.retain(|s, v| !*v || snapshot.enabled(*s));
        }
    }
    fn configure_catalog(&self, catalog: &mut crate::economy::EconomyCatalog) {
        use System::*;
        catalog.production.enabled = self.enabled(Production);
        catalog.production.adaptive_labor = self.enabled(AdaptiveLabor);
        catalog.production.food_security_labor = self.enabled(FoodSecurityLabor);
        catalog.production.food_security_maintenance = self.enabled(FoodSecurityMaintenance);
        catalog.production.replacement_tool_jobs = self.enabled(ReplacementToolJobs);
        catalog.production.toolmaking_expertise = self.enabled(ToolmakingExpertise);
        catalog.production.workshops = self.enabled(Workshops);
        catalog.production.persistent_storage = self.enabled(PersistentStorage);
        catalog.production.persistent_housing = self.enabled(PersistentHousing);
        catalog.production.waterworks = self.enabled(Waterworks);
        catalog.production.waterworks_repair_priority = self.enabled(WaterworksRepairPriority);
        catalog.production.specialized_workshops = self.enabled(SpecializedWorkshops);
        catalog.production.export_contracts = self.enabled(ExportContracts);
        catalog.production.supplier_profitability = self.enabled(SupplierProfitability);
        catalog.market.adaptive_prices = self.enabled(AdaptivePrices);
        catalog.market.network_trade = self.enabled(NetworkTrade);
        if let Some(a) = &mut catalog.agriculture {
            a.fisheries_enabled = self.enabled(Fisheries);
            a.fishery.adaptive = self.enabled(AdaptiveFishing);
            a.fishery.primitive_gear = self.enabled(PrimitiveFishingGear);
            a.fishery.opportunity_cost = self.enabled(FishingOpportunityCost);
        }
    }
    pub fn validate(&self) -> Result<()> {
        for (&s, &enabled) in &self.overrides {
            ensure!(
                !enabled || self.enabled(s),
                "{} explicitly enabled but a prerequisite is disabled",
                s.label()
            );
        }
        Ok(())
    }
}
impl Generator {
    /// Configure a new history, or add missing baselines to an existing history.
    /// Never removes established baselines: their inventories must remain accounted for.
    pub fn apply_systems(&mut self, options: &Systems) -> Result<()> {
        use System::*;
        options.validate()?;
        self.validate_living_boundary()?;
        ensure!(
            self.progress.stage == crate::gpu::Stage::Boundary,
            "systems require a completed boundary"
        );
        let h = self
            .civilizations
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("found civilizations first"))?;
        let existing = [
            (Society, h.society.is_some()),
            (Politics, h.politics.is_some()),
            (Governance, h.governance.is_some()),
            (Offices, h.offices.is_some()),
            (Shipping, h.shipping.is_some()),
            (Expeditions, h.expeditions.is_some()),
            (
                Discoveries,
                h.expeditions
                    .as_ref()
                    .is_some_and(|x| x.discoveries.is_some()),
            ),
            (LivingWorld, h.living.is_some()),
            (SharedResources, h.resources.is_some()),
            (
                MineralProcessing,
                h.resources
                    .as_ref()
                    .is_some_and(|r| !r.mineral_catalog.is_empty()),
            ),
            (
                AlloyProcessing,
                h.resources.as_ref().is_some_and(|r| r.alloy_processing),
            ),
        ];
        for (s, exists) in existing {
            ensure!(
                !exists || options.enabled(s),
                "{} already has a baseline; disable it before founding a new history",
                s.label()
            );
        }
        if h.version == 1 {
            self.upgrade_economy()?;
        }
        let mut catalog = self
            .civilizations
            .as_ref()
            .unwrap()
            .economy_catalog
            .clone()
            .unwrap();
        options.configure_catalog(&mut catalog);
        self.configure_economy(catalog)?;
        self.set_diversified_farming(options.enabled(DiversifiedFarming))?;
        for (system, exists) in existing {
            if exists || !options.enabled(system) {
                continue;
            }
            match system {
                Society => self.enable_society()?,
                Politics => self.enable_politics()?,
                Governance => self.enable_governance()?,
                Offices => self.enable_offices()?,
                Shipping => self.enable_shipping()?,
                Expeditions => self.enable_expeditions()?,
                Discoveries => self.enable_discoveries()?,
                LivingWorld => self.enable_living_history()?,
                SharedResources => self.enable_shared_resources()?,
                MineralProcessing => self.enable_mineral_processing()?,
                AlloyProcessing => self.enable_alloy_processing()?,
                _ => unreachable!(),
            }
        }
        if options.enabled(Society) {
            if options.enabled(SocialIndicators) {
                if self
                    .civilizations
                    .as_ref()
                    .unwrap()
                    .society
                    .as_ref()
                    .unwrap()
                    .indicators
                    .is_none()
                {
                    self.enable_social_indicators()?;
                }
            } else {
                self.civilizations
                    .as_mut()
                    .unwrap()
                    .society
                    .as_mut()
                    .unwrap()
                    .indicators = None;
            }
            self.set_enterprises(options.enabled(Enterprises))?;
            self.set_occupational_payroll(options.enabled(OccupationalPayroll))?;
        }
        if options.enabled(Governance) {
            self.set_negotiated_autonomy(options.enabled(NegotiatedAutonomy))?;
            self.civilizations
                .as_mut()
                .unwrap()
                .governance
                .as_mut()
                .unwrap()
                .petitions_enabled = options.enabled(CivicPetitions);
        }
        if let Some(x) = self.civilizations.as_mut().unwrap().expeditions.as_mut() {
            x.rules.automatic = options.enabled(AutomaticExpeditions);
        }
        if options.enabled(EnvironmentalReturns) {
            self.enable_environmental_returns()?;
        } else if let Some(l) = self.civilizations.as_mut().unwrap().living.as_mut() {
            l.environmental_returns = false;
        }
        self.config.systems = options.clone();
        Ok(())
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn each_individual_disable_produces_a_valid_catalog() {
        for &system in System::ALL {
            let mut options = Systems::default();
            options.overrides.insert(system, false);
            let mut catalog = crate::economy::EconomyCatalog::bundled().unwrap();
            options.configure_catalog(&mut catalog);
            catalog
                .validate()
                .unwrap_or_else(|e| panic!("{}: {e}", system.label()));
        }
        let mut options = Systems::default();
        options.select(System::Society, false);
        options.select(System::Discoveries, true);
        options.validate().unwrap();
        assert!(options.enabled(System::Society));
        options.select(System::Shipping, false);
        assert!(!options.enabled(System::Discoveries));
        options.validate().unwrap();
    }
    #[test]
    fn defaults_dependencies_and_conflicts() {
        let mut options = Systems::default();
        assert!(System::ALL.iter().all(|s| options.enabled(*s)));
        options.overrides.insert(System::Society, false);
        assert!(!options.enabled(System::Discoveries));
        assert!(options.enabled(System::LivingWorld));
        options.validate().unwrap();
        options.overrides.insert(System::Expeditions, true);
        assert!(options.validate().is_err());
        let restored: Systems = toml::from_str(&toml::to_string(&options).unwrap()).unwrap();
        assert_eq!(restored, options);
    }
}

#[cfg(test)]
mod gpu_tests {
    use super::*;
    #[test]
    #[ignore = "requires a GPU"]
    fn all_on_and_disabled_startups_resume() {
        let gpu = pollster::block_on(crate::gpu::ContextGpu::headless()).unwrap();
        for disabled in [false, true] {
            let mut g = Generator::new(
                gpu.clone(),
                crate::config::Config {
                    resolution: 32,
                    ecology_resolution: 16,
                    ..Default::default()
                },
                crate::catalog::Catalog::bundled().unwrap(),
            )
            .unwrap();
            g.found_civilizations(3).unwrap();
            let mut options = Systems::default();
            if disabled {
                for &s in System::ALL {
                    options.overrides.insert(s, false);
                }
            }
            g.apply_systems(&options).unwrap();
            let h = g.civilizations.as_ref().unwrap();
            assert_eq!(h.society.is_some(), !disabled);
            assert_eq!(h.living.is_some(), !disabled);
            assert_eq!(h.resources.is_some(), !disabled);
            assert_eq!(h.offices.is_some(), !disabled);
            assert_eq!(h.expeditions.is_some(), !disabled);
            assert_eq!(
                h.economy_catalog
                    .as_ref()
                    .unwrap()
                    .production
                    .toolmaking_expertise,
                !disabled
            );
            assert_eq!(
                h.economy_catalog.as_ref().unwrap().market.adaptive_prices,
                !disabled
            );
            if !disabled {
                assert!(h.resources.as_ref().unwrap().alloy_processing);
                assert!(h.expeditions.as_ref().unwrap().discoveries.is_some());
                assert!(h.living.as_ref().unwrap().environmental_returns);
                let before = serde_json::to_value(h).unwrap();
                let mut invalid = options.clone();
                invalid.overrides.insert(System::Society, false);
                assert!(g.apply_systems(&invalid).is_err());
                assert_eq!(
                    before,
                    serde_json::to_value(g.civilizations.as_ref().unwrap()).unwrap()
                );
            }
            g.advance_history(1).unwrap();
            let path = std::env::temp_dir()
                .join(format!("systems-{}-{disabled}.world", std::process::id()));
            g.save(&path).unwrap();
            let mut resumed = Generator::load(gpu.clone(), &path).unwrap();
            std::fs::remove_file(path).unwrap();
            g.advance_history(2).unwrap();
            resumed.advance_history(1).unwrap();
            resumed.advance_history(1).unwrap();
            assert_eq!(
                serde_json::to_value(&g.civilizations).unwrap(),
                serde_json::to_value(&resumed.civilizations).unwrap()
            );
            assert_eq!(g.config.systems, resumed.config.systems);
        }
    }
}
