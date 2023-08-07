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
        if let Some(body) = self.resp.body.clone() {
            String::from_utf8(body).unwrap()
        } else {
            "".to_string()
        }
    }
}
