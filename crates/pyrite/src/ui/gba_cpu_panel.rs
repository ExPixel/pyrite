use egui::{Color32, Ui};
use gba::Gba;

use crate::gba_runner::SharedGba;

pub fn render(ui: &mut Ui, gba: &SharedGba) {
    egui::Frame::central_panel(ui.style()).show(ui, |ui| {
        gba.with(|data| render_register_grid(ui, &data.gba));
    });
}

fn render_register_grid(ui: &mut Ui, gba: &Gba) {
    egui::Grid::new("register_grid").show(ui, |ui| {
        let mut items_on_row = 0;
        let mut add_register_row = |label: String, value: u32| {
            ui.colored_label(Color32::LIGHT_BLUE, label);
            ui.label(format!("0x{value:08X}")).on_hover_ui(|ui| {
                ui.label(format!("{value} ({})", value as i32));
            });

            items_on_row += 1;
            if items_on_row == 4 {
                items_on_row = 0;
                ui.end_row();
            }
        };

        for register in 0..16 {
            let label = match register {
                13 => "SP".to_owned(),
                14 => "LR".to_owned(),
                15 => "PC".to_owned(),
                _ => format!("R{register}"),
            };
            add_register_row(label, gba.cpu.registers.read(register));
        }

        add_register_row("CPSR".to_owned(), gba.cpu.registers.read_cpsr());
        add_register_row("SPSR".to_owned(), gba.cpu.registers.read_spsr());
    });
}
