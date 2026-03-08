use glam::Vec3;
use winit::{dpi::PhysicalSize, event::Event, event_loop::EventLoopWindowTarget};

use crate::{
    event_loop::make_window,
    gpu::{projection::View, window::Window},
    input::InputEventFilter,
};
use voxine::{
    ComposableGenerator, DeltaTimeMeter, Frustum, Gen2D, Gen3D, MaterialGenerator, Noise,
    SphereConfig, cam_controller::CamController,
};

// mod collision;
// mod console;

mod event_loop;
mod gpu;
mod input;
#[allow(unused)]
mod playground;

const NUM_CPUS: usize = 3;

const STARTING_POS: Vec3 = Vec3::new(0., 0., 0.);

const FULL_DETAL_DISTANCE: f32 = 12.;
const RENDER_DISTANCE: f32 = 20_000. / 32.;
const MAX_CHUNKS: usize = 5000;
const GRAVITY: f32 = 9.81;
const WALK_JUMP_SPEED: f32 = 5000.;

pub const FOV: f32 = std::f32::consts::FRAC_PI_3;
pub const NEAR_PLANE: f32 = 0.1;

fn main() {
    env_logger::init(); // this logs error messages

    make_window::<EventHandler>();
}

struct EventHandler<'a> {
    delta_time: DeltaTimeMeter,

    gpu: gpu::Gpu<'a>,

    engine_channel: voxine::RenderThreadChannels,

    input_event_filter: InputEventFilter,
    frames_drawn: usize,
    change_mesh: bool,
    toggle_impl: bool,
    paused: bool,
}

impl event_loop::EventHandler<'static> for EventHandler<'static> {
    fn new(window: &'static winit::window::Window) -> Self {
        let delta_time = DeltaTimeMeter::new();

        let seed: u64 = 0x6b_fb_99_99_77_f4_cd_52; //random::get_random(0, u64::MAX);
        println!("world seed: {:16x}", seed);

        Self {
            engine_channel: voxine::create_engine_thread(
                NUM_CPUS,
                SphereConfig {
                    full_detail_range: FULL_DETAL_DISTANCE,
                    radius: RENDER_DISTANCE,
                    max_chunks: MAX_CHUNKS,
                },
                CamController::new(STARTING_POS, 0., 0., true, delta_time.reader()),
                ComposableGenerator::gen_2d(
                    Gen2D {
                        noise: Noise::new((seed ^ 0x19_af_2b_7c_e8_9a_7d_d3) as u32),
                        octaves: 3,
                        base_height: -1.,
                        x_scale: 5000.,
                        y_scale: 13.,
                        z_scale: 5000.,
                    },
                    Some(MaterialGenerator::new(seed)),
                ) + ComposableGenerator::gen_3d(
                    Gen3D {
                        noise: Noise::new(seed as u32),
                        octaves: 8,
                        x_scale: 100.,
                        y_scale: 100.,
                        z_scale: 100.,
                        exponent: 3.,
                        threshold: 0.2,
                    },
                    None,
                ),
            )
            .unwrap(),
            gpu: pollster::block_on(gpu::Gpu::connect_to(
                &window,
                wgpu::PresentMode::AutoNoVsync,
            )),

            input_event_filter: input::InputEventFilter::new().expect("input event filter"),
            frames_drawn: 0,
            change_mesh: true,
            toggle_impl: true,
            paused: false,
            delta_time,
        }
    }

    fn could_handle(
        &mut self,
        event: &Event<()>,
        own_window_id: winit::window::WindowId,
        keyboard_focus: bool,
    ) -> bool {
        self.input_event_filter
            .could_handle(event, own_window_id, keyboard_focus)
    }

    fn generate_frame(
        &mut self,
        window: &mut Window<'static>,
        control_flow: &EventLoopWindowTarget<()>,
    ) {
        let inputs = self.input_event_filter.get();

        if inputs.pause {
            self.paused = !self.paused;
        }
        if inputs.remesh {
            self.change_mesh = !self.change_mesh;
        }
        if inputs.toggle_impl {
            self.toggle_impl = !self.toggle_impl;
        }

        let frustum = {
            let mut camera = self.engine_channel.player.write();
            if !self.paused && window.focused() {
                let prev_cam_pos = camera.pos();

                if inputs.free_cam {
                    // camera.toggle_free_cam();
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

                camera.advance_pos(|_start_pos, intended_pos| intended_pos);

                if inputs.status {
                    println!(
                        "FPS: {}\tpos: [{}]\tvel: {:+12.5} kmh",
                        1. / camera.delta_time(),
                        camera
                            .pos()
                            .to_array()
                            .map(|n| format!("{:+10.3}", n))
                            .into_iter()
                            .reduce(|acc, s| acc + ", " + &s)
                            .unwrap(),
                        (camera.pos() - prev_cam_pos).length() / camera.delta_time() / 3.6
                    );
                }

                self.gpu.update_view(View::new(camera.pos(), camera.dir()));
            }
            Frustum {
                cam_pos: camera.pos(),
                direction: camera.dir(),
                fov: FOV,
                aspect_ratio: window.aspect_ratio,
                max_chunks: MAX_CHUNKS,
                max_distance: RENDER_DISTANCE,
                full_detail_range: FULL_DETAL_DISTANCE,
            }
        };

        if self.change_mesh {
            self.gpu
                .update_mesh(&mut self.engine_channel.mesh_updates, 0.01);
        }

        self.gpu.draw(frustum, control_flow);

        self.input_event_filter.frame_done();
        self.frames_drawn += 1;
        self.delta_time.update();
    }

    fn reconfigure(&mut self) {
        self.gpu.reconfigure();
    }

    fn set_window_focus(&mut self, _: bool) {}

    fn resize_window(&mut self, new_size: PhysicalSize<u32>) {
        self.gpu.resize(new_size);
    }
}

impl Drop for EventHandler<'_> {
    fn drop(&mut self) {
        self.engine_channel
            .config_updates
            .push(voxine::ConfigUpdates::ShutDown)
            .unwrap()
    }
}
