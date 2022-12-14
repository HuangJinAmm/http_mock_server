use std::collections::HashMap;
use std::ops::Index;
use std::sync::Arc;

use egui::{FontData, FontDefinitions, Id, RichText, Ui, Color32, TextStyle, style};
use egui_extras::{TableBuilder, Size};
use poll_promise::Promise;
use serde::__private::de::IdentifierDeserializer;

use crate::data::{HttpMockRequest, MockDefine, MockServerHttpResponse};
use crate::highlight::code_view_ui;

const APP_KEY: &str = "mock_server_web_ui_xxx";
const SERVER_URL:&str = "../_mock_list";
// const SERVER_URL:&str = "http://localhost:13001/_mock_list";
/// We derive Deserialize/Serialize so we can persist app state on shutdown.
pub struct TemplateApp {
    // Example stuff:
    label: String,
    filter: String,
    mock_info_list: Vec<MockDefine>,
    promise: Option<Promise<ehttp::Result<Vec<MockDefine>>>>,
}

impl Default for TemplateApp {
    fn default() -> Self {
        Self {
            filter: "".to_string(),
            // Example stuff:
            label: "".to_owned(),
            mock_info_list: Vec::new(),
            promise: None,
        }
    }
}

impl TemplateApp {
    /// Called once before the first frame.
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // This is also where you can customized the look at feel of egui using
        // `cc.egui_ctx.set_visuals` and `cc.egui_ctx.set_fonts`.
        let mut fonts = FontDefinitions::default();
        fonts.font_data.insert(
            "my_font".to_owned(),
            FontData::from_static(include_bytes!("MI_LanTing_Regular.ttf")),
        );
        fonts
            .families
            .entry(egui::FontFamily::Proportional)
            .or_default()
            .insert(0, "my_font".to_owned());

        fonts
            .families
            .entry(egui::FontFamily::Monospace)
            .or_default()
            .push("my_font".to_owned());
        
        cc.egui_ctx.set_fonts(fonts);
        
        let mut style = cc.egui_ctx.style().as_ref().clone();

        style.override_text_style = Some(TextStyle::Heading);

        cc.egui_ctx.set_style(Arc::new(style));

        let mut app = TemplateApp::default();
        let _promise = app.promise.get_or_insert_with(|| {
            // Begin download.
            // We download the image using `ehttp`, a library that works both in WASM and on native.
            // We use the `poll-promise` library to communicate with the UI thread.
            let ctx = cc.egui_ctx.clone();
            let (sender, promise) = Promise::new();
            let request = ehttp::Request::get(SERVER_URL);
            ehttp::fetch(request, move |response| {
                let mock_list = response.and_then(parse_response);
                sender.send(mock_list); // send the results back to the UI thread.
                ctx.request_repaint(); // wake up UI thread
            });
            promise
        });
        app
    }
}

impl eframe::App for TemplateApp {
    /// Called each time the UI needs repainting, which may be many times per second.
    /// Put your widgets into a `SidePanel`, `TopPanel`, `CentralPanel`, `Window` or `Area`.
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        if let Some(web_info) = frame.info().web_info.as_ref() {
            let mut hash = web_info.location.hash.clone();
            if hash.starts_with('#') {
                hash.remove(0);
            }
            self.label = hash;
        }

        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                egui::widgets::global_dark_light_mode_switch(ui);
                if ui.button("??????").clicked() {
                    //??????????????????
                    let promise_flash = {
                        let ctx = ctx.clone();
                        let (sender, promise) = Promise::new();
                        let request =
                            // ehttp::Request::get(format!("http://{}/_mock_list", "127.0.0.1:3000"));
                            ehttp::Request::get(SERVER_URL);
                        ehttp::fetch(request, move |response| {
                            let mock_list = response.and_then(parse_response);
                            sender.send(mock_list); // send the results back to the UI thread.
                            ctx.request_repaint(); // wake up UI thread
                        });
                        promise
                    };
                    self.promise = Some(promise_flash);
                }

                match self.promise.as_ref().unwrap().ready() {
                    None => {
                        ui.spinner(); // still loading
                    }
                    Some(Err(err)) => {
                        ui.colored_label(egui::Color32::RED, err); // something went wrong
                    }
                    Some(Ok(mocks)) => {
                        self.mock_info_list = mocks.clone();
                    }
                }
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.vertical(|ui| {
                ui.horizontal(|ui| {
                    ui.label("?????????");
                    let rsp = ui.text_edit_singleline(&mut self.filter);

                    if rsp.changed() {
                        let mut info = frame.info();
                        frame.set_window_title(&self.filter);
                        let mut web = info.web_info.unwrap();
                        web.location.hash = format!("#{}", &self.filter);
                        info.web_info = Some(web);
                    }
                });
                
            egui::ScrollArea::both()
                .id_source("respone_ui_scroller_1")
                .show(ui, |ui| {
                    for mock_define in self.mock_info_list.iter() {
                        let mut fm = String::new();
                        fm.push_str(mock_define.req.method.clone().unwrap_or_default().as_str());
                        fm.push_str(mock_define.req.path.clone().as_str());

                        if self.filter.is_empty() && self.label.is_empty() {
                            mock_define_ui(ui, mock_define);
                        } else {
                            if fm.contains(self.label.as_str()) {
                                if let Ok(re) = regex::Regex::new(&self.filter) {
                                    if re.is_match(fm.as_str()) {
                                        mock_define_ui(ui, mock_define);
                                    }
                                }
                            }
                        }
                    }
                });
            });
        });
    }
}

fn mock_define_ui(ui: &mut Ui, mock_define: &MockDefine) {
    let id_source = Id::new(mock_define.id);
    egui::collapsing_header::CollapsingState::load_with_default_open(ui.ctx(), id_source, true)
        .show_header(ui, |ui| {
            let method = mock_define
                .req
                .method
                .clone()
                .unwrap_or_else(|| "?????????".to_string());

            let dark_mode = ui.visuals().dark_mode;
            let faded_color = ui.visuals().window_fill();
            let faded_color = |color: Color32| -> Color32 {
                use egui::Rgba;
                let t = if dark_mode { 0.95 } else { 0.8 };
                egui::lerp(Rgba::from(color)..=Rgba::from(faded_color), t).into()
            };

            if let Some(_redict_to_url) = mock_define.relay_url.clone() {
                ui.painter().rect_filled(
                    ui.available_rect_before_wrap(),
                    0.5,
                    faded_color(Color32::GREEN),);
                ui.label("??????");
            } else {
                ui.painter().rect_filled(
                    ui.available_rect_before_wrap(),
                    0.5,
                    faded_color(Color32::GRAY),);
                    ui.label("??????");
            }
            ui.add_space(20.0);
            let l = (method.len() * 10 )as f32;
            ui.label(method_text_ui(method));
            ui.add_space(80.0 - l);
            ui.label(
                RichText::new(mock_define.get_url())
                    .size(18.0)
                    .underline()
                    .color(egui::Color32::BLUE),
            );

            // ui.label("????");
        })
        .body(|ui| mock_info_ui(ui, mock_define));
}

fn mock_info_ui(ui: &mut Ui, mock_define: &MockDefine) {

    crate::esay_md::easy_mark(ui, &mock_define.remark);
    // let remark = RichText::new(mock_define.remark.as_str()).background_color(Color32::LIGHT_GREEN);
    // ui.label(remark);
    ui.columns(2, |ui| {
        ui[0].group(|ui| mock_req_ui(ui, &mock_define.req,format!("{}-{}",mock_define.id, "req").as_str()));
        ui[1].group(|ui| {

        if let Some(url) = &mock_define.relay_url {
            ui.horizontal(|ui|{
                ui.label("????????????");
                ui.hyperlink(url);
            });
            mock_resp_ui(ui, &mock_define.resp,false);
        } else {
            mock_resp_ui(ui, &mock_define.resp,true);
        }
        });
    });
}
// fn header_vec_ui(ui: &mut Ui, map: &Vec<(String, String)>,id:&str) {
//     egui::Grid::new(id)
//         .num_columns(2)
//         .min_col_width(80.)
//         .min_row_height(20.)
//         .show(ui, |ui| {
//             for (key, value) in map {
//                 ui.label(key);
//                 ui.label(value);
//                 // ui.add_sized(ui.available_size(), egui::widgets::Label::new(key.clone()));
//                 // ui.add_sized(
//                 //     ui.available_size(),
//                 //     egui::widgets::Label::new(value.clone()),
//                 // );
//                 ui.end_row();
//             }
//         });
// }

fn vec_header_table(ui: &mut Ui, map: &Vec<(String, String)>) {
        TableBuilder::new(ui)
            .column(Size::remainder().at_least(100.0))
            .column(Size::remainder())
            .resizable(true)
            .scroll(false)
            .cell_layout(egui::Layout::left_to_right())
            // .header(20.0, |mut header| {
            //     header.col(|ui| {
            //         ui.heading("KEY");
            //     });
            //     header.col(|ui| {
            //         ui.heading("VALUE");
            //     });
            // })
            .body(|mut body| {
                
                body.rows(30.0, map.len(), |i,mut row|{
                    let (key,value) = map.index(i);
                    row.col(|ui| {
                        ui.label(key);
                    });
                    row.col(|ui| {
                        ui.label(value);
                    });
                });
            });
}

fn hashmap_header_table(ui: &mut Ui, map: &HashMap<String, String>) {
        TableBuilder::new(ui)
            .column(Size::remainder().at_least(100.0))
            .column(Size::remainder())
            .resizable(true)
            .scroll(false)
            .cell_layout(egui::Layout::left_to_right())
            // .header(20.0, |mut header| {
            //     header.col(|ui| {
            //         ui.heading("KEY");
            //     });
            //     header.col(|ui| {
            //         ui.heading("VALUE");
            //     });
            // })
            .body(|mut body| {
                for (key, value) in map {
                    body.row(30.0, |mut row| {
                        row.col(|ui| {
                            ui.label(key);
                        });
                        row.col(|ui| {
                            ui.label(value);
                        });
                    });
                }
            });
}

// fn hashmap_ui(ui: &mut Ui, map: &HashMap<String, String>,id:&str) {
//     egui::Grid::new(id)
//         .num_columns(2)
//         .min_col_width(80.)
//         .min_row_height(20.)
//         .show(ui, |ui| {
//             for (key, value) in map {
//                 ui.label(key);
//                 ui.label(value);
//                 // ui.add_sized(ui.available_size(), egui::widgets::Label::new(key.clone()));
//                 // ui.add_sized(
//                 //     ui.available_size(),
//                 //     egui::widgets::Label::new(value.clone()),
//                 // );
//                 ui.end_row();
//             }
//         });
// }

fn mock_resp_ui(ui: &mut Ui, resp: &MockServerHttpResponse,is_mock:bool) {
    let MockServerHttpResponse {
        status,
        headers,
        body,
        delay,
    } = resp;

    let status = status.unwrap_or(200);
    let delay = delay.unwrap_or_default();

    if is_mock {

        ui.horizontal(|ui|{
            ui.label("???????????????");
            ui.label("?????????");
            ui.label(status.to_string());
            ui.end_row();
            ui.label("???????????????");
            ui.label(delay.as_millis().to_string());
        });

    }
    if let Some(head_vec) = headers {
        if !head_vec.is_empty() {
            ui.label("????????????");
            ui.separator();
            ui.scope(|ui|{
                vec_header_table(ui, head_vec); 
            });
            ui.separator();
        }
    }
    if let Some(body) = body.clone() {
        if let Ok(body_string) = String::from_utf8(body) {
            ui.label("???????????????");
            code_view_ui(ui, &body_string, "json");
        }
    }

}

fn mock_req_ui(ui: &mut Ui, req: &HttpMockRequest,id:&str) {
    if let Some(header_map) = &req.headers {
        if !header_map.is_empty() {
            ui.label("????????????");
            ui.separator();
            hashmap_header_table(ui, header_map);
            ui.separator();
        }
    }
    if let Some(body) = req.body.clone() {
        if let Ok(body_string) = String::from_utf8(body) {
            ui.label("???????????????");
            code_view_ui(ui, &body_string, "json");
        }
    }
}

fn method_text_ui(method: String) -> RichText {
    let size = 18.0;
    match method.to_lowercase().as_str() {
        "get" => RichText::new(method).size(size).color(egui::Color32::GREEN),
        "post" => RichText::new(method).size(size).color(egui::Color32::RED),
        "put" => RichText::new(method).size(size).color(egui::Color32::GOLD),
        "delete" => RichText::new(method).size(size).color(egui::Color32::KHAKI),
        "option" => RichText::new(method).size(size).color(egui::Color32::BROWN),
        "trace" => RichText::new(method)
            .size(size)
            .color(egui::Color32::YELLOW),
        _ => RichText::new(method).size(size).color(egui::Color32::GRAY),
    }
}

#[allow(clippy::needless_pass_by_value)]
fn parse_response(response: ehttp::Response) -> Result<Vec<MockDefine>, String> {
    let body = response.bytes.as_slice();
    match serde_json::from_slice::<Vec<MockDefine>>(body) {
        Ok(mocks) => Ok(mocks),
        Err(e) => {
            let b = String::from_utf8_lossy(body).to_string();
            Err(e.to_string() + b.as_str())
        }
    }
}
