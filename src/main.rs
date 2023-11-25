#![windows_subsystem = "windows"] // Hides the console on windows

use pixels::{Pixels, SurfaceTexture};
use winit::dpi::LogicalSize;
use winit::event::{Event, VirtualKeyCode};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::WindowBuilder;
use winit_input_helper::WinitInputHelper;

use image::codecs::gif::GifDecoder;
use image::AnimationDecoder;

use std::env;
use std::fs::File;
use std::path::PathBuf;
use std::process::exit;
use std::time::Instant;

use rfd::FileDialog;

use colored::Colorize;

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

    let file_in = File::open(chosen_file).unwrap();
    let decoder = GifDecoder::new(file_in).unwrap();
    println!("Loaded gif...");
    let frames = decoder.into_frames();
    println!("Loaded frames...");

    // TODO: Bad!
    let frames = frames.collect_frames().expect("error decoding gif");
    println!("Collected frames...");

    let event_loop = EventLoop::new();
    let mut input = WinitInputHelper::new();
    let window = {
        let size = LogicalSize::new(
            frames[0].buffer().width() as f64,
            frames[0].buffer().height() as f64,
        );
        WindowBuilder::new()
            .with_title("GIF Viewer")
            .with_inner_size(size)
            .build(&event_loop)
            .unwrap()
    };

    window.set_decorations(false);
    window.set_resizable(true);

    let mut pixels = {
        let window_size = window.inner_size();
        let surface_texture = SurfaceTexture::new(window_size.width, window_size.height, &window);
        Pixels::new(
            frames[0].buffer().width(),
            frames[0].buffer().height(),
            surface_texture,
        )
        .unwrap()
    };

    let mut delay = frames[0].delay();
    let mut delay_ms = delay.numer_denom_ms().0 / delay.numer_denom_ms().1;
    let mut now = Instant::now();
    let mut index = 0;
    let mut paused = false;
    let mut progress = true;
    event_loop.run(move |event, _, control_flow| {
        if let Event::WindowEvent {
                event: winit::event::WindowEvent::MouseWheel { delta, .. },
                ..
            } = event { match delta {
            winit::event::MouseScrollDelta::LineDelta(_, value) => {
                index = (index + ((value) as isize) as usize) % frames.len();
            }
            winit::event::MouseScrollDelta::PixelDelta(_) => (),
        } }


        // Handle input events
        if input.update(&event) {
            // Close events
            if input.key_pressed(VirtualKeyCode::Escape) || input.close_requested() {
                *control_flow = ControlFlow::Exit;
                return;
            }

            if input.key_pressed(VirtualKeyCode::Space) {
                paused = !paused;
            }

            if input.key_pressed(VirtualKeyCode::P) {
                progress = !progress;
            }

            if input.key_pressed(VirtualKeyCode::Left) {
                index = (index - 10) % frames.len();
            }

            if input.key_pressed(VirtualKeyCode::Right) {
                index = (index + 10) % frames.len();
            }

            if input.mouse_pressed(0) && input.mouse().is_some() {
                window.drag_window().unwrap();
            } else if input.mouse_pressed(1) && input.mouse().is_some() {
                match window.drag_resize_window(winit::window::ResizeDirection::SouthEast) {
                    Ok(_) => (),
                    Err(_) => println!("Resize dragging is {}", "unsupported".red()),
                };
            }

            if let Some(size) = input.window_resized() {
                if pixels.resize_surface(size.width, size.height).is_err() {
                    *control_flow = ControlFlow::Exit;
                    return;
                }
            }
        }

        if now.elapsed() >= delay.into() {
            let time_taken = now.elapsed();
            let gif_frame = &frames[index];
            if ((delay_ms - 2)..(delay_ms + 2)).contains(&(time_taken.as_millis() as u32)) {
                //println!("Frame #{:0>5}: {}ms ≈ {}ms", index + 1, time_taken.as_millis().to_string().green(), delay_ms.to_string())
            } else if ((delay_ms - 10)..(delay_ms + 10)).contains(&(time_taken.as_millis() as u32))
            {
                println!(
                    "Frame #{:0>5}: {}ms > {}ms",
                    index + 1,
                    time_taken.as_millis().to_string().yellow(),
                    delay_ms.to_string().bold()
                )
            } else {
                println!(
                    "Frame #{:0>5}: {}ms > {}ms",
                    index + 1,
                    time_taken.as_millis().to_string().red(),
                    delay_ms.to_string().bold()
                )
            }
            now = Instant::now();
            // Update internal state and request a redraw
            let frame = pixels.frame_mut();
            let width = gif_frame.buffer().width() as usize;
            let height = gif_frame.buffer().height() as usize;
            for (i, pixel) in frame
                .chunks_exact_mut(4)
                .zip(gif_frame.buffer().chunks_exact(4))
                .enumerate()
            {
                if i > (height - 25) * width
                    && ((i % width) as f32) < ((index as f32 / frames.len() as f32) * width as f32)
                    && progress
                {
                    pixel.0[0] = 255 - pixel.1[0]; // R
                    pixel.0[1] = 255 - pixel.1[1];
                    pixel.0[2] = 255 - pixel.1[2]; // B
                    pixel.0[3] = 0xFF; // A
                    continue;
                }
                pixel.0[0] = pixel.1[0]; // R
                pixel.0[1] = pixel.1[1]; // G
                pixel.0[2] = pixel.1[2]; // B
                pixel.0[3] = 0xFF // A
            }

            // Draw it to the `SurfaceTexture`
            pixels.render().unwrap();
            window.request_redraw();
            if !paused {
                index += 1;
            }
            if index >= frames.len() {
                index = 0;
            }
            delay = gif_frame.delay();
            delay_ms = delay.numer_denom_ms().0 / delay.numer_denom_ms().1;
        }
    });
}
