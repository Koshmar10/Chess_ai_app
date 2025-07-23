use crate::engine::{board::CastleType, Board, ChessPiece, PieceColor, PieceType};


pub enum VerticalDirection {Up, Down}
pub enum HorizontalDirection{Left, Right}
#[derive(Debug)]
pub enum MoveError{
    IllegalMove,
    NoAviailableMoves,
}
impl Board{

    pub fn get_all_moves(&self, piece: &ChessPiece) ->Vec<(u8, u8)> {
        let mut all_moves = Vec::new();
        let diagonals = 
        [(VerticalDirection::Up,HorizontalDirection::Left),
        (VerticalDirection::Up,HorizontalDirection::Right),
        (VerticalDirection::Down,HorizontalDirection::Left),
        (VerticalDirection::Down,HorizontalDirection::Right)];
        match piece.kind {
            PieceType::Bishop  =>{
                for dir in diagonals {
                    all_moves.extend(self.get_diagonal_moves(piece, 8, dir));
                }
                all_moves
            }
            PieceType::King =>{
                for dir in diagonals {
                    all_moves.extend(self.get_diagonal_moves(piece, 1, dir));
                }
                for h in [VerticalDirection::Down, VerticalDirection::Up]{
                    all_moves.extend(self.get_file_moves(piece, 1, h));
                }
                for h in [HorizontalDirection::Left, HorizontalDirection::Right]{
                    all_moves.extend(self.get_rank_moves(piece, 8, h));
                }
                
                all_moves
            }
            PieceType::Knight => {
                self.get_knight_moves(piece)
            }
            PieceType::Queen => {
                for dir in diagonals {
                    all_moves.extend(self.get_diagonal_moves(piece, 8, dir));
                }
                for h in [VerticalDirection::Down, VerticalDirection::Up]{
                    all_moves.extend(self.get_file_moves(piece, 8, h));
                }
                for h in [HorizontalDirection::Left, HorizontalDirection::Right]{
                    all_moves.extend(self.get_rank_moves(piece, 8, h));
                }
                all_moves
            }
            PieceType::Rook => {
                for h in [VerticalDirection::Down, VerticalDirection::Up]{
                    all_moves.extend(self.get_file_moves(piece, 8, h));
                }
                for h in [HorizontalDirection::Left, HorizontalDirection::Right]{
                    all_moves.extend(self.get_rank_moves(piece, 8, h));
                }
                all_moves
            }
            PieceType::Pawn => {
                let dep = if piece.times_moved ==0 {2} else {1};
                all_moves.extend(self.get_file_moves(piece, dep, VerticalDirection::Up));
                for h in [(VerticalDirection::Up,HorizontalDirection::Left),
                (VerticalDirection::Up,HorizontalDirection::Right)] {
                    all_moves.extend(self.get_diagonal_moves(piece, 1, h));
                }
                all_moves
            }
            
        }
    }
    pub fn set_legal_moves(&mut self, piece: &ChessPiece){
        let (quiet, captures) = self.get_legal_moves(piece);
        self.state.quiet_moves = Some(quiet);
        self.state.capture_moves = Some(captures);
    }

    pub fn get_legal_moves(&mut self, piece: &ChessPiece) -> (Vec<(u8, u8)>, Vec<(u8, u8)>) {
        let moves = self.get_all_moves(piece);
        
        let quiet = self.filter_quiet_moves(piece, &moves);
        let quiet = self.legalize_quiet_moves(piece, quiet);
        
        let captures = self.filter_capture_moves(piece, &moves);
        let captures = self.legalize_capture_moves(piece, captures);
         
        return (quiet, captures)

    }
    
    pub fn lega_capture_moves(&self, piece: &ChessPiece) -> Vec<(u8,u8)>{
        let moves = self.get_all_moves(piece);        
        let captures = self.filter_capture_moves(piece, &moves);
        let captures = self.legalize_capture_moves(piece, captures);
        return captures
    }



    pub fn get_diagonal_moves(
        &self,
        piece: &ChessPiece,
        depth: u8,
        dir: (VerticalDirection, HorizontalDirection),
    ) -> Vec<(u8, u8)> {
        let (mut dr, mut dc) = match dir {
            (VerticalDirection::Up,    HorizontalDirection::Left)  => (if piece.color == PieceColor::White {-1} else {1}, -1),
            (VerticalDirection::Up,    HorizontalDirection::Right) => (if piece.color == PieceColor::White {-1} else {1},  1),
            (VerticalDirection::Down,  HorizontalDirection::Left)  => (if piece.color == PieceColor::White {1} else {-1}, -1),
            (VerticalDirection::Down,  HorizontalDirection::Right) => (if piece.color == PieceColor::White {1} else {-1},  1),
        };
     
        let mut moves = Vec::new();
        // start from current square
        let mut r = piece.position.0 as i8;
        let mut c = piece.position.1 as i8;
        for _ in 0..depth {
            r += dr; c += dc;
            // simple bounds check
            if !(0..8).contains(&r) || !(0..8).contains(&c) {
                break;
            }
            match self.squares[r as usize][c as usize] {
                // friendly piece ⇒ stop
                Some(p) if p.color == piece.color => break,
                // enemy ⇒ capture square, then stop
                Some(_) => {
                    moves.push((r as u8, c as u8));
                    break;
                }
                // empty ⇒ push & continue
                None => {
                    moves.push((r as u8, c as u8));
                }
            }
        }
        moves
    }


    pub fn get_rank_moves(&self, piece: &ChessPiece, depth: u8, h: HorizontalDirection) -> Vec<(u8, u8)> {
        let mut moves:Vec<(u8, u8)> = Vec::new();
        let mut  dir: i8 = match h {
            HorizontalDirection::Left => -1,
            HorizontalDirection::Right => 1,
        };
      
        let  x = piece.position.0 as i8;
        let mut y = piece.position.1 as i8;
        for _ in 0..depth {
            y+=dir;
            // simple bounds check
            if !(0..8).contains(&y)  {
                break;
            }
            match self.squares[x as usize][y as usize] {
                // friendly piece ⇒ stop
                Some(p) => {
                    if p.color == piece.color{
                        if piece.kind == PieceType::King {
                            if p.kind == PieceType::Rook {
                                moves.push((x as u8, y as u8));break;
                            }
                            else{
                                break;
                            }
                        }
                        else {break;}
                    }else {
                        moves.push((x as u8, y as u8));break;
                    }
                    
                
                }
                // empty ⇒ push & continue
                None => {
                    moves.push((x as u8, y as u8));
                }
            }
        }
        moves
    }
    pub fn get_file_moves(&self, piece: &ChessPiece, depth: u8, h: VerticalDirection) -> Vec<(u8, u8)> {
        let mut moves:Vec<(u8, u8)> = Vec::new();
        let mut dir: i8 = match h {
            VerticalDirection::Down => if piece.color == PieceColor::White {1} else {-1},
            VerticalDirection::Up => if piece.color == PieceColor::White {-1} else {1},
        };
        
        let mut x = piece.position.0 as i8;
        let  y = piece.position.1 as i8;
        for _ in 0..depth {
            x+=dir;
            // simple bounds check
            if !(0..8).contains(&x) || !(0..8).contains(&y) {
                break;
            }
            match self.squares[x as usize][y as usize] {
                // friendly piece ⇒ stop
                Some(p) if p.color == piece.color => break,
                // enemy ⇒ capture square, then stop
                Some(_) => { moves.push((x as u8, y as u8));break;}
                // empty ⇒ push & continue
                None => {
                    moves.push((x as u8, y as u8));
                }
            }
        }
        moves
    }
    pub fn get_knight_moves(&self, piece: &ChessPiece) -> Vec<(u8, u8)>{
        let (kr, kf) = (piece.position.0 as i8, piece.position.1 as i8) ;
        let mut pseudo_moves : Vec<(u8,u8)> = Vec::new();
        let knight_pseudo_moves: &[(i8,i8)] = &[(2,1), (2,-1), (1,2), (-1, 2), (-2, 1), (-2, -1), (1,-2), (-1, -2)];
        for pm in knight_pseudo_moves {
            let (rank_dif, file_dif) = *pm;
          
            if kr + rank_dif < 8 && kr + rank_dif >=0 && kf + file_dif < 8 && kf+file_dif >=0 {
                pseudo_moves.push(((kr+rank_dif) as u8, (kf + file_dif) as u8));}
                
            }
            


        
        pseudo_moves
    }
    pub fn get_attack_squares(&self, piece: &ChessPiece) -> Vec<(u8, u8)> {
        match piece.kind {
            PieceType::Pawn => {
                let mut attacks = Vec::new();
                let (r, c) = (piece.position.0 as i8, piece.position.1 as i8);
                
                // Pawns attack diagonally - use directions with POV logic
                let mut direction = if piece.color == PieceColor::White { 1 } else { -1 };
                
                // Apply POV logic
                if piece.color != self.state.pov {
                    direction *= -1;
                }
                
                let attack_rank = r + direction;
                
                if (0..8).contains(&attack_rank) {
                    for &file_offset in &[-1, 1] {
                        let attack_file = c + file_offset;
                        if (0..8).contains(&attack_file) {
                            attacks.push((attack_rank as u8, attack_file as u8));
                        }
                    }
                }
                attacks
            }
            _ => {
                // For other pieces, use existing move generation which already has POV logic
                self.get_all_moves(piece)
            }
            }
        }
        pub fn can_castle(&self, castle_side: CastleType, color: PieceColor) -> bool{
            //first we check if pieces are in designated position 
            let (rook_pos, king_pos) = match castle_side {
                CastleType::KingSide => {
                    match color{
                        PieceColor::Black => {
                            if !self.black_small_castle { return false;}
                            ((0, 7), (0,4))
                        }
                        PieceColor::White =>{
                            if !self.white_small_castle { return false;}
                            
                            ((7, 7), (7, 4))
                        }
                    }        }
                CastleType::QueenSide => {
                    match color{
                        PieceColor::Black => {
                            if !self.black_big_castle { return false;}
                            
                            ((0,0), (0, 4))
                        }
                        PieceColor::White =>{
                            if !self.white_big_castle { return false;}
                            
                            ((7, 7), (7, 4))
                        }
                    }
                }
            };
            let (way, travel)  =match castle_side {
                CastleType::KingSide => {
                    (HorizontalDirection::Right, 2)
                }
                CastleType::QueenSide => {
                    (HorizontalDirection::Left, 3)
                }
            };
            let king = match self.squares[king_pos.0 as usize][king_pos.1 as usize] {
                Some(piece) => {
                if piece.times_moved == 0 { piece } else { return false; }
                }
                None => { return false; }
            };
            let rook = match self.squares[rook_pos.0 as usize][rook_pos.1 as usize] {
                Some(piece) => {
                if piece.times_moved == 0 { piece } else { return false; }
                }
                None => { return false; }
            };
            let travel_squares = self.get_rank_moves(&king, travel, way);
            for sqaure in travel_squares{
                match self.squares[sqaure.0 as usize][sqaure.1 as usize] {
                    Some(_) => return false,
                    None=> {
                        if self.simulate_move(&king, &sqaure) {return false;}
                    }
                }
            }
            true
        }
}