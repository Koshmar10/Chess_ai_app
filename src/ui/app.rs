use eframe::{egui::{self, vec2, CentralPanel, Color32, Frame, SidePanel, Stroke}, CreationContext};

use crate::{engine::{Board, PieceColor, PieceType}, game::controller::{GameController, GameMode}, ui::{theme, ui_setting::UiSettings, DEFAULT_FEN}};

pub enum AppScreen {
    MainMenu,
    TrainWithAi,
    Multiplayer,
    History,
    Analyze,
}
#[derive(Clone)]
pub enum PopupType { GameLostPopup(String) }
pub struct MyApp {
    pub screen: AppScreen,
    pub popup: Option<PopupType>,
    pub theme: theme::ThemeLoader,
    pub board: Board,
    pub game: GameController,
    pub ui: UiSettings,
}




impl From<&CreationContext<'_>> for MyApp{

    fn from(cc : &CreationContext) -> Self {

        Self {
            screen: AppScreen::MainMenu,
            popup: None,
            theme: theme::ThemeLoader::from(cc),
            board: Board::from(&DEFAULT_FEN.to_owned()),
            game: GameController::default(),
            ui : UiSettings::default()
        }

        

    }

}
//Update loop
impl eframe::App for MyApp {

    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        match self.screen {
            AppScreen::MainMenu => {
                self.render_main_menu(ctx, _frame);
            }
            AppScreen::TrainWithAi => {
                self.render_train_with_ai(ctx, _frame);
            }
            AppScreen::History => {

            }
            AppScreen::Multiplayer => {

            }
            AppScreen::Analyze => {

            }
        } // end matchd
        if let Some(popup) = self.popup.clone() {
            self.popup_handler(&popup, ctx, _frame);
        }
        }
            

        
    }  

