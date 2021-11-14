use super::gfx::camera::Camera;
use super::raycast::Ray;

use cgmath::{Deg, InnerSpace, Matrix4, Point3, Rad, Vector3};
use winit::dpi::{PhysicalPosition, PhysicalSize};
use winit::event::*;

use std::collections::HashMap;

#[allow(dead_code)]
#[derive(Clone, Copy)]
pub enum UIEvent {
  RotateAboutObject(Point3<f32>),
  FreeMoveCamera,
  None,
}

#[derive(Clone, Copy)]
pub enum MouseEvent {
  None,
  Click,
  Move,
  Release,
}

impl From<ElementState> for MouseEvent {
  fn from(other: ElementState) -> Self {
    match other {
      ElementState::Pressed => MouseEvent::Click,
      ElementState::Released => MouseEvent::Release,
    }
  }
}

#[derive(Clone, Copy)]
pub enum KeyEvent {
  None,
  Press,
  Hold,
  Release,
}

impl From<ElementState> for KeyEvent {
  fn from(other: ElementState) -> Self {
    match other {
      ElementState::Pressed => KeyEvent::Press,
      ElementState::Released => KeyEvent::Release,
    }
  }
}

impl KeyEvent {
  pub fn is_down(&self) -> bool {
    matches!(self, KeyEvent::Press | KeyEvent::Hold)
  }
}

#[derive(Clone)]
pub struct UIState {
  pub left: MouseEvent,
  pub middle: MouseEvent,
  pub right: MouseEvent,
  pub keys: HashMap<VirtualKeyCode, KeyEvent>,
  pub position: PhysicalPosition<f64>,
  pub size: PhysicalSize<u32>,
  pub event: UIEvent,
}

impl Default for UIState {
  fn default() -> Self {
    UIState {
      left: MouseEvent::None,
      middle: MouseEvent::None,
      right: MouseEvent::None,
      keys: HashMap::new(),
      position: PhysicalPosition { x: 0.0, y: 0.0 },
      size: PhysicalSize { width: 0, height: 0 },
      event: UIEvent::None,
    }
  }
}

impl UIState {
  fn _ray(&self, camera: &Camera, size: PhysicalSize<u32>) -> Ray {
    let fovy = camera.fovy;
    let fovx = fovy * size.width as f32 / size.height as f32;
    let Rad(xang) = Deg(-((self.position.x as f32 / size.width as f32) * fovx - fovx / 2.0)).into();
    let Rad(yang) = Deg(-((self.position.y as f32 / size.height as f32) * fovy - fovy / 2.0)).into();
    let direction = camera.target - camera.eye;
    let rotated = Matrix4::from_axis_angle(camera.up, Rad(xang))
      * Matrix4::from_axis_angle(direction.cross(camera.up), Rad(yang))
      * direction.extend(0.0);
    Ray {
      eye: camera.eye,
      target: camera.eye + rotated.truncate(),
    }
  }

  fn key(&self, key: &VirtualKeyCode) -> KeyEvent {
    if let Some(value) = self.keys.get(key) {
      *value
    } else {
      KeyEvent::None
    }
  }
}

pub struct UserInterface {
  pub last_state: UIState,
  pub current_state: UIState,
}

impl Default for UserInterface {
  fn default() -> Self {
    UserInterface {
      last_state: UIState::default(),
      current_state: UIState::default(),
    }
  }
}

impl UserInterface {
  pub fn new(size: PhysicalSize<u32>) -> Self {
    let mut result = Self::default();
    result.current_state.size = size;
    result.last_state.size = size;
    result
  }

  fn mouse_angle(&self, camera: &Camera) -> (Deg<f32>, Deg<f32>) {
    let current = &self.current_state.position;
    let last = &self.last_state.position;
    let fovy = camera.fovy;
    let fovx = fovy * self.current_state.size.width as f32 / self.current_state.size.height as f32;
    let x_angle = Deg(((current.x - last.x) as f32 / (self.current_state.size.width as f32)) * fovx);
    let y_angle = Deg(((current.y - last.y) as f32 / (self.current_state.size.height as f32)) * fovy);
    (x_angle, y_angle)
  }

  pub fn _rotate_about_object(&self, position: Point3<f32>, camera: &mut Camera) {
    let current = &self.current_state.position;
    let last = &self.last_state.position;
    if current != last {
      let (x_angle, y_angle) = self.mouse_angle(camera);
      let transform = Matrix4::from_angle_x(-y_angle) * Matrix4::from_angle_y(-x_angle);
      let delta = (transform * (camera.eye - position).extend(0.0)).truncate();
      camera.eye = position + delta;
      camera.target = position;
      camera.up = (transform * camera.up.extend(0.0)).truncate();
    }
  }

  pub fn free_move(&self, camera: &mut Camera) {
    let current = &self.current_state.position;
    let last = &self.last_state.position;

    let mut move_relative: Vector3<f32> = (0.0, 0.0, 0.0).into();

    if self.current_state.key(&VirtualKeyCode::W).is_down() {
      move_relative.z += 1.0;
    }
    if self.current_state.key(&VirtualKeyCode::A).is_down() {
      move_relative.x -= 1.0;
    }
    if self.current_state.key(&VirtualKeyCode::S).is_down() {
      move_relative.z -= 1.0;
    }
    if self.current_state.key(&VirtualKeyCode::D).is_down() {
      move_relative.x += 1.0;
    }

    if move_relative.magnitude() > 0.0 {
      move_relative = move_relative.normalize() * 0.05;
    }
    let translation =
      move_relative.x * camera.right() + move_relative.y * camera.up.normalize() + move_relative.z * camera.forward();

    camera.eye += translation;
    camera.target += translation;

    if current != last {
      let (x_angle, y_angle) = self.mouse_angle(camera);
      let transform =
        Matrix4::from_axis_angle(camera.right(), -y_angle) * Matrix4::from_axis_angle(camera.up, -x_angle);
      let delta = (transform * (camera.target - camera.eye).extend(0.0)).truncate();
      camera.target = camera.eye + delta;
      camera.up = (transform * camera.up.extend(0.0)).truncate();
    }
  }
}

#[cfg(test)]
mod test {
  use super::*;
  use cgmath::{EuclideanSpace, MetricSpace};

  #[test]
  fn rotate_about_object_test() {
    let mut camera = Camera::mock();
    camera.fovy = 180.0;
    let size = PhysicalSize {
      width: 100,
      height: 100,
    };
    let mut user_interface = UserInterface::new(size);
    user_interface.last_state.position = PhysicalPosition { x: 0.0, y: 0.0 };
    user_interface.current_state.position = PhysicalPosition { x: 50.0, y: 0.0 };
    user_interface._rotate_about_object(Point3::origin(), &mut camera);
    assert_eq!(camera.up, (0.0, 1.0, 0.0).into());
    assert_eq!(camera.target, (0.0, 0.0, 0.0).into());
    assert!(camera.eye.distance((1.0, 0.0, 0.0).into()) < 0.00001);
  }
}
