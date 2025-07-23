use eframe::egui::Color32;

pub struct UiSettings {
    pub square_size: f32,
    pub timer_inside: Color32,
    pub timer_outside: Color32,
    pub title_size: f32,
    pub subtitle_size: f32,
    pub menu_quote: Option<String>,
    pub default_subtitle:String,
    
}
impl Default for  UiSettings {
    fn default() -> Self {
        Self {
            square_size : 16.0,
            title_size: 36.0,
            subtitle_size: 24.0,
            timer_inside: Color32::from_rgba_unmultiplied(38, 38, 37, 255),
            timer_outside: Color32::DARK_GRAY,
            default_subtitle: String::from("- The great chess experience -"),
            menu_quote: None
            
        }
    }
}