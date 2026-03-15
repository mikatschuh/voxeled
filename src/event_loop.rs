use winit::{
    dpi::PhysicalSize,
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop, EventLoopWindowTarget},
    window::{WindowBuilder, WindowId},
};

use crate::gpu::window::Window;

pub trait EventHandler<'a> {
    fn new(window: &'a winit::window::Window) -> Self;

    fn could_handle(
        &mut self,
        event: &Event<()>,
        own_window_id: WindowId,
        keyboard_focus: bool,
    ) -> bool;

    fn generate_frame(&mut self, window: &mut Window<'a>, control_flow: &EventLoopWindowTarget<()>);

    fn reconfigure(&mut self);
    fn set_window_focus(&mut self, focused: bool);
    fn resize_window(&mut self, new_size: PhysicalSize<u32>);
}

pub fn make_window<E: EventHandler<'static>>() {
    let event_loop = EventLoop::new().unwrap();
    let window = Box::new(
        WindowBuilder::new()
            .with_title("Voxeled")
            .with_inner_size(PhysicalSize::<u32> {
                width: 2000,
                height: 2000,
            }) // this is the window configuration
            .build(&event_loop)
            .unwrap(),
    );
    let window: &'static winit::window::Window = Box::leak(window);

    let mut event_handler = E::new(window);

    let mut window = Window::from(window, true);

    event_loop // main event loop
        .run(|event, control_flow| {
            if !event_handler.could_handle(&event, window.id(), window.focused()) {
                match event {
                    Event::WindowEvent { event, window_id } if window_id == window.id() => {
                        match event {
                            WindowEvent::Occluded(occluded) => {
                                match occluded {
                                    true => control_flow.set_control_flow(ControlFlow::Wait),
                                    false => {
                                        control_flow.set_control_flow(ControlFlow::Poll);
                                    }
                                }
                                event_handler.reconfigure();
                            }
                            WindowEvent::Focused(focused) => {
                                window.set_focus(focused);
                                event_handler.set_window_focus(focused)
                            }
                            WindowEvent::CloseRequested => control_flow.exit(),
                            WindowEvent::Resized(physical_size) => {
                                window.resize(physical_size);
                                event_handler.resize_window(physical_size)
                            }

                            WindowEvent::RedrawRequested => {
                                event_handler.generate_frame(&mut window, control_flow);
                                window.request_redraw();
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
}
