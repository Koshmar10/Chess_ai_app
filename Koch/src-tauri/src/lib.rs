// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
pub mod engine;
use std::sync::Mutex;

use sysinfo::System;
pub mod ai;
pub mod analyzer;
pub mod database;
pub mod etc;
pub mod game;
pub mod server;
use crate::analyzer::analyzer::get_threat;
use crate::analyzer::analyzer::LocalChat;

use crate::analyzer::analyzer::{
    get_analyzer_settings, set_analyzer_fen, set_engine_option, start_analyzer_thread,
    stop_analyzer,
};
use crate::analyzer::analyzer::AnalyzerController;
use crate::analyzer::board_interactions::{get_board_at_index, get_fen};
use crate::database::create::create_database;
use crate::database::create::delete_game;
use crate::database::create::get_game_by_id;
use crate::database::create::get_game_chat_by_id;
use crate::database::create::get_game_list;
use crate::database::create::load_pgn_game;
use crate::database::integrations::{is_syncing_chessdotcom, sync_with_chessdotcom};
use crate::engine::board::BoardMetaData;
use crate::game::controller::save_appgame;
use crate::game::controller::update_game_state;
use crate::game::controller::SerializedGameController;
use crate::game::controller::{change_gamemode, end_game, get_share_data, new_game, start_game};
use crate::game::puzzle::get_puzzle;
use crate::server::server::get_player_stats;
use crate::server::server::Settings;
use crate::server::server::get_system_information;
use crate::ai::message::send_llm_request;

use crate::{
    engine::Board,
    server::server::ServerState,
};
use tauri::Manager;

#[tauri::command]
fn get_game_board(state: tauri::State<'_, Mutex<ServerState>>) -> SerializedGameController {
    let state = state.lock().unwrap();
    let game = state.game_controller.serialize();
    game
}
#[tauri::command]
fn get_quote(state: tauri::State<'_, Mutex<ServerState>>) -> String {
    let state = state.lock().unwrap();
    return state.get_quote();
}

// remember to call `.manage(MyState::default())`
#[tauri::command]
fn board_fen(board: Board) -> String {
    return board.to_string();
}

#[tauri::command]
fn fetch_game_history() -> Vec<(i64, BoardMetaData)> {
    match get_game_list() {
        Ok(list) => list,
        Err(e) => {
            eprintln!("fetch_game_history DB error: {e}");
            Vec::new()
        }
    }
}
#[tauri::command]
fn fetch_default_game(state: tauri::State<'_, Mutex<ServerState>>) -> AnalyzerController {
    let mut state = state.lock().unwrap();
    let mut analyzer = AnalyzerController::default();
    analyzer.board.rerender_move_cache();
    state.analyzer_controller = analyzer.clone();
    analyzer
}
#[tauri::command]
fn fetch_game(state: tauri::State<'_, Mutex<ServerState>>, id: usize) -> AnalyzerController {
    let mut state = state.lock().unwrap();
    if state.analyzer_controller.game_id == id {
        return state.analyzer_controller.clone();
    }
    let game_chat = match get_game_chat_by_id(id) {
        Ok(chat) => chat,
        Err(e) => {
            eprintln!("fetch_game: get_game_chat_by_id DB error: {e}");
            LocalChat::default()
        }
    };

    match get_game_by_id(id) {
        Ok(list) => {
            let mut analyzer = AnalyzerController::default();
            let _move_count = list.move_list.len();
            let mut board = Board::from(&list.starting_position);
            board.meta_data = list; // includes full move_list from DB
            analyzer.board = board;
            analyzer.game_id = id;
            analyzer.current_ply = -1;
            analyzer.chat_history = game_chat;

            // Persist so do_move/undo_move operate on the same instance
            state.analyzer_controller = analyzer.clone();
            analyzer
        }
        Err(e) => {
            eprintln!("fetch_game_history DB error: {e}");
            AnalyzerController::default()
        }
    }
}
#[tauri::command]
fn get_settings(state: tauri::State<'_, Mutex<ServerState>>) -> Settings {
    let state = state.lock().unwrap();
    return state.settings.clone();
}
#[tauri::command]
fn update_settings(state: tauri::State<'_, Mutex<ServerState>>, key: String, val: String) {
    let mut state = state.lock().unwrap();
    state.settings.update(key, val);
    let _ = state.settings.save();
}
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        // Initialize state with default (None channels)
        .manage(Mutex::new(ServerState::default()))
        .setup(|app| {

            let _ = create_database();
            let mut sys = System::new_all();
            sys.refresh_all();
            let mem_bytes = sys.total_memory();
            let nb_cpu = sys.cpus().len();
            let ram_capacity = (mem_bytes / 1024 / 1024) as f64 / 1024.0;

            // Get AppHandle
            let handle = app.handle().clone();

            // Start thread with handle
            let (tx, rx) = start_analyzer_thread(handle);

            // Update state with channels
            let state = app.state::<Mutex<ServerState>>();
            let mut state = state.lock().unwrap();
            state.analyzer_tx = Some(tx);
            state.analyzer_rx = Some(rx);
            state.total_memory = ram_capacity;
            state.nbcpu = nb_cpu;

            Ok(())
        })
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            get_game_board,
            change_gamemode,
            start_game,
            update_game_state,
            get_quote,
            fetch_game_history,
            fetch_game,
            fetch_default_game,
            set_analyzer_fen,
            get_fen,
            stop_analyzer,
            board_fen,
            get_board_at_index,
            get_system_information,
            set_engine_option,
            get_analyzer_settings,
            load_pgn_game,
            get_settings,
            update_settings,
            sync_with_chessdotcom,
            get_threat,
            send_llm_request,
            end_game,
            new_game,
            save_appgame,
            get_share_data,
            get_puzzle,
            get_player_stats,
            delete_game,
            is_syncing_chessdotcom
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
