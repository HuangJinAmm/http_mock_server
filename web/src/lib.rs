#![warn(clippy::all, rust_2018_idioms)]

pub mod api_context;
mod app;
pub mod book;
pub mod component;
pub mod history_db;
pub mod request_data;
pub mod ui;
pub mod utils;
pub use app::TemplateApp;
use once_cell::sync::Lazy;

pub static PORT: Lazy<String> = Lazy::new(|| {
    dotenv::dotenv().ok();
    match std::env::var("MOCK_PORT") {
        Ok(p) => p,
        Err(_) => "13001".to_owned(),
    }
});
