use std::sync::Arc;

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

use crate::{input::Inputs, server::world_gen::Generator};

mod console;
mod gpu;
mod input;
// mod old_input;
mod netcode;
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
    println!("world seed: {:16x}", seed);
    let mut server = Server::<server::world_gen::OpenCaves>::new(seed);
    let generator = server.expose_generator();

    let mut input_event_filter = input::InputEventFilter::new().expect("input event filter");
    let mut frame_number = 0;

    // ping the server:

    netcode::connect("127.0.0.1:5000").expect("connection");

    event_loop // main event loop
        .run(|event, control_flow| {
            if !input_event_filter.could_handle(&event, drawer.window.id()) {
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

                                let inputs = input_event_filter.get();

                                if frame_number == 0 {
                                    drawer.draw(control_flow)
                                } else {
                                    update(
                                        inputs,
                                        &mut drawer,
                                        &mut server,
                                        &generator,
                                        elapsed_time,
                                        &mut threadpool,
                                    );
                                    // println!("time it took to build mesh in total: {:#?}", now.elapsed());
                                    drawer.draw(control_flow);
                                }
                                input_event_filter.frame_done();
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
pub fn update<G: Generator>(
    inputs: &mut Inputs,
    drawer: &mut gpu::Drawer<'_, SmoothController, Camera<SmoothController>>,
    server: &mut Server<G>,
    generator: &Arc<std::sync::RwLock<G>>,
    elapsed_time: f64,
    threadpool: &mut threader::Threadpool,
) {
    if inputs.esc {
        drawer.window.flip_focus()
    }

    let cam_pos = drawer.camera.controller().pos();
    let cam_dir = drawer.camera.controller().dir();
    drawer.update(inputs);

    // let now = Instant::now();
    // generator.write().unwrap().vertical_area *= 1.001;
    drawer.update_mesh(server.get_mesh(
        cam_pos,
        cam_dir,
        Camera::<SmoothController>::FOV,
        drawer.window.aspect_ratio,
        8,
        threadpool,
    ));
}
