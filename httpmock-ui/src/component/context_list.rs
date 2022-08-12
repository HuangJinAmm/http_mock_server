#![warn(non_snake_case)]
use crate::app::{ADD_ID_KEY, ID_COUNT_KEY};
use egui::{Id, InnerResponse, Ui, Color32};
use std::{collections::BTreeMap, sync::mpsc::SyncSender};

#[derive(Clone, PartialEq,Eq, Debug)]
pub enum Action {
    Keep,
    Delete(Vec<u64>),
    Selected((u64, String)),
    SyncToServer((u64,bool))
}

/*
树状列表中的元素:
*/
#[derive(Clone, Debug)]
// #[cfg_attr(feature = "serde", derive(serde::Deserialize,serde::Serialize))]
#[derive(serde::Deserialize, serde::Serialize)]
pub struct ContextTree {
    id: u64,
    title: String,
    is_sync: bool,
    selected: bool,
    sub_items: BTreeMap<u64, ContextTree>,
}

impl ContextTree {
    //添加子项目
    pub fn add_item(&mut self, id: u64, title: &str) {
        let sub = ContextTree::new(id, title);
        self.sub_items.insert(id, sub);
    }

    pub fn delete_item(&mut self, id: &u64) -> Vec<u64> {
        match self.sub_items.remove(id) {
            Some(contree) => {
                // let predict = |ct:&ContextTree| ct.is_sync;
                contree.list_all_subids().unwrap_or_default()
            },
            None => Vec::new(),
        }
    }

    pub fn disable_item(&mut self,id: u64) {
        if self.id == id {
            self.is_sync = false;
        } else {
            self.disable_sub_item(id);
        }
    }

    fn disable_sub_item(&mut self,id: u64) {
        match self.sub_items.get_mut(&id) {
            Some(sub) => {
                sub.disable_item(id)
            },
            None => {},
        }
    }

    pub fn list_all_active_ids(&self) -> Vec<u64> {
        let subs = self.sub_items.clone();
        let mut active_ids = Vec::new();

        if self.is_sync {
            active_ids.push(self.id);
        }

        let mut sub_ids:Vec<u64> = subs.values()
                    .filter( |ct|ct.is_sync)
                    // .flat_map(|ct|ct.list_all_subids())
                    .flat_map(|ct| {
                        let subs = ct.list_all_active_ids();
                        subs
                    })
                    .collect();
        active_ids.append(&mut sub_ids);
        active_ids
    }

    fn list_all_subids(&self) -> Option<Vec<u64>> {
        let subs = self.sub_items.clone();
        if subs.is_empty() {
            None
        } else {
            let sub_ids = subs.values()
                        // .filter( |ct|predicate(*ct))
                        // .flat_map(|ct|ct.list_all_subids())
                        .flat_map(|ct| {
                            let mut all_sub_id = vec![self.id];
                            if let Some(mut subs) = ct.list_all_subids() {
                                all_sub_id.append(&mut subs);
                            }
                            all_sub_id
                        })
                        .collect();
            Some(sub_ids)
        }
    }

    pub fn max_id(&self) -> u64 {
        if let Some(subids) = self.list_all_subids() {
            if let Some(max_id) = subids.iter().max() {
                return max_id.to_owned();
            }
        }
        return self.id;
    }

    pub fn new(id: u64, title: &str) -> Self {
        Self {
            id,
            selected: false,
            is_sync: false,
            title: String::from(title),
            sub_items: BTreeMap::new(),
        }
    }

    pub fn ui_impl(
        &mut self,
        ui: &mut Ui,
        selected_str: u64,
        add_title: &mut String,
        flilter: &str,
    ) -> Action {
        let id_source = ui.make_persistent_id(self.id.to_string());
        self.selected = selected_str == self.id;
        //删除
        if ui.input().key_pressed(egui::Key::Delete) {
            let del_ids = self.delete_item(&selected_str);
            if !del_ids.is_empty() {
                return Action::Delete(del_ids); 
            }
        }

        if self.sub_items.is_empty() {
            ui.horizontal(|ui| {
                // ui.label("📄");
                //显示ID
                // ui.label(
                //     RichText::new("(🆔:".to_owned() + &self.id.to_string() + ")")
                //         .color(egui::Color32::RED),
                // );
                if self.is_sync {
                    if ui.button("☑").on_hover_text("从服务器删除此条模拟规则，立即生效").clicked() {
                        self.is_sync = !self.is_sync;
                        return Action::SyncToServer((self.id,self.is_sync));
                    } 
                } else {
                    if ui.button("☐").on_hover_text("更新内容发往服务器，立即生效").clicked() {
                        self.is_sync = !self.is_sync;
                        return Action::SyncToServer((self.id,self.is_sync));
                    }
                }
                if self.is_sync {
                    let dark_mode = ui.visuals().dark_mode;
                    let faded_color = ui.visuals().window_fill();
                    let faded_color = |color: Color32| -> Color32 {
                        use egui::Rgba;
                        let t = if dark_mode { 0.95 } else { 0.8 };
                        egui::lerp(Rgba::from(color)..=Rgba::from(faded_color), t).into()
                    };
                    ui.painter().rect_filled(
                        ui.available_rect_before_wrap(),
                        0.5,
                        faded_color(Color32::RED),);
                }
                let select_resp = ui
                    .toggle_value(&mut self.selected, self.title.clone())
                    .context_menu(|ui| {
                        ui.add_space(5.);
                        ui.text_edit_singleline(add_title);
                        ui.add_space(5.);
                        ui.horizontal(|ui| {
                            let add_resp = ui.button("添加");
                            let rename_resp = ui.button("重命名");
                            if add_resp.clicked() {
                                let sub_id = self.add_sub_item(ui, add_title);
                                self.sender_add_info(ui, sub_id);
                                ui.close_menu();
                            }
                            if rename_resp.clicked() {
                                self.title = add_title.to_string();
                                ui.close_menu();
                            }
                        });
                    });
                if select_resp.clicked() {
                    Action::Selected((self.id, self.title.clone()))
                } else {
                    Action::Keep
                }
            })
            .inner
        } else {
            let (_, head_rep, body_resp) = 
                egui::collapsing_header::CollapsingState::load_with_default_open(
                    ui.ctx(),
                    id_source,
                    true,
                )
                .show_header(ui, |ui| {
                    // ui.label(
                    //     RichText::new("(🆔:".to_owned() + &self.id.to_string() + ")")
                    //         .color(egui::Color32::RED),
                    // );
                    // ui.label("📓");
                    if self.is_sync {
                        if ui.button("☑").on_hover_text("从服务器删除此条模拟规则，立即生效").clicked() {
                            self.is_sync = !self.is_sync;
                            return Action::SyncToServer((self.id,self.is_sync));
                        } 
                    } else {
                        if ui.button("☐").on_hover_text("更新内容发往服务器，立即生效").clicked() {
                            self.is_sync = !self.is_sync;
                            return Action::SyncToServer((self.id,self.is_sync));
                        }
                    }
                    if self.is_sync {
                        let dark_mode = ui.visuals().dark_mode;
                        let faded_color = ui.visuals().window_fill();
                        let faded_color = |color: Color32| -> Color32 {
                            use egui::Rgba;
                            let t = if dark_mode { 0.95 } else { 0.8 };
                            egui::lerp(Rgba::from(color)..=Rgba::from(faded_color), t).into()
                        };
                        ui.painter().rect_filled(
                            ui.available_rect_before_wrap(),
                            0.5,
                            faded_color(Color32::RED),);
                    }

                    let select_resp = ui
                    // .strong(&mut self.title)
                        .toggle_value(&mut self.selected, self.title.clone())
                        .context_menu(|ui| {
                            ui.add_space(5.);
                            ui.text_edit_singleline(add_title);
                            ui.add_space(5.);
                            ui.horizontal(|ui| {
                                let add_resp = ui.button("添加");
                                let rename_resp = ui.button("重命名");
                                if add_resp.clicked() {
                                    let sub_id = self.add_sub_item(ui, add_title);
                                    self.sender_add_info(ui, sub_id);
                                    ui.close_menu();
                                }
                                if rename_resp.clicked() {
                                    self.title = add_title.to_string();
                                    ui.close_menu();
                                }
                            });
                        });
                    if select_resp.clicked() {
                        Action::Selected((self.id, self.title.clone()))
                        // return Action::Keep;
                    } else {
                        Action::Keep
                    }
                })
                .body(|ui| self.sub_ui(ui, selected_str, add_title, flilter));
            match (head_rep.inner, body_resp) {
                (Action::Selected(head), _) => Action::Selected(head),
                (Action::SyncToServer(head), _) => Action::SyncToServer(head),
                (
                    // Action::Keep,
                    _,
                    Some(InnerResponse {
                        inner: Action::Selected(body),
                        ..
                    }),
                ) => Action::Selected(body),
                (
                    // Action::Keep,
                    _,
                    Some(InnerResponse {
                        inner: Action::SyncToServer(body),
                        ..
                    }),
                ) => Action::SyncToServer(body),
                (
                    _,
                    Some(InnerResponse {
                        inner:Action::Delete(del_ids),
                        ..
                    }),
                ) => Action::Delete(del_ids),
                _ => Action::Keep,
            }
        }
    }
    fn sender_add_info(&self, ui: &mut Ui, sub_id: u64) {
        let mut data = ui.data();
        let sender: SyncSender<(u64, u64)> = data.get_temp(Id::new(ADD_ID_KEY)).unwrap();
        let _ = sender.send((self.id, sub_id));
    }

    fn add_sub_item(&mut self, ui: &mut Ui, add_title: &mut str) -> u64 {
        let mut data = ui.data();
        let id_count: &mut u64 = data.get_persisted_mut_or_default(Id::new(ID_COUNT_KEY));
        *id_count += 1;
        let sub_id = *id_count;
        self.add_item(sub_id, add_title.to_owned().as_str());
        sub_id
    }

    pub fn sub_ui(
        &mut self,
        ui: &mut Ui,
        selected_str: u64,
        add_title: &mut String,
        flilter: &str,
    ) -> Action {
        let Self { sub_items, .. } = self;

        for (_, sub) in sub_items.iter_mut() {
            if sub.title.contains(flilter) {
                let sub_resp = sub.ui_impl(ui, selected_str, add_title, flilter);
                if let Action::Keep = sub_resp {
                    continue;
                } else {
                    return sub_resp;
                }
            }
        }
        Action::Keep
        // self.sub_items = sub_items.clone().into_iter().filter_map(|mut subtree|{
        // subtree.1.ui_impl(ui, selected_str);
        // Some(subtree)
        // if subtree.1.ui_impl(ui,selected_str) == Action::Keep{
        //     Some(subtree)
        // } else {
        //     None
        // }
        // }).collect();
    }
}

