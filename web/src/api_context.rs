use std::collections::BTreeMap;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use crate::app::TOASTS;
use crate::component::tree_ui::{self, TreeUi};
use crate::history_db::{add_new_version_mockinfo, get_history_list, get_mock};
use crate::request_data::MockData;
use crate::ui::request_ui::CollectionUi;
use crate::ui::request_ui::{RequestUi, ResponseUi};
use egui::WidgetText;
use egui_dock::TabViewer;
use egui_notify::Toasts;
use log::info;
use server::common::mock::MockDefine;
use server::common::MOCK_SERVER;

#[derive(serde::Deserialize, serde::Serialize)]
pub struct ApiContext {
    pub selected: Vec<u64>,
    pub tests: BTreeMap<u64, MockData>,
    pub docs: BTreeMap<u64, String>,
    pub tree_ui: TreeUi,
    #[serde(skip)]
    req_ui: RequestUi,
    #[serde(skip)]
    rsq_ui: ResponseUi,
    #[serde(skip)]
    docs_ui: CollectionUi,
}
impl TabViewer for ApiContext {
    type Tab = String;

    fn ui(&mut self, ui: &mut egui::Ui, tab: &mut Self::Tab) {
        let selected = *self.selected.first().unwrap_or(&0);
        match tab.as_str() {
            "请求" => {
                if let Some(req_data) = self.tests.get_mut(&selected) {
                    self.req_ui.ui(ui, &mut req_data.req, selected);
                }
            }
            "响应" => {
                if let Some(req_data) = self.tests.get_mut(&selected) {
                    self.rsq_ui.ui(ui, &mut req_data.resp, selected)
                }
            }
            "文档" => {
                if let Some(req_data) = self.tests.get_mut(&selected) {
                    self.docs_ui.ui(ui, &mut req_data.req.remark, selected);
                }
                if let Some(req_data) = self.docs.get_mut(&selected) {
                    self.docs_ui.ui(ui, req_data, selected);
                }
            }
            "记录" => {
                    // draws the logger ui.
                    egui_logger::logger_ui(ui);
            }
            "记录" => {
                let hist_list = get_history_list(selected);
                egui::ScrollArea::vertical().show(ui, |ui| {
                    for (ver_name, ver) in hist_list {
                        ui.horizontal(|ui| {
                            ui.label(ver.to_string());
                            if ui.button(ver_name).clicked() {
                                if let Some(mock) = get_mock(selected, ver) {
                                    let recode = self.tests.get_mut(&selected).unwrap();
                                    *recode = mock;
                                }
                            }
                        });
                    }
                });
            }
            "导航" => {
                let toast = TOASTS.get_or_init(|| {
                    Arc::new(Mutex::new(
                        Toasts::default().with_anchor(egui_notify::Anchor::BottomRight),
                    ))
                });
                egui::ScrollArea::both().show(ui, |ui| {
                    // ui.with_layout(Layout::top_down(egui::Align::LEFT), |ui|{
                    match self.tree_ui.ui_impl(ui) {
                        tree_ui::Action::Keep => {
                            //ignore
                        }
                        tree_ui::Action::Delete(dels) => {
                            if let Ok(mut mock_server) = MOCK_SERVER.write() {
                                for del_id in dels {
                                    info!("删除{}", del_id);
                                    self.delete_collecton(del_id);
                                    if let Some(mock_define) = self.delete_test(del_id) {
                                        let mock: MockDefine = mock_define.into();
                                        mock_server.delete(mock)
                                    }
                                }
                            } else {
                                info!("删除失败，请稍后再试");
                            }
                        }
                        tree_ui::Action::Add((adds, node_type)) => {
                            let add_id = adds.first().unwrap().to_owned();
                            info!("添加{},{:?}", &add_id, &node_type);
                            match node_type {
                                tree_ui::NodeType::Collection => {
                                    self.insert_collecton(add_id, "".to_string());
                                }
                                tree_ui::NodeType::Node => {
                                    self.insert_test(add_id, MockData::default());
                                }
                            }
                        }
                        tree_ui::Action::Rename(_adds) => {
                            //基本上不用处理
                            info!("重命名")
                        }
                        tree_ui::Action::Selected((selected_id, selected_title)) => {
                            let selected = *selected_id.first().unwrap_or(&0);
                            self.selected = selected_id;
                            if let Ok(mut toast_w) = toast.lock() {
                                toast_w
                                    .info(format!("已选中{}-标题{}", selected, selected_title))
                                    .set_duration(Some(Duration::from_secs(5)));
                            }
                        }
                        tree_ui::Action::Copy(cop) => {
                            if let Ok(mut toast_w) = toast.lock() {
                                toast_w
                                    .info(format!("已复制{}", cop.0))
                                    .set_duration(Some(Duration::from_secs(5)));
                            }
                        }
                        tree_ui::Action::Parse(mut parse) => {
                            //复制动作
                            let _ = parse.pop();
                            if let Some((sid, did)) = self.tree_ui.parse_node(parse) {
                                if let Some(copyed) = self.tests.get(&sid) {
                                    let parse = copyed.clone();
                                    self.insert_test(did, parse);
                                }
                            }
                        }
                        tree_ui::Action::SyncToServer((id, active)) => {
                            let mut msg = String::new();
                            if let Some(mockdata) = self.tests.get(&id) {
                                if let Ok(mut mock_server) = MOCK_SERVER.write() {
                                    if active {
                                        let mut mock: MockDefine = mockdata.clone().into();
                                        mock.id = id;
                                        if mock.req.path.is_empty() || mock.resp.body.is_none() {
                                            msg = "添加失败：路径或者响应为空".to_owned();
                                        } else {
                                            msg = match mock_server
                                                .add(mock, mockdata.req.priority.into())
                                            {
                                                Ok(_) => {
                                                    add_new_version_mockinfo(id, mockdata);
                                                    format!("已更新配置{}", id)
                                                }
                                                Err(e) => {
                                                    // self..disable_item(id);
                                                    e
                                                }
                                            };
                                        }
                                    } else {
                                        let mut mock: MockDefine = mockdata.clone().into();
                                        mock.id = id;
                                        mock_server.delete(mock);
                                        msg = format!("已取消配置{}", id)
                                    };
                                } else {
                                    msg = format!("获取锁失败，请重试");
                                }
                            }
                            match toast.lock() {
                                Ok(mut toast_w) => {
                                    toast_w.info(msg).set_duration(Some(Duration::from_secs(5)));
                                }
                                Err(e) => {
                                    info!("{}", e.to_string())
                                }
                            }
                        }
                    }
                    ui.add_space(ui.available_height());
                });
            }
            _ => {
                ui.label(tab.as_str());
            }
        }
    }

    fn title(&mut self, tab: &mut Self::Tab) -> WidgetText {
        tab.as_str().into()
    }

    fn on_close(&mut self, _tab: &mut Self::Tab) -> bool {
        // if let Some(index )= self.tree.find_tab(tab) {
        //     self.tree.remove_tab(index);
        // }
        true
    }
}

impl ApiContext {
    pub fn new() -> Self {
        Self {
            tests: BTreeMap::new(),
            docs: BTreeMap::new(),
            req_ui: RequestUi::default(),
            tree_ui: TreeUi::new(),
            selected: vec![0],
            rsq_ui: ResponseUi::default(),
            docs_ui: CollectionUi::default(),
        }
    }

    pub fn insert_collecton(&mut self, key: u64, value: String) -> Option<String> {
        self.docs.insert(key, value)
    }
    pub fn insert_test(&mut self, key: u64, value: MockData) -> Option<MockData> {
        self.tests.insert(key, value)
    }

    pub fn delete_test(&mut self, key: u64) -> Option<MockData> {
        self.tests.remove(&key)
    }

    pub fn delete_collecton(&mut self, key: u64) -> Option<String> {
        self.docs.remove(&key)
    }

    pub fn get_mut_collection(&mut self, key: u64) -> Option<&mut String> {
        self.docs.get_mut(&key)
    }
    pub fn get_mut_test(&mut self, key: u64) -> Option<&mut MockData> {
        self.tests.get_mut(&key)
    }
}
