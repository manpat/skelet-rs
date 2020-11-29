use crate::prelude::*;

use std::cell::Cell;
use std::fmt::Debug;


#[derive(Clone, Debug)]
pub struct Camera {
	projection_mode: ProjectionMode,
	view_mode: ViewMode,
	near: f32,
	far: f32,
	
	position: Vec3,
	yaw: f32,
	pitch: f32,

	aspect: f32,
	viewport: Vec2i,

	// Cached values for speed and profit	
	projection_matrix: Memoised<Mat4>,
	view_matrix: Memoised<Mat4>,

	inv_projection_matrix: Memoised<Mat4>,
	inv_view_matrix: Memoised<Mat4>,

	proj_view_matrix: Memoised<Mat4>,
	inv_proj_view_matrix: Memoised<Mat4>,

	orientation: Memoised<Quat>,
}


#[derive(Copy, Clone, Debug)]
pub enum ProjectionMode {
	Orthographic { view_size: f32 },
	Perspective { fov_y: f32 },
}


#[derive(Copy, Clone, Debug)]
pub enum ViewMode {
	Orbit {
		distance: f32,
	},

	FirstPerson,
}


impl Camera {
	pub fn new(projection_mode: ProjectionMode, view_mode: ViewMode) -> Self {
		Self {
			projection_mode,
			view_mode,
			near: 0.1,
			far: 100.0,

			position: Vec3::zero(),
			yaw: 0.0,
			pitch: 0.0,

			aspect: 1.0,
			viewport: Vec2i::splat(1),

			projection_matrix: Memoised::new(Mat4::ident()),
			view_matrix: Memoised::new(Mat4::ident()),
			proj_view_matrix: Memoised::new(Mat4::ident()),
			
			inv_projection_matrix: Memoised::new(Mat4::ident()),
			inv_view_matrix: Memoised::new(Mat4::ident()),
			inv_proj_view_matrix: Memoised::new(Mat4::ident()),

			orientation: Memoised::new(Quat::ident()),
		}
	}

	pub fn update(&mut self, viewport: Vec2i) {
		self.viewport = viewport;
		let viewport = viewport.to_vec2();
		let aspect = viewport.x / viewport.y;

		if (self.aspect - aspect).abs() > 0.0 {
			self.aspect = aspect;
			self.mark_projection_dirty();
		}
	}

	pub fn set_near_far(&mut self, near: f32, far: f32) {
		self.near = near;
		self.far = far;
		self.mark_view_dirty();
	}

	pub fn near_far(&self) -> (f32, f32) { (self.near, self.far) }

	pub fn viewport(&self) -> Vec2i { self.viewport }
	pub fn aspect(&self) -> f32 { self.aspect }

	pub fn position(&self) -> Vec3 { self.position }
	pub fn pitch(&self) -> f32 { self.pitch }
	pub fn yaw(&self) -> f32 { self.yaw }

	pub fn forward(&self) -> Vec3 {
		self.orientation().forward()
	}


	pub fn set_position(&mut self, p: Vec3) {
		self.position = p;
		self.mark_view_dirty();
	}

	pub fn set_pitch(&mut self, pitch: f32) {
		self.pitch = pitch;
		self.mark_orientation_dirty();
	}

	pub fn set_yaw(&mut self, yaw: f32) {
		self.yaw = yaw;
		self.mark_orientation_dirty();
	}


	pub fn orientation(&self) -> Quat {
		self.orientation.get_or_update(|| {
			Quat::from_yaw(self.yaw)
				* Quat::from_pitch(self.pitch)
		})
	}


	pub fn projection_matrix(&self) -> Mat4 {
		self.projection_matrix.get_or_update(|| {
			use self::ProjectionMode::*;

			match self.projection_mode {
				Perspective { fov_y } => Mat4::perspective(fov_y, self.aspect, self.near, self.far),
				Orthographic { view_size } => Mat4::ortho_aspect(view_size, self.aspect, self.near, self.far)
			}
		})
	}

	pub fn view_matrix(&self) -> Mat4 {
		self.view_matrix.get_or_update(|| {
			use self::ViewMode::*;

			let orientation = self.orientation();

			let position = match self.view_mode {
				Orbit { distance } => self.position - orientation.forward() * distance,
				FirstPerson => self.position,
			};

			orientation.conjugate().to_mat4() * Mat4::translate(-position)
		})
	}


	pub fn projection_view(&self) -> Mat4 {
		self.proj_view_matrix.get_or_update(|| {
			 self.projection_matrix() * self.view_matrix()
		})
	}



	pub fn inverse_projection_matrix(&self) -> Mat4 {
		self.inv_projection_matrix.get_or_update(|| {
			self.projection_matrix().inverse()
		})
	}

	pub fn inverse_view_matrix(&self) -> Mat4 {
		self.inv_view_matrix.get_or_update(|| {
			self.view_matrix().inverse()
		})
	}

	pub fn inverse_projection_view(&self) -> Mat4 {
		self.inv_proj_view_matrix.get_or_update(|| {
			self.projection_view().inverse()
		})
	}



	fn mark_projection_dirty(&mut self) {
		self.projection_matrix.mark_dirty();
		self.proj_view_matrix.mark_dirty();
		self.inv_projection_matrix.mark_dirty();
		self.inv_proj_view_matrix.mark_dirty();
	}

	fn mark_view_dirty(&mut self) {
		self.view_matrix.mark_dirty();
		self.proj_view_matrix.mark_dirty();
		self.inv_view_matrix.mark_dirty();
		self.inv_proj_view_matrix.mark_dirty();
	}

	fn mark_orientation_dirty(&mut self) {
		self.orientation.mark_dirty();
		self.mark_view_dirty();
	}
}





#[derive(Clone, Debug)]
struct Memoised<T: Copy + Clone + Debug> {
	mat: Cell<T>,
	dirty: Cell<bool>
}

impl<T> Memoised<T> where T: Copy + Clone + Debug {
	fn new(init: T) -> Self {
		Memoised {
			mat: Cell::new(init),
			dirty: Cell::new(true),
		}
	}

	fn mark_dirty(&mut self) { self.dirty.set(true) }

	fn get_or_update<F>(&self, f: F) -> T where F: FnOnce() -> T {
		if self.dirty.get() {
			self.mat.set(f());
			self.dirty.set(false);
		}

		self.mat.get()
	}
}