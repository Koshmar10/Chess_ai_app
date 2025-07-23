use crate::engine::{board::CastleType, Board, ChessPiece, PieceType};

impl Board{

    pub fn filter_capture_moves(&self, piece: &ChessPiece, moves: &Vec<(u8,u8)>) -> Vec<(u8,u8)> {
        
        moves.iter().filter(|pos| {
            match self.squares[pos.0 as usize][pos.1 as usize] {
                Some(target) => {
                    //special checks for pawn
                    if piece.kind == PieceType::Pawn{
                        if piece.position.1 != pos.1 {
                            true
                        }else {
                            false
                        }
                    } else if piece.kind == PieceType::King{
                        if piece.color == target.color {
                            if target.kind == PieceType::Rook{
                                true
                            }else {false}
                            
                        }else{
                            if piece.position.1.abs_diff(target.position.1) > 1{
                                false
                            }
                            else {true}
                        }
                    }
                    else {
                        if piece.color != target.color {true} else {false}
                    }
                    
                    
                },
                None => {
                    //keep the pawn invalid captures for en passant
                    if piece.kind == PieceType::Pawn{
                        if piece.position.1 != pos.1 {
                            true
                        }else {
                            false
                        }
                    } 
                    else {false}
                }  
            }
        }).cloned().collect()
}
pub fn legalize_capture_moves(&self, piece: &ChessPiece, capture_moves: Vec<(u8,u8)>) ->Vec<(u8,u8)>{
    let mut valid_capture_moves = Vec::new();
    if piece.kind ==PieceType::Pawn {
        for mv in capture_moves {
            match self.squares[mv.0 as usize][mv.1 as usize] {
                Some(_) => {
                    // this means that this is a non en passant move
                    
                    if !self.simulate_move(piece, &mv){
                        valid_capture_moves.push(mv);    
                    }
                } 
                None => {
                    //capture square is empty so we check for en passant
                    match self.squares[piece.position.0 as usize][mv.1 as usize] {
                        Some(adjacent_piece) => {
                            match self.en_passant_target {
                                Some(target_pos) => {
                                    if adjacent_piece.position == target_pos {
                                
                                if !self.simulate_move(piece, &mv){
                                    valid_capture_moves.push(mv);    
                                }
                            }
                                }
                                _=> {}
                            }
                            
                        }
                        None => {
                            continue;
                        }
                    }
                }
                
            }
            
        } }
        else if piece.kind == PieceType::King {
            for mv in capture_moves{
                match self.squares[mv.0 as usize][mv.1 as usize] {
                    Some(p) => {
                        if p.kind == PieceType::Rook  {
                            if p.color == piece.color {
                                
                                
                                if mv.1.abs_diff(piece.position.1) == 4 && self.can_castle(CastleType::QueenSide, p.color) {
                                    valid_capture_moves.push(mv);
                                }
                                if mv.1.abs_diff(piece.position.1) == 3 && self.can_castle(CastleType::KingSide, p.color){
                                    valid_capture_moves.push(mv);
                                }
                                
                            }
                            
                            
                            else {
                                if !self.simulate_move(piece, &mv){
                                    valid_capture_moves.push(mv);    
                                }    
                            }
                        }
                        else {
                            if !self.simulate_move(piece, &mv){
                                valid_capture_moves.push(mv);    
                            }
                        }
                    }
                    None => {
                        if !self.simulate_move(piece, &mv){
                            valid_capture_moves.push(mv);    
                        }
                    }
                }
            } }
            else{ 
                for mv in capture_moves{
                    
                    if !self.simulate_move(piece, &mv){
                        valid_capture_moves.push(mv);    
                    }
                    
                }
            }
            valid_capture_moves
        }
}