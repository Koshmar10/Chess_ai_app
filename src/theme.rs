

use std::collections::HashMap;

use std::error::Error;

use eframe::egui::Color32;
use eframe::egui::ColorImage;
use eframe::egui::TextureHandle;
use eframe::egui::TextureOptions;

use eframe::CreationContext;

use crate::chess_utils::PieceColor;
use crate::chess_utils::PieceType;




pub struct ThemeLoader{
    pub dark_square: Color32,
    pub light_square: Color32,
    pub dark_square_hover: Color32,
    pub light_square_hover: Color32,
    pub empty_texture: TextureHandle,
    pub piece_map: HashMap<(PieceType, PieceColor), Result<TextureHandle, Box<dyn Error>>>, 
    pub square_select_highlight: Color32,
    pub dark_pseudo_move_highlight: Color32,  
    pub light_pseudo_move_highlight: Color32,
            
}

impl From<&CreationContext<'_>> for ThemeLoader {
    fn from(cc : &CreationContext<'_>) -> Self {
        ThemeLoader { 
            dark_square: Color32::from_rgb(181, 136, 99), 
            light_square: Color32::from_rgb(230, 207, 171), 
            dark_square_hover: Color32::from_rgb(191, 146, 119), 
            light_square_hover: Color32::from_rgb(240, 217, 181),
            square_select_highlight: Color32::from_rgba_unmultiplied(255, 255, 0, 128),
            light_pseudo_move_highlight: Color32::from_rgba_unmultiplied(70, 70, 70, 128),
            dark_pseudo_move_highlight: Color32::from_rgba_unmultiplied( 80,  80,  80, 128),
            
            empty_texture: cc.egui_ctx.load_texture(
                "empty_piece",
                ColorImage::new([1, 1], Color32::TRANSPARENT),
                TextureOptions::default(),
            ),
            piece_map: {
                let mut map = HashMap::with_capacity(12);
                for &color in &[PieceColor::White, PieceColor::Black] {
                    for &piece in &[
                        PieceType::Pawn,
                        PieceType::Bishop,
                        PieceType::King,
                        PieceType::Knight,
                        PieceType::Queen,
                        PieceType::Rook,
                    ] {
                        let image_path = format!(
                            "assets/pieces/{}_{}.png",
                            color.to_string().to_lowercase(),
                            piece.to_string().to_lowercase()
                        );
                        map.insert((piece, color), load_texture(cc, &image_path));
                    }
                }
                map
            },

        }
    }
}
fn load_texture(cc: &CreationContext, image_path: &str) ->Result<TextureHandle, Box<dyn Error>>{
    let image = image::open(image_path)
    .expect("Failed to load img")
    .to_rgba8();
    let (w, h) = image.dimensions();
    let pixels = image
    .chunks_exact(4)
    .map(|rgba| { Color32::from_rgba_unmultiplied(rgba[0], rgba[1], rgba[2], rgba[3])})
    .collect();
    let color_image = ColorImage { size:[w as usize,h as usize], pixels: pixels};
    
    let texture_name = image_path.split("/").last().unwrap().strip_suffix(".png").unwrap();
    
    let texture = cc.egui_ctx.load_texture(texture_name, color_image, TextureOptions::default());
    Ok(texture)
}


