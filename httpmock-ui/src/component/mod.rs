pub mod context_list;
pub mod highlight;
// pub mod pop_windows;
// pub mod toggle;
pub mod template_tools;
pub mod mock_path_ui;


#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Local;

    #[test]
    fn test_chrono() {
        let now = Local::now();
        println!("{}", now.timestamp_millis());
    }
}
