use cgmath::{SquareMatrix, Point3, Vector3, Matrix4};
use wgpu::{Buffer, BindGroup, BindGroupLayout, Device};
use wgpu::util::DeviceExt;

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
      view_proj: Matrix4::identity().into()
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
  uniform: CameraUniform,
  buffer: Buffer,
  bind_group: BindGroup,
}

impl Camera {
  pub fn new(device: &Device) -> Self {
    let uniform = CameraUniform::default();

    let buffer = device.create_buffer_init(
      &wgpu::util::BufferInitDescriptor {
        label: Some("Camera Buffer"),
        contents: bytemuck::cast_slice(&[uniform]),
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
      }
    );

    let camera_bind_group_layout = Self::layout(device);

    let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
      layout: &camera_bind_group_layout,
      entries: &[
        wgpu::BindGroupEntry {
          binding: 0,
          resource: buffer.as_entire_binding(),
        }
      ],
      label: Some("camera_bind_group"),
    });

    Self {
      eye: (0.0, 0.0, 0.0).into(),
      target: (0.0, 0.0, 1.0).into(),
      up: Vector3::unit_y(),
      fovy: 60.0,
      znear: 0.001,
      zfar: 1000.0,
      aspect: 1.0,
      uniform,
      buffer,
      bind_group,
    }
  }

  pub fn update(&mut self, device: &Device) {
    let view = Matrix4::look_at_rh(self.eye, self.target, self.up);
    let proj = cgmath::perspective(cgmath::Deg(self.fovy), self.aspect, self.znear, self.zfar);
    self.uniform.view_proj = (OPENGL_TO_WGPU_MATRIX * proj * view).into();
    self.buffer = device.create_buffer_init(
      &wgpu::util::BufferInitDescriptor {
        label: Some("Camera Buffer"),
        contents: bytemuck::cast_slice(&[self.uniform]),
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
      }
    );
    let camera_bind_group_layout = Self::layout(device);
    self.bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
      layout: &camera_bind_group_layout,
      entries: &[
        wgpu::BindGroupEntry {
          binding: 0,
          resource: self.buffer.as_entire_binding(),
        }
      ],
      label: Some("camera_bind_group"),
    });
  }

  pub fn layout(device: &Device) -> BindGroupLayout {
    device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
      entries: &[
        wgpu::BindGroupLayoutEntry {
          binding: 0,
          visibility: wgpu::ShaderStages::VERTEX,
          ty: wgpu::BindingType::Buffer {
            ty: wgpu::BufferBindingType::Uniform,
            has_dynamic_offset: false,
            min_binding_size: None,
          },
          count: None,
        }
      ],
      label: Some("camera_bind_group_layout"),
    })
  }

  pub fn bind_group<'a>(&'a self) -> &'a BindGroup {
    &self.bind_group
  }
}
