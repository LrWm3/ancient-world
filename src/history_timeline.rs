//! Recorded monthly observations, never reconstructed from present-day town state.
use crate::civilization::History;
use eframe::egui;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

pub const RETAINED_MONTHS: usize = 600;
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Observation {
    pub month: u32,
    pub population: f32,
    /// Aggregate food reserve; excludes unprocessed crop and household inventories.
    pub food: f32,
    pub treasury: f32,
    pub abandoned: bool,
}
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Timeline {
    pub samples: VecDeque<Observation>,
}
impl Timeline {
    pub(crate) fn valid(&self, founded: u32, current: u32) -> bool {
        self.samples.len() <= RETAINED_MONTHS
            && self.samples.iter().all(|s| {
                s.month >= founded
                    && s.month <= current
                    && [s.population, s.food, s.treasury]
                        .iter()
                        .all(|v| v.is_finite() && *v >= 0.)
            })
            && self
                .samples
                .iter()
                .zip(self.samples.iter().skip(1))
                .all(|(a, b)| a.month < b.month)
    }
    fn record(&mut self, sample: Observation) {
        if self.samples.back().is_some_and(|s| s.month == sample.month) {
            self.samples.pop_back();
        }
        self.samples.push_back(sample);
        while self.samples.len() > RETAINED_MONTHS {
            self.samples.pop_front();
        }
    }
}
impl History {
    pub(crate) fn record_timeline(&mut self) {
        for s in &mut self.sites {
            s.lifecycle.timeline.record(Observation {
                month: self.month,
                population: s.stocks.stock[0],
                food: s.stocks.stock[1],
                treasury: s.economy.finance[0],
                abandoned: s.abandoned,
            });
        }
    }
}
#[derive(Default)]
pub struct TimelineView {
    month: Option<u32>,
    site: Option<usize>,
    event: Option<u64>,
    all_events: bool,
    routine: bool,
}
fn date(month: u32) -> String {
    format!("Y{} M{}", month / 12, month % 12 + 1)
}
fn chart(
    ui: &mut egui::Ui,
    samples: &VecDeque<Observation>,
    month: u32,
    label: &str,
    value: fn(&Observation) -> f32,
) {
    let first = samples.front().unwrap().month;
    let last = samples.back().unwrap().month;
    let max = samples.iter().map(value).fold(0_f32, f32::max).max(1.);
    let selected = samples.iter().find(|s| s.month == month);
    ui.label(format!(
        "{label}: {} · scale 0–{max:.0}",
        selected.map_or_else(|| "no observation".into(), |s| format!("{:.1}", value(s)))
    ));
    let (rect, _) = ui.allocate_exact_size(
        egui::vec2(ui.available_width().max(100.), 62.),
        egui::Sense::hover(),
    );
    let painter = ui.painter_at(rect);
    painter.rect_filled(rect, 3., ui.visuals().extreme_bg_color);
    let x = |m| rect.left() + rect.width() * (m - first) as f32 / (last - first).max(1) as f32;
    let pos =
        |s: &Observation| egui::pos2(x(s.month), rect.bottom() - rect.height() * value(s) / max);
    for i in 1..samples.len() {
        if samples[i].month == samples[i - 1].month + 1 {
            painter.line_segment(
                [pos(&samples[i - 1]), pos(&samples[i])],
                egui::Stroke::new(1.5, egui::Color32::LIGHT_GREEN),
            );
        }
    }
    for s in samples {
        painter.circle_filled(pos(s), 1.3, egui::Color32::LIGHT_GREEN);
    }
    painter.line_segment(
        [
            egui::pos2(x(month), rect.top()),
            egui::pos2(x(month), rect.bottom()),
        ],
        egui::Stroke::new(1., egui::Color32::GOLD),
    );
    ui.horizontal(|ui| {
        ui.small(date(first));
        ui.small(format!("→ {}", date(last)));
    });
}
impl TimelineView {
    /// Returns a recorded event anchor or current site cell; does not rewind the world.
    pub fn show(
        &mut self,
        ui: &mut egui::Ui,
        h: &History,
        selection: &mut Option<usize>,
    ) -> Option<u32> {
        let mut focus = None;
        egui::ComboBox::from_id_salt("timeline_site")
            .selected_text(
                selection
                    .and_then(|i| h.sites.get(i))
                    .map_or("Select a settlement", |s| s.name.as_str()),
            )
            .show_ui(ui, |ui| {
                for (i, s) in h.sites.iter().enumerate() {
                    if ui.selectable_value(selection, Some(i), &s.name).clicked() {
                        focus = Some(s.cell);
                    }
                }
            });
        if self.site != *selection {
            self.site = *selection;
            self.month = None;
            self.event = None;
        }
        let Some(site) = selection.and_then(|i| h.sites.get(i)) else {
            return focus;
        };
        if ui.small_button("Locate on present-day map").clicked() {
            focus = Some(site.cell);
        }
        ui.small("Recorded monthly observations · latest 600 months per town. Map and terrain remain present-day. Food is the aggregate reserve, not total edible goods or household access.");
        let samples = &site.lifecycle.timeline.samples;
        if let (Some(first), Some(last)) = (samples.front(), samples.back()) {
            let mut month = self
                .month
                .unwrap_or(last.month)
                .clamp(first.month, last.month);
            ui.add(egui::Slider::new(&mut month, first.month..=last.month).text("Recorded month"));
            self.month = Some(month);
            ui.label(format!(
                "{} · {}",
                date(month),
                samples
                    .iter()
                    .find(|s| s.month == month)
                    .map_or("no observation", |s| if s.abandoned {
                        "abandoned"
                    } else {
                        "occupied"
                    })
            ));
            chart(ui, samples, month, "Population (people)", |s| s.population);
            chart(
                ui,
                samples,
                month,
                "Aggregate food reserve (kg equivalent)",
                |s| s.food,
            );
            chart(
                ui,
                samples,
                month,
                "Town treasury (abstract currency)",
                |s| s.treasury,
            );
        } else {
            ui.label("No observations retained. Advance history to start recording; earlier values are not reconstructed.");
        }
        ui.separator();
        ui.checkbox(&mut self.all_events, "Show entire recorded town chronicle");
        ui.small("Otherwise: events within six months of the cursor. Event links are recorded causes, not inferred from these graphs.");
        ui.checkbox(
            &mut self.routine,
            "Include routine harvest and market notices",
        );
        ui.small("Up to 100 latest matching events.");
        let month = self.month.unwrap_or(h.month);
        egui::ScrollArea::vertical()
            .id_salt("timeline_events")
            .max_height(190.)
            .show(ui, |ui| {
                let mut count = 0;
                for e in h
                    .events
                    .iter()
                    .rev()
                    .filter(|e| {
                        self.routine
                            || !matches!(
                                e.kind.as_str(),
                                "harvest" | "market_sale" | "market_arrival"
                            )
                    })
                    .filter(|e| {
                        (e.site == Some(site.id)
                            || e.other == Some(site.id)
                            || e.subjects
                                .iter()
                                .any(|(kind, id)| kind == "site" && *id == site.id))
                            && (self.all_events || e.month.abs_diff(month) <= 6)
                    })
                    .take(100)
                {
                    count += 1;
                    if ui
                        .selectable_label(
                            self.event == Some(e.id),
                            format!("{} · {} · #{}", date(e.month), e.kind, e.id),
                        )
                        .clicked()
                    {
                        self.event = Some(e.id);
                    }
                }
                if count == 0 {
                    ui.label("No linked events in this interval.");
                }
            });
        if let Some(e) = self
            .event
            .and_then(|id| h.events.iter().find(|e| e.id == id))
        {
            ui.separator();
            ui.label(format!("#{} · {} · {}", e.id, date(e.month), e.kind));
            ui.label(&e.detail);
            if let Some(anchors) = &e.spatial {
                ui.horizontal_wrapped(|ui| {
                    for anchor in anchors {
                        let label = match anchor.role {
                            crate::spatial::EventRole::Milestone => "Locate recorded milestone",
                            crate::spatial::EventRole::AssociatedSite => {
                                "Locate recorded site association"
                            }
                            crate::spatial::EventRole::AssociatedOtherSite => {
                                "Locate recorded other-site association"
                            }
                        };
                        if ui.small_button(label).clicked() {
                            focus = Some(anchor.cell);
                        }
                    }
                    if anchors.is_empty() {
                        ui.small("No geographic anchor recorded.");
                    }
                });
            } else {
                ui.small("Legacy event: only current site associations are available.");
            }
            ui.horizontal_wrapped(|ui| {
                for id in [e.site, e.other].into_iter().flatten() {
                    if let Some(place) = h.sites.get(id as usize) {
                        if ui.small_button(format!("Current {}", place.name)).clicked() {
                            focus = Some(place.cell);
                        }
                    }
                }
            });
            ui.horizontal_wrapped(|ui| {
                ui.label("Recorded causes:");
                for cause in &e.causes {
                    if ui.small_button(format!("#{cause}")).clicked() {
                        self.event = Some(*cause);
                    }
                }
                if e.causes.is_empty() {
                    ui.small("none recorded");
                }
            });
            ui.horizontal_wrapped(|ui| {
                ui.label("Linked consequences:");
                for effect in h
                    .events
                    .iter()
                    .filter(|next| next.causes.contains(&e.id))
                    .take(30)
                {
                    if ui
                        .small_button(format!("#{} {}", effect.id, effect.kind))
                        .clicked()
                    {
                        self.event = Some(effect.id);
                    }
                }
            });
            ui.small("Narrative text can contain attributed beliefs; causal links alone do not establish a supernatural or scientific explanation.");
        }
        focus
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn observations_are_bounded_and_resume_without_duplicate_months() {
        let mut t = Timeline::default();
        for month in 1..=720 {
            t.record(Observation {
                month,
                population: month as f32,
                food: 0.,
                treasury: 0.,
                abandoned: false,
            });
        }
        assert_eq!(t.samples.len(), 600);
        assert_eq!(t.samples.front().unwrap().month, 121);
        let mut resumed: Timeline =
            serde_json::from_str(&serde_json::to_string(&t).unwrap()).unwrap();
        resumed.record(t.samples.back().unwrap().clone());
        assert_eq!(
            serde_json::to_value(&resumed).unwrap(),
            serde_json::to_value(&t).unwrap()
        );
        assert!(t.valid(0, 720));
        assert!(!t.valid(0, 719));
        let mut invalid = t.clone();
        invalid.samples.back_mut().unwrap().population = f32::NAN;
        assert!(!invalid.valid(0, 720));
        let old: crate::civilization::SettlementLifecycle = serde_json::from_str("{}").unwrap();
        assert!(old.timeline.samples.is_empty());
        let ctx = egui::Context::default();
        let _ = ctx.run(Default::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                chart(ui, &t.samples, 600, "Population", |s| s.population)
            });
        });
    }
}
