use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::engine::board::GamePhase;
#[derive(Clone, Debug, TS, Serialize, Deserialize)]
#[ts(export)]
pub struct GameMoves {
    uci_moves: Vec<String>,
    san_moves: Vec<String>,
}

#[derive(Clone, TS, Debug, Serialize, Deserialize)]
#[ts(export)]
pub struct LlmContext {
    pub player_color: String,
    pub fen: String,
    pub game_moves: Option<GameMoves>,
    pub game_phase: Option<GamePhase>,
    pub msg: String,
}

#[derive(Clone, Debug, TS, Serialize, Deserialize)]
pub struct EvalScore {
    pub centipawns: Option<i32>, // None if mate score
    pub mate_in: Option<i32>,    // e.g. +3 (white mates in 3), -2 (black mates in 2)
    pub is_from_white_perspective: bool,
}
#[derive(Clone, Debug, Serialize, Deserialize)]

pub struct LlmStockfishContext {
    pub engine_depth: Option<u32>,
    pub current_eval: Option<EvalScore>,
    pub best_move: Option<String>,     // keep as UCI string e.g. "e2e4"
    pub best_move_san: Option<String>, // add SAN e.g. "e4" — LLMs reason better with this
    pub best_lines: Option<Vec<PvLine>>, // see below
    pub main_threat: Option<String>,   // opponent's best reply
    pub is_check: bool,                // simple but useful for move explanation
    pub is_game_over: bool,
}
#[derive(Clone, Debug, Serialize, Deserialize)]

pub struct PvLine {
    pub eval: EvalScore,
    pub moves: Vec<String>,     // UCI move sequence
    pub moves_san: Vec<String>, // SAN sequence for LLM readability
}
