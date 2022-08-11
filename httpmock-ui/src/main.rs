#![warn(clippy::all, rust_2018_idioms)]
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

#[macro_use]
extern crate lazy_static;

mod app;
mod component;
use app::TemplateApp;
mod history_db;
pub mod esay_md;

const PORT:&str = dotenv_codegen::dotenv!("PORT");

// When compiling natively:
#[cfg(not(target_arch = "wasm32"))]
fn main() {
    // Log to stdout (if you run with `RUST_LOG=debug`).

    tracing_subscriber::fmt::init();
    
    let path = format!("0.0.0.0:{}", PORT);
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
        format!("HTTP模拟服务器-端口{}", PORT).as_str(),
        native_options,
        Box::new(|cc| Box::new(TemplateApp::new(cc))),
    );
}
