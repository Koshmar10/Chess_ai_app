use crate::engine::{Board, PieceType};

impl Board {
    pub fn encode_san_move(&self, _from: (u8, u8), _to: (u8, u8), _promotion: Option<PieceType>) {}
}
