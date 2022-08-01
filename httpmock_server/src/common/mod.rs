use std::{
    collections::{BTreeMap, HashMap},
    sync::{Arc,RwLock}, borrow::BorrowMut,
};
use poem::Result;
use poem::error::Error;
use reqwest::StatusCode;

use crate::{matchers::{targets::{StringBodyTarget, MethodTarget, QueryParameterTarget, HeaderTarget, JSONBodyTarget}, comparators::{StringExactMatchComparator, JSONRegexMatchComparator, StringRegexMatchComparator}}};

use self::{
    data::{HttpMockRequest, Tokenizer, MockServerHttpResponse},
    filter::{
        JinjaTemplateHandler, MockFilter, MockFilterWrapper, RequestFilter, SingleValueMatcher, TEMP_ENV, MultiValueMatcher, RelayServerHandler, RegexValueMatcher,
    },
    mock::MockDefine,
    radix_tree::RadixTree,
};

pub mod data;
pub mod filter;
pub mod mock;
pub mod radix_tree;
// pub mod util;

lazy_static! {
    pub static ref MOCK_SERVER: Arc<RwLock<MockServer>> = {

        let server = Arc::new(RwLock::new(MockServer::new()));
        server
    };
    pub static ref FILTERS: Arc<RequestFilter> = 
            Arc::new(RequestFilter {
                mathcher: vec![
                    //方法
                    Box::new(SingleValueMatcher {
                        entity_name: "method",
                        target: Box::new(MethodTarget::new()),
                        comparator:  Box::new(StringExactMatchComparator::new(false)),
                        with_reason: false,
                        diff_with: Some(Tokenizer::Word),
                    }),
                    //请求方法参数
                    Box::new(MultiValueMatcher {
                        entity_name: "query parameter",
                        key_comparator: Box::new(StringExactMatchComparator::new(true)),
                        value_comparator: Box::new(StringRegexMatchComparator::new()),
                        target: Box::new(QueryParameterTarget::new()),
                        weight: 1,
                    }),
                
                    Box::new(MultiValueMatcher {
                        entity_name: "header",
                        key_comparator: Box::new(StringExactMatchComparator::new(true)),
                        value_comparator: Box::new(StringRegexMatchComparator::new()),
                        target: Box::new(HeaderTarget::new()),
                        weight: 1,
                    }),
                ],
                handler: JinjaTemplateHandler {},
                relay: RelayServerHandler{},
                body_mather: vec![
                    Box::new(RegexValueMatcher{
                        entity_name: "body",
                        comparator: Box::new(JSONRegexMatchComparator::new()),
                        target: Box::new(JSONBodyTarget::new()),
                        with_reason: true,
                        // weight: 1,
                    }),
                    Box::new(SingleValueMatcher::<String> {
                        entity_name: "body",
                        target: Box::new(StringBodyTarget::new()),
                        comparator:  Box::new(StringRegexMatchComparator::new()),
                        with_reason: false,
                        diff_with: Some(Tokenizer::Word),
                    }),
                ],
            });
}
pub struct MockServer{
    handler_dispatch: Arc<RwLock<RadixTree<u64>>>,
    handlers: Arc<RwLock<HashMap<u64, MockDefine>>>,
}

impl MockServer {
    pub fn new() -> Self {
        MockServer {
            handler_dispatch: Arc::new(RwLock::new(RadixTree::default())),
            handlers: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn list_all(&self) -> String{
        let server = self.handlers.read().unwrap();
        let all:Vec<MockDefine> = server.values().map(|mock|mock.clone()).collect();
        let all_string = serde_json::to_string(&all);
        match all_string {
            Ok(all) => all,
            Err(e) => e.to_string(),
        }
    }

    pub fn add(&mut self, mock: MockDefine) -> Result<(), String> {
        let mut dispath = self.handler_dispatch.write().unwrap();
        let mut server = self.handlers.write().unwrap();
        let id = mock.id;

        if let Some(template) = mock.resp.body.clone() {
            let temp_str = String::from_utf8(template).unwrap();
            if let Ok(mut lock) = TEMP_ENV.write() {
                let env = lock.borrow_mut();
                let mut source = env.source().unwrap().clone();
                let body_temp_key = id.to_string() + "_body";
                let _add_result = source.add_template(body_temp_key.as_str(), temp_str).map_err(|e| e.to_string())?;
                env.set_source(source);
                let url = mock.get_url();
                server.insert(id, mock.to_owned());
                let _route_result = dispath.add(url.as_str(), id).map_err(|e|e.to_string())?;
                return Ok(());
            }
        }
        Err("添时锁冲突".to_string())
    }

    // pub fn change(&mut self,mock:MockDefine) -> Result<(),RouteError> {
    //     self.add(mock)
    // }

    pub fn delete(&mut self, mock: MockDefine) {
        let mut server = self.handlers.write().unwrap();
        server.remove(&mock.id);
    }

    // pub async fn handle(&self, req: &mut HttpMockRequest) -> Result<MockServerHttpResponse> {
    //     let mut handler_wrap:Option<MockFilterWrapper> = None;
    //     if let Ok(server) = self.handler_dispatch.read() {
    //         if let Some(mock) = server.matches(&req.path) {
    //             if let Ok(handlers) = self.handlers.read() {
    //                 let exact_params =
    //                     mock.params
    //                         .into_iter()
    //                         .fold(BTreeMap::new(), |mut map, (key, value)| {
    //                             map.insert(key, value);
    //                             map
    //                         });
    //                 if let Some(handler) = handlers.get(mock.data) {
    //                     let hander_clone = handler.to_owned();
    //                     handler_wrap = Some(MockFilterWrapper {
    //                                                 mock_define: hander_clone,
    //                                                 mis_matchs: None,
    //                                                 req: req.clone(),
    //                                                 resp: None,
    //                                                 req_values: Some(exact_params),
    //                                             });
    //                 }
    //             }
    //         }
    //     }

    //     if let Some(mut hander_w) = handler_wrap {
    //         FILTERS.filter(&mut hander_w).await;
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
}

impl Default for MockServer {
    fn default() -> Self {
        Self::new()
    }
}

pub async fn handle_mock_requset(req: &mut HttpMockRequest) -> Result<MockServerHttpResponse> {
    let mut handler_wrap:Option<MockFilterWrapper> = None;
    if let Ok(mock_server) = MOCK_SERVER.read() {
        if let Ok(server) = mock_server.handler_dispatch.read() {
            if let Some(mock) = server.matches(&req.path) {
                if let Ok(handlers) = mock_server.handlers.read() {
                    let exact_params =
                        mock.params
                            .into_iter()
                            .fold(BTreeMap::new(), |mut map, (key, value)| {
                                map.insert(key, value);
                                map
                            });
                    if let Some(handler) = handlers.get(mock.data) {
                        let hander_clone = handler.to_owned();
                        handler_wrap = Some(MockFilterWrapper {
                                                    mock_define: hander_clone,
                                                    mis_matchs: None,
                                                    req: req.clone(),
                                                    resp: None,
                                                    req_values: Some(exact_params),
                                                });
                    }
                }
            }
        }
    }

    if let Some(mut hander_w) = handler_wrap {
        FILTERS.filter(&mut hander_w).await;
        if let Some(resp) = hander_w.resp {
            return Ok(resp);
        } else if let Some(mis_match) = hander_w.mis_matchs {
            let resp = serde_json::to_string_pretty(&mis_match).unwrap();
            let not_found = Error::from_string(resp,StatusCode::BAD_REQUEST);
            return Err(not_found);
        } else {
            return Err(Error::from_string("服务器未返回任何数据",StatusCode::INTERNAL_SERVER_ERROR));
        }
    } else {
        return Err(Error::from_string("未找到相应的配置",StatusCode::NOT_FOUND));
    }
}
