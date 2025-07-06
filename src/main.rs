
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release
#![allow(rustdoc::missing_crate_level_docs)] // it's an example



use chess_app::{chess_utils::{ Board, ChessPiece}, etc::DEFAULT_FEN, theme, };
use eframe::{egui::{self, pos2, vec2, CentralPanel, Color32, Painter, Pos2, Rect, Sense, SidePanel, Stroke, Ui}, CreationContext};
use::chess_app::theme::ThemeLoader;
use chess_app::ui;
fn main() -> eframe::Result {
    //env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([320.0, 240.0]),
        ..Default::default()
    };
    eframe::run_native(
        "My egui App",
        options,
        Box::new(|cc| {
            // This gives us image support:
            egui_extras::install_image_loaders(&cc.egui_ctx);

            Ok(Box::new(MyApp::from(cc)))
        }),
    )
}
pub struct MyApp {
    pub theme: theme::ThemeLoader,
    pub board: Board,
    
}



impl From<&CreationContext<'_>> for MyApp{

    fn from(cc : &CreationContext) -> Self {

        Self {

            theme: ThemeLoader::from(cc),
            board: Board::from(&DEFAULT_FEN.to_owned()),
        }

        

    }

}



impl eframe::App for MyApp {

    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
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
                if ui.button("flip").clicked() {
                    self.board.flip_board();
                };
                if let Some(quiet_moves) = &self.board.state.quiet_moves {
                    ui.separator();
                    ui.heading("Pseudo Moves:");
                    
                    for (row, col) in quiet_moves {
                        ui.label(format!("({}, {})", row, col));
                    }
                    
                }
                if let Some(capture_moves) = &self.board.state.capture_moves {
                    ui.separator();
                    ui.heading("Pseudo Moves:");
                    
                    for (row, col) in capture_moves {
                        ui.label(format!("({}, {})", row, col));
                    }
                    
                }
            
            });
            CentralPanel::default().show(ctx, |ui|{
                    //UI SETUP
                    ui.spacing_mut().item_spacing= vec2(0.0, 0.0);
                  
                    self.render_board(ui);
                });

        

    }  

}
impl MyApp {
    pub fn render_board(&mut self, ui : &mut Ui) {
        let available_size = ui.available_size();
                    let square_size = available_size.x / 16.0;
                    let board_size = square_size * 8.0;
                    // Find top-left of centered board

                    let top_left = egui::pos2(
                        ui.min_rect().center().x - board_size / 2.0,
                        ui.min_rect().center().y - board_size / 2.0,
                    );
                    let painter = ui.painter();
                    // clone the squares so we don't immutably borrow self.board
                    let squares = self.board.squares.clone();
                    for (i, row) in squares.iter().enumerate() {

                        for (j, col) in row.iter().enumerate() {

                            let x = top_left.x + j as f32 * square_size;
                            let y = top_left.y + (7 - i) as f32 * square_size;

                            let rect = egui::Rect::from_min_size(
                                egui::pos2(x, y),
                                egui::vec2(square_size, square_size),
                            );

                            let id  = ui.make_persistent_id((j, i));
                            let response = ui.interact(rect, id, Sense::all());

                            //rendering the color of teh boar
                            let color = if (i + j) % 2 == 0 {
                                if response.hovered(){
                                   self.theme.light_square_hover.to_opaque()
                                }
                                else{
                                    self.theme.light_square.to_opaque()
                                }
                            } else {
                                if response.hovered(){
                                    self.theme.dark_square_hover.to_opaque() // dark
                                }else{
                                    self. theme.dark_square.to_opaque() // dark
                                }
                            };
                            painter.rect_filled(rect, 0.0, color);

                            // Board interactions
                            
                            /* 
                            );
                            self.board.state.quiet_moves = None;
                            self.board.state.selected_piece = None;
                            */
                                                   
                            //Board ui rendering
                            self.render_selected(col, &rect, &painter);
                            self.render_piece(col, &rect, &painter);
                            self.render_quiet_move(&(i as u8,j as u8), &rect, &painter);
                            self.render_capture_move(&(i as u8,j as u8), &rect, &painter);

                            //rendering the piece
                            if response.secondary_clicked() {
                              self.board.deselect_piece();
                            }
                            if response.clicked() {
                              match self.board.state.selected_piece {
                                Some(selected_piece) => {
                                    //if a piece is already selected
                                    match col {
                                        Some(piece) => {
                                            if piece.color !=  selected_piece.color {
                                                match self.board.move_piece(piece.position){
                                                    Ok(_) => { println!("Ok");}
                                                    Err(_) => {
                                                        println!("Not Ok");
                                                    }
                                                }
                                                self.board.deselect_piece();
                                            }
                                            else{
                                                if piece.color == self.board.turn{

                                                    self.board.select_piece(*piece);
                                                    
                                                }
                                            }
                                        }
                                        None => {
                                            match self.board.move_piece((i as u8,j as u8)){
                                                Ok(_) => { println!("Ok");}
                                                Err(_) => {
                                                    println!("Not Ok");
                                                }
                                            }
                                            self.board.deselect_piece();
                                        }
                                        
                                    }
                                }
                                None => {
                                    //if piece not selected already select piece
                                    match col {
                                        Some(piece) =>{
                                            if piece.color == self.board.turn{
                                                self.board.select_piece(*piece);
                                               }
                                        }
                                        None => {
                                            self.board.deselect_piece();
                                        }
                                    }
                                }
                              }
                            }
                        }
                    }
    }

    pub fn render_quiet_move(&self, poz :&(u8, u8), rect: &Rect, painter: &Painter){
        match &self.board.state.quiet_moves {
            Some(moves) => {
                if moves.contains(poz) {
                    let center = rect.center();
                    let radius = rect.width().min(rect.height()) * 0.25;
                    painter.circle_filled(center, radius, 
                     if (poz.0 + poz.1) % 2 ==0 {self.theme.light_pseudo_move_highlight} else {self.theme.dark_pseudo_move_highlight});
                }
                else {return;}
            }
            None => {return;}
        }
    }
    pub fn render_capture_move(&self, poz :&(u8, u8), rect: &Rect, painter: &Painter){
            match &self.board.state.capture_moves {
                Some(moves) => {
                    if moves.contains(poz) {
                                        let center = rect.center();
                                        let radius = rect.width().min(rect.height()) * 0.25;
                                        painter.circle_stroke(
                                            center,
                                            radius,
                                            Stroke::from((
                                                3.0,
                                                if (poz.0 + poz.1) % 2 == 0 {
                                                    self.theme.light_pseudo_move_highlight
                                                } else {
                                                    self.theme.dark_pseudo_move_highlight
                                                },
                                            )),
                                        );
                                    }
                                
                            }
                            None => return,
                        }
                        
                    
    }
    pub fn render_selected(&self, piece: &Option<ChessPiece>, rect: &Rect, painter: &Painter){
        match piece {    
            Some(p) =>{
                match self.board.state.selected_piece{
                    Some (selected_piece) => {

                        if p.position == selected_piece.position {
                            painter.rect_filled(*rect, 0.0, self.theme.square_select_highlight);
                        }
                    }
                    None => {

                    }
                }
            }
            None => {

            }
        }
    }
    pub fn render_piece(&self, piece: &Option<ChessPiece>, rect: &Rect, painter: &Painter){
        match piece {
            Some(p) =>{
                painter.image(
                match self.theme.piece_map.get(&(p.kind, p.color)) {
                    Some(rez) => {
                        match rez {
                            Ok(texture) => {
                                texture.id()
                            }
                            Err(err) =>{
                                self.theme.empty_texture.id()
                            }
                        }
                    }
                    None => {
                        self.theme.empty_texture.id()
                    }
                },
                *rect,
                Rect { min: Pos2 { x: 0.0, y: 0.0 }, max: Pos2{ x: 1.0, y:1.0}},
                Color32::WHITE
            );
        }
        None => {

        }
    }
}
}




