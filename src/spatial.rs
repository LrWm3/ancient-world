//! Read-only spatial adapters. Geometry references existing simulation entities and stocks.
use crate::{expeditions::Phase, gpu::Generator, grid};
use anyhow::{ensure, Result};
use serde::{Deserialize, Serialize};

pub(crate) fn new_world_id() -> String {
    use std::sync::atomic::{AtomicU64, Ordering};
    static NEXT: AtomicU64 = AtomicU64::new(0);
    format!(
        "world-{:x}-{:x}-{:x}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos(),
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    )
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct GridRef {
    pub world: String,
    pub resolution: u32,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct CellRef {
    pub grid: GridRef,
    pub cell: u32,
}
impl CellRef {
    pub fn direction(&self) -> Result<[f32; 3]> {
        ensure!(!self.grid.world.is_empty(), "missing world identity");
        let n = self.grid.resolution;
        ensure!(
            n.is_power_of_two() && (8..=1024).contains(&n) && self.cell < 6 * n * n,
            "invalid terrain cell reference"
        );
        Ok(grid::cell_direction(self.cell, n))
    }
    pub fn lon_lat(&self) -> Result<[f64; 2]> {
        let [x, y, z] = self.direction()?.map(f64::from);
        Ok([
            x.atan2(z).to_degrees(),
            y.clamp(-1., 1.).asin().to_degrees(),
        ])
    }
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "type", content = "cells")]
pub enum Geometry {
    Point(CellRef),
    Path(Vec<CellRef>),
    CellRegion(Vec<CellRef>),
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EntityRef {
    pub kind: String,
    pub id: u64,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Precision {
    ParentCellCoverage,
    CellRepresentative,
    ModelCellPath,
    LegacyRouteAssociation,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Feature {
    pub id: String,
    pub entity: EntityRef,
    pub role: String,
    pub label: String,
    pub geometry: Geometry,
    pub precision: Precision,
    pub history_month: u32,
    pub evidence: Vec<u64>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FeatureCollection {
    pub version: u32,
    pub grid: GridRef,
    pub radius_m: f64,
    pub epoch: u32,
    pub ecology_month: u64,
    pub history_month: Option<u32>,
    pub features: Vec<Feature>,
}
/// Identity and parent grid of a generated survey. Old surveys have no inferred identity.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SurveyRef {
    pub id: u64,
    pub grid: GridRef,
    pub radius_m: f64,
}
impl FeatureCollection {
    fn resource(&mut self, source: &crate::resources::Source) -> Result<()> {
        let cell = CellRef {
            grid: self.grid.clone(),
            cell: source.cell,
        };
        cell.direction()?;
        self.features.push(Feature {
            id: format!(
                "{}/resource_source/{}/location",
                self.grid.world, source.cell
            ),
            entity: EntityRef {
                kind: "resource_source".into(),
                id: source.cell as u64,
            },
            role: "location".into(),
            label: format!(
                "{} and clay source",
                source.mineral.as_deref().unwrap_or("Ore")
            ),
            geometry: Geometry::Point(cell),
            precision: Precision::CellRepresentative,
            history_month: self.history_month.unwrap_or(0),
            evidence: vec![],
        });
        Ok(())
    }
}
impl crate::region::Region {
    /// Frozen survey coverage and source/site snapshots, not live inventories.
    pub fn spatial_features(&self) -> Result<FeatureCollection> {
        let survey = self.spatial.as_ref().ok_or_else(|| {
            anyhow::anyhow!("legacy survey lacks a world/grid identity; regenerate the survey")
        })?;
        ensure!(
            survey.radius_m.is_finite() && survey.radius_m > 0.,
            "invalid survey identity or radius"
        );
        let mut result = FeatureCollection {
            version: 1,
            grid: survey.grid.clone(),
            radius_m: survey.radius_m,
            epoch: self.epoch,
            ecology_month: self.ecological_month,
            history_month: self.history_month,
            features: vec![],
        };
        let parents: std::collections::BTreeSet<_> =
            self.cells.iter().map(|c| c.route[2]).collect();
        ensure!(!parents.is_empty(), "survey has no coverage");
        let cells: Vec<_> = parents
            .iter()
            .map(|&cell| CellRef {
                grid: survey.grid.clone(),
                cell,
            })
            .collect();
        for cell in &cells {
            cell.direction()?;
        }
        result.features.push(Feature {
            id: format!("{}/survey/{}/coverage", survey.grid.world, survey.id),
            entity: EntityRef {
                kind: "survey".into(),
                id: survey.id,
            },
            role: "parent_cell_coverage".into(),
            label: format!(
                "{} km survey · {}² local grid",
                self.width_km, self.resolution
            ),
            geometry: Geometry::CellRegion(cells),
            precision: Precision::ParentCellCoverage,
            history_month: self.history_month.unwrap_or(0),
            evidence: vec![],
        });
        for source in &self.resource_sources {
            ensure!(
                parents.contains(&source.cell),
                "survey source outside coverage"
            );
            result.resource(source)?;
        }
        for site in &self.historical_sites {
            ensure!(parents.contains(&site.cell), "survey site outside coverage");
            result.features.push(Feature {
                id: format!("{}/site/{}/location", survey.grid.world, site.id),
                entity: EntityRef {
                    kind: "site".into(),
                    id: site.id as u64,
                },
                role: "location".into(),
                label: site.name.clone(),
                geometry: Geometry::Point(CellRef {
                    grid: survey.grid.clone(),
                    cell: site.cell,
                }),
                precision: Precision::CellRepresentative,
                history_month: self.history_month.unwrap_or(0),
                evidence: vec![],
            });
        }
        Ok(result)
    }
}
impl Generator {
    /// Current sparse features; no GPU readback, routing decisions or new inventories.
    pub fn spatial_features(&self) -> Result<FeatureCollection> {
        let grid = GridRef {
            world: self
                .config
                .spatial_world_id
                .clone()
                .ok_or_else(|| anyhow::anyhow!("world identity unavailable"))?,
            resolution: self.config.resolution,
        };
        let mut result = FeatureCollection {
            version: 1,
            grid: grid.clone(),
            radius_m: self.config.radius_km as f64 * 1000.,
            epoch: self.progress.epoch,
            ecology_month: self.ecology.clock.month,
            history_month: self.civilizations.as_ref().map(|h| h.month),
            features: vec![],
        };
        let Some(h) = &self.civilizations else {
            return Ok(result);
        };
        let cell = |id| CellRef {
            grid: grid.clone(),
            cell: id,
        };
        let mut add = |kind: &str,
                       id: u64,
                       role: &str,
                       label: String,
                       ids: &[u32],
                       point: bool,
                       precision: Precision,
                       month: u32,
                       evidence: Vec<u64>|
         -> Result<()> {
            if ids.is_empty() {
                return Ok(());
            }
            let cells: Vec<_> = ids.iter().map(|&i| cell(i)).collect();
            for c in &cells {
                c.direction()?;
            }
            result.features.push(Feature {
                id: format!("{}/{kind}/{id}/{role}", grid.world),
                entity: EntityRef {
                    kind: kind.into(),
                    id,
                },
                role: role.into(),
                label,
                geometry: if point {
                    Geometry::Point(cells[0].clone())
                } else {
                    Geometry::Path(cells)
                },
                precision,
                history_month: month,
                evidence,
            });
            Ok(())
        };
        for s in &h.sites {
            add(
                "site",
                s.id as u64,
                "location",
                s.name.clone(),
                &[s.cell],
                true,
                Precision::CellRepresentative,
                h.month,
                vec![],
            )?;
        }
        if let Some(s) = &h.society {
            for r in &s.routes {
                add(
                    "road",
                    r.id as u64,
                    "route",
                    format!(
                        "{} – {}",
                        h.sites[r.from as usize].name, h.sites[r.to as usize].name
                    ),
                    &r.cells,
                    false,
                    Precision::ModelCellPath,
                    h.month,
                    vec![],
                )?;
            }
        }
        if let Some(s) = &h.shipping {
            for (id, p) in s.ports.iter().enumerate() {
                add(
                    "port",
                    id as u64,
                    "location",
                    format!("{} harbor", h.sites[p.site as usize].name),
                    &[p.water_cell],
                    true,
                    Precision::CellRepresentative,
                    h.month,
                    vec![],
                )?;
            }
            for (id, r) in s.lanes.iter().enumerate() {
                add(
                    "sea_lane",
                    id as u64,
                    "route",
                    format!("Sea lane {id}"),
                    &r.cells,
                    false,
                    Precision::ModelCellPath,
                    h.month,
                    vec![],
                )?;
            }
        }
        if let Some(x) = &h.expeditions {
            for e in &x.voyages {
                let legacy = e.planned_cells.is_none();
                let ids = e
                    .planned_cells
                    .as_deref()
                    .or_else(|| x.routes.get(e.route as usize).map(|r| r.cells.as_slice()))
                    .unwrap_or(&[]);
                let label = format!("Expedition {} · {:?} · {:?}", e.id, e.objective, e.phase);
                add(
                    "expedition",
                    e.id as u64,
                    "planned_route",
                    label.clone(),
                    ids,
                    false,
                    if legacy {
                        Precision::LegacyRouteAssociation
                    } else {
                        Precision::ModelCellPath
                    },
                    e.departed,
                    vec![e.cause],
                )?;
                add(
                    "expedition",
                    e.id as u64,
                    "origin",
                    label.clone(),
                    &[h.sites[e.origin as usize].cell],
                    true,
                    Precision::CellRepresentative,
                    e.departed,
                    vec![e.cause],
                )?;
                // Only camp/stranding is a known current milestone. No invented in-transit track.
                if matches!(e.phase, Phase::Camp | Phase::Stranded) {
                    if let Some(&last) = ids.last() {
                        add(
                            "expedition",
                            e.id as u64,
                            "camp",
                            label.clone(),
                            &[last],
                            true,
                            if legacy {
                                Precision::LegacyRouteAssociation
                            } else {
                                Precision::CellRepresentative
                            },
                            h.month,
                            vec![e.cause],
                        )?;
                    }
                }
                if let Some(f) = e.heritage.as_ref().and_then(|c| c.find.as_ref()) {
                    add(
                        "expedition",
                        e.id as u64,
                        "findspot",
                        f.description.clone(),
                        &[f.cell],
                        true,
                        Precision::CellRepresentative,
                        h.events
                            .get(f.observed as usize)
                            .map_or(h.month, |event| event.month),
                        vec![f.observed],
                    )?;
                }
            }
        }
        if let Some(resources) = &h.resources {
            for source in resources.sources.values() {
                result.resource(source)?;
            }
        }
        Ok(result)
    }
}
impl FeatureCollection {
    /// Fictional-planet visualization using GeoJSON syntax, not Earth/WGS84 geodesy.
    pub fn geojson(&self) -> Result<serde_json::Value> {
        let mut features = Vec::new();
        for f in &self.features {
            let cells: &[CellRef] = match &f.geometry {
                Geometry::Point(c) => std::slice::from_ref(c),
                Geometry::Path(c) | Geometry::CellRegion(c) => c,
            };
            ensure!(
                cells.iter().all(|c| c.grid == self.grid),
                "foreign world or grid in feature"
            );
            let geometry = match &f.geometry {
                Geometry::Point(c) => {
                    serde_json::json!({"type":"Point","coordinates":c.lon_lat()?})
                }
                Geometry::Path(cells) => {
                    if cells.len() < 2 {
                        anyhow::bail!("path requires at least two cells");
                    }
                    let points: Vec<_> =
                        cells.iter().map(CellRef::lon_lat).collect::<Result<_>>()?;
                    serde_json::json!({"type":"MultiLineString","coordinates":split_dateline(&points)})
                }
                Geometry::CellRegion(cells) => {
                    ensure!(!cells.is_empty(), "empty cell region");
                    let mut seen = std::collections::BTreeSet::new();
                    ensure!(
                        cells.iter().all(|c| seen.insert(c.cell)),
                        "duplicate region cell"
                    );
                    let points = cells
                        .iter()
                        .map(CellRef::lon_lat)
                        .collect::<Result<Vec<_>>>()?;
                    serde_json::json!({"type":"MultiPoint","coordinates":points})
                }
            };
            features.push(serde_json::json!({"type":"Feature","id":f.id,"geometry":geometry,"properties":{"entity":{"kind":f.entity.kind,"id":f.entity.id.to_string()},"role":f.role,"label":f.label,"precision":f.precision,"native_geometry":match &f.geometry {Geometry::Point(_) => "point",Geometry::Path(_) => "path",Geometry::CellRegion(_) => "cell_region_representatives"},"history_month":f.history_month,"evidence":f.evidence.iter().map(|id|id.to_string()).collect::<Vec<_>>()}}));
        }
        Ok(
            serde_json::json!({"type":"FeatureCollection","features":features,"ancient_world":{"world":self.grid.world,"radius_m":self.radius_m,"terrain_resolution":self.grid.resolution,"epoch":self.epoch,"ecology_month":self.ecology_month,"history_month":self.history_month,"coordinate_system":"fictional sphere; longitude/latitude degrees; not WGS84"}}),
        )
    }
}
/// Linear longitude/latitude display segments with explicit dateline endpoints.
pub fn split_dateline(points: &[[f64; 2]]) -> Vec<Vec<[f64; 2]>> {
    let Some(&first) = points.first() else {
        return vec![];
    };
    let mut out = Vec::new();
    let mut current = vec![first];
    for pair in points.windows(2) {
        let (a, b) = (pair[0], pair[1]);
        if (b[0] - a[0]).abs() > 180. {
            let end = if a[0] > 0. { 180. } else { -180. };
            let bx = b[0] + if a[0] > 0. { 360. } else { -360. };
            let t = (end - a[0]) / (bx - a[0]);
            let lat = a[1] + t * (b[1] - a[1]);
            current.push([end, lat]);
            out.push(current);
            current = vec![[-end, lat], b];
        } else {
            current.push(b);
        }
    }
    if current.len() > 1 {
        out.push(current);
    }
    out
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn export_rejects_foreign_grids_and_keeps_large_ids_as_text() {
        let grid = GridRef {
            world: "a".into(),
            resolution: 16,
        };
        let mut collection = FeatureCollection {
            version: 1,
            grid: grid.clone(),
            radius_m: 1000.,
            epoch: 0,
            ecology_month: 0,
            history_month: Some(0),
            features: vec![Feature {
                id: "a/site/1/location".into(),
                entity: EntityRef {
                    kind: "site".into(),
                    id: u64::MAX,
                },
                role: "location".into(),
                label: "Site".into(),
                geometry: Geometry::Point(CellRef {
                    grid: grid.clone(),
                    cell: 0,
                }),
                precision: Precision::CellRepresentative,
                history_month: 0,
                evidence: vec![u64::MAX],
            }],
        };
        let exported = collection.geojson().unwrap();
        assert_eq!(
            exported["features"][0]["properties"]["entity"]["id"],
            u64::MAX.to_string()
        );
        assert_eq!(
            exported["features"][0]["properties"]["evidence"][0],
            u64::MAX.to_string()
        );
        if let Geometry::Point(c) = &mut collection.features[0].geometry {
            c.grid.world = "b".into();
        }
        assert!(collection.geojson().is_err());
        collection.features[0].geometry = Geometry::CellRegion(vec![CellRef { grid, cell: 0 }]);
        assert_eq!(
            collection.geojson().unwrap()["features"][0]["geometry"]["type"],
            "MultiPoint"
        );
        if let Geometry::CellRegion(cells) = &mut collection.features[0].geometry {
            cells.push(cells[0].clone());
        }
        assert!(collection.geojson().is_err());
        collection.features[0].geometry = Geometry::CellRegion(vec![]);
        assert!(collection.geojson().is_err());
    }
    #[test]
    fn coordinates_and_dateline() {
        let g = GridRef {
            world: "test".into(),
            resolution: 16,
        };
        for i in 0..6 * 16 * 16 {
            let c = CellRef {
                grid: g.clone(),
                cell: i,
            };
            assert_eq!(grid::index(c.direction().unwrap(), 16), i);
            let [lon, lat] = c.lon_lat().unwrap();
            assert!(lon.abs() <= 180. && lat.abs() <= 90.);
        }
        assert!(CellRef {
            grid: g,
            cell: 6 * 16 * 16
        }
        .direction()
        .is_err());
        assert_eq!(
            split_dateline(&[[179., 0.], [-179., 2.]]),
            vec![vec![[179., 0.], [180., 1.]], vec![[-180., 1.], [-179., 2.]]]
        );
    }
}
