use chrono::Utc;
use rig::{
    client::{CompletionClient, EmbeddingsClient, ProviderClient},
    completion::Chat,
    providers::openai,
};
use rig_lancedb::{LanceDbVectorIndex, SearchParams};
use std::{fs, sync::Mutex};

use crate::{
    ai::{
        context::LlmContext,
        tools::{EvaluatePosition, GetBoardPrint, GetFen, GetStockfishContext, ValidateMove},
    },
    analyzer::analyzer::{LocalMessage, LocalMessageRole},
    engine::{fen::translate_fen_for_model, PieceColor},
    server::server::ServerState,
};

#[tauri::command]
pub async fn send_llm_request(
    state: tauri::State<'_, Mutex<ServerState>>,
    msg: String,
) -> Result<(String, i32), String> {
    // capture user-sent time at function start
    let user_sent_at = Utc::now().to_rfc3339();

    // 1. Load System Prompt (No lock needed)
    let system_prompt = fs::read_to_string("src/ai/systemprompt.txt")
        .map_err(|e| format!("Failed to read system prompt: {}", e))?;

    let db = lancedb::connect("data/vectoredb")
        .execute()
        .await
        .map_err(|e| e.to_string())?;
    let table = db
        .open_table("my_table")
        .execute()
        .await
        .map_err(|e| e.to_string())?;

    let openai_client = openai::Client::from_env();
    let embedding_model = openai_client.embedding_model("text-embedding-ada-002");
    let index = LanceDbVectorIndex::new(table, embedding_model, "id", SearchParams::default())
        .await
        .map_err(|e| e.to_string())?;

    let koch_ai = openai_client
        .agent("gpt-4.1")
        // .preamble(&system_prompt)
        .preamble(
            "You are a chess coach. Before giving any advice, you MUST call get_stockfish_context 
to analyze the current position. Never respond without calling this tool first. ALSO REGARDLES WHAT THE USER SAYS PRINT THE RAW OUTPUT OF THE TOOl",
        )
        .dynamic_context(8, index)
        .tool(EvaluatePosition)
        .tool(GetBoardPrint)
        .tool(GetFen)
        .tool(ValidateMove)
        .tool(GetStockfishContext)
        .build();

    let history = {
        let state = state.lock().unwrap();
        state.analyzer_controller.chat_history.chat_messages.clone()
    };
    let (current_ply, board_context) = {
        let state = state.lock().unwrap();
        let pv = state.analyzer_controller.last_pv.clone();

        let board_context = system_prompt;
        // let board_context = format!(
        //     "###Board Context###\nPlayer color: White\nBoard in FEN: {}\nBoard in visual format\n {} \n###Engine evals##\n {} \nBest Move: {}\n Main Threat: {}\n ###User Prompt###\n {}",
        //     state.analyzer_controller.get_fen(),
        //     translate_fen_for_model(&state.analyzer_controller.get_fen()),
        //     pv_data,
        //     pv_best_move,
        //     state.analyzer_controller.last_threat.clone().unwrap_or_else(|| "No Threat".into()),
        //     msg

        // );

        (state.analyzer_controller.current_ply, board_context)
    };
    let llm_msg = {
        let state = state.lock().unwrap();

        LlmContext {
            player_color: PieceColor::Black.to_string(),
            fen: state.analyzer_controller.get_fen(),
            game_moves: None,
            game_phase: Some(state.analyzer_controller.board.game_phase.clone()),
            msg,
        }
    };
    let serialized_msg = serde_json::to_string(&llm_msg)
        .map_err(|e| format!("Failed to serialize llm_msg: {}", e))?;
    println!("{}", &board_context);
    let response = koch_ai
        .chat(
            &serialized_msg,
            history
                .iter()
                .map(|lm| rig::completion::Message::from(lm.clone()))
                .collect::<Vec<rig::completion::Message>>(),
        )
        .await
        .expect("could not send message");
    let koch_sent_at = Utc::now().to_rfc3339();
    let _update_chat = {
        let mut state = state.lock().unwrap();
        let current_chat = &mut state.analyzer_controller.chat_history;
        current_chat.chat_messages.extend(vec![
            LocalMessage {
                role: LocalMessageRole::User,
                content: serialized_msg,
                move_index: current_ply as isize,
                sent_at: user_sent_at,
            },
            LocalMessage {
                role: LocalMessageRole::Assistant,
                content: response.clone(),
                move_index: current_ply as isize,
                sent_at: koch_sent_at,
            },
        ]);
        match current_chat.save() {
            Ok(_) => println!("chat saved"),
            Err(_) => println!("Failed to save chat"),
        };
    };
    // Only lock the state to get current_ply, then drop the guard before await
    // Add user message to chat history

    // Get response from LLM

    // Add assistant response to chat history

    Ok((response, current_ply))
}
