use super::app_window::{AppWindow, AppWindowWrapper};
use crate::gba_runner::SharedGba;
use ahash::HashSet;
use arm::disasm::MemoryView as _;
use arm::{disasm::AnyInstr, emu::InstructionSet};
use egui::{epaint::PathShape, Color32, RichText, Sense, Stroke, ViewportId};
use parking_lot::Mutex;
use std::fmt::Write as _;
use std::sync::Arc;

pub struct DisassemblyWindow {
    gba: SharedGba,
    first_visible_address: u32,
    instruction_set: Option<InstructionSet>,
    goto_address: String,
}

impl DisassemblyWindow {
    fn new(gba: SharedGba) -> Self {
        Self {
            gba,
            first_visible_address: 0,
            instruction_set: None,
            goto_address: String::new(),
        }
    }

    pub fn wrapped(windows: Arc<Mutex<HashSet<ViewportId>>>, gba: SharedGba) -> AppWindowWrapper {
        AppWindowWrapper::new::<Self>(windows, Self::new(gba))
    }
}

impl AppWindow for DisassemblyWindow {
    type State = Self;

    fn ui(state: &mut Self::State, ctx: &egui::Context) {
        let gba_data = state.gba.read();

        let mut should_scroll_to_current = false;

        egui::TopBottomPanel::top("disassembly_controls_panel").show(ctx, |ui| {
            ui.horizontal(|ui| {
                state.goto_address.retain(|c| c.is_ascii_hexdigit());
                let goto_address_text_edit =
                    egui::TextEdit::singleline(&mut state.goto_address).char_limit(8);
                let response = ui.add(goto_address_text_edit);
                let mut should_goto_address = false;
                if response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                    should_goto_address = true;
                }

                should_goto_address |= ui.button("Goto").clicked();

                if should_goto_address {
                    state.goto_address.retain(|c| c.is_ascii_hexdigit());
                    if let Ok(address) = u32::from_str_radix(&state.goto_address, 16) {
                        state.first_visible_address = address;
                        state.goto_address.clear();
                    }
                }

                should_scroll_to_current = ui
                    .button("Goto Next Executed Instruction")
                    .on_hover_text("Scroll to the next instruction executed by the CPU")
                    .clicked();

                egui::ComboBox::new("disassembly_isa_combobox", "Instruction Set")
                    .selected_text(match state.instruction_set {
                        None => "Auto",
                        Some(arm::emu::InstructionSet::Arm) => "ARM",
                        Some(arm::emu::InstructionSet::Thumb) => "Thumb",
                    })
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut state.instruction_set, None, "Auto");
                        ui.selectable_value(
                            &mut state.instruction_set,
                            Some(arm::emu::InstructionSet::Arm),
                            "ARM",
                        );
                        ui.selectable_value(
                            &mut state.instruction_set,
                            Some(arm::emu::InstructionSet::Thumb),
                            "Thumb",
                        );
                    })
            });
        });

        let instruction_set = match state.instruction_set {
            None => {
                if gba_data.gba.cpu.registers.get_flag(arm::emu::CpsrFlag::T) {
                    arm::emu::InstructionSet::Thumb
                } else {
                    arm::emu::InstructionSet::Arm
                }
            }
            Some(instruction_set) => instruction_set,
        };
        let instruction_width: u32 = match instruction_set {
            arm::emu::InstructionSet::Arm => 4,
            arm::emu::InstructionSet::Thumb => 2,
        };

        match instruction_set {
            InstructionSet::Arm => state.first_visible_address &= !3,
            InstructionSet::Thumb => state.first_visible_address &= !1,
        }

        if should_scroll_to_current {
            state.first_visible_address = gba_data
                .gba
                .cpu
                .registers
                .read(15)
                .wrapping_sub(instruction_width);
        }

        egui::SidePanel::left("disassembly_registers_panel").show(ctx, |ui| {
            egui::Grid::new("registers_grid")
                .num_columns(3)
                .show(ui, |ui| {
                    ui.monospace("Register");
                    ui.monospace("Hex");
                    ui.monospace("Decimal");
                    ui.end_row();

                    for register in 0..16 {
                        match register {
                            13 => ui.monospace("sp"),
                            14 => ui.monospace("lr"),
                            15 => ui.monospace("pc"),
                            _ => ui.monospace(format!("r{}", register)),
                        };

                        let hex = format!("0x{:08X}", gba_data.gba.cpu.registers.read(register));
                        let dec = format!("{}", gba_data.gba.cpu.registers.read(register));
                        let col = if register == 15 {
                            Color32::GREEN
                        } else {
                            Color32::LIGHT_BLUE
                        };
                        ui.monospace(RichText::new(hex).color(col));
                        ui.monospace(RichText::new(dec).color(col));
                        ui.end_row();
                    }
                });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            let text_height = ui.text_style_height(&egui::style::TextStyle::Monospace);
            let available_height = ui.available_height();
            let spacing = ui.spacing().item_spacing.y;
            let rows_visible = (available_height / (text_height + spacing)).ceil();
            let address_range = (state.first_visible_address as u64)
                ..(state.first_visible_address as u64
                    + ((rows_visible as u64) * instruction_width as u64));
            let cursor_padding = 2.0;

            let response = egui::Grid::new("disassembly")
                .striped(true)
                .num_columns(4)
                .show(ui, |ui| {
                    ui.allocate_exact_size(egui::vec2(text_height, text_height), Sense::hover());
                    ui.monospace("Address");
                    ui.monospace("Bytes");
                    ui.monospace("Disassembly");
                    ui.monospace("Comment");
                    ui.end_row();

                    let mut comment_buffer = String::with_capacity(32);
                    for address in address_range.step_by(instruction_width as usize) {
                        let address = address as u32;

                        let bytes: u32;
                        let disassembled: AnyInstr;

                        match instruction_set {
                            InstructionSet::Arm => {
                                bytes = gba_data.gba.mapped.view32(address);
                                disassembled =
                                    AnyInstr::from(arm::disasm::arm::disasm(bytes, address));
                            }
                            InstructionSet::Thumb => {
                                bytes = gba_data.gba.mapped.view16(address) as u32;
                                disassembled = AnyInstr::from(arm::disasm::thumb::disasm(
                                    bytes as u16,
                                    address,
                                ))
                            }
                        }

                        let mnemonic = disassembled.mnemonic();
                        let arguments = disassembled.arguments(address, Some(&gba_data.gba.mapped));
                        let comment = disassembled.comment(address, Some(&gba_data.gba.mapped));

                        let (cursor_rect, _response) = ui.allocate_exact_size(
                            egui::vec2(text_height, text_height),
                            Sense::hover(),
                        );
                        let cursor_rect = cursor_rect.shrink(cursor_padding);

                        if address
                            == gba_data
                                .gba
                                .cpu
                                .registers
                                .read(15)
                                .wrapping_sub(instruction_width)
                        {
                            let cursor_center = cursor_rect.center();
                            let cursor_shape = PathShape::convex_polygon(
                                vec![
                                    cursor_center
                                        - egui::vec2(
                                            cursor_rect.width() * 0.5,
                                            cursor_rect.height() * -0.5,
                                        ),
                                    cursor_center + egui::vec2(cursor_rect.width() * 0.5, 0.0),
                                    cursor_center
                                        - egui::vec2(
                                            cursor_rect.width() * 0.5,
                                            cursor_rect.height() * 0.5,
                                        ),
                                ],
                                Color32::YELLOW,
                                Stroke::NONE,
                            );
                            ui.painter().add(cursor_shape);
                        }

                        ui.monospace(
                            RichText::new(format!("{:08X}", address)).color(Color32::GREEN),
                        );

                        match instruction_set {
                            InstructionSet::Arm => ui.monospace(
                                RichText::new(format!("{:08X}", bytes)).color(Color32::LIGHT_BLUE),
                            ),
                            InstructionSet::Thumb => ui.monospace(
                                RichText::new(format!("{:04X}", bytes)).color(Color32::LIGHT_BLUE),
                            ),
                        };

                        ui.monospace(format!(
                            "{mnemonic:<12} {arguments:<32}",
                            mnemonic = mnemonic,
                            arguments = arguments,
                        ));

                        comment_buffer.clear();
                        write!(&mut comment_buffer, "{comment}").unwrap();

                        if !comment_buffer.is_empty() {
                            let comment_string = format!("{comment_buffer:<32}");
                            ui.horizontal(|ui| {
                                ui.monospace(RichText::new("; ").color(Color32::LIGHT_GREEN));
                                ui.monospace(
                                    RichText::new(comment_string).color(Color32::LIGHT_GREEN),
                                );
                            });
                        }
                        ui.allocate_space(egui::vec2(ui.available_width(), 0.0));
                        ui.end_row();
                    }
                })
                .response;

            if response.hovered() {
                let scrolled_by = ui.input(|input| input.scroll_delta.y);
                if scrolled_by > 0.0 {
                    state.first_visible_address =
                        state.first_visible_address.wrapping_sub(instruction_width);
                } else if scrolled_by < 0.0 {
                    state.first_visible_address =
                        state.first_visible_address.wrapping_add(instruction_width);
                }
            }
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
