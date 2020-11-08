
use std::marker::PhantomData;


#[derive(Copy, Clone, Debug)]
pub struct TextureBufferID<T: Copy>(pub(super) usize, pub(super) PhantomData<*const T>);


// Manual implementations required because of PhantomData
// see: https://github.com/rust-lang/rust/issues/26925
impl<T: Copy> std::hash::Hash for TextureBufferID<T> {
	#[inline]
	fn hash<H: std::hash::Hasher>(&self, h: &mut H) { self.0.hash(h) }
}

impl<T: Copy> std::cmp::PartialEq for TextureBufferID<T> {
    fn eq(&self, o: &TextureBufferID<T>) -> bool { self.0.eq(&o.0) }
}

impl<T: Copy> std::cmp::Eq for TextureBufferID<T> {}



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