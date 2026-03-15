use rig::tool::Tool;
use serde::{Deserialize, Serialize};
use std::io::{BufRead, BufReader, Write};
use std::process::{Command, Stdio};
use stockfish::Stockfish;
use stockfish::{EngineEval, EvalType};

use crate::ai::context::{EvalScore, LlmStockfishContext, PvLine};
use crate::analyzer::analyzer::flip_fen_turn;
use crate::engine::fen::fen_parser;
use crate::engine::{Board, PieceType};

fn eval_string(engine_eval: &EngineEval) -> String {
    format!("{}{}", engine_eval.value(), engine_eval.eval_type())
}

/// PositionEvaluator
#[derive(Serialize, Deserialize)]
pub struct EvaluatePosition;

#[derive(Serialize, Deserialize)]
pub struct EvaluatePositionArgs {
    pub fen: String,
}

#[derive(Debug, thiserror::Error)]
#[error("EvaluatePosition error: {0}")]
pub struct EvaluatePositionError(String);

impl Tool for EvaluatePosition {
    const NAME: &'static str = "evaluate_position";

    type Error = EvaluatePositionError;
    type Args = EvaluatePositionArgs;
    type Output = String;

    async fn definition(&self, _prompt: String) -> rig::completion::request::ToolDefinition {
        rig::completion::request::ToolDefinition {
            name: Self::NAME.to_string(),
            description: "Evaluates a chess position given in FEN notation and returns the evaluation score (e.g. '+1.2' means white is better, '-0.5' means black is better, 'M3' means mate in 3).".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "fen": {
                        "type": "string",
                        "description": "FEN notation of the chess position to evaluate (e.g. 'rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1')"
                    }
                },
                "required": ["fen"]
            }),
        }
    }

    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        let mut stockfish = Stockfish::new("/bin/stockfish")
            .map_err(|e| EvaluatePositionError(format!("Failed to start Stockfish: {}", e)))?;

        stockfish
            .set_fen_position(&args.fen)
            .map_err(|e| EvaluatePositionError(format!("Failed to set FEN: {}", e)))?;

        match stockfish.go() {
            Ok(engine_output) => Ok(eval_string(&engine_output.eval())),
            Err(e) => Err(EvaluatePositionError(e.to_string())),
        }
    }
}

/// GetFen - converts UCI move sequences to FEN
#[derive(Serialize, Deserialize)]
pub struct GetFen;

#[derive(Serialize, Deserialize)]
pub struct GetFenArgs {
    pub moves: String,
}

#[derive(Debug, thiserror::Error)]
#[error("GetFen error: {0}")]
pub struct GetFenError(String);

impl Tool for GetFen {
    const NAME: &'static str = "get_fen";

    type Error = GetFenError;
    type Args = GetFenArgs;
    type Output = String;

    async fn definition(&self, _prompt: String) -> rig::completion::request::ToolDefinition {
        rig::completion::request::ToolDefinition {
            name: Self::NAME.to_string(),
            description: "Converts a sequence of UCI chess moves into the resulting FEN position. Use this when the user describes moves in algebraic notation.".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "moves": {
                        "type": "string",
                        "description": "Space-separated UCI moves from the starting position (e.g. 'e2e4 e7e5 g1f3' for 1.e4 e5 2.Nf3)"
                    }
                },
                "required": ["moves"]
            }),
        }
    }

    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        let moves: Vec<&str> = args.moves.split_whitespace().collect();

        let mut stockfish = Stockfish::new("/bin/stockfish")
            .map_err(|e| GetFenError(format!("Failed to start Stockfish: {}", e)))?;

        stockfish
            .play_moves(&moves)
            .map_err(|e| GetFenError(format!("Failed to play moves: {}", e)))?;

        // Get the current FEN position
        let fen = stockfish
            .get_fen()
            .map_err(|e| GetFenError(format!("Failed to get FEN: {}", e)))?;

        Ok(fen)
    }
}

/// GetBoardPrint - visualizes a FEN position
#[derive(Serialize, Deserialize)]
pub struct GetBoardPrint;

#[derive(Serialize, Deserialize)]
pub struct GetBoardPrintArgs {
    pub fen: String,
}

#[derive(Debug, thiserror::Error)]
#[error("GetBoardPrint error: {0}")]
pub struct GetBoardPrintError(String);

impl Tool for GetBoardPrint {
    const NAME: &'static str = "get_board_print";

    type Error = GetBoardPrintError;
    type Args = GetBoardPrintArgs;
    type Output = String;

    async fn definition(&self, _prompt: String) -> rig::completion::request::ToolDefinition {
        rig::completion::request::ToolDefinition {
            name: Self::NAME.to_string(),
            description: "Returns a visual ASCII representation of a chess board from a FEN position. Use this to show the user what the board looks like.".to_string(),
            parameters: serde_json::json!({
              "type": "object",
              "properties": {
                "fen": {
                  "type": "string",
                  "description": "FEN notation of the chess position (e.g. 'rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq e3 0 1')"
                }
              },
              "required": ["fen"]
            }),
        }
    }

    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        let mut stockfish = Stockfish::new("/bin/stockfish")
            .map_err(|e| GetBoardPrintError(format!("Failed to start Stockfish: {}", e)))?;

        stockfish
            .set_fen_position(&args.fen)
            .map_err(|e| GetBoardPrintError(format!("Failed to set FEN position: {}", e)))?;

        let board = stockfish
            .get_board_display()
            .map_err(|e| GetBoardPrintError(format!("Failed to get board display: {}", e)))?;

        Ok(board)
    }
}

/// ValidateMove - checks whether a single UCI move is legal in the given FEN
#[derive(Serialize, Deserialize)]
pub struct ValidateMove;

#[derive(Serialize, Deserialize)]
pub struct ValidateMoveArgs {
    pub fen: String,
    pub mv: String,
}

#[derive(Debug, thiserror::Error)]
#[error("ValidateMove error: {0}")]
pub struct ValidateMoveError(String);

impl Tool for ValidateMove {
    const NAME: &'static str = "validate_move";

    type Error = ValidateMoveError;
    type Args = ValidateMoveArgs;
    type Output = bool;

    async fn definition(&self, _prompt: String) -> rig::completion::request::ToolDefinition {
        rig::completion::request::ToolDefinition {
            name: Self::NAME.to_string(),
            description: "Validates whether a single UCI move is legal in the provided FEN position. Returns true when the move is legal, false otherwise.".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "fen": { "type": "string", "description": "FEN notation of the chess position" },
                    "move": { "type": "string", "description": "UCI move to validate (e.g. 'e2e4', 'e7e8q')" }
                },
                "required": ["fen", "move"]
            }),
        }
    }

    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        let mut stockfish = Stockfish::new("/bin/stockfish")
            .map_err(|e| ValidateMoveError(format!("Failed to start Stockfish: {}", e)))?;

        stockfish
            .set_fen_position(&args.fen)
            .map_err(|e| ValidateMoveError(format!("Failed to set FEN: {}", e)))?;

        // Try to play the single move. If play_moves returns Ok(()) the move was legal.
        let mv = args.mv;
        let moves = vec![mv.as_str()];
        match stockfish.play_moves(&moves) {
            Ok(_) => Ok(true),
            Err(_) => Ok(false),
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct GetStockfishContext;

#[derive(Serialize, Deserialize)]
pub struct GetStockfishContextArgs {
    pub fen: String,
}

#[derive(Debug, thiserror::Error)]
#[error("GetStockfishContext error: {0}")]
pub struct GetStockfishContextError(String);

impl Tool for GetStockfishContext {
    const NAME: &'static str = "get_stockfish_context";

    type Error = GetStockfishContextError;
    type Args = GetStockfishContextArgs;
    type Output = LlmStockfishContext;

    async fn definition(&self, _prompt: String) -> rig::completion::request::ToolDefinition {
        rig::completion::request::ToolDefinition {
      name: Self::NAME.to_string(),
      description: "Returns diagnostic/context information from Stockfish for a given FEN. Useful for debugging or exposing board state and basic engine info.".to_string(),
      parameters: serde_json::json!({
        "type": "object",
        "properties": {
          "fen": {
            "type": "string",
            "description": "FEN notation of the chess position to inspect"
          }
        },
        "required": ["fen"]
      }),
    }
    }
    //     pub struct LlmStockfishContext {
    //     pub current_eval: Option<EvalScore>,
    //     pub best_move: Option<String>,     // keep as UCI string e.g. "e2e4"
    //     pub best_move_san: Option<String>, // add SAN e.g. "e4" — LLMs reason better with this
    //     pub best_lines: Option<Vec<PvLine>>, // see below
    //     pub main_threat: Option<String>,   // opponent's best reply
    //     pub is_check: bool,                // simple but useful for move explanation
    //     pub is_game_over: bool,
    // }
    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        eprintln!(
            "[Tool:get_stockfish_context] Starting for FEN: {}",
            args.fen
        );
        let mut best_move: Option<String> = None;
        let mut main_threat: Option<String> = None;
        let mut best_move_san: Option<String> = None;
        let mut eval_score: Option<EvalScore> = None;
        let mut engine_depth: Option<u32> = None;
        let is_from_white_perspective = args.fen.contains("w");
        let mut stockfish = Stockfish::new("/bin/stockfish")
            .map_err(|e| GetStockfishContextError(format!("Failed to start Stockfish: {}", e)))?;
        eprintln!("[Tool:get_stockfish_context] Stockfish started");

        match stockfish.set_option("MultiPV", "3".into()) {
            Ok(_) => {
                eprintln!("[Tool:get_stockfish_context] Set MultiPV=3");
            }
            Err(e) => {
                eprintln!(
                    "[Tool:get_stockfish_context] Set MultiPV to {} failed: {}",
                    3, e
                )
            }
        }
        stockfish.set_elo(3500);
        eprintln!("[Tool:get_stockfish_context] ELO set to 3500");
        stockfish.set_fen_position(&args.fen);
        eprintln!("[Tool:get_stockfish_context] FEN position set");

        //get best move / threat
        match stockfish.go() {
            Ok(output) => {
                best_move = Some(output.best_move().clone());
                engine_depth = Some(output.depth().clone());
                eprintln!(
                    "[Tool:get_stockfish_context] First go() returned best_move={} depth={}",
                    best_move.as_deref().unwrap_or("<none>"),
                    engine_depth.unwrap_or(0)
                );
                eval_score = {
                    let mut mate = None;
                    let mut centipawns = None;
                    let eval = output.eval().clone();
                    match eval.eval_type() {
                        EvalType::Centipawn => centipawns = Some(eval.value()),
                        EvalType::Mate => mate = Some(eval.value()),
                    };
                    eprintln!(
                        "[Tool:get_stockfish_context] Eval from engine: type={:?} value={}",
                        eval.eval_type(),
                        eval.value()
                    );
                    Some(EvalScore {
                        centipawns: centipawns,
                        mate_in: mate,
                        is_from_white_perspective,
                    })
                }
            }
            Err(e) => return Err(GetStockfishContextError(format!("{}", e))),
        }
        let mut board =
            fen_parser(&args.fen).map_err(|e| GetStockfishContextError(format!("{:?}", e)))?;
        let in_check = board.is_in_check(if args.fen.contains("b") {
            crate::engine::PieceColor::Black
        } else {
            crate::engine::PieceColor::White
        });
        let game_over = board.is_game_over();
        eprintln!(
            "[Tool:get_stockfish_context] Board parsed: in_check={} game_over={}",
            in_check, game_over
        );
        // If we got a best move from Stockfish, decode it and compute SAN
        if let Some(ref uci_move) = best_move {
            eprintln!(
                "[Tool:get_stockfish_context] Decoding best move {} to compute SAN",
                uci_move
            );
            if let Some((from_sq, to_sq, promotion)) = board.decode_uci_move(uci_move) {
                // determine capture: destination occupied OR efn-passant
                let mut is_capture = false;
                if board.squares[to_sq.0 as usize][to_sq.1 as usize].is_some() {
                    is_capture = true;
                } else if let Some(piece) = &board.squares[from_sq.0 as usize][from_sq.1 as usize] {
                    if piece.kind == PieceType::Pawn && board.en_passant_target == Some(to_sq) {
                        is_capture = true;
                    }
                }

                match board.compute_san_for_move(from_sq, to_sq, promotion, is_capture) {
                    Ok(san) => {
                        best_move_san = Some(san.clone());
                        eprintln!(
                            "[Tool:get_stockfish_context] Computed SAN for best move: {}",
                            san
                        );
                    }
                    Err(e) => eprintln!("[Tool] compute_san_for_move failed: {:?}", e),
                }
            } else {
                eprintln!(
                    "[Tool:get_stockfish_context] Failed to decode UCI move {} on board",
                    uci_move
                );
            }
        }
        let flipped_fen = flip_fen_turn(&args.fen);
        eprintln!(
            "[Tool:get_stockfish_context] Flipped FEN for opponent search: {}",
            flipped_fen
        );
        stockfish.set_fen_position(&flipped_fen);
        //get best move / threat
        match stockfish.go() {
            Ok(output) => {
                best_move = Some(output.best_move().clone());
                eprintln!(
                    "[Tool:get_stockfish_context] Opponent search best_move={}",
                    best_move.as_deref().unwrap_or("<none>")
                );
            }
            Err(e) => return Err(GetStockfishContextError(format!("{}", e))),
        }
        //get alternatives
        stockfish.set_fen_position(&args.fen);
        match stockfish.go() {
            Ok(output) => {
                main_threat = Some(output.best_move().clone());
                engine_depth = Some(output.depth().clone());
                eprintln!(
                    "[Tool:get_stockfish_context] Alternatives search main_threat={} depth={}",
                    main_threat.as_deref().unwrap_or("<none>"),
                    engine_depth.unwrap_or(0)
                );
                eval_score = {
                    let mut mate = None;
                    let mut centipawns = None;
                    let eval = output.eval().clone();
                    match eval.eval_type() {
                        EvalType::Centipawn => centipawns = Some(eval.value()),
                        EvalType::Mate => mate = Some(eval.value()),
                    };
                    eprintln!(
                        "[Tool:get_stockfish_context] Eval (alternatives): type={:?} value={}",
                        eval.eval_type(),
                        eval.value()
                    );
                    Some(EvalScore {
                        centipawns: centipawns,
                        mate_in: mate,
                        is_from_white_perspective,
                    })
                }
            }
            Err(e) => return Err(GetStockfishContextError(format!("{}", e))),
        }

        let mut is_searching = true;
        let mut max_pv_depth = 0;
        let mut pv_list: Vec<PvLine> = Vec::new();
        stockfish.uci_send(&"go depth 15 movetime 20000");
        eprintln!("[Tool:get_stockfish_context] Sent long go command for PV lines");
        while is_searching {
            let line = stockfish.read_line();
            if line.is_empty() {
                continue;
            }
            eprintln!("[Tool:get_stockfish_context] PV loop line: {}", line);
            //      pub struct PvLine {
            //     pub eval: EvalScore,
            //     pub moves: Vec<String>,     // UCI move sequence
            //     pub moves_san: Vec<String>, // SAN sequence for LLM readability
            // }
            if line.contains(" multipv ") && line.contains(" pv ") {
                let parts: Vec<&str> = line.split_whitespace().collect();
                let mut multipv_idx: u8 = 1;
                let mut depth: u32 = 0;
                let mut moves = String::new();
                let mut pv_score = EvalScore {
                    centipawns: None,
                    mate_in: None,
                    is_from_white_perspective,
                };
                let mut i = 0;
                while i < parts.len() {
                    match parts[i] {
                        "multipv" if i + 1 < parts.len() => {
                            multipv_idx = parts[i + 1].parse().unwrap_or(1);
                            i += 1;
                        }
                        "depth" if i + 1 < parts.len() => {
                            depth = parts[i + 1].parse().unwrap_or(0);
                            i += 1;
                        }
                        "score" if i + 2 < parts.len() => {
                            let mut mate = None;
                            let mut centipawns = None;
                            let score_value = parts[i + 2].parse().unwrap_or(0);
                            if parts[i + 1] == "mate" {
                                centipawns = Some(score_value);
                            } else {
                                mate = Some(score_value);
                            };
                            pv_score.centipawns = centipawns;
                            pv_score.mate_in = mate;
                            i += 2;
                        }
                        "pv" => {
                            moves = parts[i + 1..].join(" ");
                            break;
                        }
                        _ => {}
                    }
                    i += 1;
                }
                eprintln!(
                    "[Tool:get_stockfish_context] Parsed PV: multipv={} depth={} moves=\"{}\" score={:?}",
                    multipv_idx, depth, moves, pv_score
                );
                if depth >= 15 {
                    is_searching = false;
                    eprintln!(
                        "[Tool:get_stockfish_context] Reached target PV depth {}, stopping search",
                        depth
                    );
                }

                if depth >= max_pv_depth {
                    max_pv_depth = depth;
                    let uci_moves: Vec<String> =
                        moves.split_whitespace().map(|s| s.to_string()).collect();
                    pv_list.push(PvLine {
                        eval: pv_score,
                        moves: uci_moves.clone(),
                        moves_san: uci_moves,
                    });
                    eprintln!(
                        "[Tool:get_stockfish_context] Added PV line, total pv_list length={}",
                        pv_list.len()
                    );
                }
                if line.starts_with("bestmove") {
                    eprintln!(
                        "[Tool:get_stockfish_context] Received bestmove line, breaking PV loop"
                    );
                    break;
                }
            }
        }
        eprintln!(
            "[Tool:get_stockfish_context] Returning context: best_move={:?} best_move_san={:?} main_threat={:?} engine_depth={:?} eval_score={:?} pv_lines={}",
            best_move, best_move_san, main_threat, engine_depth, eval_score, pv_list.len()
        );
        return Ok(LlmStockfishContext {
            engine_depth,
            current_eval: eval_score,
            best_move,
            best_move_san,
            best_lines: Some(pv_list),
            main_threat: main_threat,
            is_check: in_check,
            is_game_over: game_over,
        });
    }
}
