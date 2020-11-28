// use crate::prelude::*;
use super::vertex::*;
use std::marker::PhantomData;


#[derive(Copy, Clone, Debug)]
pub struct MeshID<V: Vertex>(pub(super) usize, pub(super) PhantomData<*const V>);

#[derive(Copy, Clone, Debug)]
pub struct BasicMeshID<V: Vertex>(pub(super) usize, pub(super) PhantomData<*const V>);

#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
pub(super) enum UntypedMeshID {
	Indexed(usize),
	Basic(usize),
}

impl<V: Vertex> From<MeshID<V>> for UntypedMeshID {
	fn from(MeshID(o, _): MeshID<V>) -> UntypedMeshID {
		UntypedMeshID::Indexed(o)
	}
}

impl<V: Vertex> From<BasicMeshID<V>> for UntypedMeshID {
	fn from(BasicMeshID(o, _): BasicMeshID<V>) -> UntypedMeshID {
		UntypedMeshID::Basic(o)
	}
}



// Manual implementations required because of PhantomData
// see: https://github.com/rust-lang/rust/issues/26925
impl<V: Vertex> std::hash::Hash for MeshID<V> {
	#[inline] fn hash<H: std::hash::Hasher>(&self, h: &mut H) { self.0.hash(h) }
}

impl<V: Vertex> std::cmp::PartialEq for MeshID<V> {
    fn eq(&self, o: &MeshID<V>) -> bool { self.0.eq(&o.0) }
}

impl<V: Vertex> std::cmp::Eq for MeshID<V> {}



impl<V: Vertex> std::hash::Hash for BasicMeshID<V> {
	#[inline] fn hash<H: std::hash::Hasher>(&self, h: &mut H) { self.0.hash(h) }
}

impl<V: Vertex> std::cmp::PartialEq for BasicMeshID<V> {
    fn eq(&self, o: &BasicMeshID<V>) -> bool { self.0.eq(&o.0) }
}

impl<V: Vertex> std::cmp::Eq for BasicMeshID<V> {}




pub(super) struct Mesh {
	pub(super) descriptor: Descriptor,
	pub(super) element_count: u32,
	pub(super) vbo: u32,
	pub(super) ebo: u32
}

impl Mesh {
	pub(super) fn new(descriptor: Descriptor) -> Mesh {
		unsafe {
			let mut buffers = [0; 2];
			gl::GenBuffers(2, buffers.as_mut_ptr());

			let [vbo, ebo] = buffers;
			Mesh {
				descriptor,
				element_count: 0,
				vbo, ebo
			}
		}
	}

	pub(super) fn bind(&self) {
		unsafe {
			gl::BindBuffer(gl::ARRAY_BUFFER, self.vbo);
			gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, self.ebo);
		}
	}
}




pub(super) struct BasicMesh {
	pub(super) descriptor: Descriptor,
	pub(super) vertex_count: u32,
	pub(super) vbo: u32,
}

impl BasicMesh {
	pub(super) fn new(descriptor: Descriptor) -> BasicMesh {
		unsafe {
			let mut vbo = 0;
			gl::GenBuffers(1, &mut vbo);

			BasicMesh {
				descriptor,
				vertex_count: 0,
				vbo
			}
		}
	}

	pub(super) fn bind(&self) {
		unsafe {
			gl::BindBuffer(gl::ARRAY_BUFFER, self.vbo);
		}
	}
}
