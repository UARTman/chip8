use super::ExecutionState;
use egui::{DragValue, Ui};

impl super::Cpu {
    pub fn draw_ui(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.label(format!("Cpu state: {:?}", self.execution_state));

            if ui
                .add(
                    egui::Button::new(if self.execution_state == ExecutionState::Running {
                        "Pause"
                    } else {
                        "Resume"
                    })
                    .enabled(
                        self.execution_state == ExecutionState::Running
                            || self.execution_state == ExecutionState::Paused,
                    ),
                )
                .clicked()
            {
                if self.execution_state == ExecutionState::Running {
                    self.pause()
                } else {
                    self.resume()
                }
            }

            ui.separator();

            ui.set_enabled(self.execution_state == ExecutionState::Paused);
            ui.label("Program counter:");
            ui.add(
                egui::DragValue::u16(&mut self.pc).clamp_range(0x200 as f32..=4096 as f32), // .prefix("Program Counter:"),
            );
        });

        ui.set_enabled(self.execution_state == ExecutionState::Paused);

        ui.separator();

        ui.label("Registers:");

        egui::Grid::new("my_grid")
            .striped(false)
            .min_col_width(10.0)
            .show(ui, |ui| {
                for i in 0..16 {
                    ui.label(format!("P{}", i));
                }
                ui.end_row();
                for i in self.reg.iter_mut() {
                    ui.add(DragValue::u8(i));
                }
            });

        ui.horizontal(|ui| {
            ui.label("I");
            ui.add(DragValue::u16(&mut self.reg_i));
        });

        ui.separator();

        ui.label("Timers:");
        ui.horizontal(|ui| {
            ui.label("Delay:");
            ui.add(DragValue::u8(&mut self.tim_delay));
            ui.separator();
            ui.label("Sound:");
            ui.add(DragValue::u8(&mut self.tim_sound));
        });
        ui.separator();
        ui.label("Stack:");
        ui.horizontal(|ui| {
            for i in 0..(self.sp as usize) {
                ui.add(DragValue::u16(&mut self.stack[i]));
            }
        });
    }
}
