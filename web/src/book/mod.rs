use mdbook::book::{Chapter, SectionNumber, Summary};
use mdbook::config::Config;
use mdbook::{BookItem, MDBook};
use minijinja::context;

use crate::api_context::ApiContext;
use crate::component::tree_ui::{TreeUi, TreeUiNode, NodeType};
use crate::request_data::MockData;

const ROOT_DIR: &str = "./docs";

pub fn build_book(tree_ui: &TreeUi, api: &ApiContext) -> Result<(), anyhow::Error> {
    let mut cfg = Config::default();
    cfg.book.multilingual = true;
    cfg.build.create_missing = false;
    // cfg.book.title = Some("Mock服务器".to_string());
    // cfg.book.authors.push("Mocks".to_string());
    // cfg.book.description = Some("模拟数据服务器文档".into());
    let sum = Summary {
        title: Some("Mock目录".to_owned()),
        prefix_chapters: vec![],
        numbered_chapters: vec![],
        suffix_chapters: vec![],
    };
    let mut book = MDBook::load_with_config_and_summary(ROOT_DIR, cfg, sum).expect("创建book失败");
    let mut num: u32 = 1;
    for level_one_node in tree_ui.get_sub_nodes() {
        let level_one_chapter = gen_chapter(vec![num], level_one_node, api);
        book.book.push_item(BookItem::Chapter(level_one_chapter));
        num += 1;
    }
    book.build()
}

fn gen_chapter(num: Vec<u32>, node: &TreeUiNode, api: &ApiContext) -> Chapter {
    let mut count_vec = num.clone();
    let mut p = gen_node_chapter(num, node, api);
    let mut count = 1;
    for sub in node.get_sub_nodes() {
        let mut num_vec = count_vec.clone();
        num_vec.push(count);
        let mut sub_chapter = gen_chapter(num_vec, sub, api);
        sub_chapter.parent_names.push(node.title.clone());
        p.sub_items.push(mdbook::BookItem::Chapter(sub_chapter));
        count += 1;
    }
    p.sub_items.push(mdbook::BookItem::Separator);
    p
}

fn rander_mockdata(mock:&MockData) -> String {

    let r_headers:Vec<(String,String)> = mock.req.headers.iter().filter(|h|h.selected).map(|h|(h.key.clone(),h.value.clone())).collect();

    let ctx = context! {
        req_doc => mock.req.remark,
        req_method => mock.req.method.to_string(),
        req_url => mock.req.path,
        req_headers => r_headers,
        req_body => mock.req.body
    };
    let mock_template = r#"${req_doc}
---
- ## 请求

**${req_method}** `${req_url}`

**Headers:**
|Key|Value|
|--|--|
#{for head in req_headers}
|${head.0}|${head.1}|
#{endfor}
**Json Schema**

```json
${req_body}
```

**Body**

```json
${req_body}
```

- ## 响应
**code:** `200`

**Headers:**
|Key|Value|
|--|--|
|Content-Type|application/json|

**Body**

```json
{
    "a":1,
    "b":"b"
}
```

"#;
todo!()
}

fn gen_node_chapter(num: Vec<u32>, node: &TreeUiNode, api: &ApiContext) -> Chapter {
    let des = "无数据".to_owned();
    let content = match node.node_type {
        NodeType::Collection => api.docs.get(&node.id),
        NodeType::Node => {
            api.tests.get(&node.id).map(|m| {
                // let mut node_md = String::new();
                let md = &m.req.remark;
                // node_md.push_str(md);
                md
            })
        },
    }
    .unwrap_or(&des);
    let mut path = String::from(ROOT_DIR);
    path.push('/');
    path.push_str(node.id.to_string().as_str());
    Chapter {
        name: node.title.clone(),
        content: content.to_owned(),
        number: Some(SectionNumber(num)),
        sub_items: vec![],
        path: Some(path.into()),
        source_path: None,
        parent_names: vec![],
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mdbook::book::{Chapter, Link, SectionNumber, Summary, SummaryItem};
    use mdbook::config::Config;
    use mdbook::{BookItem, MDBook};

    #[test]
    fn test_name() {
        let root_dir = "./book";

        // create a default config and change a couple things
        let mut cfg = Config::default();
        cfg.book.multilingual = true;
        cfg.build.create_missing = false;
        cfg.book.title = Some("Mock服务器".to_string());
        cfg.book.authors.push("Mocks".to_string());

        // let sum = Summary {
        //     title: Some("Mock目录".to_owned()),
        //     prefix_chapters: vec![SummaryItem::Link(Link {
        //         name: "a".into(),
        //         location: None,
        //         number: None,
        //         nested_items: vec![
        //             SummaryItem::Link(Link {
        //                 name: "aa".into(),
        //                 location: None,
        //                 number: None,
        //                 nested_items: vec![],})
        //         ],
        //     })],
        //     numbered_chapters: vec![SummaryItem::Link(Link {
        //         name: "b".into(),
        //         location: None,
        //         number: None,
        //         nested_items: vec![],
        //     })],
        //     suffix_chapters: vec![SummaryItem::Link(Link {
        //         name: "c".into(),
        //         location: None,
        //         number: None,
        //         nested_items: vec![],
        //     })],
        // };

        let sum = Summary::default();
        // let mut book = MDBook::init(root_dir)
        //             .create_gitignore(true)
        //             .with_config(cfg)
        //             .build()
        //             .expect("Book generation failed");
        let mut book =
            MDBook::load_with_config_and_summary(root_dir, cfg, sum).expect("创建book失败");
        // let mut book = MDBook::load_with_config(root_dir, cfg).expect("book failed");
        let aa = Chapter {
            name: "aa".into(),
            content: r"# a
## b
- [ ] 文本内容"
                .into(),
            number: Some(SectionNumber(vec![1, 2, 3])),
            sub_items: vec![BookItem::PartTitle("测试子标题".to_owned())],
            path: None,
            source_path: None,
            parent_names: vec!["a".into(), "b".into()],
        };
        let a = Chapter {
            name: "a".into(),
            content: "- [ ] 文本内容".into(),
            number: Some(SectionNumber(vec![1, 2])),
            sub_items: vec![mdbook::BookItem::Chapter(aa)],
            path: Some("./a".into()),
            source_path: None,
            parent_names: vec!["b".into()],
        };
        let b = Chapter {
            name: "b".into(),
            content: "- [ ] 文本内容".into(),
            number: Some(SectionNumber(vec![1])),
            sub_items: vec![
                // BookItem::PartTitle("测试2的白头".to_owned()),
                BookItem::Chapter(a),
            ],
            path: Some("./b".into()),
            source_path: None,
            parent_names: vec!["a".into()],
        };
        let section = BookItem::Chapter(b);
        book.book.push_item(section);
        book.build();
    }
}
