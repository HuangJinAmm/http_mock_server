use std::sync::{Arc, Mutex};

use egui::{text::LayoutJob, Galley, Id, Key, Modifiers, Pos2, Ui};

/// View some code with syntax highlighting and selection.
pub fn code_view_ui(ui: &mut egui::Ui, mut code: &str, language: &str) {
    let theme = CodeTheme::from_memory(ui.ctx());

    let mut layouter = |ui: &egui::Ui, string: &str, _wrap_width: f32| {
        let layout_job = highlight(ui.ctx(), &theme, string, language);
        // layout_job.wrap.max_width = wrap_width; // no wrapping
        ui.fonts().layout_job(layout_job)
    };

    ui.add(
        // egui::TextEdit::multiline(&mut code)
        egui::text_edit::TextEdit::multiline(&mut code)
            .font(egui::TextStyle::Heading) // for cursor height
            // .code_editor()
            // .desired_rows(1)
            // .lock_focus(true)
            .layouter(&mut layouter),
    );
}

pub fn highlight_layouter<'a>(
    ui: &'a mut egui::Ui,
    language: &'a str,
) -> impl FnMut(&Ui, &str, f32) -> Arc<Galley> + 'a {
    let theme = CodeTheme::from_memory(ui.ctx());

    let layouter = move |ui: &egui::Ui, string: &str, _wrap_width: f32| {
        let layout_job = highlight(ui.ctx(), &theme, string, language);
        // layout_job.wrap.max_width = wrap_width; // no wrapping
        ui.fonts().layout_job(layout_job)
    };
    layouter
}

/// Memoized Code highlighting
pub fn highlight(ctx: &egui::Context, theme: &CodeTheme, code: &str, language: &str) -> LayoutJob {
    impl egui::util::cache::ComputerMut<(&CodeTheme, &str, &str), LayoutJob> for Highlighter {
        fn compute(&mut self, (theme, code, lang): (&CodeTheme, &str, &str)) -> LayoutJob {
            self.highlight(theme, code, lang)
        }
    }

    type HighlightCache<'a> = egui::util::cache::FrameCache<LayoutJob, Highlighter>;

    let mut memory = ctx.memory();
    let highlight_cache = memory.caches.cache::<HighlightCache<'_>>();
    highlight_cache.get((theme, code, language))
}

// ----------------------------------------------------------------------------

#[derive(Clone, Copy, PartialEq, serde::Deserialize, serde::Serialize, enum_map::Enum)]
enum TokenType {
    Comment,
    Keyword,
    Literal,
    StringLiteral,
    Punctuation,
    Whitespace,
}

#[derive(Clone, Hash, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct CodeTheme {
    dark_mode: bool,

    formats: enum_map::EnumMap<TokenType, egui::TextFormat>,
}

impl Default for CodeTheme {
    fn default() -> Self {
        Self::dark()
    }
}

impl CodeTheme {
    pub fn from_style(style: &egui::Style) -> Self {
        if style.visuals.dark_mode {
            Self::dark()
        } else {
            Self::light()
        }
    }

    pub fn from_memory(ctx: &egui::Context) -> Self {
        if ctx.style().visuals.dark_mode {
            ctx.data()
                .get_persisted(egui::Id::new("dark"))
                .unwrap_or_else(CodeTheme::dark)
        } else {
            ctx.data()
                .get_persisted(egui::Id::new("light"))
                .unwrap_or_else(CodeTheme::light)
        }
    }

    pub fn store_in_memory(self, ctx: &egui::Context) {
        if self.dark_mode {
            ctx.data().insert_persisted(egui::Id::new("dark"), self);
        } else {
            ctx.data().insert_persisted(egui::Id::new("light"), self);
        }
    }
}

impl CodeTheme {
    pub fn dark() -> Self {
        let font_id = egui::FontId::monospace(20.0);
        use egui::{Color32, TextFormat};
        Self {
            dark_mode: true,
            formats: enum_map::enum_map![
                TokenType::Comment => TextFormat::simple(font_id.clone(), Color32::from_gray(120)),
                TokenType::Keyword => TextFormat::simple(font_id.clone(), Color32::from_rgb(255, 100, 100)),
                TokenType::Literal => TextFormat::simple(font_id.clone(), Color32::from_rgb(87, 165, 171)),
                TokenType::StringLiteral => TextFormat::simple(font_id.clone(), Color32::from_rgb(109, 147, 226)),
                TokenType::Punctuation => TextFormat::simple(font_id.clone(), Color32::LIGHT_GRAY),
                TokenType::Whitespace => TextFormat::simple(font_id.clone(), Color32::TRANSPARENT),
            ],
        }
    }

    pub fn light() -> Self {
        let font_id = egui::FontId::monospace(20.0);
        use egui::{Color32, TextFormat};
        Self {
            dark_mode: false,
            #[cfg(not(feature = "syntect"))]
            formats: enum_map::enum_map![
                TokenType::Comment => TextFormat::simple(font_id.clone(), Color32::GRAY),
                TokenType::Keyword => TextFormat::simple(font_id.clone(), Color32::from_rgb(235, 0, 0)),
                TokenType::Literal => TextFormat::simple(font_id.clone(), Color32::from_rgb(153, 134, 255)),
                TokenType::StringLiteral => TextFormat::simple(font_id.clone(), Color32::from_rgb(37, 203, 105)),
                TokenType::Punctuation => TextFormat::simple(font_id.clone(), Color32::DARK_GRAY),
                TokenType::Whitespace => TextFormat::simple(font_id.clone(), Color32::TRANSPARENT),
            ],
        }
    }

    pub fn ui(&mut self, ui: &mut egui::Ui) {
        ui.horizontal_top(|ui| {
            let selected_id = egui::Id::null();
            let mut selected_tt: TokenType = *ui
                .data()
                .get_persisted_mut_or(selected_id, TokenType::Comment);

            ui.vertical(|ui| {
                ui.set_width(150.0);
                egui::widgets::global_dark_light_mode_buttons(ui);

                ui.add_space(8.0);
                ui.separator();
                ui.add_space(8.0);

                ui.scope(|ui| {
                    for (tt, tt_name) in [
                        (TokenType::Comment, "// comment"),
                        (TokenType::Keyword, "keyword"),
                        (TokenType::Literal, "literal"),
                        (TokenType::StringLiteral, "\"string literal\""),
                        (TokenType::Punctuation, "punctuation ;"),
                        (TokenType::Whitespace, "whitespace"),
                    ] {
                        let format = &mut self.formats[tt];
                        ui.style_mut().override_font_id = Some(format.font_id.clone());
                        ui.visuals_mut().override_text_color = Some(format.color);
                        ui.radio_value(&mut selected_tt, tt, tt_name);
                    }
                });

                let reset_value = if self.dark_mode {
                    CodeTheme::dark()
                } else {
                    CodeTheme::light()
                };

                if ui
                    .add_enabled(*self != reset_value, egui::Button::new("Reset theme"))
                    .clicked()
                {
                    *self = reset_value;
                }
            });

            ui.add_space(16.0);

            ui.data().insert_persisted(selected_id, selected_tt);

            egui::Frame::group(ui.style())
                .inner_margin(egui::Vec2::splat(2.0))
                .show(ui, |ui| {
                    // ui.group(|ui| {
                    ui.style_mut().override_text_style = Some(egui::TextStyle::Small);
                    ui.spacing_mut().slider_width = 128.0; // Controls color picker size
                    egui::widgets::color_picker::color_picker_color32(
                        ui,
                        &mut self.formats[selected_tt].color,
                        egui::color_picker::Alpha::Opaque,
                    );
                });
        });
    }
}

#[derive(Default)]
struct Highlighter {}

impl Highlighter {
    #[allow(clippy::unused_self, clippy::unnecessary_wraps)]
    fn highlight(&self, theme: &CodeTheme, mut text: &str, _language: &str) -> LayoutJob {
        // Extremely simple syntax highlighter for when we compile without syntect

        let mut job = LayoutJob::default();

        while !text.is_empty() {
            if text.starts_with("//") {
                let end = text.find('\n').unwrap_or(text.len());
                job.append(&text[..end], 0.0, theme.formats[TokenType::Comment].clone());
                text = &text[end..];
            } else if text.starts_with('"') {
                let end = text[1..]
                    .find('"')
                    .map(|i| i + 2)
                    .or_else(|| text.find('\n'))
                    .unwrap_or(text.len());
                let word = &text[..end];
                let tt = if is_template_word(word) {
                    TokenType::Keyword
                } else {
                    TokenType::StringLiteral
                };
                job.append(word, 0.0, theme.formats[tt].clone());
                text = &text[end..];
            } else if text.starts_with(|c: char| c.is_ascii_alphanumeric()) {
                let end = text[1..]
                    .find(|c: char| !c.is_ascii_alphanumeric())
                    .map_or_else(|| text.len(), |i| i + 1);
                let word = &text[..end];
                let tt = if is_template_word(word) {
                    TokenType::Keyword
                } else {
                    TokenType::Literal
                };
                job.append(word, 0.0, theme.formats[tt].clone());
                text = &text[end..];
            } else if text.starts_with(|c: char| c.is_ascii_whitespace()) {
                let end = text[1..]
                    .find(|c: char| !c.is_ascii_whitespace())
                    .map_or_else(|| text.len(), |i| i + 1);
                job.append(
                    &text[..end],
                    0.0,
                    theme.formats[TokenType::Whitespace].clone(),
                );
                text = &text[end..];
            } else {
                let mut it = text.char_indices();
                it.next();
                let end = it.next().map_or(text.len(), |(idx, _chr)| idx);
                job.append(
                    &text[..end],
                    0.0,
                    theme.formats[TokenType::Punctuation].clone(),
                );
                text = &text[end..];
            }
        }

        job
    }
}

fn is_template_word(word: &str) -> bool {
    // word.starts_with("{{") && word.ends_with("}}")
    matches!(
        word,
        "for"
            | "in"
            | "range"
            | "endfor"
            | "endif"
            | "elif"
            | "else"
            | "NAME_ZH"
            | "NAME_EN"
            | "NUM"
            | "NUM_STR"
            | "HEX"
            | "STR"
            | "EMAIL"
            | "USERNAME"
            | "IPV4"
            | "IPV6"
            | "MAC"
            | "USERAGENT"
            | "PASSWORD"
            | "UUID"
            | "UUID_SIMPLE"
            | "NOW"
            | "DATE_BEFORE"
            | "DATE_AFTER"
            | "DATE"
            | "BASE64_EN"
            | "BASE64_DE"
    )
}

fn is_keyword(word: &str) -> bool {
    matches!(
        word,
        "as" | "async"
            | "await"
            | "break"
            | "const"
            | "continue"
            | "crate"
            | "dyn"
            | "else"
            | "enum"
            | "extern"
            | "false"
            | "fn"
            | "for"
            | "if"
            | "impl"
            | "in"
            | "let"
            | "loop"
            | "match"
            | "mod"
            | "move"
            | "mut"
            | "pub"
            | "ref"
            | "return"
            | "self"
            | "Self"
            | "static"
            | "struct"
            | "super"
            | "trait"
            | "true"
            | "type"
            | "unsafe"
            | "use"
            | "where"
            | "while"
    )
}
