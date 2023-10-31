use once_cell::sync::Lazy;
use poem::error::Error;
use poem::Result;
use reqwest::StatusCode;
use serde_json::Value;
use std::{
    borrow::BorrowMut,
    collections::{BTreeMap, HashMap},
    sync::{Arc, RwLock},
};

use crate::{
    matchers::{
        comparators::{
            JSONRegexMatchComparator, JsonSchemaMatchComparator, StringExactMatchComparator,
            StringRegexMatchComparator,
        },
        targets::{
            HeaderTarget, JSONBodyTarget, JSONSchemaTarget, MethodTarget, QueryParameterTarget,
            StringBodyTarget,
        },
    },
    template::TEMP_ENV,
};

use self::{
    data::{HttpMockRequest, MockServerHttpResponse, Tokenizer},
    filter::{
        JinjaTemplateHandler, JsonSchemaMatcher, MockFilter, MockFilterWrapper, MultiValueMatcher,
        RegexValueMatcher, RelayServerHandler, RequestFilter, SingleValueMatcher,
    },
    mock::MockDefine,
    radix_tree::RadixTree,
};

pub mod data;
pub mod filter;
pub mod mock;
pub mod radix_tree;
// pub mod util;

pub static MOCK_SERVER: Lazy<Arc<RwLock<MockServer>>> = Lazy::new(|| {
    let server = Arc::new(RwLock::new(MockServer::new()));
    server
});
pub static FILTERS: Lazy<Arc<RequestFilter>> = Lazy::new(|| {
    Arc::new(RequestFilter {
        mathcher: vec![
            //方法
            Box::new(SingleValueMatcher {
                entity_name: "method",
                target: Box::new(MethodTarget::new()),
                comparator: Box::new(StringExactMatchComparator::new(false)),
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
        relay: RelayServerHandler {},
        body_mather: vec![
            Box::new(JsonSchemaMatcher {
                entity_name: "body schema match",
                target: Box::new(JSONBodyTarget::new()),
                source: Box::new(JSONSchemaTarget::new()),
                comparator: Box::new(JsonSchemaMatchComparator::new()),
                with_reason: false,
            }),
            Box::new(RegexValueMatcher {
                entity_name: "body json regex match",
                comparator: Box::new(JSONRegexMatchComparator::new()),
                target: Box::new(JSONBodyTarget::new()),
                with_reason: true,
                // weight: 1,
            }),
            Box::new(SingleValueMatcher::<String> {
                entity_name: "body string match",
                target: Box::new(StringBodyTarget::new()),
                comparator: Box::new(StringRegexMatchComparator::new()),
                with_reason: false,
                diff_with: Some(Tokenizer::Word),
            }),
        ],
    })
});
pub struct MockServer {
    handler_dispatch: Arc<RwLock<RadixTree<Vec<u64>>>>,
    handlers: Arc<RwLock<HashMap<u64, MockDefine>>>,
}

impl MockServer {
    pub fn new() -> Self {
        MockServer {
            handler_dispatch: Arc::new(RwLock::new(RadixTree::default())),
            handlers: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn list_all(&self) -> String {
        let server = self.handlers.read().unwrap();
        let all: Vec<MockDefine> = server.values().map(|mock| mock.clone()).collect();
        let all_string = serde_json::to_string(&all);
        match all_string {
            Ok(all) => all,
            Err(e) => e.to_string(),
        }
    }

    pub fn add(&mut self, mock: MockDefine, priority: usize) -> Result<(), String> {
        let mut dispath = self.handler_dispatch.write().unwrap();
        let mut server = self.handlers.write().unwrap();
        let id = mock.id;
        // if let Some(template) = mock.resp.body.clone() {
        // let temp_str = String::from_utf8(template).unwrap();
        // if let Ok(mut lock) = TEMP_ENV.write() {
        // let env = lock.borrow_mut();
        // let mut source = env.source().unwrap().clone();

        //添加header的值到模板
        // if let Some(headers) = mock.resp.headers.as_ref() {
        //     for (key,val) in headers {
        //         if val.contains("{{") && val.contains("}}") {
        //             let header_key = format!("{}_header_{}", id, key);
        //             source.add_template(header_key, val).map_err(|e| e.to_string())?;
        //         }
        //     }
        // }

        //添加body到模板
        // let body_temp_key = id.to_string() + "_body";
        // let _add_result = source.add_template(body_temp_key.as_str(), temp_str).map_err(|e| e.to_string())?;
        // env.set_source(source);
        let url = mock.get_url();
        server.insert(id, mock.to_owned());

        if let Some(matches) = dispath.matches(url.as_str()) {
            let mut exist_data = matches.data.clone();
            if !exist_data.contains(&id) {
                exist_data.insert(priority, id);
                // exist_data.push(id);
                let _route_result = dispath
                    .add(url.as_str(), exist_data)
                    .map_err(|e| e.to_string())?;
            }
        } else {
            let _route_result = dispath
                .add(url.as_str(), vec![id])
                .map_err(|e| e.to_string())?;
        }
        return Ok(());
        // }
        // }
        // Err("添时锁冲突".to_string())
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
    log::info!("开始处理请求{}", &req.path);
    let mut handler_wrap: Vec<MockFilterWrapper> = Vec::new();

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
                    let ids = mock.data;
                    log::debug!("匹配到的模板ids：{:#?}", &ids);
                    log::debug!("提取请求变量：{:#?}", &exact_params);
                    for id in ids {
                        if let Some(handler) = handlers.get(id) {
                            let hander_clone = handler.to_owned();
                            let handler_wrap_item = MockFilterWrapper {
                                mock_define: hander_clone,
                                mis_matchs: None,
                                req: req.clone(),
                                resp: None,
                                req_values: Some(exact_params.clone()),
                            };
                            handler_wrap.push(handler_wrap_item);
                        }
                    }
                }
            }
        }
    }

    if handler_wrap.is_empty() {
        log::info!("未找到对应的配置");
        return Err(Error::from_string(
            "未找到相应的配置",
            StatusCode::NOT_FOUND,
        ));
    }
    let mut all_mis_matches = Vec::new();
    for mut hander_w in handler_wrap {
        FILTERS.filter(&mut hander_w).await;
        if let Some(resp) = hander_w.resp {
            log::debug!("返回响应:{:#?}", &resp);
            return Ok(resp);
        } else if let Some(mis_match) = hander_w.mis_matchs {
            all_mis_matches.extend(mis_match);
        }
    }

    if all_mis_matches.is_empty() {
        log::info!("服务器未返回任何数据");
        return Err(Error::from_string(
            "服务器未返回任何数据",
            StatusCode::INTERNAL_SERVER_ERROR,
        ));
    } else {
        let resp = serde_json::to_string_pretty(&all_mis_matches).unwrap();
        log::info!("匹配失败:{}", &resp);
        let not_found = Error::from_string(resp, StatusCode::BAD_REQUEST);
        return Err(not_found);
    }
}
