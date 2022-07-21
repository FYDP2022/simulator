use cgmath::{InnerSpace, Matrix4, Point3, SquareMatrix, Vector3};
use wgpu::util::DeviceExt;
use wgpu::{BindGroup, BindGroupLayout, Buffer, Device};

#[rustfmt::skip]
pub const OPENGL_TO_WGPU_MATRIX: Matrix4<f32> = Matrix4::new(
  1.0, 0.0, 0.0, 0.0,
  0.0, 1.0, 0.0, 0.0,
  0.0, 0.0, 0.5, 0.0,
  0.0, 0.0, 0.5, 1.0,
);

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct CameraUniform {
  view_proj: [[f32; 4]; 4],
}

impl Default for CameraUniform {
  fn default() -> Self {
    Self {
      view_proj: Matrix4::identity().into(),
    }
  }
}

#[derive(Debug)]
pub struct Camera {
  pub eye: Point3<f32>,
  pub target: Point3<f32>,
  pub up: Vector3<f32>,
  pub fovy: f32,
  pub znear: f32,
  pub zfar: f32,
  pub aspect: f32,
  private: Option<CameraPrivate>,
}

#[derive(Debug)]
struct CameraPrivate {
  uniform: CameraUniform,
  buffer: Buffer,
  bind_group: BindGroup,
}

impl Camera {
  pub fn new(device: &Device) -> Self {
    let uniform = CameraUniform::default();

    let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
      label: Some("Camera Buffer"),
      contents: bytemuck::cast_slice(&[uniform]),
      usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
    });

    let camera_bind_group_layout = Self::layout(device);

    let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
      layout: &camera_bind_group_layout,
      entries: &[wgpu::BindGroupEntry {
        binding: 0,
        resource: buffer.as_entire_binding(),
      }],
      label: Some("camera_bind_group"),
    });

    Self {
      eye: (0.0, 0.0, -1.0).into(),
      target: (0.0, 0.0, 0.0).into(),
      up: Vector3::unit_y(),
      fovy: 60.0,
      znear: 0.001,
      zfar: 1000.0,
      aspect: 1.0,
      private: Some(CameraPrivate {
        uniform,
        buffer,
        bind_group,
      }),
    }
  }

  #[cfg(test)]
  pub fn mock() -> Self {
    Self {
      eye: (0.0, 0.0, -1.0).into(),
      target: (0.0, 0.0, 0.0).into(),
      up: Vector3::unit_y(),
      fovy: 60.0,
      znear: 0.001,
      zfar: 1000.0,
      aspect: 1.0,
      private: None,
    }
  }

  pub fn update(&mut self, device: &Device) {
    let view = Matrix4::look_at_rh(self.eye, self.target, self.up);
    let proj = cgmath::perspective(cgmath::Deg(self.fovy), self.aspect, self.znear, self.zfar);
    let private = self.private.as_mut().unwrap();
    private.uniform.view_proj = (OPENGL_TO_WGPU_MATRIX * proj * view).into();
    private.buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
      label: Some("Camera Buffer"),
      contents: bytemuck::cast_slice(&[private.uniform]),
      usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
    });
    let camera_bind_group_layout = Self::layout(device);
    private.bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
      layout: &camera_bind_group_layout,
      entries: &[wgpu::BindGroupEntry {
        binding: 0,
        resource: private.buffer.as_entire_binding(),
      }],
      label: Some("camera_bind_group"),
    });
  }

  pub fn forward(&self) -> Vector3<f32> {
    (self.target - self.eye).normalize()
  }

  pub fn right(&self) -> Vector3<f32> {
    let delta = self.target - self.eye;
    delta.cross(self.up).normalize()
  }

  pub fn layout(device: &Device) -> BindGroupLayout {
    device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
      entries: &[wgpu::BindGroupLayoutEntry {
        binding: 0,
        visibility: wgpu::ShaderStages::VERTEX,
        ty: wgpu::BindingType::Buffer {
          ty: wgpu::BufferBindingType::Uniform,
          has_dynamic_offset: false,
          min_binding_size: None,
        },
        count: None,
      }],
      label: Some("camera_bind_group_layout"),
    })
  }

  pub fn bind_group(&self) -> &BindGroup {
    &self.private.as_ref().unwrap().bind_group
  }
}
