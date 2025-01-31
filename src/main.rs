use winit::{dpi::PhysicalSize, event::*, event_loop::EventLoop, window::WindowBuilder};

mod gpu;
mod input;
mod playground;
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
        })
        .build(&event_loop)
        .unwrap();
    let mut drawer = gpu::Drawer::connect_to(&window).await; // this creates the state of the program
    let mut delta_time = time::DeltaTime::new();
    let mut keys = input::Keys::default();
    event_loop // main event loop
        .run(move |event, control_flow| {
            if !keys.handled_event(drawer.window().id(), &event) {
                match event {
                    Event::WindowEvent { event, window_id }
                        if window_id == drawer.window().id() =>
                    {
                        match event {
                            WindowEvent::CloseRequested => control_flow.exit(),
                            WindowEvent::Resized(physical_size) => {
                                drawer.resize(physical_size);
                            }
                            WindowEvent::RedrawRequested => {
                                // This tells winit that we want another frame after this one
                                drawer.window().request_redraw();

                                // some check that cannot be done
                                // if !surface_configured {
                                //     return;
                                // }

                                drawer.update(&keys, delta_time.update());
                                match drawer.render() {
                                    Ok(_) => {}
                                    // Reconfigure the surface if it's lost or outdated
                                    Err(
                                        wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated,
                                    ) => drawer.resize(drawer.window.size),
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
