
use std::sync::{Arc, Mutex};

use chrono::{Local, DateTime, Utc, Duration, TimeZone};
use minijinja::{Environment, Source};
use minijinja::{Error, ErrorKind, State};

use fake::faker::name::en::Name as NameEn;
use fake::faker::name::zh_cn::Name as NameZh;
use fake::StringFaker;
use fake::Fake;
use uuid::Uuid;


// const REQ_TEMPLATE: &str = "req_template";
// const ASCII: &str = "0123456789abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ!\"#$%&\'()*+,-./:;<=>?@";
const ASCII_HEX: &str = "0123456789ABCDEF";
const ASCII_NUM: &str = "0123456789";
lazy_static! {
    static ref TEMP_ENV: Arc<Mutex<Environment<'static>>> = {
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
        t_env.add_function("DATE",fake_datetime);


        t_env.add_function("BASE64_EN",fake_base64_en);
        t_env.add_function("BASE64_DE",fake_base64_de);
        let source = Source::new();
        t_env.set_source(source);
        Arc::new(Mutex::new(t_env))
    };
}

// weak password generator

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

fn fake_name_zh(_state: &State<'_, '_>) -> Result<String, Error> {
    let name = NameZh().fake();
    Ok(name)
}

fn fake_name_en(_state: &State<'_, '_>) -> Result<String, Error> {
    let name = NameEn().fake();
    Ok(name)
}

fn fake_hex(_state: &State<'_, '_>, low: usize, mut high: usize) -> Result<String, Error> {
    if high <= low {
        high = low + 1;
    }
    let f = StringFaker::with(Vec::from(ASCII_HEX), low..high);
    let a: String = f.fake();
    Ok(a)
}

fn fake_str(_state: &State<'_, '_>, low: usize, mut high: usize) -> Result<String, Error> {
    if high <= low {
        high = low + 1;
    }
    let a: String = (low..high).fake();
    Ok(a)
}



fn fake_num_str(_state: &State<'_, '_>, low: usize, mut high: usize) -> Result<String, Error> {
    if high <= low {
        high = low + 1;
    }
    let f = StringFaker::with(Vec::from(ASCII_NUM), low..high);
    let a: String = f.fake();
    Ok(a)
}

fn fake_num(_state: &State<'_, '_>, low: usize, mut high: usize) -> Result<String, Error> {
    if high <= low {
        high = low + 1;
    }
    let a: usize = (low..high).fake();
    Ok(a.to_string())
}

fn fake_email(_state: &State<'_, '_>) -> Result<String, Error> {
    let f: String = fake::faker::internet::en::FreeEmail().fake();
    Ok(f)
}

fn fake_username(_state: &State<'_, '_>) -> Result<String, Error> {
    let f: String = fake::faker::internet::en::Username().fake();
    Ok(f)
}
fn fake_ip4(_state: &State<'_, '_>) -> Result<String, Error> {
    let f: String = fake::faker::internet::en::IPv4().fake();
    Ok(f)
}
fn fake_ip6(_state: &State<'_, '_>) -> Result<String, Error> {
    let f: String = fake::faker::internet::en::IPv6().fake();
    Ok(f)
}
fn fake_useragent(_state: &State<'_, '_>) -> Result<String, Error> {
    let f: String = fake::faker::internet::en::UserAgent().fake();
    Ok(f)
}
fn fake_mac(_state: &State<'_, '_>) -> Result<String, Error> {
    let f: String = fake::faker::internet::en::MACAddress().fake();
    Ok(f)
}

fn fake_password(_state: &State<'_, '_>, low: usize, mut high: usize) -> Result<String, Error> {
    if high <= low {
        high = low + 1;
    }
    let f: String = fake::faker::internet::en::Password(low..high).fake();
    Ok(f)
}

fn fake_uuid(_state: &State<'_, '_>) -> Result<String, Error> {
    let f = Uuid::new_v4(); 
    Ok(f.hyphenated().to_string())
}

fn fake_uuid_s(_state: &State<'_, '_>) -> Result<String, Error> {
    let f = Uuid::new_v4(); 
    Ok(f.simple().to_string())
}

fn fake_base64_en(_state: &State<'_, '_>,fmt:String) -> Result<String, Error> {
    let f = base64::encode(fmt);
    Ok(f)
}

fn fake_base64_de(_state: &State<'_, '_>,fmt:String) -> Result<String, Error> {
    let df = base64::decode(fmt.clone());
    if let Ok(bytes) = df {
        if let Ok(f) = String::from_utf8(bytes) {
            return Ok(f);
        }
    }
    Ok(fmt)
}
fn fake_now(_state: &State<'_, '_>,fmt:String) -> Result<String, Error> {
    let local = Local::now();
    let fmt_data = local.format(fmt.as_str());
    Ok(fmt_data.to_string())
}

fn fake_datetime(_state: &State<'_, '_>,fmt:String) -> Result<String, Error> {
    let local =Utc::now();
    let ten_years = Duration::days(3660);
    let start = local.checked_sub_signed(ten_years).unwrap();
    let end = local.checked_add_signed(ten_years).unwrap();
    fake_date_between(_state, fmt.as_str(), start, end)
}

fn fake_datetime_after(_state: &State<'_, '_>,fmt:String,date:String) -> Result<String, Error> {
    let local =Utc::now();
    let ten_years = Duration::days(3660);
    let end = local.checked_add_signed(ten_years).unwrap();
    if let Ok(start) = Utc.datetime_from_str(date.as_str(), "%Y-%m-%dT%H:%M:%S") {
        fake_date_between(_state, fmt.as_str(), start, end)
    } else {
        Err(Error::new(ErrorKind::SyntaxError, format!("{}与{}格式不匹配", date,"%Y-%m-%dT%H:%M:%S")))
    }
}

fn fake_datetime_before(_state: &State<'_, '_>,fmt:String,date:String) -> Result<String, Error> {
    let local =Utc::now();
    let ten_years = Duration::days(3660);
    let start = local.checked_sub_signed(ten_years).unwrap();
    if let Ok(end) = Utc.datetime_from_str(date.as_str(), "%Y-%m-%dT%H:%M:%S") {
        fake_date_between(_state, fmt.as_str(), start, end)
    } else {
        Err(Error::new(ErrorKind::SyntaxError, format!("{}与{}格式不匹配", date,"%Y-%m-%dT%H:%M:%S")))
    }
}

fn fake_date_between(_state: &State<'_, '_>,fmt:&str,start:DateTime<Utc>,end:DateTime<Utc>) -> Result<String, Error> {
    let f:String = fake::faker::chrono::zh_cn::DateTimeBetween(start,end).fake();
    let d = f.parse::<DateTime<Utc>>().unwrap();
    Ok(d.format(fmt).to_string())
}
