use macroquad::prelude::*;
use ply_engine::prelude::*;
use rchip::Emulator;

use std::cell::{Cell, RefCell};
use std::fs;
use std::rc::Rc;
use std::time::Duration;

const KEY_MAP: [(KeyCode, u8); 16] = [
    (KeyCode::Key1, 0x1),
    (KeyCode::Key2, 0x2),
    (KeyCode::Key3, 0x3),
    (KeyCode::Key4, 0xC),
    (KeyCode::Q, 0x4),
    (KeyCode::W, 0x5),
    (KeyCode::E, 0x6),
    (KeyCode::R, 0xD),
    (KeyCode::A, 0x7),
    (KeyCode::S, 0x8),
    (KeyCode::D, 0x9),
    (KeyCode::F, 0xE),
    (KeyCode::Z, 0xA),
    (KeyCode::X, 0x0),
    (KeyCode::C, 0xB),
    (KeyCode::V, 0xF),
];

fn window_conf() -> macroquad::conf::Conf {
    macroquad::conf::Conf {
        miniquad_conf: miniquad::conf::Conf {
            window_title: "rchip - CHIP-8 Emulator".to_owned(),
            window_width: 800,
            window_height: 600,
            high_dpi: true,
            sample_count: 4,
            platform: miniquad::conf::Platform {
                webgl_version: miniquad::conf::WebGLVersion::WebGL2,
                ..Default::default()
            },
            ..Default::default()
        },
        draw_call_vertex_capacity: 100000,
        draw_call_index_capacity: 100000,
        ..Default::default()
    }
}

fn handle_keyboard_input(emulator: &mut Emulator) {
    for (physical_key, chip8_key) in KEY_MAP {
        if is_key_pressed(physical_key) {
            emulator.set_key(chip8_key, true);
        }

        if is_key_released(physical_key) {
            emulator.set_key(chip8_key, false);
        }
    }
}

fn draw_chip8_display(pixels: &[bool; 2048], x: f32, y: f32, pixel_size: f32) {
    for row in 0..32 {
        for col in 0..64 {
            let index = row * 64 + col;
            let color = if pixels[index] { WHITE } else { BLACK };

            draw_rectangle(
                x + col as f32 * pixel_size,
                y + (31 - row) as f32 * pixel_size,
                pixel_size,
                pixel_size,
                color,
            );
        }
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    static DEFAULT_FONT: FontAsset = FontAsset::Path("assets/fonts/jetbrains_mono.ttf");

    let mut ply = Ply::<()>::new(&DEFAULT_FONT).await;
    let mut emulator = Emulator::new(700);

    let rom = fs::read("roms/Pong (1 player).ch8").expect("Failed to read ROM");

    emulator.load_program(&rom).expect("Failed to load ROM");

    let paused = Rc::new(Cell::new(false));
    let reset_requested = Rc::new(Cell::new(false));
    let rom_picker_requested = Rc::new(Cell::new(false));

    let rom_name = Rc::new(RefCell::new("Pong (1 player).ch8".to_string()));

    loop {
        clear_background(BLACK);

        // Handle pause keyboard shortcut.
        if is_key_pressed(KeyCode::P) {
            paused.set(!paused.get());
        }

        // Handle ROM reset requests.
        if reset_requested.replace(false) {
            emulator.reset();
            paused.set(false);
        }

        // Handle CHIP-8 keyboard input.
        handle_keyboard_input(&mut emulator);

        // Handle ROM loading requests.
        if rom_picker_requested.replace(false) {
            if let Some(path) = rfd::FileDialog::new()
                .add_filter("CHIP-8 ROM", &["ch8"])
                .pick_file()
            {
                match fs::read(&path) {
                    Ok(program) => match emulator.load_program(&program) {
                        Ok(()) => {
                            if let Some(filename) = path.file_name() {
                                *rom_name.borrow_mut() = filename.to_string_lossy().into_owned();
                            }

                            paused.set(false);
                            println!("Loaded ROM: {}", path.display());
                        }

                        Err(error) => {
                            eprintln!("Failed to load ROM: {error}");
                        }
                    },

                    Err(error) => {
                        eprintln!("Failed to read ROM: {error}");
                    }
                }
            }
        }

        // Advance the emulator.
        if !paused.get() {
            let elapsed = Duration::from_secs_f32(get_frame_time());

            if let Err(error) = emulator.advance(elapsed) {
                eprintln!("Emulator error: {error}");
                paused.set(true);
            }
        }

        // Render CHIP-8 display to a texture.
        let display_texture = render_to_texture(640.0, 320.0, || {
            clear_background(BLACK);
            draw_chip8_display(emulator.pixels(), 0.0, 0.0, 10.0);
        });

        let registers = emulator.registers();
        let mut ui = ply.begin();

        ui.element()
            .width(grow!())
            .height(grow!())
            .layout(|l| {
                l.direction(TopToBottom)
                    .gap(12)
                    .padding(16)
            })
            .children(|ui| {
                // Header.
                ui.element()
                    .width(grow!())
                    .height(fixed!(64.0))
                    .background_color(0x20242C)
                    .layout(|l| l.align(CenterX, CenterY).padding(16))
                    .children(|ui| {
                        ui.text("RCHIP", |t| {
                            t.font_size(28).color(0xFFFFFF)
                        });
                    });

                // Main content.
                ui.element()
                    .width(grow!())
                    .height(grow!())
                    .layout(|l| l.direction(LeftToRight).gap(12))
                    .children(|ui| {
                        // Display panel.
                        ui.element()
                            .id("display_panel")
                            .width(percent!(0.68))
                            .height(grow!())
                            .background_color(0x20242C)
                            .layout(|l| {
                                l.direction(TopToBottom)
                                    .gap(16)
                                    .padding(16)
                            })
                            .children(|ui| {
                                ui.text("DISPLAY", |t| {
                                    t.font_size(20).color(0xFFFFFF)
                                });

                                ui.element()
                                    .width(grow!())
                                    .height(grow!())
                                    .aspect_ratio(2.0)
                                    .image(display_texture.clone())
                                    .empty();
                            });

                        // CPU information panel.
                        let button_paused = Rc::clone(&paused);

                        ui.element()
                            .width(grow!())
                            .height(grow!())
                            .background_color(0x20242C)
                            .layout(|l| {
                                l.direction(TopToBottom)
                                    .gap(12)
                                    .padding(16)
                            })
                            .children(|ui| {
                                ui.text("CPU STATE", |t| {
                                    t.font_size(20).color(0xFFFFFF)
                                });

                                let current_rom = rom_name.borrow();

                                ui.text(
                                    &format!("ROM: {}", *current_rom),
                                    |t| t.font_size(14).color(0xA3A3A3),
                                );

                                ui.text(
                                    if paused.get() {
                                        "STATUS: PAUSED"
                                    } else {
                                        "STATUS: RUNNING"
                                    },
                                    |t| t.font_size(14).color(0xFFFFFF),
                                );

                                // Pause / resume button.
                                ui.element()
                                    .id("pause_button")
                                    .width(fit!())
                                    .height(fit!())
                                    .on_press(move |_, _| {
                                        button_paused.set(!button_paused.get());
                                    })
                                    .accessibility(|a| {
                                        a.button("Pause / Resume")
                                    })
                                    .children(|ui| {
                                        let bg = if ui.pressed() {
                                            0x1D4ED8
                                        } else if ui.hovered() || ui.focused() {
                                            0x3B82F6
                                        } else {
                                            0x2563EB
                                        };

                                        ui.element()
                                            .width(fit!())
                                            .height(fit!())
                                            .background_color(bg)
                                            .corner_radius(6.0)
                                            .layout(|l| {
                                                l.padding(10).align(CenterX, CenterY)
                                            })
                                            .children(|ui| {
                                                ui.text(
                                                    if paused.get() {
                                                        "RESUME"
                                                    } else {
                                                        "PAUSE"
                                                    },
                                                    |t| {
                                                        t.font_size(14).color(0xFFFFFF)
                                                    },
                                                );
                                            });
                                    });

                                // Reset button.
                                let button_reset = Rc::clone(&reset_requested);

                                ui.element()
                                    .id("reset_button")
                                    .width(fit!())
                                    .height(fit!())
                                    .on_press(move |_, _| {
                                        button_reset.set(true);
                                    })
                                    .accessibility(|a| {
                                        a.button("Reset ROM")
                                    })
                                    .children(|ui| {
                                        let bg = if ui.pressed() {
                                            0xB91C1C
                                        } else if ui.hovered() || ui.focused() {
                                            0xEF4444
                                        } else {
                                            0xDC2626
                                        };

                                        ui.element()
                                            .width(fit!())
                                            .height(fit!())
                                            .background_color(bg)
                                            .corner_radius(6.0)
                                            .layout(|l| {
                                                l.padding(10).align(CenterX, CenterY)
                                            })
                                            .children(|ui| {
                                                ui.text("RESET ROM", |t| {
                                                    t.font_size(14).color(0xFFFFFF)
                                                });
                                            });
                                    });

                                // Load ROM button.
                                let button_rom_picker =
                                    Rc::clone(&rom_picker_requested);

                                ui.element()
                                    .id("load_rom_button")
                                    .width(fit!())
                                    .height(fit!())
                                    .on_press(move |_, _| {
                                        button_rom_picker.set(true);
                                    })
                                    .accessibility(|a| {
                                        a.button("Load ROM")
                                    })
                                    .children(|ui| {
                                        let bg = if ui.pressed() {
                                            0x047857
                                        } else if ui.hovered() || ui.focused() {
                                            0x10B981
                                        } else {
                                            0x059669
                                        };

                                        ui.element()
                                            .width(fit!())
                                            .height(fit!())
                                            .background_color(bg)
                                            .corner_radius(6.0)
                                            .layout(|l| {
                                                l.padding(10).align(CenterX, CenterY)
                                            })
                                            .children(|ui| {
                                                ui.text("LOAD ROM", |t| {
                                                    t.font_size(14).color(0xFFFFFF)
                                                });
                                            });
                                    });

                                // General-purpose registers.
                                ui.text("REGISTERS", |t| {
                                    t.font_size(16).color(0xAAAAAA)
                                });

                                for row in 0..4 {
                                    let start = row * 4;

                                    let register_text = format!(
                                        "V{:X}: {:02X}  V{:X}: {:02X}  V{:X}: {:02X}  V{:X}: {:02X}",
                                        start,
                                        registers[start],
                                        start + 1,
                                        registers[start + 1],
                                        start + 2,
                                        registers[start + 2],
                                        start + 3,
                                        registers[start + 3],
                                    );

                                    ui.text(&register_text, |t| {
                                        t.font_size(14).color(0xFFFFFF)
                                    });
                                }

                                // Special registers.
                                ui.text("SPECIAL REGISTERS", |t| {
                                    t.font_size(16).color(0xAAAAAA)
                                });

                                ui.text(
                                    &format!("PC: {:04X}", emulator.pc()),
                                    |t| t.font_size(14).color(0xFFFFFF),
                                );

                                ui.text(
                                    &format!("I:  {:04X}", emulator.i_register()),
                                    |t| t.font_size(14).color(0xFFFFFF),
                                );

                                // Timers.
                                ui.text("TIMERS", |t| {
                                    t.font_size(16).color(0xAAAAAA)
                                });

                                ui.text(
                                    &format!("DT: {:02X}", emulator.delay_timer()),
                                    |t| t.font_size(14).color(0xFFFFFF),
                                );

                                ui.text(
                                    &format!("ST: {:02X}", emulator.sound_timer()),
                                    |t| t.font_size(14).color(0xFFFFFF),
                                );
                            });
                    });
            });

        ui.show(|_| {}).await;
        next_frame().await;
    }
}
