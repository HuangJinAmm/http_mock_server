pub mod aes_tool;
pub mod common;
mod error;
mod matchers;
pub mod template;

use std::{
    borrow::BorrowMut,
    io::Error,
    sync::{Arc, RwLock},
};

use common::data::{HttpMockRequest, MockServerHttpResponse};
use poem::{endpoint::StaticFilesEndpoint, middleware::Cors, post, web::Json, Result};
use poem::{
    get, handler,
    http::{Method, Uri},
    listener::TcpListener,
    middleware::Tracing,
    Body, EndpointExt, Request, RequestBody, Response, Route, RouteScheme, Server,
};

use crate::common::{handle_mock_requset, mock::MockDefine, MockServer, MOCK_SERVER};

pub async fn serve(path: &str) -> Result<(), Error> {
    let cors = Cors::default();
    let controller = get(mock_handle)
        .put(mock_handle)
        .delete(mock_handle)
        .options(mock_handle)
        .delete(mock_handle)
        .trace(mock_handle)
        .post(mock_handle);
    let app = Route::new()
        .at("/mock_list", get(list_all))
        .at("/mock_add", post(add_mock))
        .at("/mock_remove", post(remove_mock))
        .nest(
            "/mock_info",
            StaticFilesEndpoint::new("./docs/book").index_file("index.html"),
        )
        .at("/*", controller)
        .with(cors)
        .with(Tracing);
    log::info!("启动服务...");
    Server::new(TcpListener::bind(path)).run(app).await
}

#[handler]
async fn mock_handle(mut req: HttpMockRequest) -> Result<MockServerHttpResponse> {
    handle_mock_requset(&mut req).await
}

#[handler]
fn list_all() -> String {
    let mock_server = MOCK_SERVER.read().unwrap();
    mock_server.list_all()
}

#[handler]
fn add_mock(mock: Json<MockDefine>) -> String {
    let mut mock_server = MOCK_SERVER.write().unwrap();
    match mock_server.add(mock.0) {
        Ok(_) => "添加成功".to_string(),
        Err(s) => s,
    }
}

#[handler]
fn remove_mock(mock: Json<MockDefine>) -> String {
    let mut mock_server = MOCK_SERVER.write().unwrap();
    mock_server.delete(mock.0);
    "删除成功".into()
}

// async fn handle(req: &mut HttpMockRequest) -> Result<MockServerHttpResponse> {
//     let mut handler_wrap:Option<MockFilterWrapper> = None;
//     if let Ok(mock_server) = MOCK_SERVER.read() {
//         if let Ok(server) = mock_server.handler_dispatch.read() {
//             if let Some(mock) = server.matches(&req.path) {
//                 if let Ok(handlers) = mock_server.handlers.read() {
//                     let exact_params =
//                         mock.params
//                             .into_iter()
//                             .fold(BTreeMap::new(), |mut map, (key, value)| {
//                                 map.insert(key, value);
//                                 map
//                             });
//                     if let Some(handler) = handlers.get(mock.data) {
//                         let hander_clone = handler.to_owned();
//                         handler_wrap = Some(MockFilterWrapper {
//                                                     mock_define: hander_clone,
//                                                     mis_matchs: None,
//                                                     req: req.clone(),
//                                                     resp: None,
//                                                     req_values: Some(exact_params),
//                                                 });
//                     }
//                 }
//             }
//         }
//     }

//     if let Some(mut hander_w) = handler_wrap {
//         for filter in self.filters.iter() {
//             filter.filter(&mut hander_w).await;
//         }
//         if let Some(resp) = hander_w.resp {
//             return Ok(resp);
//         } else if let Some(mis_match) = hander_w.mis_matchs {
//             let resp = serde_json::to_string_pretty(&mis_match).unwrap();
//             let not_found = Error::from_string(resp,StatusCode::BAD_REQUEST);
//             return Err(not_found);
//         } else {
//             return Err(Error::from_string("服务器未返回任何数据",StatusCode::INTERNAL_SERVER_ERROR));
//         }
//     } else {
//         return Err(Error::from_string("未找到相应的配置",StatusCode::NOT_FOUND));
//     }
// }
