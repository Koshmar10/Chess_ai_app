use crate::analyzer::analyzer::{AnalyzerController, EngineCommand};
use crate::game::puzzle::LichessPuzzleResponse;
use crate::{database, engine::Board, game::controller::GameController};

use serde::{Deserialize, Serialize};

use std::cmp::Ordering;
use std::collections::HashMap;
use std::error::Error;
use std::fs::{self, File};
use std::io::{self, BufRead, BufReader, Read};
use std::path::Path;
use std::sync::mpsc::Receiver;
use std::sync::mpsc::SyncSender;
use std::sync::Mutex;
use stockfish::Stockfish;
use ts_rs::TS;

#[derive(Clone, Debug, TS, Serialize, Deserialize)]
#[ts(export)]
pub enum EvalKind {
    Mate,
    Centipawn,
}

#[derive(Clone, Debug, Serialize, Deserialize, ts_rs::TS)]
#[ts(export)]
pub struct PvLineData {
    pub moves: String,
    pub eval_kind: EvalKind,
    pub eval_value: i32,
}

impl std::fmt::Display for PvLineData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.eval_kind {
            EvalKind::Mate => write!(f, "{} | Eval: (mate {})", self.moves, self.eval_value),
            EvalKind::Centipawn => write!(f, "{} | Eval:  (cp {})", self.moves, self.eval_value),
        }
    }
}

#[derive(Clone, Debug, serde::Serialize, ts_rs::TS)]
#[ts(export)]
pub struct PvObject {
    pub fen: String,
    pub depth: u32,
    // Changed: lines now map to PvLineData instead of just String
    pub lines: HashMap<u8, PvLineData>,
}

impl std::fmt::Display for PvObject {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        //writeln!(f, "FEN: {}", self.fen)?;
        //writeln!(f, "PV FEN: {}", self.fen)?;
        writeln!(f, "Engine Lines:")?;
        if self.lines.is_empty() {
            writeln!(f, "  <no lines>")?;
            return Ok(());
        }
        let mut entries: Vec<(&u8, &PvLineData)> = self.lines.iter().collect();
        entries.sort_by_key(|(k, _)| *k);
        for (k, line) in entries {
            writeln!(f, "  {}. {}", k, line)?;
        }
        Ok(())
    }
}
impl PvObject {
    /// Return the first move (as a string) of the highest-rated PV line, if any.
    /// The "highest-rated" line is chosen by a simple numeric score:
    /// - Centipawn evaluations use their raw value.
    /// - Mate evaluations are given very large magnitude values so mate results
    ///   outrank centipawn scores (positive mate is very good, negative mate very bad).
    pub fn best_first_move(&self) -> Option<String> {
        fn score_of(line: &PvLineData) -> f64 {
            const MATE_BASE: f64 = 1_000_000.0;
            match line.eval_kind {
                EvalKind::Mate => {
                    if line.eval_value >= 0 {
                        MATE_BASE - (line.eval_value as f64)
                    } else {
                        -MATE_BASE - (line.eval_value as f64)
                    }
                }
                EvalKind::Centipawn => line.eval_value as f64,
            }
        }

        fn first_move_from_moves(moves: &str) -> Option<String> {
            for token in moves.split_whitespace() {
                let t = token.trim();
                // skip move numbers like "1." or "1..."
                if t.contains('.') {
                    continue;
                }
                // remove common trailing annotation characters but keep SAN like O-O, exd5, etc.
                let cleaned = t.trim_end_matches(|c: char| matches!(c, '+' | '#' | '!' | '?'));
                if !cleaned.is_empty() {
                    return Some(cleaned.to_string());
                }
            }
            None
        }

        let best = self.lines.values().max_by(|a, b| {
            score_of(a)
                .partial_cmp(&score_of(b))
                .unwrap_or(Ordering::Equal)
        })?;

        first_move_from_moves(&best.moves)
    }
}
impl Default for PvObject {
    fn default() -> Self {
        Self {
            fen: String::new(),
            depth: 0,
            lines: std::collections::HashMap::new(),
        }
    }
}

// Commands the UI can send to the Engine
/*
"rnbqkbnr/ppp1pppp/8/3p4/3PP3/8/PPP2PPP/RNBQKBNR b KQkq - 0 2": {
    "name": "Blackmar-Diemer Gambit",
    "eco": "D00",
    "moves": "1. d4 d5 2. e4",
        "src": "eco_tsv",
        "scid": "D00l",
        "aliases": {
            "scid": "Blackmar-Diemer Gambit (BDG): 2.e4",
            "ct": "Blackmar-Diemer Gambit, General",
            "chronos": "Blackmar gambit"
        }
    }, */
#[derive(Debug, Serialize, Deserialize, Clone, TS)]
#[ts(export)]
pub struct Settings {
    pub corrupted: bool,
    pub map: HashMap<String, String>,
}
impl Settings {
    pub fn update(&mut self, key: String, val: String) {
        let sv = self.map.entry(key).or_insert("".to_string());
        *sv = val;
    }
    pub fn save(&mut self) -> Result<(), Box<dyn Error>> {
        let mut settings_string = String::new();
        // FIX: Iterate by reference (&self.map) instead of draining (moving) the values
        for (key, value) in &self.map {
            let line = format!("{}={}\n", key, value);
            settings_string.push_str(&line);
        }
        settings_string = settings_string.trim().to_string();

        // FIX: Write to ../koch.config to avoid triggering the src-tauri file watcher
        fs::write("../koch.config", settings_string)?;
        Ok(())
    }
}
#[derive(Deserialize)]
pub struct OpeningEntry {
    pub name: String,
    pub moves: Vec<String>,
}

/// Load the opening index from the bundled openings.json file.
/// Returns `None` if the file is missing or cannot be parsed.
pub fn load_opening_index() -> Option<HashMap<String, OpeningEntry>> {
    let path = Path::new(concat!(env!("CARGO_MANIFEST_DIR"), "/src/openings.json"));
    let mut file = match File::open(path) {
        Ok(f) => f,
        Err(_) => return None,
    };
    let mut buf = String::new();
    if file.read_to_string(&mut buf).is_err() {
        return None;
    }
    match serde_json::from_str::<HashMap<String, OpeningEntry>>(&buf) {
        Ok(index) => Some(index),
        Err(e) => {
            eprintln!("Failed to parse openings.json: {e}");
            None
        }
    }
}

#[derive(Debug, Copy, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct PlayerStatistics {
    pub current_elo: u32,
    pub accuracy: f32,
    pub time_played: f32,
    pub games_played: u32,
    pub win_rate: f32,
}
impl Default for PlayerStatistics {
    fn default() -> Self {
        Self {
            current_elo: Default::default(),
            accuracy: Default::default(),
            time_played: Default::default(),
            games_played: Default::default(),
            win_rate: Default::default(),
        }
    }
}
pub struct ServerState {
    pub engine: Option<Stockfish>,
    pub opening_index: Option<HashMap<String, OpeningEntry>>,
    pub game_controller: GameController,
    pub analyzer_controller: AnalyzerController,
    pub chess_puzzle: Option<(Board, LichessPuzzleResponse)>,
    pub analyzer_tx: Option<SyncSender<EngineCommand>>,
    pub analyzer_rx: Option<Receiver<PvObject>>,
    pub total_memory: f64,
    pub nbcpu: usize,
    pub settings: Settings,
    pub syncing_with_chessdotcom: bool,
    pub current_puzzle_solved: bool,
}
impl Default for ServerState {
    fn default() -> Self {
        let settings = load_settings().unwrap_or_else(|_| Settings {
            corrupted: true,
            map: HashMap::new(),
        });
        let stockfish_elo = settings
            .map
            .get("StockfishElo")
            .unwrap_or(&"600".to_string())
            .parse()
            .unwrap_or(600);

        let stockfish_path = settings
            .map
            .get("StockfishPath")
            .cloned()
            .unwrap_or_else(|| crate::etc::DEFAULT_STOCKFISH_PATH.to_string());

        let engine = match Stockfish::new(&stockfish_path) {
            Ok(mut s) => {
                if s.setup_for_new_game().is_ok() {
                    let _ = s.set_elo(stockfish_elo);
                    Some(s)
                } else {
                    None
                }
            }
            Err(_) => None,
        };
        let game_controller = GameController::default();
        let analyzer_controller = AnalyzerController::default();

        let opening_index = load_opening_index();

        database::create::create_database()
            .inspect_err(|e| eprintln!("{e}"))
            .ok();
        return ServerState {
            engine,
            game_controller,
            analyzer_controller,
            chess_puzzle: None,
            analyzer_rx: None,
            analyzer_tx: None,
            opening_index: opening_index,
            total_memory: 0.0,
            nbcpu: 1,
            settings: settings,
            syncing_with_chessdotcom: false,
            current_puzzle_solved: false,
        };
    }
}
impl ServerState {
    pub fn update_elo(&mut self, elo_delta: i32) {
        let string_elo = self.settings.map.get("PlayerElo");
        match string_elo {
            Some(elo) => {
                let num_elo = elo.parse::<i32>().unwrap_or(500);
                let updated_elo = num_elo + elo_delta;
                self.settings
                    .update("PlayerElo".to_string(), updated_elo.to_string());
                self.settings
                    .update("StockfishElo".to_string(), (updated_elo + 50).to_string());
            }
            None => {}
        };
    }
}
#[tauri::command]
pub fn get_system_information(state: tauri::State<'_, Mutex<ServerState>>) -> (f64, usize) {
    let state = state.lock().unwrap();

    (state.total_memory, state.nbcpu)
}
#[tauri::command]
pub fn get_player_stats(state: tauri::State<'_, Mutex<ServerState>>) -> PlayerStatistics {
    let state = state.lock().unwrap();
    let elo = state
        .settings
        .map
        .get("PlayerElo")
        .unwrap_or(&"0".into())
        .parse::<u32>()
        .unwrap_or(0);
    let games_played = state
        .settings
        .map
        .get("GamesPlayed")
        .unwrap_or(&"0".into())
        .parse::<u32>()
        .unwrap_or(0);
    let games_won = state
        .settings
        .map
        .get("GamesWon")
        .unwrap_or(&"0".into())
        .parse::<u32>()
        .unwrap_or(0);
    let mut stats = PlayerStatistics::default();
    stats.current_elo = elo;
    stats.games_played = games_played;
    stats.win_rate = if games_played > 0 {
        games_won as f32 / games_played as f32 * 100.0
    } else {
        0.0
    };
    stats
}
pub fn load_settings() -> Result<Settings, io::Error> {
    let mut settings_map = HashMap::new();
    let mut corrupted = false;

    // FIX: Read from ../koch.config
    let settings = fs::File::open("../koch.config")?;
    let reader = BufReader::new(settings);
    for line in reader.lines() {
        match line {
            Ok(line) => {
                let split_line: Vec<&str> = line.split("=").collect();
                settings_map.insert(split_line[0].to_string(), split_line[1].to_string());
            }
            Err(_) => {
                corrupted = true;
            }
        }
    }

    Ok(Settings {
        corrupted: corrupted,
        map: settings_map,
    })
}
