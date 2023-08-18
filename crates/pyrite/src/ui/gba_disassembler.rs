use arm::{disasm::DisasmOptions, emu::InstructionSet};
use egui::{epaint::ahash::AHashMap, Ui};
use egui_extras::{Column, TableBuilder};
use gba::Gba;

use crate::gba_runner::SharedGba;

pub struct GbaDisassemblerUi {
    address: u32,
    cache: AHashMap<(/* address */ u32, /* instruction */ u32), InstrEntry>,
    gba: SharedGba,
    isa: Option<InstructionSet>,
}

struct InstrEntry {
    used_this_frame: bool,
    frames_without_use: u32,
    address: String,
    instruction: String,
    mnemonic: String,
    arguments: String,
    comment: String,
}

impl InstrEntry {
    pub fn new(isa: InstructionSet, gba: &Gba, address: u32, instr: u32) -> Self {
        let options = DisasmOptions::default();
        if isa == InstructionSet::Arm {
            let disasm = arm::disasm::disasm_arm(instr, address, &options);
            Self {
                address: format!("{address:08X}"),
                instruction: format!("{instr:08X}"),
                mnemonic: format!("{}", disasm.mnemonic(&options)),
                arguments: format!("{}", disasm.arguments(&options)),
                comment: String::new(),
                used_this_frame: false,
                frames_without_use: 0,
            }
        } else {
            let disasm = arm::disasm::disasm_thumb(instr as u16, address, &options);
            Self {
                address: format!("{address:08X}"),
                instruction: format!("{instr:08X}"),
                mnemonic: format!("{}", disasm.mnemonic(&options)),
                arguments: format!("{}", disasm.arguments(&options)),
                comment: String::new(),
                used_this_frame: false,
                frames_without_use: 0,
            }
        }
    }
}

impl GbaDisassemblerUi {
    pub fn new(gba: SharedGba) -> Self {
        Self {
            address: 0,
            cache: AHashMap::default(),
            isa: None,
            gba,
        }
    }

    pub fn ui(&mut self, ui: &mut Ui) {
        ui.with_layout(egui::Layout::top_down(egui::Align::Max), |ui| {
            self.render(ui)
        });
        self.trim_cache();
    }

    fn render(&mut self, ui: &mut Ui) {
        let m_font_id = ui.style().text_styles.get(&egui::TextStyle::Body).unwrap();
        let (m_width, font_height) =
            ui.fonts(|f| (f.glyph_width(m_font_id, 'M'), f.row_height(m_font_id)));
        let table = TableBuilder::new(ui)
            .striped(true)
            .resizable(true)
            .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
            .column(Column::initial(m_width * 9.0))
            .column(Column::initial(m_width * 9.0))
            .column(Column::initial(m_width * 12.0))
            .column(Column::remainder())
            .column(Column::initial(m_width * 32.0));
        let gba = self.gba.read();
        let gba = &gba.gba;
        let isa = self.isa.unwrap_or_else(|| gba.cpu.get_instruction_set());
        let row_count = if isa == InstructionSet::Arm {
            1 << 30
        } else {
            1 << 31
        };
        table
            .header(font_height + 4.0, |mut header| {
                header.col(|ui| {
                    ui.strong("Address");
                });

                header.col(|ui| {
                    ui.strong("Instruction");
                });

                header.col(|ui| {
                    ui.strong("Mnemonic");
                });

                header.col(|ui| {
                    ui.strong("Arguments");
                });

                header.col(|ui| {
                    ui.strong("Comment");
                });
            })
            .body(|body| {
                body.rows(font_height + 4.0, row_count, |idx, mut row| {
                    let address = idx as u32 * isa.instruction_size();
                    let instr = if isa == InstructionSet::Arm {
                        gba.mapped.view32(address)
                    } else {
                        gba.mapped.view16(address) as u32
                    };
                    let entry = self
                        .cache
                        .entry((address, instr))
                        .or_insert_with(|| InstrEntry::new(isa, gba, address, instr));
                    entry.used_this_frame = true;

                    row.col(|ui| {
                        ui.label(&entry.address);
                    });

                    row.col(|ui| {
                        ui.label(&entry.instruction);
                    });

                    row.col(|ui| {
                        ui.label(&entry.mnemonic);
                    });

                    row.col(|ui| {
                        ui.label(&entry.arguments);
                    });

                    row.col(|ui| {
                        ui.label(&entry.comment);
                    });
                })
            });
    }

    pub fn trim_cache(&mut self) {
        self.cache.retain(|_, entry| {
            if !std::mem::take(&mut entry.used_this_frame) {
                entry.frames_without_use += 1;
                entry.frames_without_use < 4
            } else {
                entry.frames_without_use = 0;
                true
            }
        });
    }
}
