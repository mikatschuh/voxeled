use gpu::{
    camera::{Camera, Camera3d},
    camera_controller::SmoothController,
};
use server::chunk::generate_mesh;
use threader::task::Task;
use winit::{
    dpi::PhysicalSize,
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::{Window, WindowBuilder},
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
    let mut camera: Camera<SmoothController> = gpu::camera::Camera::new(
        glam::Vec3::new(0.0, 40.0, 0.0),
        glam::Vec3::new(1.0, 1.0, 0.0),
    );
    let mut drawer = gpu::Drawer::connect_to(
        &window,
        wgpu::PresentMode::Fifo,
        generate_mesh(
            glam::IVec3::new(0, 0, 0),
            server::chunk::Chunk::from_white_noise().create_faces(),
        ),
        &mut camera,
    )
    .await; // this connectes a drawer to the window
    let mut delta_time = time::DeltaTime::now();
    let mut keys = input::Keys::new();

    let mut elapsed_time = 0.0;
    let noise = server::voxel::AnimatedNoise::new(
        42,  // Seed für Reproduzierbarkeit
        1.0, // time_scale - kleinere Werte = langsamere Animation
        0.1, // space_scale - kleinere Werte = größere Strukturen
    );

    let mut threadpool = threader::Threadpool::new(None);
    event_loop // main event loop
        .run(move |event, control_flow| {
            if !keys.handled_event(drawer.window.id(), &event) {
                match event {
                    Event::NewEvents(StartCause::Init) => {
                        // Initial frame time
                        delta_time = time::DeltaTime::now();
                        for num in 0..10 {
                            threadpool.add_priority(Task::new_benchmark(format!("priority {}", num).leak()))
                        }
                        for num in 0..10 {
                            threadpool.add_to_first(Task::new_benchmark(format!("first {}", num).leak()))
                        }
                        for num in 0..10 {
                            threadpool.add_to_second(Task::new_benchmark(format!("second {}", num).leak()))
                        }
                    }
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
                                control_flow.exit()
                            },
                            WindowEvent::Resized(physical_size) => {
                                drawer.resize(physical_size);
                            }
                            WindowEvent::RedrawRequested => {
                                // This tells winit that we want another frame after this one
                                drawer.window.request_redraw();

                                if keys.esc.just_pressed() { drawer.window.flip_focus() }

                                let delta_time = delta_time.update();
                                elapsed_time += delta_time as f64 / 1_000_000_000.0;
                                // drawer.update_mesh(&generate_mesh(    glam::IVec3::new(0, 0, 0),server::chunk::Chunk::from_perlin_noise(&noise, elapsed_time).create_faces(),));
                                drawer.update_mesh(&generate_mesh(
                                    glam::IVec3::new(0, 0, 0),
                                    server::chunk::Chunk::from_pyramide().create_faces(),
                                ));

                                if drawer.window.focused() { drawer.update(&keys, delta_time as f32) }

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
