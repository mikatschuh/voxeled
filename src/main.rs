use winit::{dpi::PhysicalSize, event::*, event_loop::EventLoop, window::WindowBuilder};

mod gpu;
mod input;
mod playground;
mod time;

fn main() {
    pollster::block_on(run());
}
const WIDTH: u32 = 2000;
const HEIGHT: u32 = 2000;
async fn run() {
    env_logger::init(); // this logs error messages
    let event_loop = EventLoop::new().unwrap();
    let window = WindowBuilder::new()
        .with_title("Voxeled")
        .with_inner_size(PhysicalSize::<u32> {
            width: WIDTH,
            height: HEIGHT,
        })
        .build(&event_loop)
        .unwrap();

    let mut state = gpu::Drawer::connect_to(&window).await; // this creates the state of the program
    let mut delta_time = time::DeltaTime::new();
    let mut keys = input::Keys::default();
    event_loop // main event loop
        .run(move |event, control_flow| {
            if !keys.handled_event(state.window().id(), &event) {
                match event {
                    Event::WindowEvent { event, window_id } if window_id == state.window().id() => {
                        match event {
                            WindowEvent::CloseRequested => control_flow.exit(),
                            WindowEvent::Resized(physical_size) => {
                                state.resize(physical_size);
                            }
                            WindowEvent::RedrawRequested => {
                                // This tells winit that we want another frame after this one
                                state.window().request_redraw();

                                // some check that cannot be done
                                // if !surface_configured {
                                //     return;
                                // }

                                state.update(&keys, delta_time.update());
                                match state.render() {
                                    Ok(_) => {}
                                    // Reconfigure the surface if it's lost or outdated
                                    Err(
                                        wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated,
                                    ) => state.resize(state.size),
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
