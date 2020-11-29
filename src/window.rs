use crate::prelude::*;
use std::error::Error;

use glutin::event_loop::EventLoop;

pub struct Window {
	context: glutin::WindowedContext<glutin::PossiblyCurrent>,
	event_loop: EventLoop<()>,
	focussed: bool,
	should_capture: bool,
	retry_capture: bool,
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
			focussed: true,
			should_capture: false,
			retry_capture: false,
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

	pub fn focussed(&self) -> bool { self.focussed }

	pub fn poll_events<F>(&mut self, mut f: F) where F: FnMut(glutin::event::Event<'_, ()>) {
		use glutin::platform::desktop::EventLoopExtDesktop;
		use glutin::event::{Event, WindowEvent};

		let mut focus_event = None;

		self.event_loop.run_return(|event, _target, control_flow| {
			match event {
				Event::WindowEvent{event: WindowEvent::Focused(focussed), ..} => {
					focus_event = Some(focussed);
				}

				Event::WindowEvent{..} | Event::DeviceEvent{..} => {
					f(event);
				}

				Event::MainEventsCleared | Event::RedrawEventsCleared => {
					*control_flow = glutin::event_loop::ControlFlow::Exit;
				}
				_ => {}
			}
		});

		// if the focus state has changed, make sure we're grabbing or releasing as appropriate
		if let Some(focus) = focus_event {
			self.focussed = focus;
			self.retry_capture = true;
		}

		if self.retry_capture {
			let capture = self.should_capture && self.focussed;
			if self.context.window().set_cursor_grab(capture).is_ok() {
				self.context.window().set_cursor_visible(!capture);
				self.retry_capture = false;
			}
		}
	}

	pub fn set_cursor_capture(&mut self, enable: bool) {
		self.should_capture = enable;
		self.retry_capture = true;
	}

	pub fn swap(&mut self) {
		self.context.swap_buffers().unwrap();
	}
}
