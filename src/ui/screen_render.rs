use eframe::egui::{self, vec2, Button, CentralPanel, Color32, CornerRadius, Frame, SidePanel, Stroke};

use crate::{engine::{Board, PieceColor, PieceType}, game::controller::{GameController, GameMode}, ui::{app::{AppScreen, MyApp}, DEFAULT_FEN}};



impl MyApp{
    pub fn render_main_menu(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame){
            
            // Take up one third of the available width:
            let third = ctx.available_rect().width() / 6.0;
            SidePanel::left("left-separator")
                .min_width(third)
                .max_width(third)
                .show(ctx, |_| {});
            SidePanel::right("right-separator")
            .min_width(third)
            .max_width(third)
            .show(ctx, |_| {});
        
            
            CentralPanel::default()
                .frame(Frame::default().fill(Color32::BLACK))
                .show(ctx, |ui| {
                    // Center both horizontally and vertically:
                    
                    ui.add_space(ui.available_height()* 0.2);
                    let scale = if ui.available_width() < 400.0 { 0.8 } else { 1.0 };
                        ctx.request_repaint();
                        let title_size = self.ui.title_size * scale;
                        let subtitle_size = self.ui.subtitle_size * scale;
                    ui.vertical_centered(|ui| {
                        // Scale down to 80% if the available width is small
                       

                        // Title: bold & large
                        ui.label(
                            egui::RichText::new("Koch")
                                .heading()
                                .strong()
                                .size(title_size),
                        );
                        ui.add_space(12.0 * scale);

                        // Subtitle: smaller & fainter
                        // Before calling ui.label, compute a dynamic size based on the quote’s length:
                        let quote = match &self.ui.menu_quote {
                            None => {
                                self.ui.menu_quote = Some(self.get_quote());
                                self.ui.default_subtitle.clone()
                            }
                            Some(s) => s.clone(),
                        };
                        let len = quote.chars().count().max(1) as f32;
                        // we cap the width we expect it to occupy
                        let max_text_width = ui.available_width() * 0.8;
                        // derive a font size that shrinks for longer text, clamped to a reasonable range
                        let computed_size = (max_text_width / len * 1.5).clamp(12.0, subtitle_size);

                        ui.label(
                            egui::RichText::new(quote)
                                .weak()
                                .size(computed_size),
                        );
                        ui.add_space(12.0 * scale);
                    });
                    
                    ui.vertical_centered(|ui|{
                        let button_width = ui.available_width()*0.48;
                        
                        let train_btn = Button::new(egui::RichText::new("Train").raised().strong().size(18.0))
                        .corner_radius(CornerRadius::from(5.0))
                        .min_size(vec2(button_width, 40.0)); 
                        
                        ui.add_space(12.0 * scale);

                        let history_btn = Button::new(egui::RichText::new("Game History").raised().strong().size(18.0))
                        .corner_radius(CornerRadius::from(5.0))
                        .min_size(vec2(button_width, 40.0)); 

                        ui.add_space(12.0 * scale);

                        if ui.add(train_btn).clicked() {
                            self.screen = AppScreen::TrainWithAi;
                        }
                        ui.add_space(4.0);
                        if ui.add(history_btn).clicked() {
                            self.screen = AppScreen::History;
                        }
                            
                    } );
                    
                   
                });
                }
            
            

    
    pub fn render_train_with_ai(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame){
        SidePanel::left("menu")
            .resizable(true)
            .min_width(250.0)
            .default_width(250.0)
            .show(ctx, |ui| {
                ui.heading("Chess");
                ui.label(&self.board.to_string());
                ui.label(format!(
                    "piesa: {:?}",
                    match self.board.state.selected_piece {
                        Some(p) => self.board.squares[p.position.0 as usize][p.position.1 as usize],
                        None => None,
                    }
                ));
                ui.label(format!(
                    "passant square: {:?}",
                    match self.board.en_passant_target {
                        Some(p) => self.board.squares[p.0 as usize][p.1 as usize],
                        None => None,
                    }
                ));
                if ui.button("flip").clicked() {
                    self.board.state.pov = match self.board.state.pov {
                        PieceColor::White => PieceColor::Black,
                        PieceColor::Black => PieceColor::White,
                    };
                    ctx.request_repaint();
                };
                if ui.button("reset-board").clicked() {
                    self.board = Board::from(&DEFAULT_FEN.to_owned());
                };
                ui.separator();
                if ui.button("gameMode: Sandbox").clicked() {
                    self.game.mode = GameMode::Sandbox;
                }
                if ui.button("gameMode: PvE").clicked() {
                    self.game.mode = GameMode::PvE;
                }
                if ui.button("start-game").clicked() {
                   
                    self.board = Board::from(&DEFAULT_FEN.to_owned());
                   
                    self.game.game_over = false;
                    let colors = [PieceColor::White, PieceColor::Black];
                    let player_color = colors[rand::random::<i32>() as usize % 2];
                    self.game.player = player_color;
                    self.game.enemey = match player_color {
                        PieceColor::White => PieceColor::Black,
                        PieceColor::Black => PieceColor::White,
                    };
                    if self.board.state.pov != self.game.player {
                        self.board.state.pov = match self.board.state.pov {
                            PieceColor::White => PieceColor::Black,
                            PieceColor::Black => PieceColor::White,
                        };
                        ctx.request_repaint();
                    }
                    
                    self.start_stockfish();       // ← start the cmd_rx loop right away
                    
                    
                    
                };
                ui.label(format!("{:?}", self.game.game_over));
                ui.label(format!("{:?}{:?}", self.game.player, self.game.enemey));
                if ui.button("end-game").clicked() {
                    self.board = Board::from(&DEFAULT_FEN.to_owned());
                    self.game.game_over = true;
                    self.game = GameController::default();
                }

                ui.vertical(|ui| {
                    let check = if self.board.is_in_check(PieceColor::White) {"true"} else {"false"};
                    ui.label(format!("white_check: {}", check));
                    let check = if self.board.is_in_check(PieceColor::Black) {"true"} else {"false"};
                    ui.label(format!("black_check: {}", check));
                    let stale:bool = self.board.is_stale_mate();
                    ui.label(format!("satelmate {:?}", stale));
                    let mate :bool = self.board.is_chackmate();
                    ui.label(format!("checkmate {:?}", mate));

                });
                ui.label(format!("{:?}", self.board.state.pov));
                if let Some(quiet_moves) = &self.board.state.quiet_moves {
                    ui.separator();
                    ui.heading("Pseudo Moves:");
                    
                    for (row, col) in quiet_moves {
                        ui.label(format!("({}, {})", row, col));
                    }
                    
                }
                if let Some(capture_moves) = &self.board.state.capture_moves {
                   
                    
                    for (row, col) in capture_moves {
                        ui.label(format!("({}, {})", row, col));
                    }
                    
                }
            
            });
            CentralPanel::default() .frame(
        Frame::default()
            .fill(Color32::from_rgb(0x30, 0x30, 0x30))        // your background color            // optional corner rounding
            .stroke(Stroke::new(0.2, Color32::WHITE))   // optional border
    ).show(ctx, |ui|{
                    //UI SETUP
                    ui.spacing_mut().item_spacing= vec2(0.0, 0.0);
                    
                    let board_size = &self.ui.square_size * 8.0;
                    // Find top-left of centered board
                    let top_left = egui::pos2(
                        ui.min_rect().center().x - board_size / 2.0,
                        ui.min_rect().center().y - board_size / 2.0,
                    );
                    self.render_board(top_left, ui);
                    self.render_game_info(top_left, ui);
                    if let Some((x, y)) = self.board.state.promtion_pending {
                        egui::Window::new("Promote Pawn")
                            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                            .collapsible(false)
                            .resizable(false)
                            .show(ctx, |ui| {
                                ui.vertical_centered(|ui| {
                                    ui.label("Choose piece to promote to:");
                                    for kind in [
                                        PieceType::Bishop,
                                        PieceType::Knight,
                                        PieceType::Queen,
                                        PieceType::Rook,
                                    ] {
                                        if ui.button(kind.to_string()).clicked() {
                                            self.board.promote_pawn((x, y), kind);
                                            self.board.state.promtion_pending = None;
                                            ctx.request_repaint();
                                        }
                                    }
                                });
                            });
                    }
                    

                });
    }
}