#![warn(clippy::all, rust_2018_idioms)]
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

#[macro_use]
extern crate lazy_static;

mod app;
mod component;
use app::TemplateApp;
// use log4rs::{append::{console::ConsoleAppender, file::FileAppender}, encode::pattern::PatternEncoder, Config, config::{Appender, Root, Logger} };
mod history_db;
pub mod esay_md;
pub use httpmock_ui::PORT;

// When compiling natively:
#[cfg(not(target_arch = "wasm32"))]
fn main() {
    // Log to stdout (if you run with `RUST_LOG=debug`).
    // log4rs::init_file("log.yml", Default::default()).unwrap();
    // let logsub = tracing_subscriber::FmtSubscriber::builder()
    //         .with_max_level(Level::DEBUG)
    //         .with_env_filter("httpmock_ui=debug,httpmock_server=debug")
    //         .finish();

    // tracing::subscriber::set_global_default(logsub).expect("日志初始化失败");

    // tracing_subscriber::fmt().with_max_level(Level::INFO).with_env_filter("httpmock_ui=debug,httpmock_server=debug").init();

    let path = format!("0.0.0.0:{}", PORT.to_string());
    log::info!("服务器地址：{}",path);
    // use std::thread;

    // use tracing::Level;

    use std::thread;
    thread::spawn(move ||{
        tokio::runtime::Builder::new_multi_thread().worker_threads(1)
        .enable_all().build().unwrap().block_on(async {
            log::info!("启动....");
            let _ = httpmock_server::serve(path.as_str()).await;
        });
    });
    let native_options = eframe::NativeOptions{
        // icon_data: todo!(),
        initial_window_size: Some(egui::Vec2{x:1200.0,y:600.0}),
        min_window_size: Some(egui::Vec2{x:1200.0,y:600.0}),
        ..Default::default()
    };
    eframe::run_native(
        format!("HTTP模拟服务器-端口{}", PORT.to_string()).as_str(),
        native_options,
        Box::new(|cc| Box::new(TemplateApp::new(cc))),
    );
}

// fn init_log() {
//     let stdout = ConsoleAppender::builder()
//         .encoder(Box::new(PatternEncoder::new("{d} - {l} -{t} - {m}{n}")))
//         .build();

//     let file = FileAppender::builder()
//         .encoder(Box::new(PatternEncoder::new("{d} - {l} - {t} - {m}{n}")))
//         .build("log/test.log")
//         .unwrap();

//     let config = Config::builder()
//         .appender(Appender::builder().build("stdout", Box::new(stdout)))
//         .appender(Appender::builder().build("file", Box::new(file)))
//         .logger(Logger::builder()
//             .appender("file")
//             .additive(false)
//             .build("app", LevelFilter::Info))
//         .build(Root::builder().appender("stdout").build(LevelFilter::Debug))
//         .unwrap();

//     let _ = log4rs::init_config(config).unwrap();
// }