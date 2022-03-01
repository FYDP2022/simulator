pub mod feature;

use wgpu::{Device, ShaderModule};

pub fn basic(device: &Device) -> ShaderModule {
  device.create_shader_module(&wgpu::ShaderModuleDescriptor {
    label: Some("Basic Shader"),
    source: wgpu::ShaderSource::Wgsl(include_str!("basic.wgsl").into()),
  })
}
