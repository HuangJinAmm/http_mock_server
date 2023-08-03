use crate::history_db::add_new_version_mockinfo;
use crate::{
    api_context::ApiContext,
    component::tree_ui::{self, TreeUi},
    request_data::{MockData, RequestData, ResponseData},
};
use egui::{global_dark_light_mode_switch, Color32, FontData, FontDefinitions, Frame, Id, Window};
use egui_dock::{DockArea, Style, Tree};
use egui_file::{DialogType, FileDialog};
use egui_notify::Toasts;
use log::info;
use once_cell::sync::Lazy;
use once_cell::sync::OnceCell;
use reqwest::{Client, Request};
// use rhai::Scope;
use server::common::{mock::MockDefine, MOCK_SERVER};
use std::time::Duration;
use std::{io::BufReader, sync::Mutex};
use std::{path::PathBuf, sync::Arc};
use tokio::{
    runtime::Runtime,
    sync::mpsc::{Receiver, Sender},
};
/**
 * 全局变量
 */
const TEMP_GLOBAL_KEY: &str = "PRE_HTTP";
const APP_KEY: &str = "egui-http-mock-server";

static TABS: OnceCell<Vec<String>> = OnceCell::new();
// id.times
pub static mut TASK_CHANNEL: Lazy<(Sender<(u64, u32, u32)>, Receiver<(u64, u32, u32)>)> =
    Lazy::new(|| tokio::sync::mpsc::channel(100));
// id,time
// pub static mut RESULTE_CHANNEL: Lazy<(
//     Sender<(u64, i64, ResponseData)>,
//     Receiver<(u64, i64, ResponseData)>,
// )> = Lazy::new(|| tokio::sync::mpsc::channel(100));
// pub static mut M_RESULTE_CHANNEL: Lazy<(
//     Sender<(u64, usize, i64, ResponseData)>,
//     Receiver<(u64, usize, i64, ResponseData)>,
// )> = Lazy::new(|| tokio::sync::mpsc::channel(100));
pub static TOASTS: OnceCell<Arc<Mutex<Toasts>>> = OnceCell::new();
pub static TOKIO_RT: Lazy<Runtime> = Lazy::new(|| {
    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        // .worker_threads(16)
        .build()
        .unwrap()
});
// pub static mut CLIENT: Lazy<Client> = Lazy::new(|| {
//     Client::builder()
//         .danger_accept_invalid_certs(true)
//         .build()
//         .unwrap_or_default()
// });

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct TemplateApp {
    show_log: bool,

    test: String,
    // text: DockUi
    // pub tabs: HashSet<String>,
    pub tree: Tree<String>,
    tree_ui: TreeUi,
    api_data: ApiContext,
    #[serde(skip)]
    opened_file: Option<PathBuf>,
    #[serde(skip)]
    open_file_dialog: Option<FileDialog>,
    // #[serde(skip)]
    // script_engine: ScriptEngine,
}

impl Default for TemplateApp {
    fn default() -> Self {
        let mut api_context = ApiContext::new();
        api_context.insert_collecton(0, "".to_owned());
        Self {
            show_log: false,
            test: "".to_owned(),
            tree_ui: TreeUi::new(),
            // tabs:vec![],
            tree: Tree::new(vec![]),
            api_data: api_context,
            // script_engine: ScriptEngine::new(),
            open_file_dialog: None,
            opened_file: None,
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

        // This is also where you can customize the look and feel of egui using
        // `cc.egui_ctx.set_visuals` and `cc.egui_ctx.set_fonts`.

        // Load previous app state (if any).
        // Note that you must enable the `persistence` feature for this to work.
        if let Some(storage) = cc.storage {
            let app = eframe::get_value(storage, APP_KEY).unwrap_or_default();
            return app;
        }
        TemplateApp::default()
    }

    // pub fn load_ui(&mut self,title: String,tab_ui: Box<dyn TabUi<T = ApiContext>>) {
    //     self.tabs.insert(title, tab_ui);
    // }

    // pub fn register_ui(&mut self, title: String, tab_ui: Box<dyn TabUi<T = ApiContext>>) {
    //     let _ = self.tabs.insert(title.clone(), tab_ui);
    //     self.tree.push_to_focused_leaf(title);
    // }

    pub fn trigger_tab(&mut self, tab: &String) {
        if let Some(index) = self.tree.find_tab(tab) {
            self.tree.remove_tab(index);
        } else {
            self.tree.push_to_focused_leaf(tab.clone());
        }
    }

    pub fn open_tab(&mut self, tab: &String) {
        if let None = self.tree.find_tab(tab) {
            self.tree.push_to_focused_leaf(tab.clone());
        }
    }

    pub fn is_open(&self, title: &String) -> bool {
        if let Some(_index) = self.tree.find_tab(title) {
            true
        } else {
            false
        }
    }

    pub fn close_tab(&mut self, title: &String) -> bool {
        if let Some(index) = self.tree.find_tab(title) {
            let _rm = self.tree.remove_tab(index);
        }
        true
    }
}

impl eframe::App for TemplateApp {
    /// Called by the frame work to save state before shutdown.
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, APP_KEY, self);
    }

    /// Called each time the UI needs repainting, which may be many times per second.
    /// Put your widgets into a `SidePanel`, `TopPanel`, `CentralPanel`, `Window` or `Area`.
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        // Examples of how to create different panels and windows.
        // Pick whichever suits you.
        // Tip: a good default choice is to just keep the `CentralPanel`.
        // For inspiration and more examples, go to https://emilk.github.io/egui

        if ctx.style().visuals.dark_mode {
            catppuccin_egui::set_theme(&ctx, catppuccin_egui::FRAPPE);
            // } else {
            //     catppuccin_egui::set_theme(&ctx, catppuccin_egui::LATTE);
        }
        let toast = TOASTS.get_or_init(|| {
            Arc::new(Mutex::new(
                Toasts::default().with_anchor(egui_notify::Anchor::BottomRight),
            ))
        });
        if self.show_log {
            Window::new("Log").title_bar(true).show(ctx, |ui| {
                // draws the logger ui.
                egui_logger::logger_ui(ui);
            });
        }

        // #[cfg(not(target_arch = "wasm32"))] // no File->Quit on web pages!
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            // The top panel is often a good place for a menu bar:
            egui::menu::bar(ui, |ui| {
                global_dark_light_mode_switch(ui);

                #[cfg(not(target_arch = "wasm32"))] // no File->Quit on web pages!
                ui.menu_button("File", |ui| {
                    if (ui.button("Import")).clicked() {
                        let mut dialog = FileDialog::open_file(self.opened_file.clone())
                            .show_rename(false)
                            .filter(Box::new(|p| p.to_string_lossy().ends_with("json")));
                        dialog.open();
                        self.open_file_dialog = Some(dialog);
                    }

                    if (ui.button("Export")).clicked() {
                        let mut dialog = FileDialog::save_file(self.opened_file.clone())
                            .default_filename("app.json");
                        dialog.open();
                        self.open_file_dialog = Some(dialog);
                    }

                    if ui.button("Quit").clicked() {
                        frame.close();
                    }
                });
                if ui.selectable_label(self.show_log, "打开日志").clicked() {
                    self.show_log = !self.show_log;
                }
                if !frame.is_web() {
                    ui.menu_button("Zoom", |ui| {
                        egui::gui_zoom::zoom_menu_buttons(ui, frame.info().native_pixels_per_point);
                    });
                }

                ui.menu_button("View", |ui| {
                    // allow certain tabs to be toggled
                    for tab in TABS
                        .get_or_init(|| {
                            vec![
                                "请求".to_owned(),
                                "响应".to_owned(),
                                // "设置".to_owned(),
                                "文档".to_owned(),
                                "记录".to_owned(),
                                "脚本".to_owned(),
                            ]
                        })
                        .iter()
                    {
                        if ui
                            .selectable_label(self.is_open(tab), tab.clone())
                            .clicked()
                        {
                            self.trigger_tab(tab);
                            ui.close_menu();
                        }
                    }
                });
            });
        });

        if let Some(dialog) = &mut self.open_file_dialog {
            if dialog.show(ctx).selected() {
                if let Some(file) = dialog.path() {
                    self.opened_file = Some(file.clone());
                    match dialog.dialog_type() {
                        DialogType::OpenFile => {
                            if let Ok(rfile) = std::fs::File::open(file.clone()) {
                                let reader = BufReader::new(rfile);
                                let app: TemplateApp = serde_json::from_reader(reader).unwrap();
                                *self = app;
                                // self.records = app.records;
                                // self.records_list = app.records_list;
                                // self.list_selected = app.list_selected;
                                // self.list_selected_str = app.list_selected_str;
                            }
                        }
                        DialogType::SaveFile => {
                            let app_json =
                                std::fs::File::open(file.clone()).unwrap_or_else(|_err| {
                                    std::fs::File::create(file.clone()).unwrap()
                                });
                            if let Err(err) = serde_json::to_writer_pretty(app_json, self) {
                                if let Ok(mut toast_w) = toast.lock() {
                                    toast_w
                                        .error(format!("save file error:{}", err.to_string()))
                                        .set_duration(Some(Duration::from_secs(5)));
                                }
                            } else {
                                if let Ok(mut toast_w) = toast.lock() {
                                    toast_w
                                        .info(format!(
                                            "file saved success:{}",
                                            file.to_string_lossy()
                                        ))
                                        .set_duration(Some(Duration::from_secs(5)));
                                }
                            }
                        }
                        _ => {}
                    }
                }
            }
        }

        egui::SidePanel::left("side_panel")
            .max_width(240.0)
            .show(ctx, |ui| {
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
                                    self.api_data.delete_collecton(del_id);
                                    if let Some(mock_define) = self.api_data.delete_test(del_id) {
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
                                    self.api_data.insert_collecton(add_id, "".to_string());
                                }
                                tree_ui::NodeType::Node => {
                                    self.api_data.insert_test(add_id, MockData::default());
                                }
                            }
                        }
                        tree_ui::Action::Rename(_adds) => {
                            //基本上不用处理
                            info!("重命名")
                        }
                        tree_ui::Action::Selected((selected_id, selected_title)) => {
                            let selected = *selected_id.first().unwrap_or(&0);
                            self.api_data.selected = selected_id;
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
                                if let Some(copyed) = self.api_data.tests.get(&sid) {
                                    let parse = copyed.clone();
                                    self.api_data.insert_test(did, parse);
                                }
                            }
                        }
                        tree_ui::Action::SyncToServer((id, active)) => {
                            let mut msg = String::new();
                            if let Some(mockdata) = self.api_data.tests.get(&id) {
                                
                                if let Ok(mut mock_server) = MOCK_SERVER.write() {
                                    if active {
                                        let mut mock: MockDefine = mockdata.clone().into();
                                        mock.id = id;
                                        if mock.req.path.is_empty() || mock.resp.body.is_none() {
                                            msg = "添加失败：路径或者响应为空".to_owned();
                                        } else {
                                            msg = match mock_server.add(mock) {
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
                //    });
            });

        egui::CentralPanel::default()
            .frame(Frame::central_panel(&ctx.style()).inner_margin(0.))
            .show(ctx, |ui| {
                let mut dst = Style::from_egui(ui.style());

                dst.separator.color_dragged = Color32::RED;
                // dst.tab_text_color_active_focused = Color32::BROWN;
                DockArea::new(&mut self.tree)
                    .style(dst)
                    .show_inside(ui, &mut self.api_data);
            });

        if let Ok(mut toast_w) = toast.lock() {
            toast_w.show(ctx);
        }
    }
}
