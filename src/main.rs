use glam::Vec3;
use server::Server;
use winit::{
    dpi::PhysicalSize,
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

use crate::{
    input::Inputs,
    physics::{Aabb, CamController, DeltaTimeMeter},
    server::{frustum::Frustum, world_gen::Generator},
};

// mod collision;
// mod console;
#[allow(dead_code)]
mod data_structures;
mod gpu;
mod input;
mod netcode;
#[allow(unused)]
mod physics;
#[allow(unused)]
mod playground;
mod random;
mod server;
mod threadpool;

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

    let mut delta_time = DeltaTimeMeter::new();

    let mut camera = CamController::new(STARTING_POS, 0., 0., true, delta_time.reader());
    let mut drawer = pollster::block_on(gpu::Drawer::connect_to(
        &window,
        wgpu::PresentMode::AutoNoVsync,
        camera.pos(),
    )); // this connectes a drawer to the window
    /*
    let Ok(_) = console::Console::init(delta_time.reader()) else {
        println!("{}", "# failed to launch console".red());
        return;
    };*/

    let mut threadpool = threadpool::Threadpool::new(6); //num_cpus::get() - 1);

    let seed = 0x6bfb999977f4cd52; //random::get_random(0, u64::MAX);
    println!("world seed: {:16x}", seed);

    let mut server = Server::new(server::world_gen::Box::new(seed, 2. * 32.));

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
                                let inputs = input_event_filter.get();

                                if frame_number == 0 {
                                    drawer.draw(control_flow)
                                } else {
                                    update(
                                        &mut camera,
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
                                frame_number += 1;
                                delta_time.update();
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

const STARTING_POS: Vec3 = Vec3::new(0.,-20., 0.);

const FULL_DETAL_DISTANCE: f32 = 6.;
const RENDER_DISTANCE: f32 = 8.;
const GRAVITY: f32 = 9.81;
const WALK_JUMP_SPEED: f32 = 5000.;

pub const FOV: f32 = std::f32::consts::FRAC_PI_2;
pub const NEAR_PLANE: f32 = 0.1;
pub const FAR_PLANE: f32 = 10_000.0;

#[inline]
pub fn update<G: Generator>(
    camera: &mut CamController,
    change_mesh: &mut bool,
    inputs: & Inputs,
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

    if drawer.window.focused() {
        if inputs.free_cam {
            camera.toggle_free_cam();
        }

        if let Some(mouse_motion) = inputs.mouse_motion {
            camera.rotate_around_angle(mouse_motion.x as f32, -mouse_motion.y as f32);
        }

        if let Some(scroll) = inputs.mouse_wheel {
            camera.update_speed(scroll.y)
        }

        let free_cam = camera.free_cam();
        let on_ground = false; // !free_cam && collision::is_on_ground(server, camera.pos());

        let input_vector = inputs.input_vector();
        camera.add_input(input_vector);
        if !free_cam {
            camera.add_acc(Vec3::new(0.0, GRAVITY, 0.0));
            if inputs.space.just_pressed() && on_ground {
                camera.add_acc(Vec3::new(0.0, -WALK_JUMP_SPEED, 0.0));
            }
        }

        camera.advance_pos(|start_pos, intended_pos| {
            Aabb::player(start_pos).compute_sweep(server, intended_pos - start_pos)
        });

        // if inputs.status {
        println!("FPS: {}\tpos: {},", 1. / camera.delta_time(), camera.pos());
        //}

        drawer.update_view(camera.view());
    }

    if *change_mesh {
        // let now = Instant::now();
        // generator.write().unwrap().vertical_area *= 1.001;
        drawer.update_mesh(server.get_mesh(
            Frustum {
                cam_pos: camera.pos(),
                direction: camera.dir(),
                fov: FOV,
                aspect_ratio: drawer.window.aspect_ratio,
                render_distance: RENDER_DISTANCE,
            },
            threadpool,
        ));
    }
}
