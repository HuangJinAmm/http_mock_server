use std::{collections::HashMap, fmt, time::Duration};

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct MockDefine {
    pub id: u64,
    pub req: HttpMockRequest,
    pub resp: MockServerHttpResponse,
    pub relay_url: Option<String>,
}

impl MockDefine {
    pub fn get_url(&self) -> String {
        let mut path = self.req.path.clone();
        if let Some(query_params) = self.req.query_params.clone() {
            let mut q_params =
                query_params
                    .into_iter()
                    .fold(String::from("?"), |mut s, (key, value)| {
                        s.push_str(format!("{}={}&", key, value).as_str());
                        s
                    });
            q_params.pop(); //去掉最后一个&
            path.push_str(q_params.as_str());
        }
        path
    }

    pub fn get_response(&self) -> String {
        if let Some(body) = self.resp.body.clone() {
            String::from_utf8(body).unwrap()
        } else {
            "".to_string()
        }
    }
}
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct HttpMockRequest {
    pub path: String,
    pub method: Option<String>,
    pub headers: Option<HashMap<String, String>>,
    pub query_params: Option<HashMap<String, String>>,
    pub body: Option<Vec<u8>>,
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

impl Default for MockServerHttpResponse {
    fn default() -> Self {
        Self::new()
    }
}
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
