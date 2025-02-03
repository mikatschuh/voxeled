use winit::{
    dpi::PhysicalSize,
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};
mod gpu;
mod library;
mod playground;
use library::*;

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
    let mut drawer = gpu::Drawer::connect_to(&window, wgpu::PresentMode::Fifo).await; // this connectes a drawer to the window
    let mut delta_time = time::DeltaTime::now();
    let mut keys = input::Keys::new();
    event_loop // main event loop
        .run(move |event, control_flow| {
            if !keys.handled_event(drawer.window.id(), &event) {
                match event {
                    Event::NewEvents(StartCause::Init) => {
                        // Initial frame time
                        delta_time = time::DeltaTime::now();
                    }
                    Event::WindowEvent { event, window_id } // checks if its the right window
                        if window_id == drawer.window.id() =>
                    {
                        match event {
                            WindowEvent::Occluded(occluded) => if occluded {
                                control_flow.set_control_flow(ControlFlow::Wait);
                            } else {
                                drawer.reconfigure()
                            }

                            WindowEvent::CloseRequested => control_flow.exit(),
                            WindowEvent::Resized(physical_size) => {
                                drawer.resize(physical_size);
                            }
                            WindowEvent::RedrawRequested => {
                                // This tells winit that we want another frame after this one
                                drawer.window.request_redraw();

                                if keys.esc.just_pressed() { drawer.window.flip_focus() }

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
