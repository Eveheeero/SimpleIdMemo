#![cfg_attr(not(test), windows_subsystem = "windows")]

mod fonts;
use clipboard::{ClipboardContext, ClipboardProvider};
use eframe::egui::{self, RichText, Sense};
use serde::{Deserialize, Serialize};
use std::io::{Read, Write};

#[allow(unused)]
pub fn main() -> eframe::Result<()> {
    let mut native_options = eframe::NativeOptions::default();
    native_options.centered = true;
    eframe::run_native(
        "Simple Id Memo",
        native_options,
        Box::new(|cc| Ok(Box::new(Memo::new(cc)))),
    )
}

#[derive(Serialize, Deserialize, Default)]
struct Memo {
    data: Vec<MemoEntry>,

    #[serde(skip)]
    focus: bool,

    #[serde(skip)]
    _input: (String, String, String),
}

impl Memo {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // 폰트 설정
        cc.egui_ctx.set_fonts(fonts::get_fonts());

        Self::deser("memo")
    }
}

impl Drop for Memo {
    fn drop(&mut self) {
        self.serde("memo")
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Eq, Ord, PartialOrd, PartialEq)]
struct MemoEntry {
    local_id: usize,
    id: String,
    name: String,
    description: String,
}

impl Memo {
    fn submit(&mut self) {
        if self._input.0.is_empty() {
            return;
        }
        if !self.data.iter().any(|e| e.id == self._input.0) {
            self.data.push(MemoEntry {
                local_id: self.data.last().map(|e| e.local_id + 1).unwrap_or(0),
                id: self._input.0.clone(),
                name: self._input.1.clone(),
                description: self._input.2.clone(),
            });
        } else {
            let entry = self
                .data
                .iter_mut()
                .find(|e| e.id == self._input.0)
                .unwrap();
            entry.name = self._input.1.clone();
            entry.description = self._input.2.clone();
        }

        self._input.0.clear();
        self._input.1.clear();
        self._input.2.clear();
    }

    fn serde(&self, name: impl AsRef<str>) {
        let mut memo_path = std::env::current_dir().unwrap();
        if !cfg!(feature = "postcard") {
            memo_path.push(format!("{}.cbor", name.as_ref()));
            let file = std::fs::File::create(memo_path).unwrap();
            let writer = std::io::BufWriter::new(file);
            serde_cbor::to_writer(writer, &self).unwrap();
        } else {
            memo_path.push(format!("{}.postcard", name.as_ref()));
            let file = std::fs::File::create(memo_path).unwrap();
            let mut writer = std::io::BufWriter::new(file);
            let data = postcard::to_stdvec(&self).unwrap();
            writer.write_all(&data).unwrap();
            writer.flush().unwrap();
        }
    }
    fn deser(name: impl AsRef<str>) -> Self {
        let mut memo_path = std::env::current_dir().unwrap();
        if !cfg!(feature = "postcard") {
            memo_path.push(format!("{}.cbor", name.as_ref()));
        } else {
            memo_path.push(format!("{}.postcard", name.as_ref()));
        }
        if memo_path.exists() && !cfg!(feature = "postcard") {
            let file = std::fs::File::open(memo_path).unwrap();
            let reader = std::io::BufReader::new(file);
            let memo: Memo = serde_cbor::from_reader(reader).unwrap();
            memo
        } else if memo_path.exists() && cfg!(feature = "postcard") {
            let mut file = std::fs::File::open(memo_path).unwrap();
            let mut data = Vec::new();
            file.read_to_end(&mut data).unwrap();
            let memo: Memo = postcard::from_bytes(&data).unwrap();
            memo
        } else {
            Self::default()
        }
    }
}

impl eframe::App for Memo {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let mut delete_content = None;
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Memo");
            ui.separator();
            ui.horizontal(|ui| {
                ui.add_sized(
                    [60.0, 14.0],
                    egui::Label::new(RichText::new("Local ID").size(14.0)),
                );
                ui.add_sized(
                    [80.0, 14.0],
                    egui::Label::new(RichText::new("ID").size(14.0)),
                );
                ui.add_sized(
                    [80.0, 14.0],
                    egui::Label::new(RichText::new("Name").size(14.0)),
                );
                ui.add_sized(
                    [ui.available_width(), 14.0],
                    egui::Label::new(RichText::new("Description").size(14.0)),
                );
            });
            ui.separator();
            ui.horizontal(|ui| {
                ui.add_space(70.0);
                let id_box =
                    ui.add(egui::TextEdit::singleline(&mut self._input.0).desired_width(80.0));
                if self.focus {
                    id_box.request_focus();
                    self.focus = false;
                }
                ui.add(egui::TextEdit::singleline(&mut self._input.1).desired_width(80.0));
                ui.add_sized(
                    [ui.available_width() - 50.0, 14.0],
                    egui::TextEdit::singleline(&mut self._input.2),
                );
                let button = ui.add_sized([40.0, 14.0], egui::Button::new("Add"));
                if button.clicked() {
                    self.submit();
                }
                ui.input(|key| {
                    if key.key_pressed(egui::Key::Tab) {
                        if button.has_focus() {
                            self.focus = true;
                        }
                    } else if key.key_pressed(egui::Key::Enter) {
                        self.submit();
                        self.focus = true;
                    }
                });
            });

            ui.separator();

            egui::ScrollArea::new([true, true]).show(ui, |ui| {
                for entry in &self.data {
                    ui.horizontal(|ui| {
                        if ui.add_sized([14.0, 14.0], egui::Button::new("-")).clicked() {
                            delete_content = Some(entry.local_id);
                        }
                        ui.label(entry.local_id.to_string());
                        let id_field = ui
                            .add_sized([80.0, 14.0], egui::Label::new(entry.id.to_string()))
                            .interact(Sense::click());
                        let name_field = ui
                            .add_sized([80.0, 14.0], egui::Label::new(entry.name.to_string()))
                            .interact(Sense::click());
                        ui.label(entry.description.to_string());
                        ui.add_space(ui.available_size().x);

                        if id_field.clicked() {
                            let mut clipboard: ClipboardContext = ClipboardProvider::new().unwrap();
                            clipboard.set_contents(entry.id.to_string()).unwrap();
                        }
                        if name_field.clicked() {
                            let mut clipboard: ClipboardContext = ClipboardProvider::new().unwrap();
                            clipboard.set_contents(entry.name.to_string()).unwrap();
                        }
                    });
                }
            });
        });
        if let Some(local_id) = delete_content {
            self.data.retain(|e| e.local_id != local_id);
        }
    }
}
