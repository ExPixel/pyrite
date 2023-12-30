use std::{os::windows, sync::Arc};

use ahash::HashSet;
use egui::ViewportId;
use egui_extras::{Column, TableBuilder};
use parking_lot::Mutex;

use crate::gba_runner::SharedGba;

use super::app_window::{AppWindow, AppWindowWrapper};

pub struct DisassemblyWindow {
    gba: SharedGba,
}

impl DisassemblyWindow {
    fn new(gba: SharedGba) -> Self {
        Self { gba }
    }

    pub fn wrapped(windows: Arc<Mutex<HashSet<ViewportId>>>, gba: SharedGba) -> AppWindowWrapper {
        AppWindowWrapper::new::<Self>(windows, Self::new(gba))
    }
}

impl AppWindow for DisassemblyWindow {
    type State = Self;

    fn ui(state: &mut Self::State, ctx: &egui::Context) {
        egui::CentralPanel::default().show(ctx, |ui| {
            let font_id = egui::style::TextStyle::Monospace.resolve(ui.style());
            let m_width = ui.fonts(|f| f.glyph_width(&font_id, 'M'));
            let available_height = ui.available_height();
            TableBuilder::new(ui)
                .column(Column::exact(m_width * 12.0).resizable(false))
                .column(Column::exact(m_width * 12.0).resizable(false))
                .column(Column::remainder().resizable(false))
                .max_scroll_height(available_height)
                .header(20.0, |mut header| {
                    header.col(|ui| {
                        ui.heading("Address");
                    });

                    header.col(|ui| {
                        ui.heading("Bytes");
                    });

                    header.col(|ui| {
                        ui.heading("Instruction");
                    });
                })
                .body(|body| {
                    body.rows(18.0, 0x40000000, |row_index, mut row| {
                        let gba_data = state.gba.read();

                        let address = (row_index as u32) * 4;
                        let bytes = gba_data.gba.mapped.view32(address);
                        let disassembled = arm::disasm::arm::disasm(bytes);
                        let mnemonic = disassembled.mnemonic();
                        let arguments = disassembled.arguments();
                        let comment = disassembled.comment();

                        row.col(|ui| {
                            ui.monospace(format!("{:08X}", address));
                        });

                        row.col(|ui| {
                            ui.monospace(format!("{:08X}", bytes));
                        });

                        row.col(|ui| {
                            ui.monospace(format!("{mnemonic:<16} {arguments:<32} ; {comment}"));
                        });
                    });
                });
        });
    }

    fn title() -> String {
        "Disassembly".to_owned()
    }

    fn viewport_id() -> ViewportId {
        egui::ViewportId::from_hash_of("disassembly")
    }

    fn category() -> super::app_window::AppWindowCategory {
        super::app_window::AppWindowCategory::Gba
    }
}
