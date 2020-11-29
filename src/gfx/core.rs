
use std::marker::PhantomData;

use crate::prelude::*;

use super::shader::*;
use super::vertex::*;
use super::mesh::*;
use super::texture_buffer::*;


pub enum BlendMode {
	None,
	Alpha,
	PremultipliedAlpha,
	Add, // Linear Dodge
	Subtract,
	Multiply,

	Darken,
	Lighten,
}


pub struct Core {
	capabilities: Capabilities,

	shaders: Vec<Shader>,
	meshes: Vec<Mesh>,
	basic_meshes: Vec<BasicMesh>,
	texture_buffers: Vec<TextureBuffer>,

	bound_shader: Option<ShaderID>,
	bound_mesh: Option<UntypedMeshID>,
}


impl Core {
	pub fn new() -> Core {
		let capabilities = Capabilities::new();

		println!("capabilities: {:#?}", capabilities);

		Core {
			capabilities,

			shaders: Vec::new(),
			meshes: Vec::new(),
			basic_meshes: Vec::new(),
			texture_buffers: Vec::new(),

			bound_shader: None,
			bound_mesh: None,
		}
	}

	pub fn capabilities(&self) -> &Capabilities { &self.capabilities }

	pub fn set_bg_color(&mut self, c: Color) {
		unsafe {
			let (r,g,b,a) = c.to_tuple();
			gl::ClearColor(r, g, b, a);
		}
	}

	pub fn set_viewport(&mut self, size: Vec2i) {
		unsafe {
			let Vec2i{x, y} = size;
			gl::Viewport(0, 0, x, y);
		}
	}

	pub fn set_depth_test(&mut self, enable: bool) {
		unsafe {
			if enable {
				gl::Enable(gl::DEPTH_TEST)
			} else {
				gl::Disable(gl::DEPTH_TEST)
			}
		}
	}

	pub fn set_color_write(&mut self, enable: bool) {
		unsafe {
			let v = if enable { gl::TRUE } else { gl::FALSE };
			gl::ColorMask(v, v, v, v);
		}
	}

	pub fn set_blend_mode(&mut self, blend_mode: BlendMode) {
		use self::BlendMode::*;

		let (equation, source, target) = match blend_mode {
			None => unsafe {
				gl::Disable(gl::BLEND);
				return
			},

			Alpha => (gl::FUNC_ADD, gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA),
			PremultipliedAlpha => (gl::FUNC_ADD, gl::ONE, gl::ONE_MINUS_SRC_ALPHA),
			Add => (gl::FUNC_ADD, gl::ONE, gl::ONE),
			Subtract => (gl::FUNC_REVERSE_SUBTRACT, gl::ONE, gl::ONE),
			Multiply => (gl::FUNC_ADD, gl::DST_COLOR, gl::ZERO),

			Darken => (gl::MIN, gl::ONE, gl::ONE),
			Lighten => (gl::MAX, gl::ONE, gl::ONE),
		};
		
		unsafe {
			gl::Enable(gl::BLEND);
			gl::BlendEquation(equation);
			gl::BlendFunc(source, target);
		}
	}

	pub fn clear(&mut self) {
		unsafe {
			gl::Clear(gl::COLOR_BUFFER_BIT|gl::DEPTH_BUFFER_BIT|gl::STENCIL_BUFFER_BIT);
		}
	}


	// Shaders
	pub fn new_shader(&mut self, vsrc: &str, fsrc: &str, attribs: &[&str]) -> ShaderID {
		self.shaders.push(Shader::new(vsrc, fsrc, attribs));
		ShaderID(self.shaders.len()-1)
	}

	pub fn use_shader(&mut self, id: ShaderID) {
		unsafe {
			let shader = self.shaders.get(id.0).expect("Tried to use invalid shader");
			gl::UseProgram(shader.handle);
			self.bound_shader = Some(id);


			for i in 0..shader.attribute_count {
				gl::EnableVertexAttribArray(i);
			}

			for i in shader.attribute_count..8 {
				gl::DisableVertexAttribArray(i);
			}
		}
	}

	fn get_uniform_location(&self, name: &str) -> i32 {
		unsafe {
			let shader_id = self.bound_shader.unwrap();
			let shader = &self.shaders[shader_id.0];
			let name = std::ffi::CString::new(name.as_bytes()).unwrap();
			gl::GetUniformLocation(shader.handle, name.as_ptr())
		}
	}

	pub fn set_uniform_i32(&mut self, name: &str, value: i32) {
		let loc = self.get_uniform_location(name);
		unsafe { gl::Uniform1i(loc, value) }
	}

	pub fn set_uniform_mat4(&mut self, name: &str, value: &Mat4) {
		let loc = self.get_uniform_location(name);
		unsafe {
			gl::UniformMatrix4fv(loc, 1, 0, &value.transpose() as *const _ as *const f32);
		}
	}

	pub fn set_uniform_texture_buffer<V: Copy>(&mut self, name: &str, buffer: TextureBufferID<V>, slot: u32) {
		let loc = self.get_uniform_location(name);
		let buffer = self.texture_buffers.get(buffer.0).expect("Tried to bind invalid texture buffer");
		unsafe {
			gl::ActiveTexture(gl::TEXTURE0 + slot);
			gl::BindTexture(gl::TEXTURE_BUFFER, buffer.texture_id);
			gl::Uniform1i(loc, slot as _)
		}
	}


	// Meshes
	pub fn new_mesh<V: Vertex>(&mut self) -> MeshID<V> {
		self.meshes.push(Mesh::new(V::descriptor()));
		MeshID(self.meshes.len()-1, PhantomData)
	}

	pub fn new_basic_mesh<V: Vertex>(&mut self) -> BasicMeshID<V> {
		self.basic_meshes.push(BasicMesh::new(V::descriptor()));
		BasicMeshID(self.basic_meshes.len()-1, PhantomData)
	}

	pub fn update_mesh<V: Vertex>(&mut self, id: MeshID<V>, vs: &[V], es: &[u16]) {
		id.bind_mesh(self);

		let mesh = self.meshes.get_mut(id.0).expect("Tried to bind invalid mesh");
		mesh.element_count = es.len() as _;

		unsafe {
			gl::BufferData(
				gl::ARRAY_BUFFER,
				(vs.len() * std::mem::size_of::<V>()) as _,
				vs.as_ptr() as *const _,
				gl::STATIC_DRAW
			);

			gl::BufferData(
				gl::ELEMENT_ARRAY_BUFFER,
				(es.len() * std::mem::size_of::<u16>()) as _,
				es.as_ptr() as *const _,
				gl::STATIC_DRAW
			);
		}
	}

	pub fn update_basic_mesh<V: Vertex>(&mut self, id: BasicMeshID<V>, vs: &[V]) {
		id.bind_mesh(self);

		let mesh = self.basic_meshes.get_mut(id.0).expect("Tried to bind invalid mesh");
		mesh.vertex_count = vs.len() as _;

		unsafe {
			gl::BufferData(
				gl::ARRAY_BUFFER,
				(vs.len() * std::mem::size_of::<V>()) as _,
				vs.as_ptr() as *const _,
				gl::STATIC_DRAW
			);
		}
	}

	pub fn draw_mesh<ID: MeshIDLike>(&mut self, id: ID) {
		id.bind_mesh(self);
		id.draw_mesh(self, gl::TRIANGLES);
	}

	pub fn draw_mesh_lines<ID: MeshIDLike>(&mut self, id: ID) {
		id.bind_mesh(self);
		id.draw_mesh(self, gl::LINES);
	}

	pub fn draw_mesh_points<ID: MeshIDLike>(&mut self, id: ID) {
		id.bind_mesh(self);
		id.draw_mesh(self, gl::POINTS);
	}


	// TextureBuffers
	pub fn new_texture_buffer<V: Copy>(&mut self) -> TextureBufferID<V> {
		self.texture_buffers.push(TextureBuffer::new());
		TextureBufferID(self.texture_buffers.len()-1, PhantomData)
	}

	pub fn update_texture_buffer<V: Copy>(&mut self, id: TextureBufferID<V>, data: &[V]) {
		let buffer_size = data.len() * std::mem::size_of::<V>();
		assert!(buffer_size < self.capabilities.texture_buffer_size,
			"Texture buffer size exceeds minimum guaranteed value of GL_MAX_TEXTURE_BUFFER_SIZE");

		assert!(buffer_size % (std::mem::size_of::<f32>() * 4) == 0,
			"Texture buffer data mis-sized; currently only support 4xf32 format data");

		let buffer = self.texture_buffers.get(id.0).expect("Tried to update invalid texture buffer");

		unsafe {
			gl::BindBuffer(gl::TEXTURE_BUFFER, buffer.buffer_id);
			gl::BufferData(
				gl::TEXTURE_BUFFER,
				buffer_size as _,
				data.as_ptr() as _,
				gl::STREAM_DRAW
			);
		}
	}
}






pub trait MeshIDLike {
	// type Vertex: Vertex;
	// type Mesh;

	fn bind_mesh(&self, core: &mut Core);
	fn draw_mesh(&self, core: &mut Core, draw_mode: u32);
}


impl<V: Vertex> MeshIDLike for MeshID<V> {
	// type Vertex = V;

	fn bind_mesh(&self, core: &mut Core) {
		let untyped_id = UntypedMeshID::from(*self);
		if core.bound_mesh == Some(untyped_id) {
			return;
		}

		let mesh = core.meshes.get(self.0).expect("Tried to bind invalid mesh");
		mesh.bind();
		core.bound_mesh = Some(untyped_id);
	}

	fn draw_mesh(&self, core: &mut Core, draw_mode: u32) {
		let mesh = core.meshes.get(self.0).expect("Tried to bind invalid mesh");
		mesh.descriptor.bind();

		unsafe {
			gl::DrawElements(
				draw_mode,
				mesh.element_count as _,
				gl::UNSIGNED_SHORT,
				std::ptr::null()
			);
		}
	}
}

impl<V: Vertex> MeshIDLike for BasicMeshID<V> {
	// type Vertex = V;

	fn bind_mesh(&self, core: &mut Core) {
		let untyped_id = UntypedMeshID::from(*self);
		if core.bound_mesh == Some(untyped_id) {
			return;
		}

		let mesh = core.basic_meshes.get(self.0).expect("Tried to bind invalid mesh");
		mesh.bind();
		core.bound_mesh = Some(untyped_id);
	}

	fn draw_mesh(&self, core: &mut Core, draw_mode: u32) {
		let mesh = core.basic_meshes.get(self.0).expect("Tried to bind invalid mesh");
		mesh.descriptor.bind();

		unsafe {
			gl::DrawArrays(draw_mode, 0, mesh.vertex_count as _);
		}
	}
}





#[derive(Debug)]
pub struct Capabilities {
	pub texture_buffer_size: usize,
}

impl Capabilities {
	fn new() -> Capabilities {
		Capabilities {
			texture_buffer_size: unsafe {
				let mut v = 0;
				gl::GetIntegerv(gl::MAX_TEXTURE_BUFFER_SIZE, &mut v);
				v as usize
			},
		}
	}
}
