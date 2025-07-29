use std::{sync::Arc, time::Instant};

use colored::Colorize;
use glam::Vec3;
use gpu::{
    camera::{Camera, Camera3d},
    camera_controller::{CameraController, SmoothController},
};
use pollster::block_on;
use server::Server;
use winit::{
    dpi::PhysicalSize,
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

use crate::input::InputEventFilter;
mod console;
mod gpu;
mod input;
mod playground;
mod random;
mod server;
mod threader;
mod time;

fn main() {
    env_logger::init(); // this logs error messages
    let event_loop = EventLoop::new().unwrap();
    let window = WindowBuilder::new()
        .with_title("Voxeled")
        .with_inner_size(PhysicalSize::<u32> {
            width: 2000,
            height: 2000,
        }) // this is the window configuration
        .build(&event_loop)
        .unwrap();

    let mut delta_time = time::DeltaTimeMeter::new();
    let mut camera: Camera<SmoothController> = gpu::camera::Camera::new(
        Vec3::new(0.0, 50.0, 0.0),
        Vec3::new(1.0, 0.0, 0.0),
        delta_time.reader(),
    );

    let mut drawer = pollster::block_on(gpu::Drawer::connect_to(
        &window,
        wgpu::PresentMode::Fifo,
        &mut camera,
    )); // this connectes a drawer to the window
    let Ok(_) = console::Console::init(delta_time.reader()) else {
        println!("{}", "# failed to launch console".red());
        return;
    };

    let mut threadpool = threader::Threadpool::new();
    threadpool.launch(None);

    let elapsed_time = 0.0;
    let seed = random::get_random(0, u64::MAX);
    let noise = Arc::new(random::AnimatedNoise::new(
        seed as u32, // Seed für Reproduzierbarkeit
        1.0,         // time_scale - kleinere Werte = langsamere Animation
        0.06,        // space_scale - kleinere Werte = größere Strukturen
    ));
    println!("world seed: {:16x}", seed);
    let mut world = Server::new();

    let mut input_event_filter = input::InputEventFilter::new();
    let mut frame_number = 0;

    event_loop // main event loop
        .run(|event, control_flow| {
            if !input_event_filter.handled_event(&event, drawer.window.id()) {
                match event {
                    Event::WindowEvent { event, window_id } if window_id == drawer.window.id() => {
                        match event {
                            WindowEvent::Occluded(occluded) => match occluded {
                                true => control_flow.set_control_flow(ControlFlow::Wait),
                                false => {
                                    control_flow.set_control_flow(ControlFlow::Poll);
                                    drawer.reconfigure()
                                }
                            },
                            WindowEvent::Focused(focused) => match focused {
                                true => drawer.window.set_focus(true),
                                false => drawer.window.set_focus(false),
                            },
                            WindowEvent::CloseRequested => control_flow.exit(),
                            WindowEvent::Resized(physical_size) => {
                                drawer.resize(physical_size);
                            }
                            WindowEvent::RedrawRequested => {
                                delta_time.update();
                                threadpool.update();

                                if frame_number == 0 {
                                    block_on(drawer.draw(control_flow))
                                } else {
                                    update(
                                        &mut input_event_filter,
                                        &mut drawer,
                                        &mut world,
                                        &noise,
                                        elapsed_time,
                                        &mut threadpool,
                                    );
                                    // println!("time it took to build mesh in total: {:#?}", now.elapsed());
                                    block_on(drawer.draw(control_flow));
                                }
                                drawer.window.request_redraw(); // This tells winit that we want another frame after this one
                                frame_number += 1
                            }
                            _ => {}
                        }
                    }
                    Event::Suspended => control_flow.exit(),
                    _ => {}
                }
            }
        })
        .expect("event loop failed");
    threadpool.drop()
}
#[inline]
pub fn update(
    input_event_filter: &mut InputEventFilter,
    drawer: &mut gpu::Drawer<'_, SmoothController, Camera<SmoothController>>,
    world: &mut Server,
    noise: &Arc<random::AnimatedNoise>,
    elapsed_time: f64,
    threadpool: &mut threader::Threadpool,
) {
    let key_map = input_event_filter.get();

    if key_map.esc.just_pressed() {
        drawer.window.flip_focus()
    }

    let cam_pos = drawer.camera.controller().pos();
    let cam_dir = drawer.camera.controller().dir();
    drawer.update(&key_map);

    let now = Instant::now();
    drawer.update_mesh(world.get_mesh(
        cam_pos,
        cam_dir,
        Camera::<SmoothController>::FOV,
        drawer.window.aspect_ratio,
        12,
        noise.clone(),
        elapsed_time,
        threadpool,
    ));
}
