use std::collections::HashMap;

use serde_json::Value;

use crate::common::data::HttpMockRequest;

pub(crate) trait ValueTarget<T> {
    fn parse_from_request(&self, req: &HttpMockRequest) -> Option<T>;
}

pub(crate) trait ValueRefTarget<T> {
    fn parse_from_request<'a>(&self, req: &'a HttpMockRequest) -> Option<&'a T>;
}

pub(crate) trait MultiValueTarget<T, U> {
    fn parse_from_request(&self, req: &HttpMockRequest) -> Option<Vec<(T, Option<U>)>>;
}

pub(crate) trait KeyValueTarget<K, V> {
    fn parse_from_request<'a>(&self, req: &'a HttpMockRequest) -> Option<&'a HashMap<K, Option<V>>>;
}
// *************************************************************************************
// StringBodyTarget
// *************************************************************************************
pub(crate) struct StringBodyTarget {}

impl StringBodyTarget {
    pub fn new() -> Self {
        Self {}
    }
}

impl ValueTarget<String> for StringBodyTarget {
    fn parse_from_request(&self, req: &HttpMockRequest) -> Option<String> {
        req.body
            .as_ref()
            .map(|b| String::from_utf8_lossy(b).to_string()) // FIXME: Avoid copying here. Create a "ValueRefTarget".
    }
}

pub(crate) struct JSONSchemaTarget {}

impl JSONSchemaTarget {
    pub fn new() -> Self {
        Self {}
    }
}
impl ValueTarget<Value> for JSONSchemaTarget {
    fn parse_from_request(&self, req: &HttpMockRequest) -> Option<Value> {
        let body = req.body_schema.as_ref();
        if body.is_none() || body.unwrap().is_empty(){
            return None;
        }
        let body_vec = body.unwrap();
        if let Ok(body_str) = String::from_utf8(body_vec.to_owned()) {
            // let re = regex::Regex::new("\\{#.+?#\\}").unwrap();
            // let dealed_body = re.replace_all(&body_str, "");
            match serde_json::from_str(body_str.as_ref()) {
                Ok(v) => {return Some(v)},
                Err(e) => {
                    log::trace!("paser json error:{}",e);
                    return None;
                },
            }
        }
        match serde_json::from_slice(body.unwrap()) {
            Err(e) => {
                log::trace!("Cannot parse json value: {}", e);
                None
            }
            Ok(v) => Some(v),
        }
    }
}
// *************************************************************************************
// JSONBodyTarget
// *************************************************************************************
pub(crate) struct JSONBodyTarget {}

impl JSONBodyTarget {
    pub fn new() -> Self {
        Self {}
    }
}

impl ValueTarget<Value> for JSONBodyTarget {
    fn parse_from_request(&self, req: &HttpMockRequest) -> Option<Value> {
        let body = req.body.as_ref();
        if body.is_none() || body.unwrap().is_empty(){
            return None;
        }
        let body_vec = body.unwrap();
        if let Ok(body_str) = String::from_utf8(body_vec.to_owned()) {
            // let re = regex::Regex::new("\\{#.+?#\\}").unwrap();
            // let dealed_body = re.replace_all(&body_str, "");
            match serde_json::from_str(body_str.as_ref()) {
                Ok(v) => {return Some(v)},
                Err(e) => {
                    log::trace!("paser json error:{}",e);
                    return None;
                },
            }
        }
        match serde_json::from_slice(body.unwrap()) {
            Err(e) => {
                log::trace!("Cannot parse json value: {}", e);
                None
            }
            Ok(v) => Some(v),
        }
    }
}

// *************************************************************************************
// CookieTarget
// *************************************************************************************
// #[cfg(feature = "cookies")]
// pub(crate) struct CookieTarget {}

// #[cfg(feature = "cookies")]
// impl CookieTarget {
//     pub fn new() -> Self {
//         Self {}
//     }
// }

// #[cfg(feature = "cookies")]
// impl MultiValueTarget<String, String> for CookieTarget {
//     fn parse_from_request(&self, req: &HttpMockRequest) -> Option<Vec<(String, Option<String>)>> {
//         let req_cookies = match matchers::parse_cookies(req) {
//             Ok(v) => v,
//             Err(err) => {
//                 log::info!(
//                 "Cannot parse cookies. Cookie matching will not work for this request. Error: {}",
//                 err
//             );
//                 return None;
//             }
//         };

//         Some(req_cookies.into_iter().map(|(k, v)| (k, Some(v))).collect())
//     }
// }

// *************************************************************************************
// HeaderTarget
// *************************************************************************************
pub(crate) struct HeaderTarget {}

impl HeaderTarget {
    pub fn new() -> Self {
        Self {}
    }
}

impl MultiValueTarget<String, String> for HeaderTarget {
    fn parse_from_request(&self, req: &HttpMockRequest) -> Option<Vec<(String, Option<String>)>> {
        req.headers.as_ref().map(|headers| {
            headers
                .into_iter()
                .map(|(k, v)| (k.to_string(), Some(v.to_string())))
                .collect()
        })
    }
}

// *************************************************************************************
// HeaderTarget
// *************************************************************************************
pub(crate) struct QueryParameterTarget {}

impl QueryParameterTarget {
    pub fn new() -> Self {
        Self {}
    }
}

impl MultiValueTarget<String, String> for QueryParameterTarget {
    fn parse_from_request(&self, req: &HttpMockRequest) -> Option<Vec<(String, Option<String>)>> {
        req.query_params.as_ref().map(|headers| {
            headers
                .into_iter()
                .map(|(k, v)| (k.to_string(), Some(v.to_string())))
                .collect()
        })
    }
}

// *************************************************************************************
// PathTarget
// *************************************************************************************
// pub(crate) struct PathTarget {}

// impl PathTarget {
//     pub fn new() -> Self {
//         Self {}
//     }
// }

// impl ValueTarget<String> for PathTarget {
//     fn parse_from_request(&self, req: &HttpMockRequest) -> Option<String> {
//         Some(req.path.to_string()) // FIXME: Avoid copying here. Create a "ValueRefTarget".
//     }
// }

// *************************************************************************************
// MethodTarget
// *************************************************************************************
pub(crate) struct MethodTarget {}

impl MethodTarget {
    pub fn new() -> Self {
        Self {}
    }
}

impl ValueTarget<String> for MethodTarget {
    fn parse_from_request(&self, req: &HttpMockRequest) -> Option<String> {
        // match req.method.clone() {
        //     Some(m) if m == "NONE"=> Some("*".into()),
        //     _ => req.method.clone(),
        // }
        req.method.clone()
    }
}

// *************************************************************************************
// FullRequestTarget
// *************************************************************************************
// pub(crate) struct FullRequestTarget {}

// impl FullRequestTarget {
//     pub fn new() -> Self {
//         Self {}
//     }
// }

// impl ValueRefTarget<HttpMockRequest> for FullRequestTarget {
//     fn parse_from_request<'a>(&self, req: &'a HttpMockRequest) -> Option<&'a HttpMockRequest> {
//         Some(req)
//     }
// }

// // *************************************************************************************
// // XWWWFormUrlEncodedBodyTarget
// // *************************************************************************************
// pub(crate) struct XWWWFormUrlEncodedBodyTarget {}

// impl XWWWFormUrlEncodedBodyTarget {
//     pub fn new() -> Self {
//         Self {}
//     }
// }

// impl MultiValueTarget<String, String> for XWWWFormUrlEncodedBodyTarget {
//     fn parse_from_request(&self, req: &HttpMockRequest) -> Option<Vec<(String, Option<String>)>> {
//         req.body.as_ref().map(|body| {
//             form_urlencoded::parse(body)
//                 .into_owned()
//                 .map(|(k, v)| (k, Some(v)))
//                 .collect()
//         })
//     }
// }
