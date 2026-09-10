//! Explicit counterfactual custody. Restricted tools are unavailable to all town consumers.
//! No decay or confiscation occurs in this experimental boundary store.
use crate::gpu::{Generator, Stage};
use anyhow::{ensure, Result};
impl Generator {
    /// Set accessible ordinary tools to a fraction of available + restricted tools.
    /// This is a one-time stock intervention, not a cap on later production or imports.
    /// Fraction 1 releases reserves; new goods acquired subsequently remain usable.
    pub fn set_tool_stock_access(&mut self, site: u32, fraction: f64) -> Result<()> {
        ensure!(
            fraction.is_finite() && (0. ..=1.).contains(&fraction),
            "tool fraction must be in [0,1]"
        );
        self.validate_living_boundary()?;
        ensure!(
            self.progress.stage == Stage::Boundary,
            "tool access requires a completed boundary"
        );
        let h = self
            .civilizations
            .as_mut()
            .ok_or_else(|| anyhow::anyhow!("found civilizations first"))?;
        ensure!((site as usize) < h.sites.len(), "unknown tool site");
        ensure!(
            h.economy_catalog
                .as_ref()
                .is_some_and(|c| c.goods.get(3).is_some_and(|g| g.id == "tools")),
            "ordinary tool catalog required"
        );
        let old = h.sites[site as usize].economy.goods[3];
        let held = h
            .experimental_tool_reserves
            .get(&site)
            .copied()
            .unwrap_or(0.);
        let total = old as f64 + held;
        ensure!(
            total.is_finite() && total >= 0. && total <= f32::MAX as f64,
            "invalid tool inventory"
        );
        let mut available = (total * fraction) as f32;
        // Never round a release above the available total. Retain any sub-ULP remainder.
        if available as f64 > total {
            available = f32::from_bits(available.to_bits() - 1);
        }
        let restricted = total - available as f64;
        if old == available {
            return Ok(());
        }
        h.sites[site as usize].economy.goods[3] = available;
        if restricted == 0. {
            h.experimental_tool_reserves.remove(&site);
        } else {
            h.experimental_tool_reserves.insert(site, restricted);
        }
        h.event("experimental_tool_access",Some(site),None,format!("Counterfactual custody: ordinary tools {:.6} kg accessible, {:.6} kg restricted; requested fraction {:.6}; subsequent production/imports remain accessible",available,restricted,fraction));
        Ok(())
    }
}
