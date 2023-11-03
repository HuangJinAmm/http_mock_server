use std::{error::Error, fs::File, io::Read};

use log;
use serde::{Serialize, Deserialize};
use server::common::{mock::MockDefine, MOCK_SERVER};

#[tokio::main]
async fn main() {
    // tracing_subscriber::registry()
    // .with(fmt::layer())
    // .init();
    if std::env::var_os("RUST_LOG").is_none() {
        std::env::set_var("RUST_LOG", "mock_server=DEBUG,server=DEBUG");
    }
    env_logger::init();
    // tracing_subscriber::fmt::init();
    // log::set_max_level(log::LevelFilter::Info);
    log::info!("启动....");
    let api = parse_config("./api.json5").unwrap();
    let url = format!("0.0.0.0:{}",api.port);
    {
        let mut mock_server = MOCK_SERVER.write().unwrap();
        for mock in api.apis {
            match mock_server.add(mock.clone(), 0) {
                Ok(_) => log::info!("add:{:?}",&mock),
                Err(s) => log::error!("add {:?},error:{}",&mock,s),
            };
        }
    }
    log::info!("服务地址:{}",url); 
    let _ = server::serve(&url).await;
}

fn parse_config(path:&str) -> Result<ApiConfig,String> {
    match File::open(path) {
        Ok(mut f) => {
            let mut buf = String::new();
            let _ = f.read_to_string(&mut buf);
            let apic:ApiConfig = json5::from_str(&buf).unwrap();
            Ok(apic)
        },
        Err(e) => {
            Err(e.to_string())
        },
    }
}

#[derive(Serialize, Deserialize, Clone)]
struct ApiConfig {
    port: u16,
    apis: Vec<MockDefine>
}