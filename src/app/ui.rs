use super::widgets::{centered_combo, centered_row, job_title, ERROR_TEXT};
use super::App;
use crate::model::{Job, Theme};
use chrono::Utc;
use eframe::egui;

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if ctx.input(|input| input.viewport().close_requested()) {
            if !self.persist() {
                eprintln!("Could not save settings on exit: {}", self.error);
            }
            return;
        }
        self.tick(ctx);
        if let Some(id) = self.selected {
            self.ensure_summary(id, ctx);
        }
        ctx.set_theme(theme_preference(self.config.theme));
        let before = self.config.clone();
        self.show_header(ctx);
        self.show_footer(ctx);
        self.show_job_list(ctx);
        self.show_job_editor(ctx);
        self.show_history(ctx);
        if self.config != before {
            self.dirty = true;
        }
    }
}

impl App {
    fn show_header(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::top("header").show(ctx, |ui| {
            ui.add_space(12.0);
            centered_row(ui, |ui| {
                ui.heading("portbak");
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    centered_combo(ui, |ui| {
                        egui::ComboBox::from_id_salt("theme")
                            .selected_text(self.config.theme.label())
                            .show_ui(ui, |ui| {
                                for theme in Theme::ALL {
                                    ui.selectable_value(
                                        &mut self.config.theme,
                                        theme,
                                        theme.label(),
                                    );
                                }
                            })
                    });
                });
            });
            ui.add_space(10.0);
        });
    }

    fn show_footer(&self, ctx: &egui::Context) {
        egui::TopBottomPanel::bottom("footer").show(ctx, |ui| {
            ui.add_space(5.0);
            if self.active.is_some() {
                centered_row(ui, |ui| {
                    ui.spinner();
                    ui.label(&self.progress);
                });
            } else {
                ui.label("Ready · scheduling runs while the app is open");
            }
            if !self.error.is_empty() {
                ui.colored_label(ERROR_TEXT, &self.error);
            }
            ui.small(format!("Settings: {}", self.path.display()));
            ui.add_space(5.0);
        });
    }

    fn show_job_list(&mut self, ctx: &egui::Context) {
        egui::SidePanel::left("jobs")
            .default_width(240.0)
            .show(ctx, |ui| {
                ui.add_space(10.0);
                ui.heading("Backup jobs");
                ui.add_space(8.0);
                let new_job = egui::Button::new("New job");
                if ui.add_enabled(!self.scheduling_paused(), new_job).clicked() {
                    self.add_job();
                }
                ui.separator();
                let mut clicked = None;
                egui::ScrollArea::vertical().show(ui, |ui| {
                    for job in &self.config.jobs {
                        if self.job_card(ui, job) {
                            clicked = Some(job.id);
                        }
                    }
                });
                if clicked.is_some() {
                    self.selected = clicked;
                }
            });
    }

    /// Returns whether the job's title was clicked.
    fn job_card(&self, ui: &mut egui::Ui, job: &Job) -> bool {
        ui.group(|ui| {
            let selected = self.selected == Some(job.id);
            let title = job_title(
                ui,
                |ui| ui.selectable_label(selected, &job.name),
                job.enabled,
            );
            ui.small(self.activity(job));
            if self.queue.contains(&job.id) {
                ui.small("Queued");
            }
            if let Some(remaining) = job.countdown(Utc::now()) {
                ui.small(format!("Next check in: {remaining}"));
            }
            ui.small(&job.status);
            title.clicked()
        })
        .inner
    }

    fn activity(&self, job: &Job) -> &'static str {
        match (self.active == Some(job.id), self.restoring, job.enabled) {
            (true, true, _) => "Restore running",
            (true, false, _) => "Backup running",
            (false, _, true) => "Schedule enabled",
            (false, _, false) => "Schedule disabled",
        }
    }
}

fn theme_preference(theme: Theme) -> egui::ThemePreference {
    match theme {
        Theme::System => egui::ThemePreference::System,
        Theme::Light => egui::ThemePreference::Light,
        Theme::Dark => egui::ThemePreference::Dark,
    }
}
