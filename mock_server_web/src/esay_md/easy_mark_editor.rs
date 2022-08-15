use egui::{text_edit::CCursorRange, *};

#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "serde", serde(default))]
pub struct EasyMarkEditor {
    #[cfg_attr(feature = "serde", serde(skip))]
    highlighter: crate::esay_md::MemoizedEasymarkHighlighter,
}

impl Default for EasyMarkEditor {
    fn default() -> Self {
        Self {
            highlighter: Default::default(),
        }
    }
}

impl EasyMarkEditor {
    // pub fn panels(&mut self, ctx: &egui::Context) {

    //     egui::CentralPanel::default().show(ctx, |ui| {
    //         self.ui(ui);
    //     });
    // }

    pub fn ui(&mut self, ui: &mut egui::Ui,code:&mut String,show_rendered:&mut bool) {

        if *show_rendered {
            ui.columns(2, |columns| {
                ScrollArea::vertical()
                    .id_source("rendered")
                    .show(&mut columns[1], |ui| {
                        // TODO(emilk): we can save some more CPU by caching the rendered output.
                        crate::esay_md::easy_mark(ui, code);
                    });
            });
        } else {
            ScrollArea::vertical()
                .id_source("source")
                .show(ui, |ui| self.editor_ui(ui,code));
        }
    }

    fn editor_ui(&mut self, ui: &mut egui::Ui,code:&mut String) {
        let Self {
            highlighter 
        } = self;

        let response = {
            let mut layouter = |ui: &egui::Ui, easymark: &str, wrap_width: f32| {
                let mut layout_job = highlighter.highlight(ui.style(), easymark);
                layout_job.wrap.max_width = wrap_width;
                ui.fonts().layout_job(layout_job)
            };

            ui.add(
                egui::TextEdit::multiline(code)
                    .desired_width(f32::INFINITY)
                    .font(egui::TextStyle::Monospace) // for cursor height
                    .layouter(&mut layouter),
            )
        };

        if let Some(mut state) = TextEdit::load_state(ui.ctx(), response.id) {
            if let Some(mut ccursor_range) = state.ccursor_range() {
                let any_change = shortcuts(ui, code, &mut ccursor_range);
                if any_change {
                    state.set_ccursor_range(Some(ccursor_range));
                    state.store(ui.ctx(), response.id);
                }
            }
        }
    }
}

fn nested_hotkeys_ui(ui: &mut egui::Ui) {
    let _ = ui.label("CTRL+B *bold*");
    let _ = ui.label("CTRL+N `code`");
    let _ = ui.label("CTRL+I /italics/");
    let _ = ui.label("CTRL+L $subscript$");
    let _ = ui.label("CTRL+Y ^superscript^");
    let _ = ui.label("ALT+SHIFT+Q ~strikethrough~");
    let _ = ui.label("ALT+SHIFT+W _underline_");
    let _ = ui.label("ALT+SHIFT+E two spaces"); // Placeholder for tab indent
}

fn shortcuts(ui: &Ui, code: &mut dyn TextBuffer, ccursor_range: &mut CCursorRange) -> bool {
    let mut any_change = false;
    if ui
        .input_mut()
        .consume_key(egui::Modifiers::ALT, Key::E)
    {
        // This is a placeholder till we can indent the active line
        any_change = true;
        let [primary, _secondary] = ccursor_range.sorted();

        let advance = code.insert_text("  ", primary.index);
        ccursor_range.primary.index += advance;
        ccursor_range.secondary.index += advance;
    }
    for (modifier, key, surrounding) in [
        (egui::Modifiers::COMMAND, Key::B, "*"),   // *bold*
        (egui::Modifiers::COMMAND, Key::N, "`"),   // `code`
        (egui::Modifiers::COMMAND, Key::I, "/"),   // /italics/
        (egui::Modifiers::COMMAND, Key::L, "$"),   // $subscript$
        (egui::Modifiers::COMMAND, Key::Y, "^"),   // ^superscript^
        (egui::Modifiers::ALT, Key::Q, "~"), // ~strikethrough~
        (egui::Modifiers::ALT, Key::W, "_"), // _underline_
    ] {
        if ui.input_mut().consume_key(modifier, key) {
            any_change = true;
            toggle_surrounding(code, ccursor_range, surrounding);
        };
    }
    any_change
}

/// E.g. toggle *strong* with `toggle_surrounding(&mut text, &mut cursor, "*")`
fn toggle_surrounding(
    code: &mut dyn TextBuffer,
    ccursor_range: &mut CCursorRange,
    surrounding: &str,
) {
    let [primary, secondary] = ccursor_range.sorted();

    let surrounding_ccount = surrounding.chars().count();

    let prefix_crange = primary.index.saturating_sub(surrounding_ccount)..primary.index;
    let suffix_crange = secondary.index..secondary.index.saturating_add(surrounding_ccount);
    let already_surrounded = code.char_range(prefix_crange.clone()) == surrounding
        && code.char_range(suffix_crange.clone()) == surrounding;

    if already_surrounded {
        code.delete_char_range(suffix_crange);
        code.delete_char_range(prefix_crange);
        ccursor_range.primary.index -= surrounding_ccount;
        ccursor_range.secondary.index -= surrounding_ccount;
    } else {
        code.insert_text(surrounding, secondary.index);
        let advance = code.insert_text(surrounding, primary.index);

        ccursor_range.primary.index += advance;
        ccursor_range.secondary.index += advance;
    }
}
