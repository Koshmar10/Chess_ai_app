use eframe::egui::Response;

use crate::{engine::{ChessPiece, PieceColor, PieceType}, game::{controller::GameMode, stockfish_engine::{StockfishCmd, StockfishResult}}, ui::app::{MyApp, PopupType}};


impl MyApp{
    pub fn handle_board_interaction_logic(&mut self,  piece: &Option<ChessPiece>, poz :&(u8, u8), response :&Response) {
    match self.game.mode {
        GameMode::PvE => {
            // start stockfish
            
            if !self.game.game_over{
                if self.game.player != self.board.turn
                {
                if !self.game.stockfish_move_pending {
                    self.game.stockfish_move_pending = true; // Mark move in progress
                    // send UCI "go" command to Stockfish
                    if let Some(tx) = &self.game.stockfish_tx {
                        let _ = tx.send(StockfishCmd::Go(self.board.to_string()));
                    }
                } else {
                    // try to receive Stockfish result and apply it
                    if let Some(rx) = &self.game.stockfish_rx {
                        if let Ok(StockfishResult::Move(mv)) = rx.try_recv() {
                            println!("recived");
                            println!("move {}", &mv);
                            match self.board.decode_uci_move(mv) {
                                Some((from, to)) => {
                                        match self.board.move_piece(from, to){
                                            Err(e) => {
                                                println!("cannot move");
                                            }
                                            _ => {}
                                        }
                                    
                                }
                                None => {}
                            }
                            
                            self.game.stockfish_move_pending = false;
                        }
                    }
                }
                      
                } else {
        
                    if response.secondary_clicked() {
                        self.board.deselect_piece();
                    }
                if response.clicked() {
                if self.board.state.promtion_pending.is_some(){
                    //we need to handle promotion before doing anything lese
                    
                    

                } else {
                match self.board.state.selected_piece {
                    Some(selected_piece) => {
                        //if a piece is already selected
                        match piece {
                            Some(piece) => {
                                if piece.color !=  selected_piece.color {
                                    match self.board.move_piece(selected_piece.position, piece.position){
                                        Ok(_) => { println!("Ok");}
                                        Err(_) => {
                                            println!("Not Ok");
                                        }
                                    }
                                    self.board.deselect_piece();
                                }
                                else{
                                    if piece.color == self.board.turn{
                                        if selected_piece.kind == PieceType::King && piece.kind == PieceType::Rook && !self.board.is_in_check(piece.color){
                                            match &self.board.state.capture_moves{
                                                Some(moves) => {
                                                    if moves.contains(&piece.position){
                                                    self.board.execute_castle(selected_piece.position, piece.position);
                                                    self.board.deselect_piece();
                                                    }
                                                }
                                                _=>{}
                                            }
                                        }else{
                                            if self.game.player == piece.color{
        
                                                self.board.select_piece(*piece);
                                            }
                                        }
                                        
                                    }
                                }
                            }
                            None => {
                                match self.board.move_piece(selected_piece.position, *poz){
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
                        match piece {
                            Some(piece) =>{
                                if self.game.player == piece.color{
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
                if self.board.has_lost() { 
                    self.game.game_over = true;
                    self.popup = Some(PopupType::GameLostPopup("ai perdut".to_owned()));
                    let king_pos = self
                        .board
                        .squares
                        .iter()
                        .enumerate()
                        .find_map(|(r, row)| {
                            row.iter().enumerate().find_map(|(c, &cell)| {
                                if let Some(piece) = cell {
                                    if piece.kind == PieceType::King && piece.color == self.board.turn {
                                        Some((r as u8, c as u8))
                                    } else {
                                        None
                                    }
                                } else {
                                    None
                                }
                            })
                        });
                    self.board.state.checkmate_square = king_pos;
                   
                }
            }
            
        }
        GameMode::Sandbox => {
            if response.secondary_clicked() {
                self.board.deselect_piece();
            }
            if response.clicked() {
            match self.board.state.selected_piece {
                Some(selected_piece) => {
                    //if a piece is already selected
                    match piece {
                        Some(piece) => {
                            if piece.color !=  selected_piece.color {
                                match self.board.move_piece(selected_piece.position, piece.position){
                                    Ok(_) => { 
                                       
                                        println!("Ok");}
                                    Err(_) => {
                                        println!("Not Ok");
                                    }
                                }
                                self.board.deselect_piece();
                            }
                            else{
                                if piece.color == selected_piece.color{
                                    if selected_piece.kind == PieceType::King && piece.kind == PieceType::Rook && !self.board.is_in_check(piece.color){
                                        match &self.board.state.capture_moves{
                                            Some(moves) => {
                                                if moves.contains(&piece.position){
                                                self.board.execute_castle(selected_piece.position, piece.position);
                                                self.board.deselect_piece();
                                                }
                                            }
                                            _=>{}
                                        }

                                    }else{
                                            self.board.select_piece(*piece);
                                        
                                    }
                                    
                                }
                            }
                        }
                        None => {
                            match self.board.move_piece(selected_piece.position, *poz){
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
                    match piece {
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
        _ => {}
    }
    
}
}