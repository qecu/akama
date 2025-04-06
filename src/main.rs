#![allow(dead_code)]
use simplelog::{LevelFilter, SimpleLogger};
mod backend;

/// holds shared data types between ui and backend;
mod common;
// mod ui;
mod screen;

use common::{BackendEvent, UiEvent};
//use backend;
//use iced_winit::runtime::Task;
use simplelog::ConfigBuilder;

#[tokio::main]
async fn main() {
    // iced::widget::radio
    // use ui::screen::dashboard::style::Theme;

    // let file = std::fs::read_to_string("mytheme.toml").unwrap();
    // let theme: Theme = toml::from_str(file.as_str()).unwrap();
    // println!("theme {:?}", theme);

    // let ss = Column::new();

    let config = ConfigBuilder::new()
        .set_thread_mode(simplelog::ThreadLogMode::Both)
        .add_filter_allow_str("akama")
        .build();
    SimpleLogger::init(LevelFilter::Debug, config).unwrap();

    let (ui_tx, ui_rx) = async_channel::bounded::<UiEvent>(100);
    let (backend_tx, backend_rx) = async_channel::bounded::<BackendEvent>(100);

    // TODO: run ui in seperate thread instead of the backend

    tokio::spawn(backend::Backend::run(backend_tx, ui_rx));

    screen::run(backend_rx, ui_tx.clone())

    //let ui_tx_c = ui_tx.clone();
    //std::thread::spawn( || {
    //tokio::spawn( async {
    //akama_ui::run(backend_rx, ui_tx.clone())
    //akama_ui::run(backend_rx, ui_tx_c)
    //});

    //let ss = iced_winit::program::run(
    //    iced_winit::Settings::default(),
    //    iced_winit::graphics::Settings::default(),
    //    None,
    //
    //    //((iced_winit::Program::State::default(), Task::none()))
    //
    //);
}
