pub mod common;
mod error;
mod matchers;
mod template;
mod aes_tool;

use std::{
    borrow::{BorrowMut},
    io::Error,
    sync::{Arc, RwLock},
};

#[macro_use]
extern crate lazy_static;

use common::{
    data::{HttpMockRequest, MockServerHttpResponse},
};
use poem::{Result, middleware::Cors, endpoint::StaticFilesEndpoint};
use poem::{
    get, handler,
    http::{Method, Uri},
    listener::TcpListener,
    middleware::Tracing,
    Body, EndpointExt, Request, RequestBody, Response, Route, RouteScheme, Server,
};

use crate::common::{MockServer, MOCK_SERVER, handle_mock_requset};


pub async fn serve(path:&str) -> Result<(), Error> {
    // {
        // let mock_define = MockDefine {
        //     id: 1,
        //     req: HttpMockRequest::new("/abc/*path".into()),
        //     resp: MockServerHttpResponse {
        //         status: Some(200),
        //         headers: None,
        //         body: Some("hello_world {{NAME_ZH()}} from {{ctx.path}}".as_bytes().to_vec()),
        //         delay: None,
        //     },
        //     relay_url: None,
        // };
        // let mock_define2 = MockDefine {
        //     id: 2,
        //     req: HttpMockRequest::new("/*".into()),
        //     resp: MockServerHttpResponse {
        //         status: Some(200),
        //         headers: None,
        //         body: Some("hello_world {{NAME_ZH()}},{{EMAIL()}}".as_bytes().to_vec()),
        //         delay: Some(Duration::from_millis(2000)),
        //     },
        //     relay_url: None,
        // };
        // let mut mock_server = MOCK_SERVER.write().unwrap();
        // mock_server.add(mock_define);
        // mock_server.add(mock_define2);
    // }
    // let app = RouteScheme::new().http(mock_handle).with(Tracing);

    let cors = Cors::default();

    let controller = get(mock_handle).put(mock_handle)
                                    .delete(mock_handle)
                                    .options(mock_handle)
                                    .delete(mock_handle)
                                    .trace(mock_handle)
                                    .post(mock_handle);
    let app = Route::new()
        .at("/_mock_list", get(list_all))
        .nest(
            "/_mock_info",
            StaticFilesEndpoint::new("./docs").index_file("index.html"),
        )
        .at("/*", controller).with(cors).with(Tracing);
    Server::new(TcpListener::bind(path))
        .run(app)
        .await
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