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
    server::{
        frustum::{Frustum, LodLevel},
        world_gen::Generator,
    },
};

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
        Vec3::new(0.0, 0.0, 0.0),
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

    let mut threadpool = threadpool::Threadpool::new(num_cpus::get() - 1);

    let seed = 0x6bfb999977f4cd52; //random::get_random(0, u64::MAX);
    println!("world seed: {:16x}", seed);

    let generator = server::world_gen::OpenCaves::new(seed);
    let mut server = Server::new(generator);

    let mut input_event_filter = input::InputEventFilter::new().expect("input event filter");
    let mut frame_number = 0;
    let mut change_mesh = true;
    let mut lod_level = 0;

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
                                        &mut lod_level,
                                        inputs,
                                        &mut drawer,
                                        &mut server,
                                        &mut threadpool,
                                    );
                                    // println!("time it took to build mesh in total: {:#?}", now.elapsed());
                                    drawer.draw(control_flow);
                                }
                                if frame_number % 60 == 0 {
                                    println!("{}\n", threadpool.debug_log())
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

#[inline]
pub fn update<G: Generator>(
    change_mesh: &mut bool,
    lod_level: &mut LodLevel,
    inputs: &mut Inputs,
    drawer: &mut gpu::Drawer<'_>,
    server: &mut Server<G>,
    threadpool: &mut threadpool::Threadpool<G>,
) {
    if inputs.esc {
        drawer.window.flip_focus()
    }
    if inputs.remesh {
        *change_mesh = !*change_mesh;
    }
    *lod_level += inputs.lod_up as u16;
    *lod_level -= inputs.lod_down as u16;

    if drawer.window.focused() {
        if let Some(mouse_motion) = inputs.mouse_motion {
            drawer.camera.rotate_around_angle(glam::Vec3::new(
                -mouse_motion.x as f32,
                -mouse_motion.y as f32,
                0.,
            ));
        }

        if let Some(scroll) = inputs.mouse_wheel {
            drawer.camera.update_acc(scroll.y)
        }

        drawer.camera.update(glam::Vec3::new(
            inputs.right.process() - inputs.left.process(),
            inputs.down.process() - inputs.up.process(),
            inputs.backwards.process() - inputs.forward.process(),
        ));
    }

    drawer.update();

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
            *lod_level,
        ));
    }
}
