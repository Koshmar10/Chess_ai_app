use std::collections::HashMap;

use crate::engine::{Board, PieceType};

impl Board{

    pub fn decode_uci_move(&mut self, uci_move: String) -> Option<((u8, u8), (u8, u8))> {
        let uci_move = uci_move.as_str();

        // Mapping for algebraic notation to board coordinates
        let file_map: HashMap<char, u8> = [
            ('a', 0), ('b', 1), ('c', 2), ('d', 3),
            ('e', 4), ('f', 5), ('g', 6), ('h', 7),
        ].iter().cloned().collect();

        // Parse standard coordinates from UCI move
        let chars: Vec<char> = uci_move.chars().collect();
        let from_file = file_map[&chars[0]];
        let from_rank = 8 - (chars[1].to_digit(10).unwrap() as u8);
        let to_file   = file_map[&chars[2]];
        let to_rank   = 8 - (chars[3].to_digit(10).unwrap() as u8);
        let from = (from_rank, from_file);
        let to = (to_rank, to_file);
        
        // 1) Handle promotions: e.g. "e7e8q"
        if uci_move.len() == 5 {
            // Execute the move first
            let result = self.move_piece(from, to);
            if result.is_err() {
                return None;
            }
            
            // Then handle the promotion
            let promotion_char = chars[4];
            let promotion_piece = match promotion_char {
                'q' | 'Q' => PieceType::Queen,
                'r' | 'R' => PieceType::Rook,
                'b' | 'B' => PieceType::Bishop,
                'n' | 'N' => PieceType::Knight,
                _ => panic!("Invalid promotion piece: {}", promotion_char),
            };
            
            // Apply the promotion
            self.promote_pawn(to, promotion_piece);
            
            // Mark that promotion is handled
            self.state.promtion_pending = None;
            
            return Some((from, to));
        }

        // 2) Handle castling (no changes needed to your castling code)
        match uci_move {
            "e1g1" => {
                let king_pos = (7, 4);
                let rook_pos = (7, 7);
                self.execute_castle(king_pos, rook_pos);
                None
            }
            "e1c1" => {
                let king_pos = (7, 4);
                let rook_pos = (7, 0);
                self.execute_castle(king_pos, rook_pos);
                None
            }
            "e8g8" => {
                let king_pos = (0, 4);
                let rook_pos = (0, 7);
                self.execute_castle(king_pos, rook_pos);
                None
            }
            "e8c8" => {
                let king_pos = (0, 4);
                let rook_pos = (0, 0);
                self.execute_castle(king_pos, rook_pos);
                None
            }
            // 3) Normal move (no changes needed)
            _ => Some((from, to))
        }
    }
}
