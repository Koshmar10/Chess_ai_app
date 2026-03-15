use std::sync::Mutex;

use crate::{database::create::parse_pgn_string, engine::Board, server::server::ServerState};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct LichessPuzzleResponse {
    pub game: LichessPuzzleGame,
    pub puzzle: LichessPuzzle,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct LichessPuzzleGame {
    pub id: String,
    pub perf: LichessPerf,
    pub rated: bool,
    pub players: Vec<LichessPlayer>,
    pub pgn: String,
    pub clock: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct LichessPerf {
    pub key: String,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct LichessPlayer {
    pub name: String,
    pub id: String,
    pub color: String,
    pub rating: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct LichessPuzzle {
    pub id: String,
    pub rating: u32,
    pub plays: u32,
    pub solution: Vec<String>,
    pub themes: Vec<String>,
    #[serde(rename = "initialPly")]
    pub initial_ply: u32,
}
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub enum GetPuzzleFlag {
    OnLoad,
    GetNew,
}
#[tauri::command]
pub async fn get_puzzle(
    state: tauri::State<'_, Mutex<ServerState>>,
    flag: GetPuzzleFlag,
) -> Result<(Board, LichessPuzzleResponse), String> {
    let make_request = match flag {
        GetPuzzleFlag::GetNew => true,
        GetPuzzleFlag::OnLoad => state.lock().unwrap().chess_puzzle.is_none(),
    };
    if make_request {
        let resp = reqwest::get("https://lichess.org/api/puzzle/next")
            .await
            .map_err(|e| format!("Failed to fetch puzzle: {e}"))?;

        let puzzle_response = resp
            .json::<LichessPuzzleResponse>()
            .await
            .map_err(|e| format!("Failed to parse puzzle json: {e}"))?;
        let mut modified_pgn = puzzle_response.game.pgn.clone();
        modified_pgn.push_str(" *");
        let metadata = parse_pgn_string(modified_pgn);
        let mut board = Board::default();
        board.turn = crate::engine::PieceColor::White;
        for mv in &metadata.move_list {
            let (from, to, promotion) = board.decode_uci_move(&mv.uci).unwrap();
            match board.move_piece(from, to, promotion) {
                Ok(_) => {},
                Err(e) => eprintln!("{:?}", e),
            }
            board.rerender_move_cache();
        }
        board.meta_data = metadata;
        state.lock().unwrap().chess_puzzle = Some((board.clone(), puzzle_response.clone()));
        return Ok((board, puzzle_response));
    }
    let puzzle_struct = state.lock().unwrap().chess_puzzle.clone().unwrap();
    Ok(puzzle_struct)
}
