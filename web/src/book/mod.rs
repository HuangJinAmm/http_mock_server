use mdbook::MDBook;
use mdbook::config::Config;

use crate::api_context::ApiContext;
use crate::component::tree_ui::TreeUi;

const ROOT_DIR: &str = "./book";

pub fn build_book(tree_ui:&TreeUi,api:&ApiContext) {
    // tree_ui.
    
}

#[cfg(test)]
mod tests {
    use super::*;
    use mdbook::book::{Chapter, SectionNumber};
    use mdbook::{MDBook, BookItem};
    use mdbook::config::Config;

    #[test]
    fn test_name() {
        let root_dir = "./book";

        // create a default config and change a couple things
        let mut cfg = Config::default();
        cfg.book.multilingual = true;
        cfg.build.create_missing = false;
        cfg.book.title = Some("Mock服务器".to_string());
        cfg.book.authors.push("Mocks".to_string());

        let mut book = MDBook::init(root_dir)
                    .create_gitignore(true)
                    .with_config(cfg)
                    .build()
                    .expect("Book generation failed");
        let chapter = Chapter {
            name: "测试1".into(),
            content: "- [ ] 文本内容".into(),
            number: None,
            sub_items: vec![],
            path: Some(root_dir.into()),
            source_path: None,
            parent_names: vec![],
        };
        let section = BookItem::Chapter(chapter);
        book.book.push_item(section);
        book.build();
    }
}
