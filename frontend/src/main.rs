use core::*;

use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::Canvas;
use sdl2::video::Window;

use std::env;
use std::fs::File;
use std::io::Read;

const SCALE: u32 = 15; // 15x native scale
const TICKS_PER_FRAME: usize = 10; // Chip-8 has any defined clock speed, this is a easier way to set refresh rate
const WINDOW_HEIGHT: u32 = (SCREEN_HEIGHT as u32) * SCALE;
const WINDOW_WIDTH: u32 = (SCREEN_WIDTH as u32) * SCALE;

fn main() {
    let args: Vec<_> = env::args().collect();
    if args.len() != 2 {
        println!("Usage: cargo run <path-to-game>");
        return;
    }

    let sdl = sdl2::init().unwrap();
    let mut emu = Emulator::new();

    let mut rom = File::open(&args[1]).expect("Unable to open the rom requested.");
    let mut buffer = Vec::new();
    rom.read_to_end(&mut buffer).expect("Unable to read rom.");
    emu.load(&buffer);

    let video_subsystem = sdl.video().unwrap();
    let window = video_subsystem
        .window("chip-r", WINDOW_WIDTH, WINDOW_HEIGHT)
        .position_centered()
        .opengl()
        .build()
        .unwrap();

    let mut canvas = window.into_canvas().present_vsync().build().unwrap();
    canvas.clear();
    canvas.present();

    let mut event_pump = sdl.event_pump().unwrap();
    'gameloop: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => {
                    break 'gameloop;
                }
                Event::KeyDown {
                    keycode: Some(key), ..
                } => {
                    let command = key_to_button(key);
                    if command.is_some() {
                        emu.keypress(command.unwrap(), true);
                    }
                }
                Event::KeyUp {
                    keycode: Some(key), ..
                } => {
                    let command = key_to_button(key);
                    if command.is_some() {
                        emu.keypress(command.unwrap(), false);
                    }
                }
                _ => (),
            }
        }

        // Refresh rate of drawing
        for _ in 0..TICKS_PER_FRAME {
            emu.tick();
        }
        emu.tick_timers();
        draw_screen(&emu, &mut canvas);
    }
}

fn draw_screen(emu: &Emulator, canvas: &mut Canvas<Window>) {
    canvas.set_draw_color(Color::RGB(0, 0, 0));
    canvas.clear();

    let screen_buffer = emu.get_display();
    canvas.set_draw_color(Color::RGB(0, 255, 0));
    for (i, pixel) in screen_buffer.iter().enumerate() {
        if *pixel {
            // Convert our 1D array's index into a 2D (x,y) position
            let x = (i % SCREEN_WIDTH) as u32;
            let y = (i / SCREEN_WIDTH) as u32;

            let rect = Rect::new((x * SCALE) as i32, (y * SCALE) as i32, SCALE, SCALE);
            canvas.fill_rect(rect).unwrap();
        }
    }

    canvas.present();
}

fn key_to_button(key: Keycode) -> Option<usize> {
    match key {
        Keycode::Num1 => Some(0x1),
        Keycode::Num2 => Some(0x2),
        Keycode::Num3 => Some(0x3),
        Keycode::Num4 => Some(0xC),
        Keycode::Q => Some(0x4),
        Keycode::W => Some(0x5),
        Keycode::E => Some(0x6),
        Keycode::R => Some(0xD),
        Keycode::A => Some(0x7),
        Keycode::S => Some(0x8),
        Keycode::D => Some(0x9),
        Keycode::F => Some(0xE),
        Keycode::Z => Some(0xA),
        Keycode::X => Some(0x0),
        Keycode::C => Some(0xB),
        Keycode::V => Some(0xF),

        _ => None,
    }
}
