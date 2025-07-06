use eframe::{egui::{self, vec2, CentralPanel, Color32, Painter, Pos2, Rect, Sense, SidePanel}, CreationContext};

use crate::{chess_utils::{Board, ChessPiece}, etc::DEFAULT_FEN, theme::ThemeLoader};

pub mod theme;
pub mod chess_utils;
pub mod etc;
pub mod ui;




