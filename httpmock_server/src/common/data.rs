use poem::http::{HeaderMap, HeaderValue};
use poem::web::IntoResponse;
use poem::{FromRequest, Request, RequestBody, Result};
use regex::Regex;
use reqwest::header::HeaderName;
use reqwest::{Method, Url, StatusCode, Body};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fmt;
use std::fmt::Debug;
use std::str::FromStr;
use std::time::Duration;
use std::{cmp::Ordering, collections::HashMap};
/// A general abstraction of an HTTP request of `httpmock`.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct HttpMockRequest {
    pub path: String,
    pub method: Option<String>,
    pub headers: Option<HashMap<String, String>>,
    pub query_params: Option<HashMap<String, String>>,
    pub body: Option<Vec<u8>>,
}

#[poem::async_trait]
impl<'a> FromRequest<'a> for HttpMockRequest {
    async fn from_request(req: &'a Request, body: &mut RequestBody) -> Result<Self> {
        let mut mock = HttpMockRequest::new(req.uri().path().to_string());
        let mut headers: HashMap<String, String> = HashMap::new();
        let method = req.method().to_string();
        mock.method(method);
        for (hn, hv) in req.headers().iter() {
            let value = hv.to_str().unwrap().to_string();
            headers.insert(hn.to_string(), value);
        }
        mock.headers(headers);
        let mut querys = HashMap::<String, String>::new();
        if let Some(query_str) = req.uri().query() {
            query_str.split("&").into_iter().for_each(|kv| {
                let kv_split: Vec<&str> = kv.split("=").collect();
                let key = kv_split[0];
                let value = kv_split[1];
                querys.insert(key.into(), value.into());
            });
        }
        mock.query_params(querys);
        if let Ok(body) = body.take() {
            let body = body.into_vec().await.unwrap();
            mock.body(body);
        }
        Ok(mock)
    }
}

impl Into<reqwest::Request> for HttpMockRequest {
    fn into(self) -> reqwest::Request {
        let HttpMockRequest {
            path,
            method,
            headers,
            query_params,
            body,
        } = self;
        let method = method.as_ref().unwrap_or(&"GET".to_string()).to_uppercase();
        let req_method = Method::from_bytes(method.as_bytes()).unwrap_or_default();
        let url;
        if let Some(query) = query_params {
            if query.is_empty() {
                url = Url::parse(path.as_str()).unwrap();
            } else {
                url = Url::parse_with_params(path.as_str(), query.iter()).unwrap();
            }
        } else {
            url = Url::parse(path.as_str()).unwrap();
        }
        let host = url.host().unwrap().to_string();
        let mut blocking_req = reqwest::Request::new(req_method, url);
        if let Some(headers) = headers {
            let req_headers = blocking_req.headers_mut();
            let mut header_map = HeaderMap::new();
            for (h, n) in headers.iter() {
                if let Ok(hdr) = HeaderName::from_str(h) {
                    let mut value = n.as_str();
                    if hdr == "host" {
                        value = host.as_str();
                    }
                    let value = HeaderValue::from_str(value).unwrap();
                    header_map.append(hdr, value);
                }
            }
            *req_headers = header_map;
        }

        if let Some(body) = body {
            let req_body = blocking_req.body_mut();
            let body = Body::from(body.clone());
            *req_body = Some(body);
        }

        blocking_req
    }
}

impl HttpMockRequest {
    pub fn new(path: String) -> Self {
        Self {
            path,
            method: None,
            headers: None,
            query_params: None,
            body: None,
        }
    }
    pub fn method(&mut self, method: String) {
        self.method = Some(method);
    }

    pub fn headers(&mut self, arg: HashMap<String, String>) {
        self.headers = Some(arg);
    }

    pub fn query_params(&mut self, arg: HashMap<String, String>) {
        self.query_params = Some(arg);
    }

    pub fn body(&mut self, arg: Vec<u8>) {
        self.body = Some(arg);
    }
}

/// A general abstraction of an HTTP response for all handlers.
#[derive(Serialize, Deserialize, Clone)]
pub struct MockServerHttpResponse {
    pub status: Option<u16>,
    pub headers: Option<Vec<(String, String)>>,
    #[serde(default, with = "opt_vector_serde_base64")]
    pub body: Option<Vec<u8>>,
    pub delay: Option<Duration>,
}

impl MockServerHttpResponse {
    pub fn new() -> Self {
        Self {
            status: None,
            headers: None,
            body: None,
            delay: None,
        }
    }
}

impl IntoResponse for MockServerHttpResponse {
    fn into_response(self) -> poem::Response {
        let MockServerHttpResponse {
            status,
            headers,
            body,
            ..
        } = self;
        let builder = poem::Response::builder();
        let status = StatusCode::from_u16(status.unwrap_or(200))
                                    .unwrap_or(StatusCode::OK);
        let builder = builder.status(status);
        let mut resp = builder.body(body.unwrap_or_default());

        let mut header_map = HeaderMap::new();
        if let Some(headers_vec) = headers {
            for (key,val) in headers_vec {
                if let Ok(hdr) = HeaderName::from_str(key.as_str()) {
                    let value = val.as_str();
                    let value = HeaderValue::from_str(value).unwrap();
                    header_map.append(hdr, value);
                }
            }
        } 
        *resp.headers_mut() = header_map;
        resp
    }
}

impl Default for MockServerHttpResponse {
    fn default() -> Self {
        Self::new()
    }
}

/// Serializes and deserializes the response body to/from a Base64 string.
mod opt_vector_serde_base64 {
    use serde::{Deserialize, Deserializer, Serializer};

    // See the following references:
    // https://github.com/serde-rs/serde/blob/master/serde/src/ser/impls.rs#L99
    // https://github.com/serde-rs/serde/issues/661
    pub fn serialize<T, S>(bytes: &Option<T>, serializer: S) -> Result<S::Ok, S::Error>
    where
        T: AsRef<[u8]>,
        S: Serializer,
    {
        match bytes {
            Some(ref value) => serializer.serialize_bytes(base64::encode(value).as_bytes()),
            None => serializer.serialize_none(),
        }
    }

    // See the following references:
    // https://github.com/serde-rs/serde/issues/1444
    pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<Vec<u8>>, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct Wrapper(#[serde(deserialize_with = "from_base64")] Vec<u8>);

        let v = Option::deserialize(deserializer)?;
        Ok(v.map(|Wrapper(a)| a))
    }

    fn from_base64<'de, D>(deserializer: D) -> Result<Vec<u8>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let vec = Vec::deserialize(deserializer)?;
        base64::decode(vec).map_err(serde::de::Error::custom)
    }
}

/// Prints the response body as UTF8 string
impl fmt::Debug for MockServerHttpResponse {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("MockServerHttpResponse")
            .field("status", &self.status)
            .field("headers", &self.headers)
            .field(
                "body",
                &self
                    .body
                    .as_ref()
                    .map(|x| String::from_utf8_lossy(x.as_ref()).to_string()),
            )
            .field("delay", &self.delay)
            .finish()
    }
}

// /// A general abstraction of an HTTP request for all handlers.
// #[derive(Serialize, Deserialize, Clone, Debug)]
// pub struct Pattern {
//     #[serde(with = "serde_regex")]
//     pub regex: Regex,
// }

// impl Pattern {
//     pub fn from_regex(regex: Regex) -> Pattern {
//         Pattern { regex }
//     }
// }

// impl Ord for Pattern {
//     fn cmp(&self, other: &Self) -> Ordering {
//         self.regex.as_str().cmp(other.regex.as_str())
//     }
// }

// impl PartialOrd for Pattern {
//     fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
//         Some(self.cmp(other))
//     }
// }

// impl PartialEq for Pattern {
//     fn eq(&self, other: &Self) -> bool {
//         self.regex.as_str() == other.regex.as_str()
//     }
// }

// impl Eq for Pattern {}

// pub type MockMatcherFunction = fn(&HttpMockRequest) -> bool;

// /// A general abstraction of an HTTP request for all handlers.
// #[derive(Serialize, Deserialize, Clone)]
// pub struct RequestRequirements {
//     pub path: Option<String>,
//     pub path_contains: Option<Vec<String>>,
//     pub path_matches: Option<Vec<Pattern>>,
//     pub method: Option<String>,
//     pub headers: Option<Vec<(String, String)>>,
//     pub header_exists: Option<Vec<String>>,
//     pub cookies: Option<Vec<(String, String)>>,
//     pub cookie_exists: Option<Vec<String>>,
//     pub body: Option<String>,
//     pub json_body: Option<Value>,
//     pub json_body_includes: Option<Vec<Value>>,
//     pub body_contains: Option<Vec<String>>,
//     pub body_matches: Option<Vec<Pattern>>,
//     pub query_param_exists: Option<Vec<String>>,
//     pub query_param: Option<Vec<(String, String)>>,
//     pub x_www_form_urlencoded_key_exists: Option<Vec<String>>,
//     pub x_www_form_urlencoded: Option<Vec<(String, String)>>,

//     #[serde(skip_serializing, skip_deserializing)]
//     pub matchers: Option<Vec<MockMatcherFunction>>,
// }

// impl Default for RequestRequirements {
//     fn default() -> Self {
//         Self::new()
//     }
// }

// impl RequestRequirements {
//     pub fn new() -> Self {
//         Self {
//             path: None,
//             path_contains: None,
//             path_matches: None,
//             method: None,
//             headers: None,
//             header_exists: None,
//             cookies: None,
//             cookie_exists: None,
//             body: None,
//             json_body: None,
//             json_body_includes: None,
//             body_contains: None,
//             body_matches: None,
//             query_param_exists: None,
//             query_param: None,
//             x_www_form_urlencoded: None,
//             x_www_form_urlencoded_key_exists: None,
//             matchers: None,
//         }
//     }

//     pub fn with_path(mut self, arg: String) -> Self {
//         self.path = Some(arg);
//         self
//     }

//     pub fn with_method(mut self, arg: String) -> Self {
//         self.method = Some(arg);
//         self
//     }

//     pub fn with_body(mut self, arg: String) -> Self {
//         self.body = Some(arg);
//         self
//     }

//     pub fn with_json_body(mut self, arg: Value) -> Self {
//         self.json_body = Some(arg);
//         self
//     }

//     pub fn with_path_contains(mut self, arg: Vec<String>) -> Self {
//         self.path_contains = Some(arg);
//         self
//     }

//     pub fn with_path_matches(mut self, arg: Vec<Pattern>) -> Self {
//         self.path_matches = Some(arg);
//         self
//     }

//     pub fn with_headers(mut self, arg: Vec<(String, String)>) -> Self {
//         self.headers = Some(arg);
//         self
//     }

//     pub fn with_header_exists(mut self, arg: Vec<String>) -> Self {
//         self.header_exists = Some(arg);
//         self
//     }

//     pub fn with_cookies(mut self, arg: Vec<(String, String)>) -> Self {
//         self.cookies = Some(arg);
//         self
//     }

//     pub fn with_cookie_exists(mut self, arg: Vec<String>) -> Self {
//         self.cookie_exists = Some(arg);
//         self
//     }

//     pub fn with_json_body_includes(mut self, arg: Vec<Value>) -> Self {
//         self.json_body_includes = Some(arg);
//         self
//     }

//     pub fn with_body_contains(mut self, arg: Vec<String>) -> Self {
//         self.body_contains = Some(arg);
//         self
//     }

//     pub fn with_body_matches(mut self, arg: Vec<Pattern>) -> Self {
//         self.body_matches = Some(arg);
//         self
//     }

//     pub fn with_query_param_exists(mut self, arg: Vec<String>) -> Self {
//         self.query_param_exists = Some(arg);
//         self
//     }

//     pub fn with_query_param(mut self, arg: Vec<(String, String)>) -> Self {
//         self.query_param = Some(arg);
//         self
//     }
// }

// /// A Request that is made to set a new mock.
// #[derive(Serialize, Deserialize, Clone)]
// pub struct MockDefinition {
//     pub request: RequestRequirements,
//     pub response: MockServerHttpResponse,
// }

// impl MockDefinition {
//     pub fn new(req: RequestRequirements, mock: MockServerHttpResponse) -> Self {
//         Self {
//             request: req,
//             response: mock,
//         }
//     }
// }

// #[derive(Serialize, Deserialize)]
// pub struct MockRef {
//     pub mock_id: usize,
// }

// impl MockRef {
//     pub fn new(mock_id: usize) -> Self {
//         Self { mock_id }
//     }
// }

// #[derive(Serialize, Deserialize, Clone)]
// pub struct ActiveMock {
//     pub id: usize,
//     pub call_counter: usize,
//     pub definition: MockDefinition,
//     pub is_static: bool,
// }

// impl ActiveMock {
//     pub fn new(id: usize, mock_definition: MockDefinition, is_static: bool) -> Self {
//         ActiveMock {
//             id,
//             definition: mock_definition,
//             call_counter: 0,
//             is_static,
//         }
//     }
// }

// #[derive(Serialize, Deserialize)]
// pub struct ClosestMatch {
//     pub request: HttpMockRequest,
//     pub request_index: usize,
//     pub mismatches: Vec<Mismatch>,
// }

// #[derive(Serialize, Deserialize)]
// pub struct ErrorResponse {
//     pub message: String,
// }

// impl ErrorResponse {
//     pub fn new<T>(message: &T) -> ErrorResponse
//     where
//         T: ToString,
//     {
//         ErrorResponse {
//             message: message.to_string(),
//         }
//     }
// }

// *************************************************************************************************
// Diff and Change correspond to difference::Changeset and Difference structs. They are duplicated
// here only for the reason to make them serializable/deserializable using serde.
// *************************************************************************************************
#[derive(PartialEq, Debug, Serialize, Deserialize)]
pub enum Diff {
    Same(String),
    Add(String),
    Rem(String),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DiffResult {
    pub differences: Vec<Diff>,
    pub distance: f32,
    pub tokenizer: Tokenizer,
}

#[derive(PartialEq, Debug, Serialize, Deserialize, Clone, Copy)]
pub enum Tokenizer {
    Line,
    Word,
    Character,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Reason {
    pub expected: String,
    pub actual: String,
    pub comparison: String,
    pub best_match: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Mismatch {
    pub title: String,
    pub reason: Option<Reason>,
    pub diff: Option<DiffResult>,
}

// #[cfg(test)]
// mod test {
//     use std::collections::BTreeMap;

//     use regex::Regex;
//     use serde_json::json;

//     use crate::common::data::{Pattern, RequestRequirements};

//     /// This test makes sure that adding the matching rules to a mock fills the struct as expected.
//     #[test]
//     fn fill_mock_requirements() {
//         // Arrange
//         let with_path = "with_path";
//         let with_path_contains = vec!["with_path_contains".into()];
//         let with_path_matches = vec![Pattern::from_regex(
//             Regex::new(r#"with_path_matches"#).unwrap(),
//         )];
//         let mut with_headers = Vec::new();
//         with_headers.push(("test".into(), "value".into()));
//         let with_method = "GET";
//         let with_body = "with_body";
//         let with_body_contains = vec!["body_contains".into()];
//         let with_body_matches = vec![Pattern::from_regex(
//             Regex::new(r#"with_body_matches"#).unwrap(),
//         )];
//         let with_json_body = json!(12.5);
//         let with_json_body_includes = vec![json!(12.5)];
//         let with_query_param_exists = vec!["with_query_param_exists".into()];
//         let mut with_query_param = Vec::new();
//         with_query_param.push(("with_query_param".into(), "value".into()));
//         let with_header_exists = vec!["with_header_exists".into()];

//         // Act
//         let rr = RequestRequirements::new()
//             .with_path(with_path.clone().into())
//             .with_path_contains(with_path_contains.clone())
//             .with_path_matches(with_path_matches.clone())
//             .with_headers(with_headers.clone())
//             .with_method(with_method.clone().into())
//             .with_body(with_body.clone().into())
//             .with_body_contains(with_body_contains.clone())
//             .with_body_matches(with_body_matches.clone())
//             .with_json_body(with_json_body.clone())
//             .with_json_body_includes(with_json_body_includes.clone())
//             .with_query_param_exists(with_query_param_exists.clone())
//             .with_query_param(with_query_param.clone())
//             .with_header_exists(with_header_exists.clone());

//         // Assert
//         assert_eq!(rr.path.as_ref().unwrap(), with_path.clone());
//         assert_eq!(
//             rr.path_contains.as_ref().unwrap(),
//             &with_path_contains.clone()
//         );
//         assert_eq!(
//             rr.path_matches.as_ref().unwrap(),
//             &with_path_matches.clone()
//         );
//         assert_eq!(rr.headers.as_ref().unwrap(), &with_headers.clone());
//         assert_eq!(rr.body.as_ref().unwrap(), with_body.clone());
//         assert_eq!(
//             rr.body_contains.as_ref().unwrap(),
//             &with_body_contains.clone()
//         );
//         assert_eq!(
//             rr.body_matches.as_ref().unwrap(),
//             &with_body_matches.clone()
//         );
//         assert_eq!(rr.json_body.as_ref().unwrap(), &with_json_body.clone());
//         assert_eq!(
//             rr.json_body_includes.as_ref().unwrap(),
//             &with_json_body_includes.clone()
//         );
//         assert_eq!(
//             rr.query_param_exists.as_ref().unwrap(),
//             &with_query_param_exists.clone()
//         );
//         assert_eq!(rr.query_param.as_ref().unwrap(), &with_query_param.clone());
//         assert_eq!(
//             rr.header_exists.as_ref().unwrap(),
//             &with_header_exists.clone()
//         );
//     }
// }
