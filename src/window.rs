use crate::prelude::*;
use std::error::Error;

use glutin::event_loop::EventLoop;

pub struct Window {
	context: glutin::WindowedContext<glutin::PossiblyCurrent>,
	event_loop: EventLoop<()>,
}


impl Window {
	pub fn new() -> Result<Self, Box<dyn Error>> {
		let event_loop = EventLoop::new();

		let window = glutin::window::WindowBuilder::new()
			.with_title("bees")
			.with_resizable(true);

		let context = glutin::ContextBuilder::new()
			.with_gl(glutin::GlRequest::Specific(glutin::Api::OpenGl, (3, 2)))
			.with_gl_profile(glutin::GlProfile::Core)
			.with_gl_debug_flag(true)
			.with_vsync(true)
			.build_windowed(window, &event_loop)?;

		let context = unsafe { context.make_current().unwrap() };

		gl::load_with(|s| context.get_proc_address(s) as *const _);


		Ok(Window {
			context,
			event_loop,
		})
	}

	pub fn size(&self) -> Vec2i {
		let (x, y): (u32, u32) = self.context.window()
			.inner_size()
			.into();

		Vec2i::new(x as i32, y as i32)
	}

	pub fn dpi(&self) -> f64 {
		self.context.window().scale_factor()
	} 

	pub fn poll_events<F>(&mut self, mut f: F) where F: FnMut(glutin::event::WindowEvent<'_>) {
		use glutin::platform::desktop::EventLoopExtDesktop;
		use glutin::event::Event;

		self.event_loop.run_return(move |event, _target, control_flow| {
			match event {
				Event::WindowEvent{event, ..} => {
					f(event);
				}

				Event::MainEventsCleared | Event::RedrawEventsCleared => {
					*control_flow = glutin::event_loop::ControlFlow::Exit;
				}
				_ => {}
			}
		});
	}

	pub fn swap(&mut self) {
		self.context.swap_buffers().unwrap();
	}
}
