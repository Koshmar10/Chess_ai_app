use std::{fmt::format, io::Seek, mem::swap};

use eframe::{egui::Color32, App};
use crate::etc::{self, DEFAULT_FEN, DEFAULT_STARTING};
// use crate::Myapp; // Removed or commented out as Myapp is not defined
#[derive(Debug, Clone, Copy,  PartialEq, Eq, Hash)]
pub enum PieceType {
    Pawn,
    Rook,
    Queen,
    Bishop,
    Knight,
    King,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PieceColor{
    White,
    Black
}
pub struct Board{
    pub squares: [[Option<ChessPiece>; 8]; 8],
    pub turn: PieceColor,
    pub wihte_big_castle: bool,
    pub black_big_castle: bool,
    pub wihte_samll_castle: bool,
    pub black_small_castle: bool,
    pub halfmove_clock: u32,
    pub fullmove_number: u32,
    pub state: BoardState,

}

pub struct BoardState {
    pub selected_piece: Option<ChessPiece>,
    pub quiet_moves: Option<Vec<(u8, u8)>>,
    pub capture_moves: Option<Vec<(u8, u8)>>,
    pub pov: PieceColor,
}

impl Default for BoardState{
    fn default() -> Self {
        Self{
            selected_piece: None,
            quiet_moves: None,
            capture_moves: None,
            pov: DEFAULT_STARTING,
        }
    }
}
#[derive(Clone, Copy, Debug)]
pub struct ChessPiece {
    pub kind: PieceType,
    pub color: PieceColor,
    pub position: (u8, u8),
    pub times_moved: i32,
}

impl ToString for PieceColor {
    fn to_string(&self) -> String {
        match self {
            PieceColor::White => "White".to_owned(),
            PieceColor::Black => "Black".to_owned(),
        }
    }
}
impl ToString for PieceType {
    fn to_string(&self) -> String {
        match self {
            PieceType::King => "King".to_owned(),
            PieceType::Queen => "Queen".to_owned(),
            PieceType::Bishop => "Bishop".to_owned(),
            PieceType::Knight => "Knight".to_owned(),
            PieceType::Pawn => "Pawn".to_owned(),
            PieceType::Rook => "Rook".to_owned(),
        
        }
    }
    
}
// Assuming you meant to implement this for a struct named MyApp, define it properly
pub enum FenError{
    InvalidChar(char)
}
pub enum MoveError{
    IllegalMove,
    NoAviailableMoves,
}

    pub fn fen_parser(fen: &String) -> Result<Board, FenError>{
        // Implementation goes here
        let mut pieces = Vec::<ChessPiece>::new();
        // split into exactly six fields
        let parts: Vec<&str> = fen.split_whitespace().collect();
        let board_representation    = parts[0];
        let to_move                 = parts[1];
        let castling_rights         = parts[2];
        let en_passant_targets      = parts[3];
        let halfmove_clock: u32     = parts[4].parse().unwrap();
        let fullmove_number: u32    = parts[5].parse().unwrap();
        let fen_files : Vec<&str> = board_representation.split("/").collect();
        let mut i: u8 = 0;
        for file in fen_files {
            let mut j: u8 =0;
            for elem in file.chars(){
                if elem.is_numeric(){
                    j+=(elem.to_digit(10).unwrap() as u8);

                }
                else{
                    let (kind, color, pos, move_count) = match elem {
                        'r' => {(PieceType::Rook, PieceColor::Black, (i, j), 0)}
                        'n' => {(PieceType::Knight, PieceColor::Black, (i, j), 0)}
                        'b' => {(PieceType::Bishop, PieceColor::Black, (i, j), 0)}
                        'k' => {(PieceType::King, PieceColor::Black, (i, j), 0)}
                        'q' => {(PieceType::Queen, PieceColor::Black, (i, j), 0)}
                        'p' => {(PieceType::Pawn, PieceColor::Black, (i, j), 0)}
                        'R' => {(PieceType::Rook, PieceColor::White, (i, j), 0)}
                        'N' => {(PieceType::Knight, PieceColor::White, (i, j), 0)}
                        'B' => {(PieceType::Bishop, PieceColor::White, (i, j), 0)}
                        'K' => {(PieceType::King, PieceColor::White, (i, j), 0)}
                        'Q' => {(PieceType::Queen, PieceColor::White, (i, j), 0)}
                        'P' => {(PieceType::Pawn, PieceColor::White, (i, j), 0)}
                        (c) => return Err(FenError::InvalidChar(c))
                    };
                    pieces.push(
                        ChessPiece { 
                            kind, 
                            color, 
                            position: pos, 
                            times_moved: move_count }
                    );
                    j+=1;
                }
            }
            i+=1;
        }
        let mut board: [[Option<ChessPiece>; 8]; 8] = [[None; 8]; 8];
        for piece in pieces {
            let (x, y) = piece.position;
            board[x as usize][y as usize] = Some(piece);
        }
        Ok(Board { squares: board, turn: 
            if to_move == "w" {PieceColor::White} else {PieceColor::Black}, 
            wihte_big_castle: if castling_rights.contains("Q") {true} else {false}, 
            black_big_castle: if castling_rights.contains("q") {true} else {false},
            wihte_samll_castle: if castling_rights.contains("K") {true} else {false}, 
            black_small_castle: if castling_rights.contains("k") {true} else {false}, 
            halfmove_clock,
            fullmove_number,
            state: BoardState::default(),
        })
    }
impl Default for Board{
    fn default() -> Self {
    match fen_parser(&DEFAULT_FEN.to_owned()){
        Ok(board) => return board,
        Err( e) => return Board{
            squares: [[None; 8]; 8],
            turn: PieceColor::White,
            wihte_big_castle: true,
            wihte_samll_castle: true,
            black_big_castle: true, 
            black_small_castle: true, 
            halfmove_clock: 0,
            fullmove_number: 1,
            state:BoardState::default(),

        },   
    }
}      
}
impl From<&String> for Board{
    fn from(fen: &String) -> Self {
        match fen_parser(fen){
            Ok(b) => b,
            Err(e) => Board::default(),
        }
    }
}
impl ToString for Board {
    fn to_string(&self) -> String {
        let mut board_string = "".to_owned();
        let to_move: &str = if self.turn ==PieceColor::White {"w"} else {"b"};
        let mut castleing_rights = 
            (if self.wihte_samll_castle {"K"} else {""}).to_string() +
            if self.wihte_big_castle {"Q"} else {""} +
            if self.black_small_castle {"k"} else {""} +
            if self.black_big_castle {"q"} else {""};
        if castleing_rights == ""{castleing_rights = "-".to_owned()}
        for (i, row) in self.squares.iter().enumerate(){
            let mut empty_squares =0;
            for (j, piece) in row.iter().enumerate(){
                match &piece{
                    Some(p) => {
                        if empty_squares != 0 {board_string+= &format!("{}", empty_squares)};
                        board_string += match (p.kind, p.color) {
                            (PieceType::King, PieceColor::White) => "K",
                            (PieceType::Queen, PieceColor::White) => "Q",
                            (PieceType::Rook, PieceColor::White) => "R",
                            (PieceType::Knight, PieceColor::White) => "N",
                            (PieceType::Bishop, PieceColor::White) => "B",
                            (PieceType::Pawn, PieceColor::White) => "P",
                            (PieceType::King, PieceColor::Black) => "k",
                            (PieceType::Queen, PieceColor::Black) => "q",
                            (PieceType::Rook, PieceColor::Black) => "r",
                            (PieceType::Knight, PieceColor::Black) => "n",
                            (PieceType::Bishop, PieceColor::Black) => "b",
                            (PieceType::Pawn, PieceColor::Black) => "p",

                            
                        };
                        empty_squares=0;
                    }
                    None => {empty_squares+=1;}
                }
            }
            if empty_squares !=0 {
                board_string+=&empty_squares.to_string();
            }
            board_string+="/";

        }
        board_string= board_string.trim_end_matches("/").to_string();
        return board_string + " " +&to_move + " " + &castleing_rights + " " + &self.halfmove_clock.to_string() + " " + &self.fullmove_number.to_string();
            
    }
}
pub enum VerticalDirection {Up, Down}
pub enum HorizontalDirection{Left, Right}
impl Board {
   

    pub fn move_piece(&mut self, new_pos: (u8, u8)) -> Result<(), MoveError>{
        let selected_piece = self.state.selected_piece.ok_or(MoveError::IllegalMove)?;
        let (old_rank, old_file) = selected_piece.position;
        let quiet_moves = if let Some(quiet_moves) = &self.state.quiet_moves {quiet_moves} else { 
            return Err(MoveError::IllegalMove);
        };
        let capture_moves = if let Some(capture) = &mut self.state.capture_moves { 
            capture
        } else { 
            return Err(MoveError::IllegalMove);
        };
      
        if quiet_moves.contains(&new_pos) || capture_moves.contains(&new_pos) {
            //move exists now make move
            match &mut self.squares[old_rank as usize][old_file as usize] {
                Some(piece) => {
                    piece.position = (new_pos.0, new_pos.1);
                    piece.times_moved+=1;
                }
                None => {

                }
            };
            // calculate en passant take
            if selected_piece.kind == PieceType::Pawn && new_pos.1 != selected_piece.position.1{
                let back_direction:i8 = if self.state.pov == PieceColor::Black {-1} else {1};
                match &self.squares[new_pos.0 as usize][new_pos.1 as usize] {
                    None => {
                            self.squares[(new_pos.0 as i8 + back_direction ) as usize ][new_pos.1 as usize] = None;
                    }
                    _ => {}                 
                }
            }
            //the piece change
            self.squares[new_pos.0 as usize][new_pos.1 as usize] = None;
            self.squares[new_pos.0 as usize][new_pos.1 as usize] = self.squares[old_rank as usize][old_file as usize].clone();
            self.squares[old_rank as usize][old_file as usize]= None;
            
            


            self.turn = match self.turn {
                PieceColor::Black => PieceColor::White,
                PieceColor::White => PieceColor::Black,
            };
            Ok(())
        } else {
            return Err(MoveError::NoAviailableMoves);
        }


      
    }

    pub fn legalize_capture_moves(&self, piece: &ChessPiece, capture_moves: Vec<(u8, u8)>) -> Vec<(u8,u8)> {
     
                capture_moves.into_iter().filter(|pos| {
                    match piece.kind {
                        PieceType::Pawn => {
                            //En passant check
                            
                            match self.squares[pos.0 as usize][pos.1 as usize] {
                                Some(_) => {
                                    //if the capture square contains a piece then it s valid
                                    return true;
                                }
                                None =>{
                                    let under = pos.0.checked_sub(1);
                                    match under {
                                        Some(under) =>{
                                            //check if under capture move is a pawn
                                             match &self.squares[under as usize][pos.1 as usize] {
                                                Some(under_piece) =>{
                                                    if under_piece.kind == PieceType::Pawn && under_piece.color!= piece.color && under_piece.times_moved == 1{return true}
                                                    return false
                                                }
                                                None => {
                                                    return false;
                                                }
                                                }
                                        
                                        }
                                        None => {
                                            return false;
                                        }
                                    }
                                }
                                
                            }
                        },
                        _ => true,
                        
                    }
                }).collect()
            }
            pub fn get_quiet_moves(&self, piece: &ChessPiece) -> Vec<(u8, u8)>{
                let mut quiet_moves:Vec<(u8, u8)> = Vec::new();
                match piece.kind {
                    PieceType::Pawn => {
                        if piece.times_moved == 0{
                            quiet_moves.extend(self.get_file_moves(piece, 2, VerticalDirection::Up));
                        }else{
                            quiet_moves.extend(self.get_file_moves(piece, 1, VerticalDirection::Up));
                        }
                    }
                    _ =>{}
                }
        
                quiet_moves
            }
        
            pub fn get_capture_moves(&self, piece: &ChessPiece) ->Vec<(u8,u8)>{
                let mut capture_moves: Vec<(u8,u8)> = Vec::new();
                match piece.kind {
                    PieceType::Pawn => {
                            capture_moves.extend(self.get_diagonal_moves(piece, 1, (VerticalDirection::Up, HorizontalDirection::Right)));
                            capture_moves.extend(self.get_diagonal_moves(piece, 1, (VerticalDirection::Up, HorizontalDirection::Left)));
        
                    }       
                    _ =>{}
                }
                self.legalize_capture_moves(piece, capture_moves)
            }
    pub fn get_diagonal_moves(
        &self,
        piece: &ChessPiece,
        depth: u8,
        dir: (VerticalDirection, HorizontalDirection),
    ) -> Vec<(u8, u8)> {
        let (mut dr, mut dc) = match dir {
            (VerticalDirection::Up,    HorizontalDirection::Left)  => (1, -1),
            (VerticalDirection::Up,    HorizontalDirection::Right) => (1,  1),
            (VerticalDirection::Down,  HorizontalDirection::Left)  => (-1, -1),
            (VerticalDirection::Down,  HorizontalDirection::Right) => (-1,  1),
        };
        if piece.color != self.state.pov {
            dc*=-1; dr*=-1;
        }
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
        if piece.color != self.state.pov {
            dir*=-1;
        }
        let mut x = piece.position.0 as i8;
        let y = piece.position.1 as i8;
        for _ in 0..depth {
            x+=dir;
            // simple bounds check
            if !(0..8).contains(&x)  {
                break;
            }
            match self.squares[x as usize][y as usize] {
                // friendly piece ⇒ stop
                Some(p) if p.color == piece.color => break,
                // enemy ⇒ capture square, then stop
                Some(_) => {
                    moves.push((x as u8, y as u8));
                    break;
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
            VerticalDirection::Down => -1,
            VerticalDirection::Up => 1,
        };
        if piece.color != self.state.pov {
            dir*=-1;
        }
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
                Some(_) => {break;}
                // empty ⇒ push & continue
                None => {
                    moves.push((x as u8, y as u8));
                }
            }
        }
        moves
    }
pub fn get_knight_moves(&self, piece: &ChessPiece){
    let (kr, kf) = piece.position;
    
}
    pub fn flip_board(&mut self) {
        let mut fliped_board: [[Option<ChessPiece>; 8]; 8] = [[None; 8]; 8];
        for (i, row) in self.squares.iter_mut().enumerate() {
            for (j, col) in row.iter_mut().enumerate() {
                match col {
                    Some(piece) =>{
                        piece.position = (7-i as u8, 7-j as u8);
                    }
                    None=> {}
                }
                fliped_board[7-i][7-j] = *col;

            }
        }
        self.state = BoardState::default();
        self.state.pov = match self.state.pov {
            PieceColor::Black => PieceColor::White,
            PieceColor::White => PieceColor::Black,
        }; 
        match self.state.selected_piece {
            Some(p) => {
                self.select_piece(p);
            }
            none=> {}
        }
        self.squares = fliped_board;
    }
    pub fn select_piece(&mut self, piece: ChessPiece) {
        let captures = self.get_capture_moves(&piece);
        let quiets   = self.get_quiet_moves(&piece);
        self.state.selected_piece = Some(piece);
        self.state.capture_moves  = Some(captures);
        self.state.quiet_moves    = Some(quiets);
    }
    pub fn deselect_piece(&mut self){
        self.state.capture_moves = None;
        self.state.selected_piece = None;
        self.state.quiet_moves = None;
    }
}
