use serde::{Deserialize, Serialize};

use super::data::{HttpMockRequest, MockServerHttpResponse};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct MockDefine {
    pub id: u64,
    pub remark: String,
    pub req_script: Option<String>,
    pub resp_script: Option<String>,
    pub req: HttpMockRequest,
    pub resp: MockServerHttpResponse,
    pub relay_url: Option<String>,
}

impl MockDefine {
    pub fn get_url(&self) -> String {
        self.req.path.clone()
    }

    pub fn get_response(&self) -> String {
        self.resp.body.clone().unwrap_or("".to_owned())
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct MockDefineConfig {
    pub remark: String,
    pub req_script: Option<String>,
    pub resp_script: Option<String>,
    pub req: HttpMockRequest,
    pub resp: MockServerHttpResponse,
    pub relay_url: Option<String>,
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_serde() {
        let mock = MockDefine {
            id: 1,
            remark: "".to_string(),
            req_script: None,
            resp_script: None,
            req: HttpMockRequest {
                path: "/a/b".to_string(),
                method: Some("GET".to_string()),
                headers: None,
                query_params: None,
                body: Some("hello world".to_owned()),
                body_schema: None,
            },
            resp: MockServerHttpResponse { status: Some(200), headers: None, body: Some("test".to_owned()), delay: None },
            relay_url: None,
        };
        let js = serde_json::to_string_pretty(&mock).unwrap();
        println!("{}", js);
    }
}
