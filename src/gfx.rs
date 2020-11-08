pub mod core;
pub mod mesh;
pub mod shader;
pub mod vertex;
pub mod texture_buffer;
pub mod mesh_builder;
pub mod camera;


pub struct Gfx {
	pub core: core::Core,
	pub camera: camera::Camera,
}


impl Gfx {
	pub fn new() -> Gfx {
		let mut core = core::Core::new();
		core.set_depth_test(true);

		unsafe {
			let mut vao = 0;
			gl::GenVertexArrays(1, &mut vao);
			gl::BindVertexArray(vao);

			gl::Enable(gl::BLEND);
			gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
			
			gl::PointSize(2.0);
			gl::LineWidth(2.0);

			gl::DebugMessageCallback(Some(gl_message_callback), std::ptr::null());
			gl::Enable(gl::DEBUG_OUTPUT_SYNCHRONOUS);

			// Disable performance messages
			gl::DebugMessageControl(
				gl::DONT_CARE,
				gl::DEBUG_TYPE_PERFORMANCE,
				gl::DONT_CARE,
				0, std::ptr::null(),
				0 // false
			);

			// Disable notification messages
			gl::DebugMessageControl(
				gl::DONT_CARE,
				gl::DONT_CARE,
				gl::DEBUG_SEVERITY_NOTIFICATION,
				0, std::ptr::null(),
				0 // false
			);
		}

		Gfx {
			core,
			camera: camera::Camera::new(),
		}
	}
}


extern "system" fn gl_message_callback(source: u32, ty: u32, _id: u32, severity: u32,
	_length: i32, msg: *const i8, _ud: *mut std::ffi::c_void)
{
	let severity = match severity {
		gl::DEBUG_SEVERITY_LOW => "low",
		gl::DEBUG_SEVERITY_MEDIUM => "medium",
		gl::DEBUG_SEVERITY_HIGH => "high",
		gl::DEBUG_SEVERITY_NOTIFICATION => "notification",
		_ => panic!("Unknown severity {}", severity),
	};

	let ty = match ty {
		gl::DEBUG_TYPE_ERROR => "error",
		gl::DEBUG_TYPE_DEPRECATED_BEHAVIOR => "deprecated behaviour",
		gl::DEBUG_TYPE_UNDEFINED_BEHAVIOR => "undefined behaviour",
		gl::DEBUG_TYPE_PORTABILITY => "portability",
		gl::DEBUG_TYPE_PERFORMANCE => "performance",
		gl::DEBUG_TYPE_OTHER => "other",
		_ => panic!("Unknown type {}", ty),
	};

	let source = match source {
		gl::DEBUG_SOURCE_API => "api",
		gl::DEBUG_SOURCE_WINDOW_SYSTEM => "window system",
		gl::DEBUG_SOURCE_SHADER_COMPILER => "shader compiler",
		gl::DEBUG_SOURCE_THIRD_PARTY => "third party",
		gl::DEBUG_SOURCE_APPLICATION => "application",
		gl::DEBUG_SOURCE_OTHER => "other",
		_ => panic!("Unknown source {}", source),
	};

	eprintln!("GL ERROR!");
	eprintln!("Source:   {}", source);
	eprintln!("Severity: {}", severity);
	eprintln!("Type:     {}", ty);

	unsafe {
		let msg = std::ffi::CStr::from_ptr(msg as _).to_str().unwrap();
		eprintln!("Message: {}", msg);
	}

	panic!("GL ERROR!");
}