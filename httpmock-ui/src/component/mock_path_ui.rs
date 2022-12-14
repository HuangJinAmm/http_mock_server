use std::{collections::HashMap, time::Duration};
use egui::{Ui, Key, Vec2};
use httpmock_server::common::{mock::MockDefine, data::{HttpMockRequest, MockServerHttpResponse}};
use crate::{app::Method, esay_md::EasyMarkEditor};

use super::highlight::{CodeTheme, highlight};

#[derive(Debug,Default, Clone, serde::Deserialize, serde::Serialize)]
pub struct MockPathUi {
    pub is_edit: bool,
    pub remark: String,
    path: String,
    method: Method,
    head:SelectKeyValueInputs,
    body: String,
    returns: MockReturns,
}

// impl Default for MockPathUi {
//     fn default() -> Self {
//         Self { is_edit: Default::default(), remark: "请添加注释".to_owned(), path: Default::default(), method: Default::default(), head: Default::default(), body: Default::default(), returns: Default::default() }
//     }
// }

impl MockPathUi {
    pub fn ui(&mut self, ui: &mut Ui) {

        // editable_label(ui, &mut self.is_edit, &mut self.remark);
        ui.columns(2, |cols|{
            self.req_set_ui(&mut cols[0]);
            self.returns.ui(&mut cols[1]);
        })
    }

    fn req_set_ui(&mut self,ui: &mut Ui) {
        ui.group(|ui|{

            ui.vertical(|ui| {
                ui.strong("请求条件设置");
                ui.add_space(5.0);
                ui.horizontal(|ui|{
                    egui::ComboBox::from_id_source("requset_method")
                        .selected_text(format!("{:?}", &mut self.method))
                        .show_ui(ui, |ui| {
                            ui.selectable_value(&mut self.method, Method::GET, "GET");
                            ui.selectable_value(&mut self.method, Method::POST, "POST");
                            ui.selectable_value(&mut self.method, Method::PUT, "PUT");
                            ui.selectable_value(&mut self.method, Method::DELETE, "DELETE");
                            ui.selectable_value(&mut self.method, Method::PATCH, "PATCH");
                            ui.selectable_value(&mut self.method, Method::OPTIONS, "OPTIONS");
                            ui.selectable_value(&mut self.method, Method::NONE, "无限制");
                        });
                    ui.label("路径");
                    egui::TextEdit::singleline(&mut self.path).desired_width(ui.available_width()).show(ui);
                });
                self.head.ui_grid_input(ui, "aaaaaa");
                super::highlight::code_editor_ui_notool(ui, &mut self.body, "json");
            });
        });
    }
}

#[derive(Debug, Clone,PartialEq, serde::Deserialize, serde::Serialize)]
enum ReturnType {
    Mock,Relay
}
impl Default for ReturnType {
    fn default() -> Self {
        ReturnType::Mock 
    }
}



#[derive(Debug, Clone, Default, serde::Deserialize, serde::Serialize)]
struct MockReturns {
    return_type:ReturnType,
    dist_url:String,
    delay: u16,
    code: u16,
    body: String,
    headers: SelectKeyValueInputs,
}

impl MockReturns {
    pub fn ui(&mut self, ui: &mut Ui) {

        ui.group(|ui| {
            ui.vertical(|ui|{
                ui.horizontal(|ui|{
                    ui.selectable_value(&mut self.return_type,ReturnType::Mock, "模拟响应");
                    ui.selectable_value(&mut self.return_type,ReturnType::Relay, "转发");
                });
                match self.return_type {
                    ReturnType::Mock => {
                        ui.columns(4, |cols| {
                            cols[0].label("延时（ms）");
                            cols[1].add(egui::DragValue::new(&mut self.delay).speed(1));
                            cols[2].label("响应码");
                            cols[3].add(egui::DragValue::new(&mut self.code).speed(1));
                        });
                        self.headers.ui_grid_input(ui, self.dist_url.as_str());
                        super::highlight::code_editor_ui(ui, &mut self.body, "json");
                    },
                    ReturnType::Relay => {
                        ui.columns(2, |cols| {
                            cols[0].label("转发服务器地址");
                            cols[1].add(egui::text_edit::TextEdit::singleline(&mut self.dist_url).hint_text("请输入转发服务器的全路径地址"));
                        });
                    },
                    }
            });
        });
    }
}

#[derive(Debug, Clone, Default, serde::Deserialize, serde::Serialize)]
pub struct MockDefineInfo {
    pub mock_define_info: MockPathUi,
}

impl MockDefineInfo {
    pub fn ui(&mut self, ui: &mut Ui) {
        self.mock_define_info.ui(ui);
    }
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct SelectKeyValueInputs {
    inputs: Vec<SelectKeyValueItem>,
}

impl Default for SelectKeyValueInputs {
    fn default() -> Self {
        Self { 
            inputs: vec![SelectKeyValueItem{ selected: true, key:reqwest::header::CONTENT_TYPE.to_string(), value: "application/json".to_string() },SelectKeyValueItem::new()],
        }
    }
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
struct SelectKeyValueItem {
    selected: bool,
    key: String,
    value: String,
}

impl SelectKeyValueItem {
    fn new() -> Self {
        Self {
            selected: true,
            key: "".into(),
            value: "".into(),
        }
    }
}

impl SelectKeyValueInputs {
    pub fn ui_grid_input(&mut self, ui: &mut Ui, id: &str) {
        ui.group(|ui| {
            egui::Grid::new(id)
                .num_columns(3)
                .min_col_width(20.)
                .min_row_height(20.)
                .show(ui, |ui| {
                    ui.add_sized(ui.available_size(), egui::widgets::Label::new("Header"));
                    ui.horizontal(|ui|{

                    let add_header = ui.small_button("➕添加");
                    let del_header = ui.small_button("➖删除");
                    if add_header.clicked() {
                        self.inputs.push(SelectKeyValueItem::new());
                    }
                    if del_header.clicked() {
                        self.inputs = self
                            .clone()
                            .inputs
                            .into_iter()
                            .filter(|item| item.selected)
                            .collect();
                    }
                    });
                    // ui.add_sized(
                    //     [120., 20.],
                    //     egui::widgets::Label::new(egui::RichText::new("Key").strong()),
                    // );
                    // ui.add_sized(
                    //     ui.available_size(),
                    //     egui::widgets::Label::new(egui::RichText::new("Value").strong()),
                    // );
                    ui.end_row();
                    for SelectKeyValueItem {
                        selected,
                        key,
                        value,
                    } in &mut self.inputs
                    {
                        ui.checkbox(selected, "");
                        let theme = CodeTheme::from_memory(ui.ctx());

                        let mut layouter = |ui: &egui::Ui, string: &str, _wrap_width: f32| {
                            let layout_job = highlight(ui.ctx(), &theme, string, "json");
                            // layout_job.wrap.max_width = wrap_width; // no wrapping
                            ui.fonts().layout_job(layout_job)
                        };
                        ui.add_sized(
                            ui.available_size(),
                            egui::text_edit::TextEdit::singleline(key),
                        );
                        ui.add_sized(
                            ui.available_size(),
                            egui::text_edit::TextEdit::singleline(value).layouter(&mut layouter),
                        );
                        ui.end_row();
                    }
                });
        });
    }
    pub fn ui_grid(&mut self, ui: &mut Ui, id: &str) {
        ui.group(|ui| {
            egui::Grid::new(id)
                .num_columns(2)
                .min_col_width(80.)
                .min_row_height(20.)
                .show(ui, |ui| {
                    ui.add_sized(
                        ui.available_size(),
                        egui::widgets::Label::new(egui::RichText::new("键").strong()),
                    );
                    ui.add_sized(
                        ui.available_size(),
                        egui::widgets::Label::new(egui::RichText::new("值").strong()),
                    );
                    ui.end_row();
                    for SelectKeyValueItem { key, value, .. } in &mut self.inputs {
                        ui.add_sized(ui.available_size(), egui::widgets::Label::new(key.clone()));
                        ui.add_sized(
                            ui.available_size(),
                            egui::widgets::Label::new(value.clone()),
                        );
                        ui.end_row();
                    }
                });
        });
    }

    // pub fn ui_table(&mut self, ui: &mut Ui) {
    //     TableBuilder::new(ui)
    //         .column(Size::remainder().at_least(100.0))
    //         .column(Size::exact(40.0))
    //         .header(20.0, |mut header| {
    //             header.col(|ui| {
    //                 ui.heading("选择");
    //             });
    //             header.col(|ui| {
    //                 ui.heading("Key");
    //             });
    //             header.col(|ui| {
    //                 ui.heading("Value");
    //             });
    //         })
    //         .body(|mut body| {
    //             body.row(30.0, |mut row| {
    //                 for SelectKeyValueItem {
    //                     selected,
    //                     key,
    //                     value,
    //                 } in &mut self.inputs
    //                 {
    //                     row.col(|ui| {
    //                         ui.checkbox(selected, "");
    //                     });
    //                     row.col(|ui| {
    //                         ui.text_edit_singleline(key);
    //                     });
    //                     row.col(|ui| {
    //                         ui.text_edit_singleline(value);
    //                     });
    //                 }
    //             });
    //         });
    // }
}

impl Into<MockDefine> for MockPathUi {
    fn into(self) -> MockDefine {
        let id = 0;
        let mut req;
        if self.path.contains('?') {
            let path_query_split:Vec<&str> = self.path.split('?').collect();
            let path = path_query_split.first().unwrap().clone().to_string();
            let query = &(*path_query_split.get(1).unwrap()).clone();
            let query_params_m:HashMap<String,String > = query.split('&')
                .map(|qr|{
                    let qrs:Vec<String>= qr.split('=').map(|qk|qk.to_string()).collect();
                    qrs
                }).fold(HashMap::new(), |mut qm,qvc|{
                    if qvc.len()>1{
                        qm.insert(qvc.get(0).unwrap().to_string(), qvc.get(1).unwrap().to_string());
                    }
                    qm
                });
            req = HttpMockRequest::new(path);
            req.query_params(query_params_m);
        } else {
            req = HttpMockRequest::new(self.path);
        }
        let headers = self.head.inputs.into_iter()
            .filter(|selected_item|selected_item.selected)
            .fold(HashMap::new(), |mut map,head_item|{
            if !head_item.key.is_empty() && !head_item.value.is_empty() {
                map.insert(head_item.key,head_item.value);
                map
            } else {
                map
            }
        });
        req.headers(headers);
        req.method(self.method.get_string());
        req.body(self.body.as_bytes().to_vec());

        let mock_ret = self.returns;

        let relay_url = match mock_ret.return_type {
            ReturnType::Mock => None,
            ReturnType::Relay => Some(mock_ret.dist_url),
        };

        let mut resp = MockServerHttpResponse::new();
        
        resp.body = Some(mock_ret.body.as_bytes().to_vec());

        resp.delay = Some(Duration::from_millis(mock_ret.delay.into()));

        resp.status = Some(mock_ret.code);

        let resp_headers = mock_ret.headers.inputs.into_iter()
            .filter(|selected_item|selected_item.selected)
            .fold(Vec::new(), |mut map,head_item|{
            if !head_item.key.is_empty() && !head_item.value.is_empty() {
                map.push((head_item.key,head_item.value));
                map
            } else {
                map
            }
        });
        resp.headers = Some(resp_headers);
        let remark = self.remark;
        MockDefine { id,remark, req, resp,relay_url}
    }
}


pub fn editable_label(ui: &mut egui::Ui, is_edit: &mut bool, value: &mut String) {
    if *is_edit {
        
        let mut text_edit_size = ui.available_size();
        text_edit_size.y = 40.;
        let rsp = ui.add_sized(
            text_edit_size,
            // egui::TextEdit::multiline(&mut code)
            egui::text_edit::TextEdit::multiline(value)
                .font(egui::TextStyle::Monospace) // for cursor height
        );
        // let rsp = ui.text_edit_multiline(value);
        if rsp.lost_focus() {
            *is_edit = false;
        }
    } else {
        ui.horizontal(|ui| {
            let resp = ui.label("备注:".to_string()+value.clone().as_str());
            let rect = resp.rect.expand2(Vec2::new(40., 10.));
            if ui.rect_contains_pointer(rect) {
                let rsp = ui.button("编辑");
                if rsp.clicked() {
                    *is_edit = !*is_edit;
                }
            }
        });
    }
}