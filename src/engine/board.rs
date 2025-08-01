use crate::{engine::{fen::fen_parser, move_gen::MoveError, ChessPiece, PieceColor, PieceType}, etc::{DEFAULT_FEN, DEFAULT_STARTING}};

#[derive(Clone)]
pub struct Board{
    pub squares: [[Option<ChessPiece>; 8]; 8],
    pub turn: PieceColor,
    pub white_big_castle: bool,
    pub black_big_castle: bool,
    pub white_small_castle: bool,
    pub black_small_castle: bool,
    pub halfmove_clock: u32,
    pub fullmove_number: u32,
    pub en_passant_target: Option<(u8,u8)>,
    pub state: BoardState,

}

#[derive(Clone)]
pub struct BoardState {
    pub selected_piece: Option<ChessPiece>,
    pub moved_from: Option<(u8, u8)>,
    pub moved_to: Option<(u8, u8)>,
    pub quiet_moves: Option<Vec<(u8, u8)>>,
    pub capture_moves: Option<Vec<(u8, u8)>>,
    pub pov: PieceColor,
    pub white_taken:     Vec<ChessPiece>,
    pub black_taken:     Vec<ChessPiece>,
    pub promtion_pending: Option<(u8, u8)>,
    pub checkmate_square: Option<(u8, u8)>

}

#[derive(Clone)]
pub enum CastleType {QueenSide, KingSide}
    

impl Default for Board{
    fn default() -> Self {
    match fen_parser(&DEFAULT_FEN.to_owned()){
        Ok(board) => return board,
        Err( e) => return Board{
            squares: [[None; 8]; 8],
            turn: PieceColor::White,
            white_big_castle: true,
            white_small_castle: true,
            black_big_castle: true, 
            black_small_castle: true, 
            halfmove_clock: 0,
            fullmove_number: 1,
            en_passant_target: None,
            state:BoardState::default(),

        },   
    }
}      
}
impl Default for BoardState{
    fn default() -> Self {
        Self{
            selected_piece: None,
            quiet_moves: None,
            capture_moves: None, 
            pov: DEFAULT_STARTING,
            moved_from: None,
            moved_to: None,
            white_taken: Vec::new(),
            black_taken: Vec::new(),
            promtion_pending: None,
            checkmate_square: None,
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
impl Board {
    pub fn move_piece(&mut self, old_pos:(u8,u8), new_pos: (u8, u8)) -> Result<(), MoveError> {
        let (old_rank, old_file) = old_pos;
        let (new_rank, new_file) = new_pos;
        
        
        // Clone the piece to compute legal moves without holding a mutable borrow on self
        let piece = match self.squares[old_rank as usize][old_file as usize].as_ref() {
            Some(p) => p.clone(),
            None => return Err(MoveError::IllegalMove),
        };
        let (quiets, captures) = self.get_legal_moves(&piece);
        let capture_piece = self.squares[new_rank as usize][new_file as usize].clone();
        // Now borrow the moving piece mutably to apply the move
        match self.squares[old_rank as usize][old_file as usize].as_mut() {
            Some(moving_piece) => {

                
                println!("Quiet moves: {:?}", quiets);
                println!("Capture moves: {:?}", captures);
                let is_quiet = quiets.contains(&new_pos);
                let is_capture = captures.contains(&new_pos);
                if is_capture {
                    self.halfmove_clock = 0;
                    //first we check for en passant because is a special capture
                    match moving_piece.kind {
                        PieceType::Pawn => {
                            //pawn capture decide
                            match capture_piece {
                                Some(_) => {
                                    //normal capture
                                    moving_piece.times_moved+=1;
                                    moving_piece.position = new_pos;
                                    self.state.moved_to = Some(moving_piece.position);
                                    self.squares[new_rank as usize][new_file as usize] = Some(*moving_piece);
                                    self.squares[old_rank as usize][old_file as usize] = None;
                                    let capture = capture_piece.unwrap();
                                    match capture.color {
                                        PieceColor::Black => {
                                            self.state.white_taken.push(capture);
                                        }
                                        PieceColor::White => {
                                            self.state.black_taken.push(capture);
                                        }
                                    }
                                    self.en_passant_target = None;
                                    self.change_turn();
                                    self.deselect_piece();
                                }
                                None => {
                                    //en passant
                                    moving_piece.times_moved+=1;
                                    moving_piece.position = new_pos;
                                    self.state.moved_to = Some(moving_piece.position);
                                    
                                    self.squares[new_rank as usize][new_file as usize] = Some(*moving_piece);
                                    self.squares[old_rank as usize][old_file as usize] = None;
                                    let (epr, epf) = self.en_passant_target.unwrap();
                                    let capture = self.squares[epr as usize][epf as usize].unwrap();
                                    match capture.color {
                                        PieceColor::Black => {
                                            self.state.white_taken.push(capture);
                                        }
                                        PieceColor::White => {
                                            self.state.black_taken.push(capture);
                                        }
                                    }
                                    self.squares[epr as usize][epf as usize]=None;
                                    self.en_passant_target = None;
                                    self.change_turn();
                                    self.deselect_piece();
                                }
                            }
                            

                        }
                        _ =>{
                            moving_piece.times_moved+=1;
                            moving_piece.position = new_pos;
                            self.state.moved_to = Some(moving_piece.position);
                            self.squares[new_rank as usize][new_file as usize] = Some(*moving_piece);
                            self.squares[old_rank as usize][old_file as usize] = None;
                            
                            self.en_passant_target = None;
                            let capture = capture_piece.unwrap();
                            match capture.color {
                                PieceColor::Black => {
                                    self.state.white_taken.push(capture);
                                }
                                PieceColor::White => {
                                    self.state.black_taken.push(capture);
                                }
                            }
                            self.change_turn();
                            self.deselect_piece();
                        }
                    }
                    
                }else 
                if is_quiet  {
                    self.en_passant_target = None;
                    if moving_piece.kind == PieceType::Pawn {
                        let mv_delta = old_rank.abs_diff(new_rank);
                        if mv_delta == 2 {
                            //set en ppassant target 
                            self.en_passant_target = Some(new_pos);
                        }
                        
                    }
                    moving_piece.times_moved+=1;
                    moving_piece.position = new_pos;
                    self.state.moved_to = Some(moving_piece.position);
                    self.squares[new_rank as usize][new_file as usize] = Some(*moving_piece);
                    self.squares[old_rank as usize][old_file as usize] = None;

                    if piece.color == PieceColor::Black {
                        self.fullmove_number+=1;
                    }
                    if piece.kind == PieceType::Pawn {
                        self.halfmove_clock = 0;
                    }

                    self.change_turn();
                    self.deselect_piece();
                    

                }
                
            }

            None =>{
                return Err(MoveError::IllegalMove);
            }
        }
        
        
        
        //iffailed return the piece to itssquare
       
        Ok(())
    }
    
    pub fn change_turn(&mut self){
        self.turn = match self.turn {
            PieceColor::Black => PieceColor::White,
            PieceColor::White => PieceColor::Black,
        };
    }
    
    pub fn execute_castle(&mut self, king_pos:(u8,u8), rook_pos: (u8,u8)) {
        //get mut king rook
        let mut king = self.squares[king_pos.0 as usize][king_pos.1 as usize].unwrap();
        let mut rook: ChessPiece = self.squares[rook_pos.0 as usize][rook_pos.1 as usize].unwrap();

        //determine if is big castleif piece.kind == PieceType::Rook || piece.kind == PieceType::King {
        let king_rook_distance = king.position.1.abs_diff(rook.position.1);
        let castle_type = if king_rook_distance == 4 { CastleType::QueenSide} else {CastleType::KingSide};       
        //befora castle we need to check that all the squares between rook and king ar not in check
        if self.can_castle(castle_type.clone(), king.color){

            match castle_type {
                CastleType::KingSide => {
                    king.position.1 = (king.position.1 as i8 + 2) as u8;
                    rook.position.1 = (rook.position.1 as i8 + -2) as u8;
                    
                }
                CastleType::QueenSide => {
                    king.position.1 = (king.position.1 as i8 + -2) as u8;
                    rook.position.1 = (rook.position.1 as i8 + 3) as u8;
                }
                
            };
            
            self.state.moved_to = Some(king.position);
            //update_ing board
        
            self.squares[king_pos.0 as usize][king_pos.1 as usize] = None;
            self.squares[rook_pos.0 as usize][rook_pos.1 as usize] = None;
            
            self.squares[king.position.0 as usize][king.position.1 as usize] = Some(king);
        self.squares[rook.position.0 as usize][rook.position.1 as usize] = Some(rook); 
        
        match king.color {
            PieceColor::Black => {
                self.black_big_castle = false;  
                self.black_small_castle = false;
            }
            PieceColor::White => {
                self.white_big_castle = false;
                self.white_small_castle = false;
            }
            
            
            
        }
        self.change_turn();
        self.deselect_piece();
    
    }


    }
    pub fn promote_pawn(&mut self, pos: (u8,u8), kind: PieceType){
        if let Some(pawn) = self.squares[pos.0 as usize][pos.1 as usize].as_mut() {
            pawn.kind = kind;
        }
    }
    pub fn select_piece(&mut self, piece: ChessPiece) {
        self.state.selected_piece = Some(piece);
        self.set_legal_moves(&piece);
    }
    pub fn deselect_piece(&mut self) {
        self.state.capture_moves = None;
        self.state.selected_piece = None;
        self.state.quiet_moves = None;
    }
    
}