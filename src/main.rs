#![windows_subsystem = "windows"] // Hides the console on windows

use gif::DisposalMethod;
use pixels::{Pixels, SurfaceTexture};
use winit::dpi::LogicalSize;
use winit::event::VirtualKeyCode;
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::WindowBuilder;
use winit_input_helper::WinitInputHelper;

use std::env;
use std::fs::File;
use std::path::PathBuf;
use std::process::exit;
use std::time::{Duration, Instant};

use rfd::FileDialog;

const DEFAULT_BG_COLOR: &[u8] = &[0x60, 0x60, 0x60, 0x00];

fn main() {
    let args: Vec<String> = env::args().collect();

    let chosen_file = if args.len() <= 1 {
        let files = FileDialog::new()
            .set_title("Select GIF")
            .set_directory("~/")
            .add_filter("gif", &["gif"])
            .pick_file();

        match files {
            Some(files) => files,
            None => exit(0),
        }
    } else {
        PathBuf::from(&args[1])
    };

    let file_in = File::open(chosen_file.clone()).unwrap();

    let mut decoder = gif::DecodeOptions::new();
    // Configure the decoder such that it will expand the image to RGBA.
    decoder.set_color_output(gif::ColorOutput::RGBA);

    let mut decoder_outer = decoder.read_info(file_in).unwrap();
    println!("Loaded gif...");

    let event_loop = EventLoop::new();
    let mut input = WinitInputHelper::new();
    let window = {
        let size = LogicalSize::new(
            decoder_outer.width() as f64,
            decoder_outer.height() as f64,
        );
        WindowBuilder::new()
            .with_title("GIF Viewer")
            .with_inner_size(size)
            .build(&event_loop)
            .unwrap()
    };

    //window.set_decorations(false);
    window.set_resizable(true);

    let mut pixels = {
        let window_size = window.inner_size();
        let surface_texture = SurfaceTexture::new(window_size.width, window_size.height, &window);
        Pixels::new(
            decoder_outer.width() as u32,
            decoder_outer.height() as u32,
            surface_texture,
        )
        .unwrap()
    };

    let mut frame_info = decoder_outer.next_frame_info().unwrap().unwrap().clone();
    let mut frame_buffer = vec![0; decoder_outer.buffer_size()];

    let mut delay_ms = frame_info.delay * 10;
    let mut now = Instant::now();

    #[allow(deprecated)]
    event_loop.run(move |event, _, control_flow| {
        // Handle input events
        if input.update(&event) {
            // Close events
            if input.key_pressed(VirtualKeyCode::Escape) || input.close_requested() {
                *control_flow = ControlFlow::Exit;
                return;
            }

            if input.mouse_pressed(0) && input.mouse().is_some() {
                window.drag_window().unwrap();
            } else if input.mouse_pressed(1) && input.mouse().is_some() {
                match window.drag_resize_window(winit::window::ResizeDirection::SouthEast) {
                    Ok(_) => (),
                    Err(_) => println!("Resize dragging is unsupported"),
                };
            }

            if let Some(size) = input.window_resized() && pixels.resize_surface(size.width, size.height).is_err() {
                *control_flow = ControlFlow::Exit;
                return;
            }
        }

        // Handle frame updates
        if now.elapsed() >= Duration::from_millis(delay_ms as u64) {
            now = Instant::now();

            let frame = pixels.frame_mut();

            decoder_outer.fill_buffer(&mut frame_buffer).unwrap();

            let mut frame_buffer_iter = frame_buffer.chunks(4);

            frame.chunks_mut(4)
                .enumerate()
                .for_each(|(i, f)| {
                    let y = i / decoder_outer.width() as usize;
                    let x = i % decoder_outer.width() as usize;

                    let mut g = DEFAULT_BG_COLOR;

                    if x < frame_info.left as usize || x >= frame_info.left as usize + frame_info.width as usize {
                        // If we are below the left offset OR above the frame width, skip this pixel
                    } else if y < frame_info.top as usize || y >= frame_info.top as usize + frame_info.height as usize {
                        // If we are above the top offset OR below the frame height, skip this pixel
                    } else {
                        g = frame_buffer_iter.next().unwrap()
                    }

                    match frame_info.dispose {
                        DisposalMethod::Any => f.copy_from_slice(g),
                        DisposalMethod::Keep if g[3] == 0x00 => (),
                        DisposalMethod::Keep => f.copy_from_slice(g),
                        DisposalMethod::Background => f.copy_from_slice(g),
                        DisposalMethod::Previous => (),
                    }
                });

            // Draw it to the `SurfaceTexture`
            pixels.render().unwrap();
            window.request_redraw();

            // Now that we've finished drawing the previous frame, get information
            // about the next one
            frame_info = match decoder_outer.next_frame_info().unwrap() {
                Some(i) => i.clone(),
                None => {
                    let file_in = File::open(chosen_file.clone()).unwrap();
                    let mut decoder = gif::DecodeOptions::new();
                    decoder.set_color_output(gif::ColorOutput::RGBA);
                    decoder_outer = decoder.read_info(file_in).unwrap();

                    decoder_outer.next_frame_info().unwrap().unwrap().clone()
                },
            };

            delay_ms = frame_info.delay * 10; // Calculate the next delay properly
        }
    });
}
