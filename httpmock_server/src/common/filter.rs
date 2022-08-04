
use std::collections::BTreeMap;
use std::fmt::Display;
use std::sync::{Arc, RwLock};

use async_trait::async_trait;
use chrono::{DateTime, Duration, Local, TimeZone, Utc};
use minijinja::value::Value;
use minijinja::{context, Environment, Source};
use minijinja::{Error, ErrorKind, State};
use serde_json::Value as JValue;

use fake::faker::name::en::Name as NameEn;
use fake::faker::name::zh_cn::Name as NameZh;
use fake::StringFaker;
use fake::{Fake};
// use tera::Tera;
use uuid::Uuid;

use crate::matchers::comparators::{match_json_key, ValueComparator};
use crate::matchers::targets::{MultiValueTarget, ValueTarget};
use crate::matchers::{diff_str, Matcher};

use super::data::{HttpMockRequest, Mismatch, MockServerHttpResponse, Reason, Tokenizer};
use super::mock::MockDefine;

// const ASCII: &str = "0123456789abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ!\"#$%&\'()*+,-./:;<=>?@";
const ASCII_HEX: &str = "0123456789ABCDEF";
const ASCII_NUM: &str = "0123456789";
lazy_static! {
    pub static ref TEMP_ENV: Arc<RwLock<Environment<'static>>> = {
        let mut t_env = Environment::new();
        t_env.add_function("NAME_ZH", fake_name_zh);
        t_env.add_function("NAME_EN", fake_name_en);
        t_env.add_function("NUM", fake_num);
        t_env.add_function("NUM_STR", fake_num_str);
        t_env.add_function("HEX", fake_hex);
        t_env.add_function("STR", fake_str);
        t_env.add_function("EMAIL", fake_email);
        t_env.add_function("USERNAME", fake_username);
        t_env.add_function("IPV4", fake_ip4);
        t_env.add_function("IPV6", fake_ip6);
        t_env.add_function("MAC", fake_mac);
        t_env.add_function("USERAGENT", fake_useragent);
        t_env.add_function("PASSWORD", fake_password);

        t_env.add_function("UUID", fake_uuid);
        t_env.add_function("UUID_SIMPLE", fake_uuid_s);

        t_env.add_function("NOW", fake_now);
        t_env.add_function("DATE_BEFORE", fake_datetime_before);
        t_env.add_function("DATE_AFTER", fake_datetime_after);
        t_env.add_function("DATE", fake_datetime);

        t_env.add_function("BASE64_EN", fake_base64_en);
        t_env.add_function("BASE64_DE", fake_base64_de);
        t_env.add_filter("INT", to_int);
        let source = Source::new();
        t_env.set_source(source);
        Arc::new(RwLock::new(t_env))
    };
}

// pub fn rander_template(template: &str) -> Result<String,Error> {
//     let mut lock = TEMP_ENV.lock().unwrap();
//     let env = lock.borrow_mut();
//     let mut source = env.source().unwrap().clone();
//     source.add_template(REQ_TEMPLATE, template)?;
//     env.set_source(source);
//     let temp = env.get_template(REQ_TEMPLATE).unwrap();
//     let result = temp
//         .render(context!(aaa=>"aaa"))
//         .unwrap_or_else(|_s| template.to_string());
//     Ok(result)
// }
fn to_int(_state: &State, value: String) -> Result<i32, Error> {
    value.parse::<i32>().map_err(|e|Error::new(ErrorKind::InvalidArguments, format!("{}cant turn to int", value)))
}


fn fake_name_zh(_state: &State) -> Result<String, Error> {
    let name = NameZh().fake();
    Ok(name)
}

fn fake_name_en(_state: &State) -> Result<String, Error> {
    let name = NameEn().fake();
    Ok(name)
}

fn fake_hex(_state: &State, low: usize, mut high: usize) -> Result<String, Error> {
    if high <= low {
        high = low + 1;
    }
    let f = StringFaker::with(Vec::from(ASCII_HEX), low..high);
    let a: String = f.fake();
    Ok(a)
}

fn fake_str(_state: &State, low: usize, mut high: usize) -> Result<String, Error> {
    if high <= low {
        high = low + 1;
    }
    let a: String = (low..high).fake();
    Ok(a)
}

fn fake_num_str(_state: &State, low: usize, mut high: usize) -> Result<String, Error> {
    if high <= low {
        high = low + 1;
    }
    let f = StringFaker::with(Vec::from(ASCII_NUM), low..high);
    let a: String = f.fake();
    Ok(a)
}

fn fake_num(_state: &State, low: usize, mut high: usize) -> Result<String, Error> {
    if high <= low {
        high = low + 1;
    }
    let a: usize = (low..high).fake();
    Ok(a.to_string())
}

fn fake_email(_state: &State) -> Result<String, Error> {
    let f: String = fake::faker::internet::en::FreeEmail().fake();
    Ok(f)
}

fn fake_username(_state: &State) -> Result<String, Error> {
    let f: String = fake::faker::internet::en::Username().fake();
    Ok(f)
}
fn fake_ip4(_state: &State) -> Result<String, Error> {
    let f: String = fake::faker::internet::en::IPv4().fake();
    Ok(f)
}
fn fake_ip6(_state: &State) -> Result<String, Error> {
    let f: String = fake::faker::internet::en::IPv6().fake();
    Ok(f)
}
fn fake_useragent(_state: &State) -> Result<String, Error> {
    let f: String = fake::faker::internet::en::UserAgent().fake();
    Ok(f)
}
fn fake_mac(_state: &State) -> Result<String, Error> {
    let f: String = fake::faker::internet::en::MACAddress().fake();
    Ok(f)
}

fn fake_password(_state: &State, low: usize, mut high: usize) -> Result<String, Error> {
    if high <= low {
        high = low + 1;
    }
    let f: String = fake::faker::internet::en::Password(low..high).fake();
    Ok(f)
}

fn fake_uuid(_state: &State) -> Result<String, Error> {
    let f = Uuid::new_v4();
    Ok(f.hyphenated().to_string())
}

fn fake_uuid_s(_state: &State) -> Result<String, Error> {
    let f = Uuid::new_v4();
    Ok(f.simple().to_string())
}

fn fake_base64_en(_state: &State, fmt: String) -> Result<String, Error> {
    let f = base64::encode(fmt);
    Ok(f)
}

fn fake_base64_de(_state: &State, fmt: String) -> Result<String, Error> {
    let df = base64::decode(fmt.clone());
    if let Ok(bytes) = df {
        if let Ok(f) = String::from_utf8(bytes) {
            return Ok(f);
        }
    }
    Ok(fmt)
}
fn fake_now(_state: &State, fmt: String) -> Result<String, Error> {
    let local = Local::now();
    let fmt_data = local.format(fmt.as_str());
    Ok(fmt_data.to_string())
}

fn fake_datetime(_state: &State, fmt: String) -> Result<String, Error> {
    let local = Utc::now();
    let ten_years = Duration::days(3660);
    let start = local.checked_sub_signed(ten_years).unwrap();
    let end = local.checked_add_signed(ten_years).unwrap();
    fake_date_between(_state, fmt.as_str(), start, end)
}

fn fake_datetime_after(_state: &State, fmt: String, date: String) -> Result<String, Error> {
    let local = Utc::now();
    let ten_years = Duration::days(3660);
    let end = local.checked_add_signed(ten_years).unwrap();
    if let Ok(start) = Utc.datetime_from_str(date.as_str(), "%Y-%m-%dT%H:%M:%S") {
        fake_date_between(_state, fmt.as_str(), start, end)
    } else {
        Err(Error::new(
            ErrorKind::SyntaxError,
            format!("{}与{}格式不匹配", date, "%Y-%m-%dT%H:%M:%S"),
        ))
    }
}

fn fake_datetime_before(_state: &State, fmt: String, date: String) -> Result<String, Error> {
    let local = Utc::now();
    let ten_years = Duration::days(3660);
    let start = local.checked_sub_signed(ten_years).unwrap();
    if let Ok(end) = Utc.datetime_from_str(date.as_str(), "%Y-%m-%dT%H:%M:%S") {
        fake_date_between(_state, fmt.as_str(), start, end)
    } else {
        Err(Error::new(
            ErrorKind::SyntaxError,
            format!("{}与{}格式不匹配", date, "%Y-%m-%dT%H:%M:%S"),
        ))
    }
}

fn fake_date_between(
    _state: &State,
    fmt: &str,
    start: DateTime<Utc>,
    end: DateTime<Utc>,
) -> Result<String, Error> {
    let f: String = fake::faker::chrono::zh_cn::DateTimeBetween(start, end).fake();
    let d = f.parse::<DateTime<Utc>>().unwrap();
    Ok(d.format(fmt).to_string())
}

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
        let mock_resp_id = req.mock_define.id.to_string();
        let mut ret_mock_resp:Option<MockServerHttpResponse> = None;
        let start = std::time::SystemTime::now();

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

        if let Ok(env) = TEMP_ENV.read() {
            //处理body模板
            let body_temp_key = format!("{}_body", mock_resp_id.as_str());
            if let Ok(body_temp) = env.get_template(body_temp_key.as_str()) {
                let mut mock_resp = req.mock_define.resp.clone();
                let rendered =
                    match body_temp.render(temp_ctx.clone()) {
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
                        let header_key = format!("{}_header_{}", mock_resp_id.as_str(), key.as_str());
                        if let Ok(header_tmp) = env.get_template(header_key.as_str()) {
                            let rander_header = match header_tmp.render(temp_ctx.clone()) {
                                Ok(s) => s,
                                Err(e) => e.to_string(),
                            };
                            (key,rander_header)
                        } else {
                            (key,val)
                        }
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
        let client = reqwest::Client::new();
        let mut req_clone = req.req.clone();
        if let Some(redict_url) = req.mock_define.relay_url.clone() {
            req_clone.path = redict_url;
            let real_req = req_clone.into(); 
            let resp = client.execute(real_req);
            match resp.await {
                Ok(r) => {
                    let status = Some(r.status().as_u16());
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
        // let mock_body = mock.body.clone().unwrap();
        // dbg!(String::from_utf8(mock_body).unwrap());
        // dbg!(&mock_value);
        match (mock_value, req_value) {
            (None, _) => return true,
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
    T: Display,
{
    fn matches(&self, req: &HttpMockRequest, mock: &HttpMockRequest) -> bool {
        let req_value = self.target.parse_from_request(req);
        let mock_value = self.target.parse_from_request(mock);
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

