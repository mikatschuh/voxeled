use colored::Colorize;
use glam::Vec3;
use gpu::camera::Camera;
use server::Server;
use winit::{
    dpi::PhysicalSize,
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

use crate::{
    input::Inputs,
    server::{frustum::Frustum, world_gen::Generator},
};

mod collision;
mod console;
mod gpu;
mod input;
// mod old_input;
#[allow(dead_code)]
mod data_structures;
mod netcode;
#[allow(unused)]
mod playground;
mod random;
mod server;
mod threadpool;
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
    let mut camera = gpu::camera::Camera::new(
        Vec3::new(0.0, -50.0, 0.0),
        Vec3::new(0.0, 0.0, 1.0),
        true,
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

    let mut threadpool = threadpool::Threadpool::new(num_cpus::get() - 1);

    let seed = 0x6bfb999977f4cd52; //random::get_random(0, u64::MAX);
    println!("world seed: {:16x}", seed);

    let generator = server::world_gen::OpenCaves::new(seed);
    let mut server = Server::new(generator);

    let mut input_event_filter = input::InputEventFilter::new().expect("input event filter");
    let mut frame_number = 0;
    let mut change_mesh = true;

    // ping the server:

    // netcode::connect("127.0.0.1:5000").expect("connection");

    event_loop // main event loop
        .run(|event, control_flow| {
            if !input_event_filter.could_handle(&event, drawer.window.id(), drawer.window.focused())
            {
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
                            WindowEvent::Focused(focused) => drawer.window.set_focus(focused),
                            WindowEvent::CloseRequested => control_flow.exit(),
                            WindowEvent::Resized(physical_size) => {
                                drawer.resize(physical_size);
                            }
                            WindowEvent::RedrawRequested => {
                                delta_time.update();

                                let inputs = input_event_filter.get();

                                if frame_number == 0 {
                                    drawer.draw(control_flow)
                                } else {
                                    update(
                                        &mut change_mesh,
                                        inputs,
                                        &mut drawer,
                                        &mut server,
                                        &mut threadpool,
                                    );
                                    // println!("time it took to build mesh in total: {:#?}", now.elapsed());
                                    drawer.draw(control_flow);
                                }
                                if frame_number % 60 == 0 {
                                    // println!("{}\n", threadpool.debug_log())
                                    threadpool.debug_log();
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

const FULL_DETAL_DISTANCE: f32 = 5.;
const RENDER_DISTANCE: f32 = 48.;
const GRAVITY: f32 = 9.81;
const WALK_JUMP_SPEED: f32 = 5000.;

#[inline]
pub fn update<G: Generator>(
    change_mesh: &mut bool,
    inputs: &mut Inputs,
    drawer: &mut gpu::Drawer<'_>,
    server: &mut Server<G>,
    threadpool: &mut threadpool::Threadpool<G>,
) {
    if inputs.pause {
        drawer.window.flip_focus()
    }
    if inputs.remesh {
        *change_mesh = !*change_mesh;
    }

    drawer.update_cam(|camera| {
        if inputs.free_cam {
            camera.toggle_free_cam();
        }

        if let Some(mouse_motion) = inputs.mouse_motion {
            camera.rotate_around_angle(glam::Vec3::new(
                -mouse_motion.x as f32,
                mouse_motion.y as f32,
                0.,
            ));
        }

        if let Some(scroll) = inputs.mouse_wheel {
            camera.update_acc(scroll.y)
        }

        let free_cam = camera.free_cam();
        let on_ground = !free_cam && collision::is_on_ground(server, camera.pos());

        let input_vector = if free_cam {
            glam::Vec3::new(
                inputs.right.process_f32() - inputs.left.process_f32(),
                inputs.down.process_f32() - inputs.up.process_f32(),
                inputs.backwards.process_f32() - inputs.forward.process_f32(),
            )
        } else if on_ground {
            _ = inputs.up.process_f32();
            _ = inputs.down.process_f32();

            glam::Vec3::new(
                inputs.right.process_f32() - inputs.left.process_f32(),
                0.0,
                inputs.backwards.process_f32() - inputs.forward.process_f32(),
            )
        } else {
            _ = inputs.right.process_f32();
            _ = inputs.left.process_f32();
            _ = inputs.up.process_f32();
            _ = inputs.down.process_f32();
            _ = inputs.backwards.process_f32();
            _ = inputs.forward.process_f32();
            Vec3::ZERO
        };
        camera.add_input(input_vector);
        if !free_cam {
            let mut vel = camera.vel();
            vel.y += GRAVITY * camera.delta_time();
            if inputs.space.just_pressed() && on_ground {
                vel.y -= WALK_JUMP_SPEED;
            }
            camera.set_vel(vel)
        };

        camera.apply_friction();

        camera.advance_pos(|start_pos, intended_pos| {
            let delta = intended_pos - start_pos;

            let sweep = collision::move_player_with_sweep(server, start_pos, delta);
            let vel = if sweep.hit {
                let vel = delta;
                let adjusted_vel = vel - sweep.normal * vel.dot(sweep.normal);
                adjusted_vel
            } else {
                Vec3::ZERO
            };
            (vel, sweep.position)
        });

        if inputs.status {
            println!("FPS: {} pos: {},", 1. / camera.delta_time(), camera.pos())
        }
    });

    if *change_mesh {
        // let now = Instant::now();
        // generator.write().unwrap().vertical_area *= 1.001;
        drawer.update_mesh(server.get_mesh(
            Frustum {
                cam_pos: drawer.camera.pos(),
                direction: drawer.camera.dir(),
                fov: Camera::FOV,
                aspect_ratio: drawer.window.aspect_ratio,
                render_distance: RENDER_DISTANCE,
            },
            threadpool,
        ));
    }
}
