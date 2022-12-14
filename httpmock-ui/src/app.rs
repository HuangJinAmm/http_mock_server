#![warn(non_snake_case)]
use std::any::Any;
use std::collections::BTreeMap;
use std::fs::{self, DirEntry};
use std::io::BufReader;
use std::path::PathBuf;
use std::process::id;
use std::str::FromStr;
use std::sync::mpsc::{Receiver};

use chrono::Local;
use egui::{FontData, FontDefinitions, Id, Label, Layout, vec2};

use httpmock_server::common::MOCK_SERVER;
use httpmock_server::common::mock::MockDefine;
use serde::{Deserialize, Serialize};

use crate::PORT;
use crate::component::context_list::Action::Selected;
use crate::component::context_list::Action::Delete;
use crate::component::context_list::Action::Keep;
use crate::component::context_list::Action::SyncToServer;
use crate::component::context_list::ContextTree;
use crate::component::mock_path_ui::MockDefineInfo;
use crate::esay_md::EasyMarkEditor;
use crate::history_db::{add_new_version_mockinfo, get_history_list, get_mock};

pub const ADD_ID_KEY: &str = "Http_mocker_recodes";
const APP_KEY: &str = "Http_mock_ui_server_xxx";
pub const ID_COUNT_KEY: &str = "Http_mocker_count_id";
const NOTIFICATION_KEY: &str = "http_mocker_Notice";
lazy_static! {
    static ref NOTIFICATION_ID: Id = Id::new(NOTIFICATION_KEY);
}
const NOTIFICATION_SHOW_TIME: i64 = 3000; //毫秒

#[warn(clippy::upper_case_acronyms)]
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum Method {
    GET,
    HEAD,
    POST,
    PUT,
    DELETE,
    CONNECT,
    OPTIONS,
    TRACE,
    PATCH,
    NONE,
}
impl Default for Method {
    fn default() -> Self {
        Method::GET
    }
}

impl Method {
    pub fn get_string(&self) -> String {
        match self {
            Method::NONE => "*".to_string(),
            _ => self.to_string(),
        }
    } 
}

impl FromStr for Method {
    type Err = String;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        match input {
            "GET" => Ok(Method::GET),
            "HEAD" => Ok(Method::HEAD),
            "POST" => Ok(Method::POST),
            "PUT" => Ok(Method::PUT),
            "DELETE" => Ok(Method::DELETE),
            "CONNECT" => Ok(Method::CONNECT),
            "OPTIONS" => Ok(Method::OPTIONS),
            "TRACE" => Ok(Method::TRACE),
            "PATCH" => Ok(Method::PATCH),
            _ => Err(format!("Invalid HTTP method {}", input)),
        }
    }
}

impl From<&str> for Method {
    fn from(value: &str) -> Self {
        value.parse().expect("Cannot parse HTTP method")
    }
}

impl std::fmt::Display for Method {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Debug::fmt(self, f)
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
enum AppTab {
    Mock,
    Req,
    // Test,
}

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct TemplateApp {
    // server_path:String,
    // is_server_path_edit:bool,
    is_exiting: bool,
    can_exit: bool,

    apptab: AppTab,
    label: String,
    is_pop: bool,
    filter: String,
    records_list: ContextTree,
    list_selected: u64,
    list_selected_str: Option<String>,
    records: BTreeMap<u64,MockDefineInfo>,
    #[serde(skip)]
    // history:Vec<PathBuf>,
    history:Vec<(String,u32)>,
    #[serde(skip)]
    add_reciever: Option<Receiver<(u64, u64)>>, // current: Option<ApiRecordDefinition>,
                                                // notifications:Vec<(u64, String)>,
                                                // method: Method,
                                                // left:ContextList,
                                                // this how you opt-out of serialization of a member
                                                // #[serde(skip)]
                                                // value: f32,
}

impl Default for TemplateApp {
    fn default() -> Self {
        Self {
            // server_path: "127.0.0.1:3000".to_string(),
            // is_server_path_edit:false,
            history:Vec::new(),
            add_reciever: None,
            is_exiting: false,
            can_exit: false,
            apptab: AppTab::Req,
            // netTestUi: Default::default(),
            label: "测试案例1".to_owned(),
            // value: 2.7,
            is_pop: false,
            records_list: unsafe { ContextTree::new(0, "HTTP测试") },
            list_selected_str: None,
            list_selected: 0,
            // notifications:Vec::new(),
            filter: "".into(),
            // current: None,
            // method: Method::GET,
            // notifications: VecDeque::new(),
            records: BTreeMap::new(),
        }
    }
}

// pub(crate) fn add_native_notifaction(notice: &str,title:&str,msg_type:rfd::MessageLevel) {
//     rfd::MessageDialog::new()
//         .set_level(msg_type)
//         .set_buttons(rfd::MessageButtons::Ok)
//         .set_description(notice)
//         .set_title(title)
//         .show();
// }

pub(crate) fn add_notification(ctx: &egui::Context, notice: &str) {
    let mut egui_data = ctx.data();
    let notice_vec: &mut Vec<(i64, String)> =
        egui_data.get_temp_mut_or_default(NOTIFICATION_ID.clone());
    let now = chrono::Local::now().timestamp_millis();
    notice_vec.push((now, notice.to_string()));
}

impl TemplateApp {
    fn display_notifications(&mut self, ctx: &egui::Context) {
        let mut offset = 22.;
        let notice_vec_clone;
        {
            let mut egui_data = ctx.data();
            let notice_vec: Vec<(i64, String)> =
                egui_data.get_temp(NOTIFICATION_ID.clone()).unwrap();
            notice_vec_clone = notice_vec.clone();
        }
        let now = chrono::Local::now().timestamp_millis();
        // let notice_own_vec = std::mem::take(notice_vec);
        notice_vec_clone
            .iter()
            .filter(|notice| notice.0 + NOTIFICATION_SHOW_TIME > now)
            .for_each(|notice| {

                if let Some(response) = egui::Window::new("通知")
                    .id(egui::Id::new(offset as u32))
                    .default_size(vec2(256.0, 256.0))
                    .collapsible(false)
                    .resizable(false)
                    .anchor(egui::Align2::RIGHT_TOP, (1.0, offset))
                    .show(ctx, |ui| {
                        ui.label(notice.1.clone());
                    })
                {
                    offset += response.response.rect.height();
                }
            });
        // *notice_vec = filted_notice_vec;

        // for (_, notification) in filted_notice_vec.iter() {
        //     if let Some(response) = egui::Window::new("通知")
        //         .id(egui::Id::new(offset as u32))
        //         .anchor(egui::Align2::RIGHT_TOP, (0., offset))
        //         .collapsible(false)
        //         .resizable(false)
        //         .show(ctx, |ui| {
        //             ui.label(notification);
        //         })
        //     {
        //         offset += dbg!(response.response.rect.height());
        //     }
        // }
        // for (_, error) in &self.errors {
        //     if let Some(response) = egui::Window::new("Error")
        //         .id(egui::Id::new(offset as u32))
        //         .anchor(egui::Align2::RIGHT_TOP, (0., offset))
        //         .collapsible(false)
        //         .resizable(false)
        //         .show(ctx, |ui| {
        //             ui.colored_label(egui::Color32::RED, error);
        //         })
        //     {
        //         offset += response.response.rect.height();
        //     }
        // }
    }

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

        // Load previous app state (if any).
        // Note that you must enable the `persistence` feature for this to work.

        let notice_id = NOTIFICATION_ID.clone();
        let notice_vec: Vec<(i64, String)> = Vec::new();
        cc.egui_ctx.data().insert_temp(notice_id, notice_vec);
        
        let (add_sender, add_reciever) = std::sync::mpsc::sync_channel::<(u64, u64)>(100);
        cc.egui_ctx
            .data()
            .insert_temp(Id::new(ADD_ID_KEY), add_sender);
        let mut app: TemplateApp = Default::default();
        app.add_reciever = Some(add_reciever);

        let mut app:TemplateApp = if let Some(storage) = cc.storage {
            log::debug!("加载存储数据==================");
            let mut app: TemplateApp = eframe::get_value(storage, APP_KEY).unwrap_or_default();
            app.is_exiting = false;
            app.can_exit = false;
            
            let all_ids = app.records_list.list_all_active_ids(); 
            if let Ok(mut mock_server) = MOCK_SERVER.write() {
                for id in all_ids {
                    if let Some(recode) = app.records.get(&id) {
                        let mut mock:MockDefine = recode.mock_define_info.clone().into();
                        mock.id = id; 
                        if let Err(e) = mock_server.add(mock) {
                            app.records_list.disable_item(id);
                            add_notification(&cc.egui_ctx, e.as_str());
                        }
                    }
                }
            }
            app
        } else {
            Default::default()
        };
        let (add_sender, add_reciever) = std::sync::mpsc::sync_channel::<(u64, u64)>(100);
        cc.egui_ctx
            .data()
            .insert_temp(Id::new(ADD_ID_KEY), add_sender);
        app.add_reciever = Some(add_reciever);
        app
    }
    //     let (add_sender, add_reciever) = std::sync::mpsc::sync_channel::<(u64, u64)>(100);
    //     cc.egui_ctx
    //         .data()
    //         .insert_temp(Id::new(ADD_ID_KEY), add_sender);
    //     println!("初始化==================");
    //     let mut app: TemplateApp = Default::default();
    //     // let path = app.server_path.clone();
    //     // thread::spawn(move ||{
    //     //     tokio::runtime::Builder::new_multi_thread().worker_threads(1)
    //     //     .enable_all().build().unwrap().block_on(async {
    //     //         log::info!("启动....");
    //     //         let _ = httpmock_web::serve(path.as_str()).await;
    //     //     });
    //     // });
    //     app.add_reciever = Some(add_reciever);
    //     app

    //     if let Some(file_name) = find_newest_backjson(".") {
    //         println!("====>加载文件<====={}",file_name.to_str().unwrap());
    //         let _ = load_app(file_name, &mut app);
    //     }

    //     let all_ids = app.records_list.list_all_active_ids(); 
    //     if let Ok(mut mock_server) = MOCK_SERVER.write() {
    //         for id in all_ids {
    //             if let Some(recode) = app.records.get(&id) {
    //                 let mut mock:MockDefine = recode.mock_define_info.clone().into();
    //                 mock.id = id; 
    //                 let _ =mock_server.add(mock);
    //             }
    //         }
    //     }
    //     app
    // }
}

impl eframe::App for TemplateApp {
    /// Called by the frame work to save state before shutdown.
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, APP_KEY, self);
    }

    fn on_exit_event(&mut self) -> bool {

        let backup_name = get_backup_name("app_mock");
        log::debug!("保存文件=>{}",backup_name);
        let _ = backup_app(self,&backup_name);
        true
        // self.is_exiting = true;
        // self.can_exit
    }

    /// Called each time the UI needs repainting, which may be many times per second.
    /// Put your widgets into a `SidePanel`, `TopPanel`, `CentralPanel`, `Window` or `Area`.
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        //显示通知
        self.display_notifications(ctx);

        if let Ok((sup, sub)) = self.add_reciever.as_ref().unwrap().try_recv() {
            let sup_record = self.records.get(&sup).unwrap();
            let sub_record = sup_record.clone();
            self.records.insert(sub, sub_record);
        }

        // Examples of how to create different panels and windows.
        // Pick whichever suits you.
        // Tip: a good default choice is to just keep the `CentralPanel`.
        // For inspiration and more examples, go to https://emilk.github.io/egui
        if !ctx.input().raw.dropped_files.is_empty() {
            let dropped_files = ctx.input().raw.dropped_files.clone();
            for file in dropped_files {
                if let Some(file_p) = file.path {
                    if let Some(ext) = file_p.extension() {
                        if ext == "json" {
                            load_app(file_p, self,ctx);
                        }
                    }
                }
            }
        }
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            // The top panel is often a good place for a menu bar:
            egui::menu::bar(ui, |ui| {
                egui::widgets::global_dark_light_mode_switch(ui);
                ui.menu_button("菜单", |ui| {
                    if ui.button("退出").clicked() {
                        frame.quit();
                    }
                    // if ui
                    //     .button("加载json文件")
                    //     .on_hover_text("将加载与app同路径下的app_mock.json文件")
                    //     .clicked()
                    // {
                    //     let ok = rfd::MessageDialog::new()
                    //         .set_level(rfd::MessageLevel::Info)
                    //         .set_buttons(rfd::MessageButtons::OkCancel)
                    //         .set_description("将加载与app同路径下的app_mock-xxx.json文件")
                    //         .set_title("加载文件")
                    //         .show();
                    //     if ok {
                    //         if let Ok(file) = std::fs::File::open("app_mock.json") {
                    //             let reader = BufReader::new(file);
                    //             let app: TemplateApp = serde_json::from_reader(reader).unwrap();
                    //             self.list_selected = app.list_selected;
                    //             self.list_selected_str = app.list_selected_str;
                    //             self.records = app.records;
                    //             self.records_list = app.records_list;
                    //         }
                    //     }
                    // }
                    if ui.button("保存为json文件").clicked() {
                        let file_name = get_backup_name("app_mock");
                        match backup_app(self, &file_name) {
                            Ok(_) => {
                                rfd::MessageDialog::new()
                                    .set_level(rfd::MessageLevel::Info)
                                    .set_buttons(rfd::MessageButtons::Ok)
                                    .set_description(format!("已将app信息保存在{}文件中", file_name).as_str())
                                    .set_title("保存文件")
                                    .show();
                            },
                            Err(e) => {
                                rfd::MessageDialog::new()
                                    .set_level(rfd::MessageLevel::Error)
                                    .set_buttons(rfd::MessageButtons::Ok)
                                    .set_description(e.to_string().as_str())
                                    .set_title("保存文件")
                                    .show();
                            },
                        }
                    }

                    if ui.button("选择json文件…").clicked() {
                        if let Some(path) = rfd::FileDialog::new()
                            .add_filter("app存储文件", &["json", "JSON"])
                            .set_file_name("app")
                            .set_directory(".")
                            .pick_file()
                        {
                            if load_app(path, self,ctx) {
                                add_notification(ctx, "加载成功");
                            } else {
                                add_notification(ctx, "加载失败");
                            }

                        }
                    }

                    // if ui.button("清除所有记录").clicked() {
                    //     self.is_pop=true;
                    // }
                    // if self.is_pop {
                    // egui::Window::new("警告")
                    //     .collapsible(false)
                    //     .resizable(false)
                    //     .fixed_size([80.,140.])
                    //     .anchor(egui::Align2::CENTER_CENTER, [0.0,0.0])
                    //     .show(ctx, |ui|{
                    //         ui.add_space(20.);
                    //         ui.label("是否清除所有记录?");
                    //         ui.add_space(20.);
                    //         ui.horizontal(|ui|{
                    //             if ui.button("是").clicked() {
                    //                 self.reset();
                    //                 self.is_pop = false;
                    //             }
                    //             if ui.button("否").clicked() {
                    //                 self.is_pop = false;
                    //             }

                    //         });
                    //     });
                    // }
                });
                // ui.label("服务器地址:");
                // editable_label(ui,&mut self.is_server_path_edit,&mut self.server_path);
                // ui.selectable_value(apptab, AppTab::Mock, "模拟");
                // ui.selectable_value(apptab, AppTab::Req, "请求");
                // ui.add(toggle(is_login));
                // ui.toggle_value(is_login, "历史记录");
                // ui.selectable_value(apptab, AppTab::Test, "测试");
                ui.with_layout(Layout::right_to_left(), |ui| {
                    if ui.selectable_label(self.is_exiting, "历史记录").clicked() {
                        self.is_exiting = !self.is_exiting;
                        if self.is_exiting {
                            // self.history = list_backjson(".", "app_mock");
                            self.history = get_history_list(self.list_selected);
                        }
                    }
                });
            });
        });

        egui::SidePanel::left("side_panel").show(ctx, |ui| {
            if ui
                .interact(
                    ui.available_rect_before_wrap(),
                    ui.id(),
                    egui::Sense::drag(),
                )
                .drag_released()
            {}

            ui.add(egui::TextEdit::singleline(&mut self.filter).hint_text("筛选条件"));
            let list_resp = self.records_list.ui_impl(
                ui,
                self.list_selected,
                &mut self.label,
                &mut self.filter,
            );
            match list_resp {
                Selected((id, title)) => {
                    self.list_selected = id;
                    self.list_selected_str = Some(title)
                }
                Delete(subids) => {
                    if let Ok(mut mock_server) = MOCK_SERVER.write() {
                        subids.iter().for_each(|id|{
                            if let Some(mock_define) = self.records.remove(id) {
                                let mock:MockDefine = mock_define.mock_define_info.into();
                                mock_server.delete(mock)
                            }
                        });
                    } else {
                        add_notification(ctx, "删除失败，请稍后再试");
                    }
                },
                SyncToServer((id,sync_bool)) => {
                    if sync_bool {
                        if let Some(mock_def) = self.records.get(&id) {
                            if let Ok(mut mock_server) = MOCK_SERVER.write() {
                                let mut mock:MockDefine = mock_def.mock_define_info.clone().into();
                                mock.id = id;
                                if mock.req.path.is_empty() 
                                    || mock.resp.body.is_none() {
                                    add_notification(ctx, "添加失败！\n路径或者响应为空");
                                    self.records_list.disable_item(id);
                                } else {
                                    match mock_server.add(mock) {
                                        Ok(_) => {
                                            add_new_version_mockinfo(id, &mock_def.mock_define_info);
                                            add_notification(ctx, "添加成功！");
                                            // add_native_notifaction("添加成功","成功",rfd::MessageLevel::Info);
                                        },
                                        Err(e) => {
                                            add_notification(ctx, e.as_str());
                                            self.records_list.disable_item(id);
                                        },
                                    }
                                }
                            } else {
                                add_notification(ctx, "添加失败！获取锁失败");
                            }
                        }
                    } else if let Some(mock_def) = self.records.get(&id) {

                        if let Ok(mut mock_server) = MOCK_SERVER.write() {
                            let mut mock:MockDefine = mock_def.mock_define_info.clone().into();
                            mock.id = id;
                            mock_server.delete(mock);
                            add_notification(ctx, "删除成功！");
                        } else {
                            add_notification(ctx, "删除失败！获取锁失败");
                        }
                    }
                },
                Keep => {},
            }

            // ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
            //     ui.horizontal(|ui| {
            //         ui.spacing_mut().item_spacing.x = 0.0;
            //         ui.label("powered by ");
            //         ui.hyperlink_to("egui", "https://github.com/emilk/egui");
            //         ui.label(" and ");
            //         ui.hyperlink_to("eframe", "https://github.com/emilk/egui/tree/master/eframe");
            //     });
            // });
        });

        if self.is_exiting {
            egui::SidePanel::right("right_panel").show(ctx, |ui| {
                ui.label("历史记录");
                egui::ScrollArea::vertical().show(ui, |ui|{
                    for (ver_name,ver) in self.history.clone() {
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
            });
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.horizontal(|ui|{
                ui.heading(
                    self.list_selected_str
                        .clone()
                        .unwrap_or_else(||"未命名".into())
                        .as_str(),
                );
                ui.selectable_value(&mut self.apptab,AppTab::Mock, "文档说明");
                ui.selectable_value(&mut self.apptab,AppTab::Req, "模拟设置");
            });
            let net_ui = if let Some(net) = self.records.get_mut(&self.list_selected) {
                net
            } else {
                let net_ui = MockDefineInfo::default();
                self.records.insert(self.list_selected.to_owned(), net_ui);
                self.records.get_mut(&self.list_selected).unwrap()
            };
            match self.apptab {
                AppTab::Mock => {
                    ui.horizontal(|ui|{
                        if net_ui.mock_define_info.is_edit {
                            if ui.button("编辑").clicked(){
                                net_ui.mock_define_info.is_edit = !net_ui.mock_define_info.is_edit;
                            }
                        } else {
                            if ui.button("预览").clicked(){
                                net_ui.mock_define_info.is_edit = !net_ui.mock_define_info.is_edit;
                            }
                        }
                        crate::esay_md::nested_hotkeys_ui(ui);
                    });
                    let mut md_editor = EasyMarkEditor::default();
                    md_editor.ui(ui, &mut net_ui.mock_define_info.remark, &mut net_ui.mock_define_info.is_edit);
                }
                AppTab::Req => {
                    net_ui.ui(ui);
                } 
            }
        });

        // if self.is_exiting {
        // let ok = rfd::MessageDialog::new()
        //     .set_level(rfd::MessageLevel::Info)
        //     .set_buttons(rfd::MessageButtons::YesNo)
        //     .set_description("是否退出应用？")
        //     .set_title("退出")
        //     .show();
        // if ok {
        //     self.can_exit = true;
        //     frame.quit();
        // }
        // egui::Window::new("确认退出?")
        //     .collapsible(false)
        //     .resizable(false)
        //     .show(ctx, |ui| {
        //         ui.horizontal(|ui| {
        //             if ui.button("暂不").clicked() {
        //                 self.is_exiting = false;
        //             }

        //             if ui.button("是的").clicked() {
        //                 self.can_exit = true;
        //                 frame.quit();
        //             }
        //         });
        //     });
        // }

        // if !*is_login {
        //     egui::Window::new("登录")
        //         .collapsible(false)
        //         .resizable(false)
        //         // .open(&mut false)
        //         .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0f32, 0f32))
        //         .show(ctx, |ui| {
        //             egui::Grid::new("login_grid")
        //                 .num_columns(2)
        //                 .spacing([40.0, 4.0])
        //                 // .striped(true)
        //                 .show(ui, |ui| {
        //                     ui.label("用户名：");
        //                     ui.text_edit_singleline(label).on_hover_text("请输入用户名");
        //                     ui.end_row();

        //                     ui.label("密  码：");
        //                     ui.add(password(label));
        //                     ui.end_row();

        //                     ui.label("验证码：");
        //                     ui.text_edit_singleline(&mut "请输入验证码");
        //                     ui.end_row();

        //                     ui.add_visible(false, egui::Label::new("zhanwein"));
        //                     ui.horizontal(|ui| {
        //                         let login_click = ui.button("登    录");
        //                         let _regist_click = ui.button("注    册");
        //                         let _forget_password = ui.button("忘记密码");

        //                         if login_click.clicked() {
        //                             *is_login = true;
        //                         }
        //                     })
        //                 });
        //         });
        // }
    }
}

// fn api_url_ui(method: &str, url: &str, mode: bool, font_size: f32) -> LayoutJob {
//     let mut job = LayoutJob::default();

//     let (default_color, strong_color, bg_color) = if mode {
//         (Color32::LIGHT_GRAY, Color32::WHITE, Color32::DARK_RED)
//     } else {
//         (Color32::DARK_GRAY, Color32::BLACK, Color32::LIGHT_RED)
//     };
//     let font = FontId::new(font_size, egui::FontFamily::Proportional);

//     job.append(
//         method,
//         0.0,
//         TextFormat {
//             color: strong_color,
//             font_id: font.clone(),
//             background: bg_color,
//             ..Default::default()
//         },
//     );

//     job.append(
//         url,
//         0.0,
//         TextFormat {
//             color: default_color,
//             font_id: font,
//             ..Default::default()
//         },
//     );
//     job
// }



fn backup_app(app:&TemplateApp,file_name:&str) -> Result<(),String> {
    let app_json = std::fs::File::open(file_name)
        .unwrap_or_else(|_err| std::fs::File::create(file_name).unwrap());
    match serde_json::to_writer_pretty(app_json, app) {
        Ok(_) => Ok(()),
        Err(e) => Err(e.to_string())
    }
}

fn get_backup_name(app_name:&str) -> String {
    let local = Local::now();
    let fmt_data = local.format("%Y%m%d");
    format!("{}-{}-{}.json",app_name,PORT.to_string(),fmt_data.to_string())
}

fn load_app(file:PathBuf,app:&mut TemplateApp,ctx:&egui::Context) -> bool {
    if let Ok(file) = std::fs::File::open(file) {
        let reader = BufReader::new(file);
        let new_app: TemplateApp = serde_json::from_reader(reader).unwrap();
        app.list_selected = new_app.list_selected;
        app.list_selected_str = new_app.list_selected_str;
        app.records = new_app.records;
        app.records_list = new_app.records_list;
        app.records_list.list_all_active_ids().into_iter().for_each(|id|app.records_list.disable_item(id));

        {
            let mut data =ctx.data();
            data.insert_persisted(Id::new(ID_COUNT_KEY), app.records_list.max_id());
        }
        true
    } else {
        false
    }
}

// fn list_backjson(path:&str,app_name:&str) -> Vec<PathBuf> {
//     let entries = fs::read_dir(path).unwrap();
//     let name_p = format!("{}-{}", app_name,PORT);
//     let mut back_json_files:Vec<DirEntry> = entries.filter(
//         |ent| 
//             ent.as_ref().ok()
//                 .map(|en| en.path())
//                 .map(|path| {
//                     path.exists()
//                     && path.extension().map(|ext|ext == "json").unwrap_or(false)
//                     && path.file_name().map(
//                                                 |name|name.to_str()
//                                                                     .map(
//                                                                             |n|n.starts_with(name_p.as_str())
//                                                                         ).unwrap_or(false)
//                                                                 ).unwrap_or(false)
//                 }).unwrap_or(false)
//     )
//     .map(|ent|ent.unwrap())
//     .collect();
//     back_json_files.sort_by(|e1,e2|{
//         e1.metadata().unwrap().modified().unwrap().cmp(&e2.metadata().unwrap().modified().unwrap())
//     });
//     dbg!(back_json_files.into_iter().map(|ent|ent.path()).collect())
// }
// fn find_newest_backjson(path:&str) -> Option<PathBuf> {
//     let entries = fs::read_dir(path).unwrap();
//     entries.filter(
//         |ent| ent.as_ref().map(
//             |en| en.metadata()
//         ).map(
//             |meta|meta.map(
//                 |me|me.modified())
//         ).is_ok()
//     ).map(|ent|ent.unwrap())
//     .max_by(|e1,e2|{
//         e1.metadata().unwrap().modified().unwrap().cmp(&e2.metadata().unwrap().modified().unwrap())
//     }).map(|ent|ent.path())
// }