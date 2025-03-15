#![allow(unused)]

use simplelog::{SimpleLogger, LevelFilter};

mod backend;
mod ui;
mod core;
use core::event::{UiEvent, BackendEvent};
//use backend;
use iced_winit::runtime::Task;
use simplelog::ConfigBuilder;


#[tokio::main]
async fn main() {

    
    let config = ConfigBuilder::new()
        .set_thread_mode(simplelog::ThreadLogMode::Both)
        .add_filter_allow_str("akama")
        .build();
    SimpleLogger::init(LevelFilter::Debug, config).unwrap();

    let mut sj = 10;

    sj = 20;

    let (ui_tx, ui_rx) = async_channel::bounded::<UiEvent>(100);
    let (backend_tx, backend_rx) = async_channel::bounded::<BackendEvent>(100);


    // TODO: run ui in seperate thread instead of the backend
 
    tokio::spawn({
        backend::run(ui_rx, backend_tx)
    });
    
    
    ui::run(backend_rx, ui_tx.clone())

    


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

