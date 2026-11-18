mod data;
mod app;
mod menus;
mod mob;
mod enemy;
mod skill;
mod item;
mod character;
mod inventory;
mod popup;

use crate::app::App;


fn main() -> Result<std::io::Result<()>, Box<dyn std::error::Error>> {
    data::init_game_data()?;
    Ok(ratatui::run(|terminal| App::new().run(terminal)))
}
