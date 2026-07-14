use crate::app::P2PApp;
use crate::audio;
use crate::engine;
use eframe::egui;
use std::sync::atomic::Ordering;
use std::time::Instant;

pub fn render(ctx: &egui::Context, app: &mut P2PApp) {
    egui::SidePanel::left("controls")
        .default_width(220.0)
        .resizable(false)
        .show(ctx, |ui| {
            ui.heading("Настройки");
            ui.separator();
            draw_connection(ui, app);
            ui.add_space(10.0);
            draw_devices(ui, app);
            ui.add_space(10.0);
            draw_controls(ui, app);

            ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
                ui.add_space(10.0);
                let status = app.status_text.lock().unwrap().clone();
                ui.label(egui::RichText::new(status).small());
            });
        });

    egui::CentralPanel::default().show(ctx, |ui| {
        ui.heading("Участники");
        ui.separator();
        draw_peers(ui, app);
    });

    if app.show_update_dialog {
        egui::Window::new("Доступно обновление!")
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .show(ctx, |ui| {
                let info = app.update_info.lock().unwrap();

                ui.label(format!(
                    "Текущая версия: {}\nПоследняя версия: {}",
                    info.current_version,
                    info.latest_version.as_deref().unwrap_or("?")
                ));

                if let Some(notes) = &info.release_notes {
                    ui.separator();
                    ui.label("Что нового:");
                    egui::ScrollArea::vertical()
                        .max_height(150.0)
                        .show(ui, |ui| {
                            ui.label(notes);
                        });
                }

                ui.separator();

                ui.horizontal(|ui| {
                    if ui.button("Обновить").clicked() && !app.is_updating {
                        app.is_updating = true;

                        std::thread::spawn(move || match crate::updater::perform_update() {
                            Ok(_) => {
                                std::process::exit(42);
                            }
                            Err(e) => {
                                eprintln!("Ошибка обновления: {}", e);
                            }
                        });
                    }

                    if ui.button("Позже").clicked() {
                        app.show_update_dialog = false;
                    }
                });

                if app.is_updating {
                    ui.spinner();
                    ui.label("Скачиваю обновления...");
                }
            });
    }
}

fn draw_connection(ui: &mut egui::Ui, app: &mut P2PApp) {
    ui.label("Ник:");
    ui.add(egui::TextEdit::singleline(&mut app.username).desired_width(f32::INFINITY));
    ui.label("Комната:");
    ui.add(egui::TextEdit::singleline(&mut app.room_name).desired_width(f32::INFINITY));
}

fn draw_devices(ui: &mut egui::Ui, app: &mut P2PApp) {
    ui.label("Микрофон:");
    egui::ComboBox::from_id_source("mic")
        .selected_text(&app.selected_input)
        .width(ui.available_width())
        .show_ui(ui, |ui| {
            for dev in &app.available_inputs {
                ui.selectable_value(&mut app.selected_input, dev.clone(), dev);
            }
        });
    ui.label("Динамики:");
    egui::ComboBox::from_id_source("out")
        .selected_text(&app.selected_output)
        .width(ui.available_width())
        .show_ui(ui, |ui| {
            for dev in &app.available_outputs {
                ui.selectable_value(&mut app.selected_output, dev.clone(), dev);
            }
        });
    if ui.button("Обновить").clicked() {
        let (ins, outs) = audio::get_audio_devices();
        app.available_inputs = ins;
        app.available_outputs = outs;
    }
}

fn draw_controls(ui: &mut egui::Ui, app: &mut P2PApp) {
    let mut muted = app.is_muted.load(Ordering::Relaxed);
    if ui.checkbox(&mut muted, "Выкл. Микрофон").changed() {
        app.is_muted.store(muted, Ordering::Relaxed);
    }
    let mut deafened = app.is_deafened.load(Ordering::Relaxed);
    if ui.checkbox(&mut deafened, "Выкл. Динамики").changed() {
        app.is_deafened.store(deafened, Ordering::Relaxed);
    }
    ui.add_space(10.0);

    if app.is_connected {
        if ui.button("Отключиться").clicked() {
            app.kill_signal.store(true, Ordering::Relaxed);
            app.is_connected = false;
            *app.status_text.lock().unwrap() = "Отключение...".to_string();

            app.active_peers.lock().unwrap().clear();

            std::thread::sleep(std::time::Duration::from_millis(200));
        }
    } else {
        if ui.button("Подключиться").clicked() {
            app.is_connected = true;
            app.kill_signal.store(false, Ordering::Relaxed);
            *app.status_text.lock().unwrap() = "Подключение...".to_string();
            engine::start_voice_engine(
                app.username.clone(),
                app.room_name.clone(),
                app.selected_input.clone(),
                app.selected_output.clone(),
                app.volume_level.clone(),
                app.status_text.clone(),
                app.kill_signal.clone(),
                app.is_muted.clone(),
                app.is_deafened.clone(),
                app.active_peers.clone(),
            );
        }
    }
}

fn draw_peers(ui: &mut egui::Ui, app: &mut P2PApp) {
    egui::ScrollArea::vertical().show(ui, |ui| {
        let mut peers = app.active_peers.lock().unwrap();
        if peers.is_empty() {
            ui.vertical_centered(|ui| {
                ui.add_space(50.0);
                ui.label(egui::RichText::new("Пока никого нет...").italics());
            });
        } else {
            let now = Instant::now();
            for (_addr, state) in peers.iter_mut() {
                egui::Frame::none()
                    .fill(ui.visuals().extreme_bg_color)
                    .rounding(5.0)
                    .inner_margin(8.0)
                    .show(ui, |ui| {
                        let is_speaking = now.duration_since(state.last_spoken).as_millis() < 300;
                        ui.horizontal(|ui| {
                            let icon = if is_speaking {
                                egui::RichText::new("🔊").color(egui::Color32::GREEN)
                            } else {
                                egui::RichText::new("🔈")
                            };
                            ui.label(icon);
                            ui.label(egui::RichText::new(&state.name).strong());
                            ui.label(format!("({} ms)", state.ping_ms));
                        });
                        ui.add(egui::Slider::new(&mut state.volume, 0.0..=2.0).text("Громкость"));
                    });
                ui.add_space(5.0);
            }
        }
    });
}
