
use std::marker::PhantomData;


#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct TextureBufferID<V: Copy>(pub(super) usize, pub(super) PhantomData<*const V>);

// #[derive(Copy, Clone, Debug, PartialEq, Eq)]
// pub(super) struct UntypedTextureBufferID(pub(super) usize);

// impl<V: Vertex> From<TextureBufferID<V>> for UntypedTextureBufferID {
// 	fn from(MeshID(o, _): MeshID<V>) -> UntypedMeshID {
// 		UntypedMeshID(o)
// 	}
// }


pub(super) struct TextureBuffer {
	pub(super) texture_id: u32,
	pub(super) buffer_id: u32
}

impl TextureBuffer {
	pub(super) fn new() -> TextureBuffer {
		let (mut texture_id, mut buffer_id) = (0, 0);

		unsafe {
			gl::GenTextures(1, &mut texture_id);
			gl::GenBuffers(1, &mut buffer_id);

			// buffer_id doesn't need to be bound here for gl::TexBuffer to work,
			// but it needs to have been bound at least once as gl::TEXTURE_BUFFER
			// before it's properly set up as such
			gl::BindBuffer(gl::TEXTURE_BUFFER, buffer_id);
			gl::BindTexture(gl::TEXTURE_BUFFER, texture_id);
			gl::TexBuffer(gl::TEXTURE_BUFFER, gl::RGBA32F, buffer_id);
		}

		TextureBuffer {texture_id, buffer_id}
	}
}