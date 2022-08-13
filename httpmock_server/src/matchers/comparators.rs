// use assert_json_diff::{assert_json_matches_no_panic, CompareMode, Config};
use regex::Regex;
use serde_json::Value;

use crate::{
    matchers::distance_for,
};

pub trait ValueComparator<S, T> {
    fn matches(&self, mock_value: &S, req_value: &T) -> bool;
    fn name(&self) -> &str;
    fn distance(&self, mock_value: &Option<&S>, req_value: &Option<&T>) -> usize;
}

pub struct  JSONRegexMatchComparator {

}

impl JSONRegexMatchComparator {
    
    pub fn new() -> Self {
        Self {}
    }
}

fn match_string_regex(regex:&str,value:&str) -> bool {
    if let Ok(re) = Regex::new(regex) {
        re.is_match(value)
    } else {
        value == regex
    }
}

pub fn match_json_key(root:String,mock:&Value,req:&Value) -> Option<String>{
    match(mock,req) {
        (Value::String(mock_str),req_v) => {

            match (mock_str.as_str(),req_v) {
                ("*",Value::Array(_)) => None,
                ("*",Value::Object(_)) => None,
                (_,Value::String(req)) => {
                    if match_string_regex(mock_str, req) {
                        None
                    }else{
                        let msg:String = format!("{}的值不匹配，要求：{},实际{}", root, mock_str, req_v);
                        Some(msg)
                    }
                },
                (_,Value::Number(num)) => {
                    if match_string_regex(mock_str, num.to_string().as_str()) {
                        None
                    }else{
                        let msg:String = format!("{}的值不匹配，要求：{},实际{}", root, mock_str, req_v);
                        Some(msg)
                    }
                },
                (_,Value::Bool(b)) => {
                    if match_string_regex(mock_str, b.to_string().as_str()) {
                        None
                    }else{
                        let msg:String = format!("{}的值不匹配，要求：{},实际{}", root, mock_str, req_v);
                        Some(msg)
                    }
                },
                _ => None,
            }

        },
        (Value::Number(mock_num),Value::Number(req_num)) => {
            if mock_num == req_num {
                None
            }else{
                let msg:String = format!("{}的值不匹配，要求：{},实际{}", root, mock_num, req_num);
                Some(msg)
            }
        },
        (Value::Bool(mock_bool),Value::Bool(req_bool)) => {
            if mock_bool == req_bool {
                None
            }else{
                let msg:String = format!("{}的值不匹配，要求：{},实际{}", root, mock_bool, req_bool);
                Some(msg)
            }
        },
        (Value::Null,Value::Null) =>{None},
        (Value::Array(mock_array),Value::Array(req_array)) => {
            if req_array.len() != mock_array.len() {
                let msg:String = format!("{}的值不匹配，要求：{},实际{}", root,mock_array.len(), req_array.len());
                Some(msg)
            } else {
                for (mock,req) in mock_array.iter().zip(req_array.iter()) {
                    if let Some(msg) = match_json_key(root.clone(), mock, req) {
                        return Some(msg);
                    }
                }
                None
            }
        },
        (Value::Object(mock_obj),Value::Object(req_obj)) => {
            mock_obj.iter().find_map(|(mock_key,value)|{
                let mut sub_root = root.clone();
                sub_root.push_str(mock_key);
                if let Some(req_clone) = req_obj.get(mock_key) {
                    match_json_key(sub_root,value, req_clone)
                } else {
                    let msg:String = format!("{}的值不匹配，要求：{},不存在", root, mock_key);
                    Some(msg)
                }
            })
        },
        (_,_) => {None}
    }
}

impl ValueComparator<Value, Value> for JSONRegexMatchComparator {

    fn matches(&self, mock_value: &Value, req_value: &Value) -> bool {
        log::debug!("JsonRegexMatchComparator执行");
        let root = "$".to_string();
        let res = match_json_key(root, mock_value, req_value).is_none();
        log::debug!("JsonRegexMatchComparator执行结果:{}",res);
        res
    }

    fn name(&self) -> &str {
        "json_equal"
    }

    fn distance(&self, mock_value: &Option<&Value>, req_value: &Option<&Value>) -> usize {

        distance_for(mock_value, req_value)
    }
}
// ************************************************************************************************
// JSONExactMatchComparator
// ************************************************************************************************
// pub struct JSONExactMatchComparator {}

// impl JSONExactMatchComparator {
//     pub fn new() -> Self {
//         Self {}
//     }
// }

// impl ValueComparator<Value, Value> for JSONExactMatchComparator {
//     fn matches(&self, mock_value: &Value, req_value: &Value) -> bool {
//         let config = Config::new(CompareMode::Strict);
//         assert_json_matches_no_panic(req_value, mock_value, config).is_ok()
//     }

//     fn name(&self) -> &str {
//         "equals"
//     }

//     fn distance(&self, mock_value: &Option<&Value>, req_value: &Option<&Value>) -> usize {
//         distance_for(mock_value, req_value)
//     }
// }

// // ************************************************************************************************
// // JSONExactMatchComparator
// // ************************************************************************************************
// pub struct JSONContainsMatchComparator {}

// impl JSONContainsMatchComparator {
//     pub fn new() -> Self {
//         Self {}
//     }
// }

// impl ValueComparator<Value, Value> for JSONContainsMatchComparator {
//     fn matches(&self, mock_value: &Value, req_value: &Value) -> bool {
//         let config = Config::new(CompareMode::Inclusive);
//         assert_json_matches_no_panic(req_value, mock_value, config).is_ok()
//     }

//     fn name(&self) -> &str {
//         "contains"
//     }

//     fn distance(&self, mock_value: &Option<&Value>, req_value: &Option<&Value>) -> usize {
//         distance_for(mock_value, req_value)
//     }
// }

// ************************************************************************************************
// StringExactMatchComparator
// ************************************************************************************************
pub struct StringExactMatchComparator {
    case_sensitive: bool,
}

impl StringExactMatchComparator {
    pub fn new(case_sensitive: bool) -> Self {
        Self { case_sensitive }
    }
}

impl ValueComparator<String, String> for StringExactMatchComparator {
    fn matches(&self, mock_value: &String, req_value: &String) -> bool {
        if mock_value == "*" {
            return true;
        }
        match self.case_sensitive {
            true => mock_value.eq(req_value),
            false => mock_value.to_lowercase().eq(&req_value.to_lowercase()),
        }
    }
    fn name(&self) -> &str {
        "equals"
    }
    fn distance(&self, mock_value: &Option<&String>, req_value: &Option<&String>) -> usize {
        distance_for(mock_value, req_value)
    }
}

// ************************************************************************************************
// StringExactMatchComparator
// ************************************************************************************************
pub struct StringContainsMatchComparator {
    case_sensitive: bool,
}

impl StringContainsMatchComparator {
    pub fn new(case_sensitive: bool) -> Self {
        Self { case_sensitive }
    }
}

impl ValueComparator<String, String> for StringContainsMatchComparator {
    fn matches(&self, mock_value: &String, req_value: &String) -> bool {
        if mock_value == "*" {
            return true;
        }
        match self.case_sensitive {
            true => req_value.contains(mock_value),
            false => req_value
                .to_lowercase()
                .contains(&mock_value.to_lowercase()),
        }
    }
    fn name(&self) -> &str {
        "contains"
    }
    fn distance(&self, mock_value: &Option<&String>, req_value: &Option<&String>) -> usize {
        distance_for(mock_value, req_value)
    }
}

// ************************************************************************************************
// StringExactMatchComparator
// ************************************************************************************************
pub struct StringRegexMatchComparator {}

impl StringRegexMatchComparator {
    pub fn new() -> Self {
        Self {}
    }
}

impl ValueComparator<String, String> for StringRegexMatchComparator {
    fn matches(&self, mock_value: &String, req_value: &String) -> bool {
        log::debug!("StringRegexMatchComparator 执行");
        let res = match_string_regex(mock_value, req_value);
        log::debug!("StringRegexMatchComparator 执行结果：{}",res);
        res
    }

    fn name(&self) -> &str {
        "matches regex"
    }

    fn distance(&self, mock_value: &Option<&String>, req_value: &Option<&String>) -> usize {
        distance_for(mock_value, req_value)
    }
}

// ************************************************************************************************
// AnyValueComparator
// ************************************************************************************************
pub struct AnyValueComparator {}

impl AnyValueComparator {
    pub fn new() -> Self {
        Self {}
    }
}

impl<T, U> ValueComparator<T, U> for AnyValueComparator {
    fn matches(&self, _: &T, _: &U) -> bool {
        true
    }
    fn name(&self) -> &str {
        "any"
    }
    fn distance(&self, _: &Option<&T>, _: &Option<&U>) -> usize {
        0
    }
}

// ************************************************************************************************
// FunctionMatchComparator
// ************************************************************************************************
// pub struct FunctionMatchesRequestComparator {}

// impl FunctionMatchesRequestComparator {
//     pub fn new() -> Self {
//         Self {}
//     }
// }

// impl ValueComparator<MockMatcherFunction, HttpMockRequest> for FunctionMatchesRequestComparator {
//     fn matches(&self, mock_value: &MockMatcherFunction, req_value: &HttpMockRequest) -> bool {
//         (*mock_value)(req_value)
//     }

//     fn name(&self) -> &str {
//         "matches"
//     }

//     fn distance(
//         &self,
//         mock_value: &Option<&MockMatcherFunction>,
//         req_value: &Option<&HttpMockRequest>,
//     ) -> usize {
//         let mock_value = match mock_value {
//             None => return 0,
//             Some(v) => v,
//         };
//         let req_value = match req_value {
//             None => return 1,
//             Some(v) => v,
//         };
//         match self.matches(mock_value, req_value) {
//             true => 0,
//             false => 1,
//         }
//     }
// }

#[cfg(test)]
mod test {
    use serde_json::json;

    use crate::matchers::comparators::{
        AnyValueComparator, 
        // JSONContainsMatchComparator, JSONExactMatchComparator,
        StringContainsMatchComparator, StringExactMatchComparator, StringRegexMatchComparator,
        ValueComparator,
    };
    use regex::Regex;

    fn run_test<S, T>(
        comparator: &dyn ValueComparator<S, T>,
        v1: &S,
        v2: &T,
        expected_match: bool,
        expected_distance: usize,
        expected_name: &str,
    ) {
        // Act
        let match_result = comparator.matches(&v1, &v2);
        let distance_result = comparator.distance(&Some(&v1), &Some(&v2));
        let name_result = comparator.name();

        // Assert
        assert_eq!(match_result, expected_match);
        assert_eq!(distance_result, expected_distance);
        assert_eq!(name_result, expected_name);
    }

    #[test]
    fn string_exact_comparator_match() {
        run_test(
            &StringExactMatchComparator::new(true),
            &"test string".to_string(),
            &"test string".to_string(),
            true,
            0, // compute distance even if values match!
            "equals",
        );
    }

    #[test]
    fn string_exact_comparator_no_match() {
        run_test(
            &StringExactMatchComparator::new(true),
            &"test string".to_string(),
            &"not a test string".to_string(),
            false,
            6, // compute distance even if values match!
            "equals",
        );
    }

    #[test]
    fn string_exact_comparator_case_sensitive_match() {
        run_test(
            &StringExactMatchComparator::new(false),
            &"TEST string".to_string(),
            &"test STRING".to_string(),
            true,
            10, // compute distance even if values match!
            "equals",
        );
    }

    #[test]
    fn string_contains_comparator_match() {
        run_test(
            &StringContainsMatchComparator::new(true),
            &"st st".to_string(),
            &"test string".to_string(),
            true,
            6, // compute distance even if values match!
            "contains",
        );
    }

    #[test]
    fn string_contains_comparator_no_match() {
        run_test(
            &StringContainsMatchComparator::new(true),
            &"xxx".to_string(),
            &"yyy".to_string(),
            false,
            3, // compute distance even if values match!
            "contains",
        );
    }

    #[test]
    fn string_contains_comparator_case_sensitive_match() {
        run_test(
            &StringContainsMatchComparator::new(false),
            &"ST st".to_string(),
            &"test STRING".to_string(),
            true,
            9, // compute distance even if values match!
            "contains",
        );
    }

    #[test]
    fn regex_comparator_match() {
        run_test(
            &StringRegexMatchComparator::new(),
            &r"^\d{4}-\d{2}-\d{2}$".to_string(),
            &"2014-01-01".to_string(),
            true,
            16, // compute distance even if values match!
            "matches regex",
        );
    }

    #[test]
    fn regex_match_fn() {
        use super::match_string_regex;

        let r = match_string_regex("^\\d{4}-\\d{2}-\\d{2}$", "2014-01-01");
        println!("{}",r);
    }

    #[test]
    fn regex_json_match() {
        use super::*;
        let mock = json!({ "name" : "P.+","other":"*" });
        let req = json!({ "name" : "Peter", "other1" : { "human" : { "surname" : "Griffin" }}});
        let root = "$$".to_string();
        let r = match_json_key(root,&mock,& req); 
        println!("{:#?}",r);
    }

    #[test]
    fn regex_comparator_no_match() {
        run_test(
            &StringRegexMatchComparator::new(),
            &r"^\d{4}-\d{2}-\d{2}$".to_string(),
            &"xxx".to_string(),
            false,
            19, // compute distance even if values match!
            "matches regex",
        );
    }

    #[test]
    fn any_comparator_match() {
        run_test(
            &AnyValueComparator::new(),
            &"00000000".to_string(),
            &"xxx".to_string(),
            true,
            0, // compute distance even if values match!
            "any",
        );
    }
}
