//! Drawing-only historical atlas overlays. Uses stored cells, never current claim owners.
use crate::{civilization::History, grid, spatial::split_dateline};
use eframe::egui;

pub(crate) struct Projection {
    pub rect: egui::Rect,
    pub pan: [f32; 2],
    pub zoom: f32,
    pub resolution: u32,
}
impl Projection {
    fn lon_lat(&self, p: [f64; 2]) -> egui::Pos2 {
        let q = egui::vec2(p[0] as f32 / 360. + 0.5, 0.5 - p[1] as f32 / 180.);
        self.rect.min
            + ((q - egui::vec2(0.5 + self.pan[0], 0.5 + self.pan[1])) * self.zoom
                + egui::vec2(0.5, 0.5))
                * self.rect.size()
    }
    fn coordinates(&self, cell: u32) -> [f64; 2] {
        let d = grid::cell_direction(cell, self.resolution);
        [
            d[0].atan2(d[2]).to_degrees() as f64,
            d[1].clamp(-1., 1.).asin().to_degrees() as f64,
        ]
    }
    fn cell(&self, cell: u32) -> egui::Pos2 {
        let mut p = self.coordinates(cell);
        p[0] += ((self.pan[0] as f64 * 360. - p[0]) / 360.).round() * 360.;
        self.lon_lat(p)
    }
}
pub(crate) fn draw(
    painter: &egui::Painter,
    projection: &Projection,
    h: &History,
    month: Option<u32>,
    journey: Option<u64>,
) {
    let painter = painter.with_clip_rect(projection.rect);
    if let Some(month) = month {
        if let Some(snapshot) = h.territory_at(month) {
            for (&cell, owners) in &snapshot.claims {
                let color = if owners.len() > 1 {
                    egui::Color32::WHITE
                } else {
                    crate::viewer::political_color(*owners.first().unwrap())
                };
                let radius = (projection.rect.width() * projection.zoom
                    / projection.resolution as f32
                    * 0.2)
                    .clamp(1.5, 10.);
                painter.circle_filled(projection.cell(cell), radius, color.gamma_multiply(0.55));
            }
            for site in &snapshot.sites {
                painter.circle_filled(
                    projection.cell(site.cell),
                    4.,
                    crate::viewer::political_color(site.controller),
                );
                painter.circle_stroke(
                    projection.cell(site.cell),
                    4.5,
                    egui::Stroke::new(1., egui::Color32::BLACK),
                );
            }
        }
    }
    if let Some(event) = journey
        .and_then(|id| h.events.get(id as usize))
        .filter(|e| journey == Some(e.id))
    {
        if let Some(path) = &event.planned_path {
            let points: Vec<_> = path
                .iter()
                .map(|&cell| projection.coordinates(cell))
                .collect();
            for segment in split_dateline(&points) {
                // Shift whole segments, not individual vertices: preserve dateline cuts.
                for turn in -1..=1 {
                    let offset = (projection.pan[0].floor() as f64 + turn as f64) * 360.;
                    painter.add(egui::Shape::line(
                        segment
                            .iter()
                            .map(|p| projection.lon_lat([p[0] + offset, p[1]]))
                            .collect(),
                        egui::Stroke::new(2.5, egui::Color32::LIGHT_BLUE),
                    ));
                }
            }
            for (index, label) in [
                (0, "Origin"),
                (path.len().saturating_sub(1), "Intended destination"),
            ] {
                if let Some(&cell) = path.get(index) {
                    let pos = projection.cell(cell);
                    painter.circle_stroke(
                        pos,
                        6.,
                        egui::Stroke::new(2., egui::Color32::LIGHT_BLUE),
                    );
                    painter.text(
                        pos + egui::vec2(8., if index == 0 { -10. } else { 10. }),
                        egui::Align2::LEFT_CENTER,
                        label,
                        egui::FontId::proportional(12.),
                        egui::Color32::LIGHT_BLUE,
                    );
                }
            }
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn projection_tracks_pan_zoom_and_dateline_segments() {
        let p = Projection {
            rect: egui::Rect::from_min_size(egui::pos2(10., 20.), egui::vec2(720., 360.)),
            pan: [0., 0.],
            zoom: 1.,
            resolution: 16,
        };
        assert_eq!(p.lon_lat([0., 0.]), egui::pos2(370., 200.));
        assert_eq!(p.lon_lat([-180., 90.]), egui::pos2(10., 20.));
        for segment in split_dateline(&[[179., 0.], [-179., 0.]]) {
            assert!((p.lon_lat(segment[0]) - p.lon_lat(segment[1])).length() < 3.);
        }
        let shifted = Projection {
            pan: [0.25, 0.],
            zoom: 2.,
            ..p
        };
        assert_eq!(shifted.lon_lat([90., 0.]), egui::pos2(370., 200.));
        let repeated = Projection {
            pan: [1.25, 0.],
            ..shifted
        };
        let cell = grid::index([1., 0., 0.], 16);
        assert!((repeated.cell(cell) - shifted.cell(cell)).length() < 0.001);
    }
}
