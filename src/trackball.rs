use super::raycast::{Ball, Intersect, Model, Plane, Ray, Transform};

use cgmath::{InnerSpace, Matrix4, Point3, Rad, Vector3};

pub struct VirtualTrackball {
  position: Point3<f32>,
  radius: f32,
  model: Model,
}

impl VirtualTrackball {
  pub fn new(position: Point3<f32>, radius: f32) -> Self {
    VirtualTrackball {
      position,
      radius,
      model: Model::Transform(
        Transform::new(Matrix4::from_translation(position.to_homogeneous().truncate()))
          .expect("ray transform to be valid"),
        Box::new(Model::Object(Box::new(Ball::new(radius)))),
      ),
    }
  }

  pub fn position(&self) -> Point3<f32> {
    self.position
  }

  fn intersect(&self, ray: &Ray) -> Point3<f32> {
    if let Some(intersect) = self.model.intersect(ray) {
      intersect.position
    } else {
      let plane = Plane {
        position: self.position,
        normal: (self.position - ray.eye).normalize(),
      };
      let delta: Vector3<f32> = plane.intersect(ray).closest().unwrap().position - self.position;
      self.position + delta.normalize() * self.radius
    }
  }

  pub fn compute(&self, start: Ray, end: Ray) -> Option<(Vector3<f32>, Rad<f32>)> {
    if start == end {
      None
    } else {
      let start_point: Vector3<f64> = (self.intersect(&start) - self.position).cast().unwrap();
      let end_point: Vector3<f64> = (self.intersect(&end) - self.position).cast().unwrap();
      let axis = start_point.cross(end_point).normalize();
      let Rad(angle) = start_point.angle(end_point);
      Some((axis.cast().unwrap(), Rad(angle as f32)))
    }
  }

  pub fn test(&self, ray: Ray) -> bool {
    self.model.intersect(&ray).is_some()
  }
}

#[cfg(test)]
mod test {
  use super::*;
  use cgmath::Deg;

  #[test]
  fn trackball_test() {
    let trackball = VirtualTrackball::new((0.0, 0.0, 0.0).into(), 1.0);
    let start = Ray {
      eye: (0.0, 0.0, -2.0).into(),
      target: (0.0, 0.0, 0.0).into(),
    };
    let end = Ray {
      eye: (-2.0, 0.0, -2.0).into(),
      target: (0.0, 0.0, 0.0).into(),
    };
    let (axis, angle) = trackball.compute(start, end).unwrap();
    assert!((axis - Vector3::unit_y()).magnitude() < 0.00001);
    assert_eq!(angle, Deg(45.0).into());
  }
}
