use std::collections::{BTreeMap };

use crate::history_db::{get_history_list, get_mock};
use crate::request_data::MockData;
use crate::ui::request_ui::CollectionUi;
use crate::ui::request_ui::{RequestUi, ResponseUi};
use egui::WidgetText;
use egui_dock::TabViewer;

#[derive(serde::Deserialize, serde::Serialize)]
pub struct ApiContext {
    pub selected: Vec<u64>,
    pub tests: BTreeMap<u64, MockData>,
    pub docs: BTreeMap<u64, String>,
    #[serde(skip)]
    req_ui: RequestUi,
    #[serde(skip)]
    rsq_ui: ResponseUi,
    #[serde(skip)]
    docs_ui:CollectionUi,
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
                let hist_list = get_history_list(selected); 
                egui::ScrollArea::vertical().show(ui, |ui|{
                    for (ver_name,ver) in hist_list {
                        ui.horizontal(|ui|{
                            ui.label(ver.to_string());
                            if ui.button(ver_name).clicked() {
                                if let Some(mock) = get_mock(self.list_selected, ver) {
                                    let mut recode = self.records.get_mut(&self.list_selected).unwrap();
                                    recode.mock_define_info = mock;
                                }
                            }
                        });
                    }
                });
            }
            _ => {
                ui.label(tab.as_str());
            }
        }
    }

    fn context_menu(&mut self, _ui: &mut egui::Ui, tab: &mut Self::Tab) {
        match tab {
            _ => {}
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
