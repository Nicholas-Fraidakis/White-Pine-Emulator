use crate::cpu::Chip8Display;
use crate::cpu::*;
use macroquad::{miniquad::window::set_window_size, prelude::*};
use std::env;
use std::fs;
use std::ops::Index;
mod cpu;

const TILE_COLOUR: Color = Color {
    r: 155.0 / 255.0,
    g: 188.0 / 255.0,
    b: 15.0 / 255.0,
    a: 1.0,
};

const BACKGROUND_COLOUR: Color = Color {
    r: 48.0 / 255.0,
    g: 98.0 / 255.0,
    b: 48.0 / 255.0,
    a: 1.0,
};

#[macroquad::main("White Pine")]
async fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        println!("[Usage] (path-to-rom)");
        std::process::exit(0);
    }
    
    let mut arg_n = 1;
    while arg_n < args.len() {
        if let Ok(_) = fs::exists(args.index(arg_n)) {
            break;
        }
        arg_n += 1;
    }

    // Creates a new Chip8 and loads our little program
    let mut chip8 = Chip8::new(
        fs::read(args.index(arg_n))
            .as_ref()
            .expect("Couldn't load program!"),
    );

    set_window_size(640, 320);

    // Emulations the Chip8
    while chip8.is_running() {
        clear_background(BACKGROUND_COLOUR);

        chip8.ctx.update_delay();
        for _ in 0..11 {
            chip8.emulation_cycle();
        }
        draw_chip8_display(chip8.get_display(), Vec2 { x: 640.0, y: 320.0 });

        next_frame().await
    }
}

fn draw_chip8_display(display: &Chip8Display, screen_size: Vec2) {
    let (tile_x_size, tile_y_size) = (screen_size.x / 64.0, screen_size.y / 32.0);
    for y in 0..32 {
        for x in 0..64 {
            if display.access_position(x, y) {
                draw_rectangle(
                    x as f32 * tile_x_size,
                    y as f32 * tile_y_size,
                    tile_x_size,
                    tile_y_size,
                    TILE_COLOUR,
                );
            }
        }
    }
}
