use log;

#[tokio::main]
async fn main() {
    // tracing_subscriber::registry()
    // .with(fmt::layer())
    // .init();
    // if std::env::var_os("RUST_LOG").is_none() {
    //     std::env::set_var("RUST_LOG", "httpmock_server=debug");
    // }
    // egui_logger::init().unwrap();
    tracing_subscriber::fmt::init();
    log::set_max_level(log::LevelFilter::Debug);
    log::info!("启动....");
    let _ = server::serve("127.0.0.1:3000").await;
}
