use eframe::egui;
use std::sync::Arc;
use parking_lot::Mutex;

#[derive(Clone, Copy, PartialEq)]
pub enum AppStatus {
    Ready,
    Recording,
    Processing,
}

pub struct VibeOverlay {
    pub status: Arc<Mutex<AppStatus>>,
    pub amplitude: Arc<Mutex<f32>>,
    pub visible: Arc<Mutex<bool>>,
}

impl VibeOverlay {
    pub fn new(status: Arc<Mutex<AppStatus>>, amplitude: Arc<Mutex<f32>>, visible: Arc<Mutex<bool>>) -> Self {
        Self { status, amplitude, visible }
    }
}

impl eframe::App for VibeOverlay {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let is_visible = *self.visible.lock();
        
        ctx.send_viewport_cmd(egui::ViewportCommand::Visible(is_visible));
        
        if !is_visible {
            return;
        }

        let panel_frame = egui::Frame {
            fill: egui::Color32::from_rgba_unmultiplied(5, 5, 10, 200),
            rounding: egui::Rounding::same(12.0),
            stroke: egui::Stroke::new(1.0, egui::Color32::from_rgba_unmultiplied(0, 255, 255, 100)),
            inner_margin: egui::Margin::same(8.0),
            ..Default::default()
        };

        egui::CentralPanel::default()
            .frame(panel_frame)
            .show(ctx, |ui| {
                ui.vertical_centered(|ui| {
                    let status = *self.status.lock();
                    let amp = *self.amplitude.lock();
                    let time = ui.input(|i| i.time);

                    match status {
                        AppStatus::Recording => {
                            ui.label(egui::RichText::new("VIBE ACTIVE").color(egui::Color32::from_rgb(0, 255, 255)).strong().size(11.0));
                            ui.add_space(8.0);
                            
                            ui.horizontal(|ui| {
                                ui.set_height(30.0);
                                let spacing = 8.0;
                                let bar_width = 3.0;
                                
                                for i in 0..9 {
                                    // Organic movement using sine waves + amplitude
                                    let offset = i as f32 * 0.5;
                                    let wave = (time as f32 * 4.0 + offset).sin() * 0.3 + 0.7;
                                    let height = (amp * 150.0 * wave).clamp(4.0, 28.0);
                                    
                                    let color = egui::Color32::from_rgb(0, 255, 255);
                                    let glow_color = egui::Color32::from_rgba_unmultiplied(0, 255, 255, 40);
                                    
                                    let (rect, _) = ui.allocate_at_least(egui::vec2(bar_width + spacing, 30.0), egui::Sense::hover());
                                    let center = rect.center();
                                    
                                    // 1. Outer Glow (Bloom)
                                    ui.painter().rect_filled(
                                        egui::Rect::from_center_size(center, egui::vec2(bar_width + 4.0, height + 4.0)),
                                        4.0,
                                        glow_color
                                    );
                                    
                                    // 2. Inner Glow
                                    ui.painter().rect_filled(
                                        egui::Rect::from_center_size(center, egui::vec2(bar_width + 2.0, height + 2.0)),
                                        3.0,
                                        egui::Color32::from_rgba_unmultiplied(0, 255, 255, 80)
                                    );
                                    
                                    // 3. Core Bar
                                    ui.painter().rect_filled(
                                        egui::Rect::from_center_size(center, egui::vec2(bar_width, height)),
                                        2.0,
                                        egui::Color32::WHITE
                                    );
                                }
                            });
                        }
                        AppStatus::Processing => {
                            let glow_alpha = ((time * 4.0).sin() * 50.0 + 150.0) as u8;
                            ui.label(egui::RichText::new("THINKING...")
                                .color(egui::Color32::from_rgba_unmultiplied(0, 255, 255, glow_alpha))
                                .strong());
                            
                            ui.add_space(10.0);
                            let progress = (time * 2.0).fract() as f32;
                            ui.add(egui::ProgressBar::new(progress).show_percentage().animate(true));
                        }
                        _ => {}
                    }
                });
            });

        ctx.request_repaint();
    }
}

pub fn run_gui(status: Arc<Mutex<AppStatus>>, amplitude: Arc<Mutex<f32>>, visible: Arc<Mutex<bool>>) {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_transparent(true)
            .with_decorations(false)
            .with_always_on_top()
            .with_inner_size([160.0, 75.0])
            .with_position([880.0, 940.0])
            .with_mouse_passthrough(true),
        ..Default::default()
    };

    let _ = eframe::run_native(
        "VibeFlow Overlay",
        options,
        Box::new(|_cc| Ok(Box::new(VibeOverlay::new(status, amplitude, visible)))),
    );
}
