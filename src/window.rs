use crate::prelude::*;
use std::error::Error;

pub struct Window {
	context: glutin::WindowedContext<glutin::PossiblyCurrent>,
	events_loop: glutin::EventsLoop,
}


impl Window {
	pub fn new() -> Result<Self, Box<dyn Error>> {
		let events_loop = glutin::EventsLoop::new();

		let window = glutin::WindowBuilder::new()
			.with_title("bees")
			.with_resizable(true);

		let context = glutin::ContextBuilder::new()
			.with_gl(glutin::GlRequest::Specific(glutin::Api::OpenGl, (3, 2)))
			.with_gl_profile(glutin::GlProfile::Core)
			.with_gl_debug_flag(true)
			.build_windowed(window, &events_loop)?;

		let context = unsafe { context.make_current().unwrap() };

		gl::load_with(|s| context.get_proc_address(s) as *const _);


		Ok(Window {
			context,
			events_loop,
		})
	}

	pub fn size(&self) -> Vec2i {
		let (x, y): (u32, u32) = self.context.window()
			.get_inner_size()
			.unwrap()
			.to_physical(self.dpi())
			.into();

		Vec2i::new(x as i32, y as i32)
	}

	pub fn dpi(&self) -> f64 {
		self.context.window().get_hidpi_factor()
	} 

	pub fn poll_events(&mut self) -> Vec<glutin::WindowEvent> {
		let mut events = Vec::new();

		self.events_loop.poll_events(|event| {
			if let glutin::Event::WindowEvent{event, ..} = event {
				events.push(event);
			}
		});

		events
	}

	pub fn swap(&mut self) {
		self.context.swap_buffers().unwrap();
	}
}
