use cgmath::{InnerSpace, EuclideanSpace, MetricSpace, Vector3, Point3, Matrix3, Matrix4};
use roots::Roots;

#[derive(Debug, PartialEq)]
pub enum Clipped<T: Sized> {
  Inside(T),
  Outside(T),
}

pub trait Clip: Sized {
  fn clip(self, plane: &Plane) -> Clipped<Self>;
}

impl Clip for Point3<f32> {
  fn clip(self, plane: &Plane) -> Clipped<Point3<f32>> {
    let v = self - plane.position;
    let dist = v.dot(plane.normal);
    if dist >= 0.0 {
      Clipped::Inside(self)
    } else {
      Clipped::Outside(self - dist * plane.normal)
    }
  }
}

pub struct Ray {
  pub eye: Point3<f32>,
  pub target: Point3<f32>,
}

impl Ray {
  pub fn transform(self, matrix: &Matrix4<f32>) -> Self {
    Ray {
      eye: Point3::from_homogeneous(matrix * self.eye.to_homogeneous()),
      target: Point3::from_homogeneous(matrix * self.target.to_homogeneous()),
    }
  }

  pub fn delta(&self) -> Vector3<f32> {
    self.target - self.eye
  }
}

#[derive(Debug, PartialEq)]
pub struct Intersection {
  pub position: Point3<f32>,
  pub normal: Vector3<f32>,
}

impl Intersection {
  pub fn transform(self, position_transform: &Matrix4<f32>, normal_transform: &Matrix3<f32>) -> Self {
    Intersection {
      position: Point3::from_homogeneous(position_transform * self.position.to_homogeneous()),
      normal: (normal_transform * self.normal).normalize(),
    }
  }
}

#[derive(Debug, PartialEq)]
pub enum IntersectResult {
  Miss,
  HitOnce(Intersection),
  HitTwice(Intersection, Intersection),
}

pub trait Intersect {
  fn intersect(&self, ray: &Ray) -> IntersectResult;
}

pub struct Plane {
  position: Point3<f32>,
  normal: Vector3<f32>,
}

impl Intersect for Plane {
  fn intersect(&self, ray: &Ray) -> IntersectResult {
    let delta = ray.delta();
    let denom = delta.dot(self.normal);
    if denom == 0.0 {
      IntersectResult::Miss
    } else {
      let t = (self.position.dot(self.normal) - ray.eye.dot(self.normal)) / denom;
      if t >= 0.0 {
        let position = ray.eye + t * delta;
        if denom < 0.0 {
          IntersectResult::HitOnce(Intersection {
            position,
            normal: self.normal,
          })
        } else {
          IntersectResult::HitOnce(Intersection {
            position,
            normal: -self.normal,
          })
        }
      } else {
        IntersectResult::Miss
      }
    }
  }
}

pub struct Ball {
  radius: f32,
}

impl Ball {
  pub fn new(radius: f32) -> Self {
    Ball {
      radius,
    }
  }
}

impl Intersect for Ball {
  fn intersect(&self, ray: &Ray) -> IntersectResult {
    let delta = ray.delta();
    let a = delta.dot(delta);
    let b = 2.0 * ray.eye.dot(delta);
    let c = ray.eye.dot(ray.eye.to_vec()) - self.radius * self.radius;
    match roots::find_roots_quadratic(a, b, c) {
      Roots::One([r1]) => {
        let p = ray.eye + r1 * delta;
        IntersectResult::HitOnce(Intersection {
          position: p,
          normal: p.to_vec().normalize(),
        })
      },
      Roots::Two([r1, r2]) => {
        let mut p1 = ray.eye + r1 * delta;
        let mut p2 = ray.eye + r2 * delta;
        if ray.eye.distance(p1) > ray.eye.distance(p2) {
          std::mem::swap(&mut p1, &mut p2)
        }
        let first = Intersection {
          position: p1,
          normal: p1.to_vec().normalize(),
        };
        let second = Intersection {
          position: p2,
          normal: p2.to_vec().normalize(),
        };
        IntersectResult::HitTwice(first, second)
      },
      _ => IntersectResult::Miss,
    }
  }
}

#[cfg(test)]
mod test {
  use super::*;

  #[test]
  fn clip_point_test() {
    let point = cgmath::point3(1.0, 1.0, 1.0);
    let plane = Plane {
      position: Point3::origin(),
      normal: Vector3::unit_z(),
    };
    let clipped = point.clip(&plane);
    assert_eq!(clipped, Clipped::Inside(cgmath::point3(1.0, 1.0, 1.0)));
    let point = cgmath::point3(1.0, 1.0, -1.0);
    let clipped = point.clip(&plane);
    assert_eq!(clipped, Clipped::Outside(cgmath::point3(1.0, 1.0, 0.0)));
  }

  #[test]
  fn ray_transform_test() {
    let ray = Ray {
      eye: Point3::origin(),
      target: (0.0, 0.0, 5.0).into(),
    };
    let transformed = ray.transform(&Matrix4::from_translation((1.0, 1.0, 1.0).into()));
    assert_eq!(transformed.eye, (1.0, 1.0, 1.0).into());
    assert_eq!(transformed.target, (1.0, 1.0, 6.0).into());
  }

  #[test]
  fn plane_intersect_test() {
    let plane = Plane {
      position: Point3::origin(),
      normal: Vector3::unit_z(),
    };
    let ray = Ray {
      eye: (0.0, 0.0, 1.0).into(),
      target: (0.0, 0.0, 2.0).into(),
    };
    assert_eq!(plane.intersect(&ray), IntersectResult::Miss);
    let ray = Ray {
      eye: (0.0, 0.0, -1.0).into(),
      target: (0.0, 0.0, 0.0).into(),
    };
    assert_eq!(plane.intersect(&ray), IntersectResult::HitOnce(Intersection {
      position: (0.0, 0.0, 0.0).into(),
      normal: -Vector3::unit_z(),
    }));
  }

  #[test]
  fn intersect_ball_test() {
    let ball = Ball {
      radius: 5.0,
    };
    let ray = Ray {
      eye: (0.0, 0.0, -10.0).into(),
      target: Point3::origin(),
    };
    assert_eq!(ball.intersect(&ray), IntersectResult::HitTwice(
      Intersection {
        position: (0.0, 0.0, -5.0).into(),
        normal: (0.0, 0.0, -1.0).into(),
      },
      Intersection {
        position: (0.0, 0.0, 5.0).into(),
        normal: (0.0, 0.0, 1.0).into(),
      },
    ));
    let ray = Ray {
      eye: (5.0, 0.0, -10.0).into(),
      target: (5.0, 0.0, 0.0).into(),
    };
    assert_eq!(ball.intersect(&ray), IntersectResult::HitOnce(Intersection {
      position: (5.0, 0.0, 0.0).into(),
      normal: Vector3::unit_x(),
    }));
  }
}
