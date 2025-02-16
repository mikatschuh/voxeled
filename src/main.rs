use std::sync::Arc;

use glam::Vec3;
use gpu::{
    camera::{Camera, Camera3d},
    camera_controller::{CameraController, SmoothController},
};
use server::Server;
use winit::{
    dpi::PhysicalSize,
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};
mod gpu;
mod input;
mod playground;
mod random;
mod server;
mod threader;
mod time;

fn main() {
    pollster::block_on(run());
}
async fn run() {
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
    let mut camera: Camera<SmoothController> =
        gpu::camera::Camera::new(Vec3::new(0.0, 40.0, 0.0), Vec3::new(1.0, 1.0, 0.0));
    let mut drawer = gpu::Drawer::connect_to(&window, wgpu::PresentMode::Fifo, &mut camera).await; // this connectes a drawer to the window
    let mut keys = input::Keys::new();

    let elapsed_time = 0.0;
    let noise = Arc::new(server::voxel::AnimatedNoise::new(
        random::get_random(0, 100), // Seed für Reproduzierbarkeit
        1.0,                        // time_scale - kleinere Werte = langsamere Animation
        0.2,                        // space_scale - kleinere Werte = größere Strukturen
    ));
    let mut world = Server::new();

    let mut delta_time = time::DeltaTime::now();

    let mut threadpool = threader::Threadpool::new();
    threadpool.launch(None);

    event_loop // main event loop
        .run(move |event, control_flow| {
            if !keys.handled_event(drawer.window.id(), &event) {
                match event {
                    Event::WindowEvent { event, window_id } // checks if its the right window
                        if window_id == drawer.window.id() =>
                    {
                        match event {
                            WindowEvent::Occluded(occluded) => if occluded {
                                control_flow.set_control_flow(ControlFlow::Wait);
                            } else {
                                control_flow.set_control_flow(ControlFlow::Poll);
                                drawer.reconfigure()
                            }

                            WindowEvent::CloseRequested => {
                                // do saving and stuff
                                threadpool.drop();
                                control_flow.exit();
                            },
                            WindowEvent::Resized(physical_size) => {
                                drawer.resize(physical_size);
                            }
                            WindowEvent::RedrawRequested => {
                                // This tells winit that we want another frame after this one
                                drawer.window.request_redraw();

                                threadpool.update();

                                if keys.esc.just_pressed() { drawer.window.flip_focus() }

                                let cam_pos = drawer.camera().controller().pos();
                                let cam_dir = drawer.camera().controller().dir();

                                drawer.update_mesh(&world.get_mesh(
                                    cam_pos,
                                    cam_dir,
                                    Camera::<SmoothController>::FOV,
                                    drawer.window.aspect_ratio,
                                    3.0,
                                    noise.clone(),
                                    elapsed_time,
                                    &mut threadpool
                                ));

                                if drawer.window.focused() { drawer.update(&keys, delta_time.update() as f32) }

                                match drawer.draw() {
                                    Ok(_) => {}
                                    // Reconfigure the surface if it's lost or outdated
                                    Err(
                                        wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated,
                                    ) => drawer.reconfigure(),
                                    // The system is out of memory, we should probably quit
                                    Err(wgpu::SurfaceError::OutOfMemory) => {
                                        log::error!("OutOfMemory");
                                        control_flow.exit();
                                    }

                                    // This happens when the a frame takes too long to present
                                    Err(wgpu::SurfaceError::Timeout) => {
                                        log::warn!("Surface timeout")
                                    }
                                }
                                keys.update()
                            }
                            _ => {}
                        }
                    }
                    _ => {}
                }
            }
        })
        .expect("event loop failed");
}
