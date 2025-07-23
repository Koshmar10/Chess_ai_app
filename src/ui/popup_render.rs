use eframe::egui;

use crate::ui::app::{MyApp, PopupType};



impl MyApp {
    pub fn popup_handler(&mut self, popup: &PopupType, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        match popup {
            PopupType::GameLostPopup(msg) =>{
                egui::Window::new("Promote Pawn")
                            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                            .collapsible(false)
                            .resizable(false)
                            .show(ctx, |ui| {
                                ui.vertical_centered(|ui| {
                                    ui.label(msg);
                                    if ui.button("x").clicked(){
                                        self.popup = None;
                                    }
                                });
                            });
            }
        }
    }
}