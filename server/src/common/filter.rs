
use std::collections::BTreeMap;
use std::error::Error;
use std::fmt::{Display, Debug};

use async_trait::async_trait;
use minijinja::value::Value;
use minijinja::{context};

use serde_json::Value as JValue;


use crate::matchers::comparators::{match_json_key, ValueComparator};
use crate::matchers::targets::{MultiValueTarget, ValueTarget};
use crate::matchers::{diff_str, Matcher};
use crate::template::{TEMP_ENV, rander_template};

use super::data::{HttpMockRequest, Mismatch, MockServerHttpResponse, Reason, Tokenizer};
use super::mock::MockDefine;

#[derive(Debug)]
pub struct MockFilterWrapper {
    pub req_values: Option<BTreeMap<String, String>>,
    pub req: HttpMockRequest,
    pub resp: Option<MockServerHttpResponse>,
    pub mock_define: MockDefine,
    pub mis_matchs: Option<Vec<Mismatch>>,
}

impl MockFilterWrapper {
    #[inline]
    pub fn is_failed(&self) -> bool {
        self.mis_matchs.is_some()
    }
}

#[async_trait]
pub trait MockFilter: Send + Sync {
    async fn filter(&self, req: &mut MockFilterWrapper);
}

#[async_trait]
pub trait MockHandler: Send + Sync {
    async fn handle(&self, req: &mut MockFilterWrapper) -> Option<MockServerHttpResponse>;
}

pub struct RequestFilter {
    pub mathcher: Vec<Box<dyn Matcher>>,
    pub body_mather: Vec<Box<dyn Matcher>>,
    pub handler: JinjaTemplateHandler,
    pub relay: RelayServerHandler,
}

#[async_trait]
impl MockFilter for RequestFilter {
    async fn filter(&self, filter_wrapper: &mut MockFilterWrapper) {
        log::debug!("开始执行过滤器逻辑");
        let req = &filter_wrapper.req;
        let mock = &mut filter_wrapper.mock_define.req;
        let matched = self
            .mathcher
            .iter()
            .all(|matcher| matcher.matches(req, mock));
        let body_matcher = self
            .body_mather
            .iter()
            .any(|matcher| matcher.matches(req, mock));

        log::debug!("请求数据匹配:{}",matched);
        log::debug!("请求Body匹配:{}",body_matcher);
        if matched && body_matcher {
            let resp;
            if let Some(_url) = filter_wrapper.mock_define.relay_url.clone() {
                resp = self.relay.handle(filter_wrapper).await;
            } else {
                resp = self.handler.handle(filter_wrapper).await;
            }
            filter_wrapper.resp = resp;
        } else {
            let mut miss:Vec<Mismatch> = Vec::new();
            if !matched {
                let mut miss_query =
                    self.mathcher
                        .iter()
                        .fold(Vec::new(), |mut mismatchs: Vec<Mismatch>, matcher| {
                            let mut sub_mis = matcher.mismatches(req, mock);
                            mismatchs.append(&mut sub_mis);
                            mismatchs
                        });
                miss.append(&mut miss_query);
            }
            if !body_matcher {
                let mut miss_body = self.body_mather.iter()
                            .fold(Vec::new(), |mut mismatchs:Vec<Mismatch>,matcher|{
                            let mut sub_mis = matcher.mismatches(req, mock);
                            mismatchs.append(&mut sub_mis);
                            mismatchs
                });
                miss.append(&mut miss_body);
            }
            if !filter_wrapper.is_failed() {
                filter_wrapper.mis_matchs = Some(Vec::new());
            }
            let mis_wrap = filter_wrapper.mis_matchs.as_mut().unwrap();
            miss.append(mis_wrap);
            filter_wrapper.mis_matchs = Some(miss);
        }
    }
}

pub struct JinjaTemplateHandler {}

#[async_trait]
impl MockHandler for JinjaTemplateHandler {
    async fn handle(&self, req: &mut MockFilterWrapper) -> Option<MockServerHttpResponse> {
        let mut ret_mock_resp:Option<MockServerHttpResponse> = None;
        let start = std::time::SystemTime::now();

        log::debug!("JinjaTemplateHandler 执行");
        //提取相关模板变量
        let path = req.req_values.clone().unwrap_or_default();

        let mut body = Value::UNDEFINED;
        let request = req.req.clone();
        if let Some(b) = &request.body {
            if let Ok(body_json_value) = serde_json::from_slice::<Value>(b.as_slice()) {
                body = body_json_value;
            } else {
                body = Value::from_safe_string(String::from_utf8_lossy(b.as_slice()).to_string());
            }
        }
        let HttpMockRequest {
            path: url,
            method,
            headers,
            query_params,
            ..
        } = request;

        let temp_ctx = context!(path, url, body, method, headers, query_params);
        log::debug!("获取到的局部变量{:#?}",&temp_ctx);
        if let Ok(env) = TEMP_ENV.read() {
            //处理body模板
            if let Some(body_tmp) = req.mock_define.resp.body.clone().map(|b|String::from_utf8(b).unwrap_or_default()) {
                let mut mock_resp = req.mock_define.resp.clone();
                let rendered = match env.render_str(&body_tmp,temp_ctx.clone()) {
                    Ok(s) => s,
                    Err(e) => e.to_string(),
                };
                mock_resp.body = Some(rendered.as_bytes().to_vec());
                ret_mock_resp = Some(mock_resp);
            }

            //处理header的模板
            if let Some(mock_headers) = req.mock_define.resp.headers.clone(){
                let dealed_headers:Vec<(String,String)> = mock_headers.into_iter()
                    .map(|(key,val)|{
                        let rander_header = match env.render_str(&val,temp_ctx.clone()) {
                            Ok(s) => s,
                            Err(e) => e.to_string(),
                        };
                        (key,rander_header)
                    }).collect();
                if let Some(ret_mock_resp_c) = ret_mock_resp.as_mut() {
                    ret_mock_resp_c.headers = Some(dealed_headers);
                }
            }
        }

        //处理延时,本应该放到另外一个handler里面的，这里偷懒了
        let end = start.elapsed().unwrap();
        if let Some(delay) = req.mock_define.resp.delay {
            if let Some(sleep) = delay.checked_sub(end) {
                if sleep.as_millis() > 120 {
                    // thread::sleep(sleep);
                    log::debug!("延时：{:#?}",&sleep);
                    let _ = tokio::time::sleep(sleep).await;
                }
            }
        }
        ret_mock_resp
    }
}

pub struct RelayServerHandler {}

#[async_trait]
impl MockHandler for RelayServerHandler {
    async fn handle(&self, req: &mut MockFilterWrapper) -> Option<MockServerHttpResponse> {
        log::debug!("转发处理逻辑开始");
        let client = reqwest::Client::new();
        let mut req_clone = req.req.clone();
        if let Some(redict_url) = req.mock_define.relay_url.clone() {
            log::debug!("转发地址：{}",&redict_url);
            req_clone.path = redict_url;
            let real_req = req_clone.into(); 
            let resp = client.execute(real_req);
            match resp.await {
                Ok(r) => {
                    let status = Some(r.status().as_u16());
                    log::debug!("转发响应状态：{:#?}",&status);
                    let mut headers: Vec<(String, String)> = Vec::new();
                    for (hn, hv) in r.headers().iter() {
                        let hns = hn.to_string();
                        let hvs = hv.to_str().unwrap().to_string();
                        headers.push((hns, hvs));
                    }
                    let body = Some(r.text().await.unwrap_or_else(|e|{e.to_string()}).as_bytes().to_vec());
                    Some(MockServerHttpResponse {
                        status,
                        headers: Some(headers),
                        body,
                        delay: None,
                    })
                }
                Err(e) => {
                    log::error!("转发响应错误:{:#?}",&e);
                    let body = Some(e.to_string().as_bytes().to_vec());
                    Some(MockServerHttpResponse {
                        status: Some(500),
                        headers: None,
                        body,
                        delay: None,
                    })
                }
            }
        } else {
            None
        }
    }
}

pub struct DoNothingHandler {}


#[async_trait]
impl MockHandler for DoNothingHandler {
    async fn handle(&self, _req: &mut MockFilterWrapper) -> Option<MockServerHttpResponse> {
        None
    }
}

pub(crate) struct RegexValueMatcher {
    pub entity_name: &'static str,
    pub target: Box<dyn ValueTarget<JValue> + Send + Sync>,
    pub comparator: Box<dyn ValueComparator<JValue, JValue> + Send + Sync>,
    pub with_reason: bool,
    // pub weight: usize,
}

impl Matcher for RegexValueMatcher {
    fn matches(&self, req: &HttpMockRequest, mock: &HttpMockRequest) -> bool {
        let req_value = self.target.parse_from_request(req);
        let mock_value = self.target.parse_from_request(mock);
        log::debug!("MockValue:{:#?}",&mock_value);
        log::debug!("ReqValue:{:#?}",&req_value);
        // let mock_body = mock.body.clone().unwrap();
        // dbg!(String::from_utf8(mock_body).unwrap());
        // dbg!(&mock_value);
        match (mock_value, req_value) {
                    (None, _) => {
                        // mock_value 为空，但是body有值的情况，走后续匹配
                        if req.body.is_some() {
                            return false;
                        } else {
                            return true;
                        }
                    },
                    (Some(_), None) => return false,
                    (Some(mock), Some(req)) => self.comparator.matches(&mock, &req),
                }
    }

    fn distance(&self, req: &HttpMockRequest, mock: &HttpMockRequest) -> usize {
        let req_value = self.target.parse_from_request(req);
        let mock_value = self.target.parse_from_request(mock);
        self.comparator.distance(&mock_value.as_ref(), &req_value.as_ref())
    }

    fn mismatches(&self, req: &HttpMockRequest, mock: &HttpMockRequest) -> Vec<Mismatch> {
        let req_value = self.target.parse_from_request(req);
        let mock_value = self.target.parse_from_request(mock);
        match (mock_value, req_value) {
            (None, _) => return Vec::new(),
            (Some(m), None) => {
                let mut mis_vec = Vec::new();
                let mis_match = Mismatch {
                    title: format!("{} 不匹配", self.entity_name),
                    reason: match self.with_reason {
                        true => Some(Reason {
                            expected: m.to_string(),
                            actual: "not found".to_string(),
                            comparison: self.comparator.name().into(),
                            best_match: false,
                        }),
                        false => None,
                    },
                    diff: None,
                };
                mis_vec.push(mis_match);
                mis_vec
            }
            (Some(mock), Some(req)) => {
                let mut mis_vec = Vec::new();
                let root = "$".to_string();
                let result = match_json_key(root, &mock, &req).unwrap();
                let mis_match = Mismatch {
                    title: result,
                    reason: match self.with_reason {
                        true => Some(Reason {
                            expected: mock.to_string(),
                            actual: req.to_string(),
                            comparison: self.comparator.name().into(),
                            best_match: false,
                        }),
                        false => None,
                    },
                    diff: None,
                };
                mis_vec.push(mis_match);
                mis_vec
            }
        }
    }
}

pub(crate) struct SingleValueMatcher<T>
where
    T: Display,
{
    pub entity_name: &'static str,
    pub target: Box<dyn ValueTarget<T> + Send + Sync>,
    pub comparator: Box<dyn ValueComparator<T, T> + Send + Sync>,
    // pub transformer: Option<Box<dyn Transformer<T, T> + Send + Sync>>,
    pub with_reason: bool,
    pub diff_with: Option<Tokenizer>,
    // pub weight: usize,
}

impl<T> Matcher for SingleValueMatcher<T>
where
    T: Display+Debug,
{
    fn matches(&self, req: &HttpMockRequest, mock: &HttpMockRequest) -> bool {
        let req_value = self.target.parse_from_request(req);
        let mock_value = self.target.parse_from_request(mock);
        log::debug!("MockValue:{:#?}",&mock_value);
        log::debug!("ReqValue:{:#?}",&req_value);
        match (mock_value, req_value) {
                    (None, _) => return true,
                    (Some(_), None) => return false,
                    (Some(mock), Some(req)) => self.comparator.matches(&mock, &req),
                }
    }

    fn distance(&self, req: &HttpMockRequest, mock: &HttpMockRequest) -> usize {
        let req_value = self.target.parse_from_request(req);
        let mock_values = self.target.parse_from_request(mock);
        self.comparator
            .distance(&mock_values.as_ref(), &req_value.as_ref())
    }

    fn mismatches(&self, req: &HttpMockRequest, mock: &HttpMockRequest) -> Vec<Mismatch> {
        let req_value = self.target.parse_from_request(req);
        let mock_value = self.target.parse_from_request(mock);
        match (mock_value, req_value) {
            (None, _) => return Vec::new(),
            (Some(m), None) => {
                let mut mis_vec = Vec::new();
                let mis_match = Mismatch {
                    title: format!("{} 不匹配", self.entity_name),
                    reason: match self.with_reason {
                        true => Some(Reason {
                            expected: m.to_string(),
                            actual: "not found".to_string(),
                            comparison: self.comparator.name().into(),
                            best_match: false,
                        }),
                        false => None,
                    },
                    diff: self.diff_with.map(|t| diff_str(&m.to_string(), "", t)),
                };
                mis_vec.push(mis_match);
                mis_vec
            }
            (Some(mock), Some(req)) => {
                let mut mis_vec = Vec::new();
                let mis_match = Mismatch {
                    title: format!("{} 不匹配", self.entity_name),
                    reason: match self.with_reason {
                        true => Some(Reason {
                            expected: mock.to_string(),
                            actual: req.to_string(),
                            comparison: self.comparator.name().into(),
                            best_match: false,
                        }),
                        false => None,
                    },
                    diff: self
                        .diff_with
                        .map(|t| diff_str(&mock.to_string(), &req.to_string(), t)),
                };
                mis_vec.push(mis_match);
                mis_vec
            }
        }
    }
}

pub(crate) struct MultiValueMatcher<TK, TV>
where
    TK: Display,
    TV: Display,
{
    pub entity_name: &'static str,
    pub target: Box<dyn MultiValueTarget<TK, TV> + Send + Sync>,
    pub key_comparator: Box<dyn ValueComparator<TK, TK> + Send + Sync>,
    pub value_comparator: Box<dyn ValueComparator<TV, TV> + Send + Sync>,
    pub weight: usize,
}

impl<TK, TV> MultiValueMatcher<TK, TV>
where
    TK: Display,
    TV: Display,
{
    fn find_unmatched<'a>(
        &self,
        req_values: &Vec<(TK, Option<TV>)>,
        mock_values: &'a Vec<(TK, Option<TV>)>,
    ) -> Vec<&'a (TK, Option<TV>)> {
        mock_values
            .into_iter()
            .filter(|(sk, sv)| {
                req_values
                    .iter()
                    .find(|(tk, tv)| {
                        let key_matches = self.key_comparator.matches(sk, &tk);
                        let value_matches = match (sv, tv) {
                            (Some(_), None) => false, // Mock required a value but none was present
                            (Some(sv), Some(tv)) => self.value_comparator.matches(sv, &tv),
                            _ => true,
                        };
                        key_matches && value_matches
                    })
                    .is_none()
            })
            .collect()
    }

    fn find_best_match<'a>(
        &self,
        sk: &TK,
        sv: &Option<TV>,
        req_values: &'a Vec<(TK, Option<TV>)>,
    ) -> Option<(&'a TK, &'a Option<TV>)> {
        if req_values.is_empty() {
            return None;
        }

        let found = req_values
            .into_iter()
            .find(|(k, v)| k.to_string().eq(&sk.to_string()));
        if let Some((fk, fv)) = found {
            return Some((fk, fv));
        }

        req_values
            .into_iter()
            .map(|(tk, tv)| {
                let key_distance = self.key_comparator.distance(&Some(sk), &Some(&tk));
                let value_distance = self.value_comparator.distance(&sv.as_ref(), &tv.as_ref());
                (tk, tv, key_distance + value_distance)
            })
            .min_by(|(_, _, d1), (_, _, d2)| d1.cmp(d2))
            .map(|(k, v, _)| (k, v))
    }
}

impl<TK, TV> Matcher for MultiValueMatcher<TK, TV>
where
    TK: Display,
    TV: Display,
{
    fn matches(&self, req: &HttpMockRequest, mock: &HttpMockRequest) -> bool {
        let req_values = self.target.parse_from_request(req).unwrap_or(Vec::new());
        let mock_values = self.target.parse_from_request(mock).unwrap_or(Vec::new());
        self.find_unmatched(&req_values, &mock_values).is_empty()
    }

    fn distance(&self, req: &HttpMockRequest, mock: &HttpMockRequest) -> usize {
        let req_values = self.target.parse_from_request(req).unwrap_or(Vec::new());
        let mock_values = self.target.parse_from_request(mock).unwrap_or(Vec::new());
        self.find_unmatched(&req_values, &mock_values)
            .into_iter()
            .map(|(k, v)| (k, v, self.find_best_match(&k, v, &req_values)))
            .map(|(k, v, best_match)| match best_match {
                None => {
                    self.key_comparator.distance(&Some(k), &None)
                        + self.value_comparator.distance(&v.as_ref(), &None)
                }
                Some((bmk, bmv)) => {
                    self.key_comparator.distance(&Some(k), &Some(&bmk))
                        + self.value_comparator.distance(&v.as_ref(), &bmv.as_ref())
                }
            })
            .map(|d| d * self.weight)
            .sum()
    }

    fn mismatches(&self, req: &HttpMockRequest, mock: &HttpMockRequest) -> Vec<Mismatch> {
        let req_values = self.target.parse_from_request(req).unwrap_or(Vec::new());
        let mock_values = self.target.parse_from_request(mock).unwrap_or(Vec::new());
        self.find_unmatched(&req_values, &mock_values)
                        .into_iter()
                        .map(|(k, v)| (k, v, self.find_best_match(&k, v, &req_values)))
                        .map(|(k, v, best_match)| Mismatch {
                            title: match v {
                                None => format!("期望 {} 存在'{}'，实际不存在", self.entity_name, &k),
                                Some(v) => format!("期望 {} 中的'{}' 的值为'{}',实际不存在", self.entity_name, &k, v),
                            },
                            reason: best_match.as_ref().map(|(bmk, bmv)| {
                                Reason {
                                    expected: match v {
                                        None => format!("{}", k),
                                        Some(v) => format!("{}={}", k, v),
                                    },
                                    actual: match bmv {
                                        None => format!("{}", bmk),
                                        Some(bmv) => format!("{}={}", bmk, bmv),
                                    },
                                    comparison: format!("key={}, value={}", self.key_comparator.name(), self.value_comparator.name()),
                                    best_match: true,
                                }
                            }),
                            diff: None,
                        })
                        .collect()
    }
}

