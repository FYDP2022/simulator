use cgmath::{Matrix4, Point3};
use wgpu::{Device, ShaderModule, VertexBufferLayout};

pub fn compile(device: &Device) -> ShaderModule {
  device.create_shader_module(&wgpu::ShaderModuleDescriptor {
    label: Some("Feature Shader"),
    source: wgpu::ShaderSource::Wgsl(include_str!("feature.wgsl").into()),
  })
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct FeatureVertex {
  pub position: [f32; 3],
}

impl From<Point3<f32>> for FeatureVertex {
  fn from(point: Point3<f32>) -> Self {
    FeatureVertex { position: point.into() }
  }
}

impl FeatureVertex {
  pub fn description<'a>() -> VertexBufferLayout<'a> {
    wgpu::VertexBufferLayout {
      array_stride: std::mem::size_of::<FeatureVertex>() as wgpu::BufferAddress,
      step_mode: wgpu::VertexStepMode::Vertex,
      attributes: &wgpu::vertex_attr_array![
        0 => Float32x3,
      ],
    }
  }
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct FeatureInstance {
  pub model: [[f32; 4]; 4],
}

impl From<Matrix4<f32>> for FeatureInstance {
  fn from(matrix: Matrix4<f32>) -> Self {
    FeatureInstance { model: matrix.into() }
  }
}

impl FeatureInstance {
  pub fn description<'a>() -> VertexBufferLayout<'a> {
    const ATTRIBUTES: [wgpu::VertexAttribute; 4] = wgpu::vertex_attr_array![
      1 => Float32x4,
      2 => Float32x4,
      3 => Float32x4,
      4 => Float32x4,
    ];
    wgpu::VertexBufferLayout {
      array_stride: std::mem::size_of::<FeatureInstance>() as wgpu::BufferAddress,
      step_mode: wgpu::VertexStepMode::Instance,
      attributes: &ATTRIBUTES,
    }
  }
}

// #[repr(C)]
// #[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
// pub struct FeatureUniform {
// }