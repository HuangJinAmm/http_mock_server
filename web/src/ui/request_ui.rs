use std::collections::HashMap;
use std::time::Duration;

use crate::app::REQ_UI_ID;
use crate::app::TASK_CHANNEL;
use crate::app::TOASTS;
use crate::app::TOKIO_RT;
use crate::component::code_editor::TextEdit;
use crate::component::header_ui::HeaderUi;
use crate::component::header_ui::SelectKeyValueItem;
use crate::component::syntax_highlight::code_view_ui;
use crate::request_data::Method;
use crate::request_data::MockData;
use crate::request_data::ReqMockData;
use crate::request_data::RspMockData;
use crate::utils::template::rander_template;
use crate::component::editeable_label::editable_label;
use egui_commonmark::CommonMarkCache;
use egui_commonmark::CommonMarkViewer;
use serde_json::Value;
use server::common::data::HttpMockRequest;
use server::common::data::MockServerHttpResponse;
use server::common::mock::MockDefine;
#[derive(Default)]
pub struct RequestUi {
    pub editor: TextEdit,
}

impl RequestUi {
    pub fn ui(&mut self, ui: &mut egui::Ui, request_data: &mut ReqMockData, id: u64) {
        let ReqMockData {
            remark,
            path,
            method,
            headers,
            body,
        } = request_data;
        let ui_id = REQ_UI_ID.get_or_init(|| ui.id());
        let req_id = ui_id.with(id);

        ui.vertical(|ui| {
            ui.add(editable_label(remark));
            ui.horizontal(|ui| {
                ui.label("请求方法:");
                egui::ComboBox::from_label("🌐")
                    .selected_text(format!("{:?}", method))
                    .show_ui(ui, |ui| {
                        ui.selectable_value(method, Method::GET, "GET");
                        ui.selectable_value(method, Method::POST, "POST");
                        ui.selectable_value(method, Method::PUT, "PUT");
                        ui.selectable_value(method, Method::DELETE, "DELETE");
                        ui.selectable_value(method, Method::PATCH, "PATCH");
                        ui.selectable_value(method, Method::OPTIONS, "OPTIONS");
                    });

                egui::TextEdit::singleline(path)
                    .desired_width(ui.available_width())
                    .hint_text("请求路径")
                    .show(ui);
            });

            egui::ScrollArea::both()
                .auto_shrink([false, false])
                .id_source("requset_ui_scroller_1")
                .show(ui, |ui| {
                    let id_source = ui.make_persistent_id("net_test_requset_ui");
                    egui::collapsing_header::CollapsingState::load_with_default_open(
                        ui.ctx(),
                        id_source,
                        false,
                    )
                    .show_header(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.label("请求头");
                            let add_header = ui.small_button("➕");
                            let del_header = ui.small_button("➖");
                            if add_header.clicked() {
                                headers.push(SelectKeyValueItem::new("", ""));
                            }
                            if del_header.clicked() {
                                let new_headers: Vec<SelectKeyValueItem> = headers
                                    .clone()
                                    .into_iter()
                                    .filter(|item| item.selected)
                                    .collect();
                                *headers = new_headers;
                            }
                        });
                    })
                    .body(|ui| {
                        HeaderUi::ui_grid_input(ui, "request_body_grid_1", headers);
                    });

                    let state_id = ui.id().with(id.to_string() + "body");
                    let (mut show_plaintext, mut template_str) = ui.data(|d| {
                        d.get_temp::<(bool, String)>(state_id)
                            .unwrap_or((false, "".to_string()))
                    });
                    ui.horizontal(|ui| {
                        ui.label("请求体：");
                    });
                    self.editor.ui(ui, body, id);
                })
        });
    }
}

#[derive(Default)]
pub struct CollectionUi {
    script_editor: TextEdit,
    cache: CommonMarkCache,
}

impl CollectionUi {
    pub fn ui(&mut self, ui: &mut egui::Ui, data: &mut String, id: u64) {
        let req_id = ui.id().with(id);
        let mut preview_state = ui.data_mut(|d| d.get_temp::<bool>(req_id).unwrap_or(false));
        ui.vertical(|ui| {
            ui.horizontal(|ui| {
                ui.label("文档");
                let add_header = ui.small_button("预览");
                if add_header.clicked() {
                    preview_state = !preview_state;
                }
            });
            if preview_state {
                CommonMarkViewer::new("viewer").show(ui, &mut self.cache, &data);
            } else {
                self.script_editor.ui(ui, data, id);
            }
        });
        ui.data_mut(|d| d.insert_temp(req_id, preview_state));
    }
}

#[derive(Default)]
pub struct ResponseUi {
    editor: TextEdit,
}

impl ResponseUi {
    pub fn ui(&mut self, ui: &mut egui::Ui, data: &mut RspMockData, id: u64) {
        let req_id = ui.id().with(id).with("resp");
        let ui_id = REQ_UI_ID.get_or_init(|| ui.id());
        let req_id = ui_id.with(id);
        let (mut view_state, mut template_str) = ui.data_mut(|d| {
            d.get_temp::<(bool, String)>(req_id)
                .unwrap_or((false, "".to_owned()))
        });
        let RspMockData {
            is_proxy,
            dist_url,
            delay,
            code,
            body,
            headers,
        } = data;
        ui.vertical(|ui| {
            ui.group(|ui| {
                egui::ScrollArea::both()
                    .id_source("respone_ui_scroller_1")
                    .show(ui, |ui| {
                        // ui.set_min_size(ui.available_size());
                        ui.add_enabled_ui(*is_proxy, |ui| {
                            ui.columns(4, |cols| {
                                cols[0].label("延时（ms）");
                                cols[1].add(egui::DragValue::new(delay).speed(1));
                                cols[2].label("响应码");
                                cols[3].add(egui::DragValue::new(code).speed(1));
                            });
                            ui.with_layout(
                                egui::Layout::top_down_justified(egui::Align::LEFT),
                                |ui| {
                                    ui.collapsing("响应头", |ui| {
                                        HeaderUi::ui_grid_input(ui, "response_grid_ui_1", headers);
                                    });
                                    if view_state {
                                        if ui.button("🔃").clicked() {
                                            if view_state {
                                                match rander_template(body.as_str()) {
                                                    Ok(parsed_temp) => template_str = parsed_temp,
                                                    Err(e) => {
                                                        if let Ok(mut toast_w) =
                                                            TOASTS.get().unwrap().lock()
                                                        {
                                                            toast_w.error(e.to_string().as_str());
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    } else {
                                        ui.add_space(29.0);
                                    }
                                    if ui.toggle_value(&mut view_state, "预览").clicked() {
                                        if view_state {
                                            let task_sender = unsafe { TASK_CHANNEL.0.clone() };
                                            TOKIO_RT.spawn(async move {
                                                if let Err(_) = task_sender.send((id, 0, 0)).await {
                                                    log::info!("receiver dropped");
                                                    return;
                                                }
                                            });
                                            let deal_temp = match rander_template(&body) {
                                                Ok(parsed_temp) => parsed_temp,
                                                Err(e) => {
                                                    let mut msg = "模板语法错误：".to_string();
                                                    msg.push_str(e.to_string().as_str());
                                                    if let Ok(mut toast_w) =
                                                        TOASTS.get().unwrap().lock()
                                                    {
                                                        toast_w.error(e.to_string().as_str());
                                                    }
                                                    body.clone()
                                                }
                                            };
                                            template_str =
                                                match json5::from_str::<Value>(&deal_temp) {
                                                    Ok(json_body) => {
                                                        serde_json::to_string_pretty(&json_body)
                                                            .unwrap_or(body.clone())
                                                    }
                                                    Err(_) => body.clone(),
                                                };
                                        }
                                    }
                                    if ui.button("格式化JSON").clicked() {
                                        let unfmt_json = body.clone();

                                        let f = json5format::Json5Format::new().unwrap();
                                        match json5format::ParsedDocument::from_str(
                                            &unfmt_json,
                                            None,
                                        ) {
                                            Ok(d) => match f.to_string(&d) {
                                                Ok(s) => {
                                                    *body = s;
                                                }
                                                Err(se) => {
                                                    if let Ok(mut toast_w) =
                                                        TOASTS.get().unwrap().lock()
                                                    {
                                                        toast_w.error(se.to_string().as_str());
                                                    }
                                                }
                                            },
                                            Err(e) => {
                                                if let Ok(mut toast_w) =
                                                    TOASTS.get().unwrap().lock()
                                                {
                                                    toast_w.error(e.to_string().as_str());
                                                }
                                            }
                                        }
                                    }

                                    if view_state {
                                        self.editor.ui(ui, body, id);
                                    } else {
                                        code_view_ui(ui, &body, "json");
                                    }
                                    ui.data_mut(|data| {
                                        data.insert_temp(req_id, (view_state, template_str))
                                    });
                                },
                            )
                        });

                        ui.add_enabled_ui(!*is_proxy, |ui| {
                            ui.columns(2, |cols| {
                                cols[0].label("转发服务器地址");
                                cols[1].add(
                                    egui::text_edit::TextEdit::singleline(dist_url)
                                        .hint_text("请输入转发服务器的全路径地址"),
                                );
                            });
                        });
                    });
            });
        });
    }
}

impl Into<MockDefine> for MockData {
    fn into(self) -> MockDefine {
        let id = 0;
        let mut req;
        if self.req.path.contains('?') {
            let path_query_split: Vec<&str> = self.req.path.split('?').collect();
            let path = path_query_split.first().unwrap().clone().to_string();
            let query = &(*path_query_split.get(1).unwrap()).clone();
            let query_params_m: HashMap<String, String> = query
                .split('&')
                .map(|qr| {
                    let qrs: Vec<String> = qr.split('=').map(|qk| qk.to_string()).collect();
                    qrs
                })
                .fold(HashMap::new(), |mut qm, qvc| {
                    if qvc.len() > 1 {
                        qm.insert(
                            qvc.get(0).unwrap().to_string(),
                            qvc.get(1).unwrap().to_string(),
                        );
                    }
                    qm
                });
            req = HttpMockRequest::new(path);
            req.query_params(query_params_m);
        } else {
            req = HttpMockRequest::new(self.req.path);
        }
        let headers = self
            .req
            .headers
            .into_iter()
            .filter(|selected_item| selected_item.selected)
            .fold(HashMap::new(), |mut map, head_item| {
                if !head_item.key.is_empty() && !head_item.value.is_empty() {
                    map.insert(head_item.key, head_item.value);
                    map
                } else {
                    map
                }
            });
        req.headers(headers);
        req.method(self.req.method.to_string());
        req.body(self.req.body.as_bytes().to_vec());

        let mock_ret = self.resp;

        let relay_url = if mock_ret.is_proxy {
            None
        } else {
            Some(mock_ret.dist_url)
        };

        let mut resp = MockServerHttpResponse::new();

        resp.body = Some(mock_ret.body.as_bytes().to_vec());

        resp.delay = Some(Duration::from_millis(mock_ret.delay.into()));

        resp.status = Some(mock_ret.code);

        let resp_headers = mock_ret
            .headers
            .into_iter()
            .filter(|selected_item| selected_item.selected)
            .fold(Vec::new(), |mut map, head_item| {
                if !head_item.key.is_empty() && !head_item.value.is_empty() {
                    map.push((head_item.key, head_item.value));
                    map
                } else {
                    map
                }
            });
        resp.headers = Some(resp_headers);
        let remark = self.req.remark;
        MockDefine {
            id,
            remark,
            req,
            resp,
            relay_url,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::{json, Map, Number, Value};

    #[test]
    fn test_json5() {
        let v =
            json5::to_string(&json!({"a": [null, true, 42, 42.42, f64::NAN, "hello"]})).unwrap();
        println!("{v}");
    }

    #[test]
    fn test_json5_str_format() {
        let config = r#" {// A traditional message.  
            message: 'hello world', 
            // A number for some reason.
            n: 42, } "#;

        println!("{config}");
        println!("===================");
        let f = json5format::Json5Format::new().unwrap();
        let d = json5format::ParsedDocument::from_str(config, None).unwrap();
        let s = f.to_string(&d).unwrap();
        println!("{s}");
    }
    #[test]
    fn test_json5_str() {
        let config = "
        {
            // A traditional message.
            message: 'hello world',

            // A number for some reason.
            n: 42,
            }
        ";

        let v = json5::from_str::<Value>(&config).unwrap();
        println!("{v}");
    }
}
