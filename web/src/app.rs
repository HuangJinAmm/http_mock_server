use crate::book::build_book;
use crate::history_db::add_new_version_mockinfo;
use crate::{
    api_context::ApiContext,
    component::tree_ui::{self, TreeUi},
    request_data::{MockData, RequestData, ResponseData},
};
use egui::{global_dark_light_mode_switch, Color32, FontData, FontDefinitions, Frame, Id, Window};
use egui_dock::{DockArea, DockState, Style, Tree};
use egui_file::{DialogType, FileDialog};
use egui_notify::Toasts;
use log::{debug, error, info};
use once_cell::sync::Lazy;
use once_cell::sync::OnceCell;
use reqwest::{Client, Request};
// use rhai::Scope;
use server::common::{mock::MockDefine, MOCK_SERVER};
use std::thread;
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

    pub tree: DockState<String>,
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
        let tabs = ["导航", "文档"].map(str::to_string).into_iter().collect();
        let dock_state = DockState::new(tabs);
        Self {
            show_log: false,
            tree: dock_state,
            api_data: api_context,
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
            let app: TemplateApp = eframe::get_value(storage, APP_KEY).unwrap_or_default();
            app.init_active();
            return app;
        }
        TemplateApp::default()
    }

    fn init_active(&self) {
        if let Some(ids) = self.api_data.tree_ui.get_all_active_nodes() {
            if let Ok(mut mock_server) = MOCK_SERVER.write() {
                for id in ids {
                    if let Some(mockdata) = self.api_data.tests.get(&id) {
                        let mut mock: MockDefine = mockdata.clone().into();
                        mock.id = id;
                        if !mock.req.path.is_empty() && mock.resp.body.is_some() {
                            match mock_server.add(mock, mockdata.req.priority.into()) {
                                Ok(_) => {
                                    debug!("id{}初始添加成功", id);
                                }
                                Err(e) => {
                                    //ignore
                                    debug!("id{}初始添加错误：{}", id, e);
                                }
                            };
                        }
                    }
                }
            }
        }
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
                                "文档".to_owned(),
                                "记录".to_owned(),
                                "日志".to_owned(),
                                "导航".to_owned(),
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

                if ui.button("build book").clicked() {
                    let tree_ui = self.api_data.tree_ui.clone();
                    let docs = self.api_data.docs.clone();
                    let tests = self.api_data.tests.clone();
                    thread::spawn(move || {
                        let build_res = build_book(tree_ui, docs, tests);
                        if let Ok(mut toast_w) = toast.lock() {
                            match build_res {
                                Ok(_) => {
                                    toast_w
                                        .info(format!("build success!"))
                                        .set_duration(Some(Duration::from_secs(5)));
                                }
                                Err(e) => {
                                    error!("{}", e);
                                    toast_w
                                        .info(format!("build success!"))
                                        .set_duration(Some(Duration::from_secs(5)));
                                    // toast_w
                                    //     .error(format!("build error:{}", e.to_string()))
                                    //     .set_duration(Some(Duration::from_secs(5)));
                                }
                            }
                        }
                    });
                }
            });
        });

        if let Some(dialog) = &mut self.open_file_dialog {
            if dialog.show(ctx).selected() {
                if let Some(file) = dialog.path() {
                    self.opened_file = Some(file.to_path_buf());
                    match dialog.dialog_type() {
                        DialogType::OpenFile => {
                            if let Ok(rfile) = std::fs::File::open(file.clone()) {
                                let reader = BufReader::new(rfile);
                                let app: ApiContext = serde_json::from_reader(reader).unwrap();
                                self.api_data = app;
                                // self.records = app.records;
                                // self.records_list = app.records_list;
                                // self.list_selected = app.list_selected;
                                // self.list_selected_str = app.list_selected_str;
                            }
                        }
                        DialogType::SaveFile => {
                            let app_json =
                                std::fs::File::open(file).unwrap_or_else(|_err| {
                                    std::fs::File::create(file).unwrap()
                                });
                            if let Err(err) = serde_json::to_writer_pretty(app_json, &self.api_data)
                            {
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
