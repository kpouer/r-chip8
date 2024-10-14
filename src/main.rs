use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

use pixels::{Pixels, SurfaceTexture};
use winit::dpi::LogicalSize;
use winit::event::Event;
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::WindowBuilder;

mod chip8;
mod cpu;
mod display;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 2 {
        println!("Usage: chip8 <rom>");
        std::process::exit(1);
    }
    let rom_name = args.get(1).unwrap();
    println!("rom_name: {}", rom_name);
    let rom_bytes = std::fs::read(rom_name).unwrap();
    let chip8 = Arc::new(Mutex::new(chip8::Chip8::new(rom_bytes)));
    let event_loop = EventLoop::new();
    let width = 64;
    let height = 32;
    let window = {
        let size = LogicalSize::new(width as f64, width as f64);
        let scaled_size = LogicalSize::new(width as f64 * 3.0, width as f64 * 3.0);
        WindowBuilder::new()
            .with_title("Chip 8")
            .with_inner_size(scaled_size)
            .with_min_inner_size(size)
            .build(&event_loop)
            .unwrap()
    };

    let mut pixels = {
        let window_size = window.inner_size();
        let surface_texture = SurfaceTexture::new(window_size.width, window_size.height, &window);
        Pixels::new(width as u32, width as u32, surface_texture).unwrap()
    };

    {
        let chip8 = chip8.clone();
        thread::spawn(move || loop {
            chip8.lock().unwrap().cycle();
        });
    }
    let sleep_duration = Duration::from_millis(2);
    let mut time = Instant::now();
    event_loop.run(move |event, _, control_flow| {
        control_flow.set_poll();
        // The one and only event that winit_input_helper doesn't have for us...
        match event {
            Event::RedrawRequested(_) => {
                {
                    let chip8 = chip8.clone();
                    let chip8 = chip8.lock().unwrap();
                    render(width, height, &chip8.display.vram, &mut pixels);
                }
                if let Err(err) = pixels.render() {
                    println!("pixels.render() failed: {}", err);
                    // log_error("pixels.render", err);
                    *control_flow = ControlFlow::Exit;
                    return;
                }
            }
            Event::WindowEvent {
                event: winit::event::WindowEvent::CloseRequested,
                ..
            } => {
                control_flow.set_exit();
            }
            _ => {}
        }
        match *control_flow {
            _ => {
                if time.elapsed() >= sleep_duration {
                    time = Instant::now();
                    if chip8.lock().unwrap().should_render() {
                        chip8.lock().unwrap().display.reset_dirty_flag();
                        window.request_redraw();
                    }
                }
            }
        }
    });
}

fn render(width: u8, height: u8, vram: &[[bool; 64]; 32], pixels: &mut Pixels) {
    const BLACK: [u8; 4] = [0x00, 0x00, 0x00, 0xff];
    const WHITE: [u8; 4] = [0xff, 0xff, 0xff, 0xff];
    let frame = pixels.frame_mut();

    for (i, pixel) in frame.chunks_exact_mut(4).enumerate() {
        let x = i / width as usize;
        let y = i % height as usize;
        let color = vram[y][x];

        if color {
            pixel.copy_from_slice(&BLACK);
        } else {
            pixel.copy_from_slice(&WHITE);
        }
    }
}
