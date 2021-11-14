use super::camera::Camera;
use super::geometry::Geometry;
use super::shader::feature::{FeatureInstance, FeatureVertex};

use cgmath::Matrix4;
use wgpu::util::DeviceExt;
use wgpu::{Buffer, Device, RenderPass, RenderPipeline, SurfaceConfiguration};

pub struct BasicRenderer {
  pipeline: RenderPipeline,
}

impl BasicRenderer {
  pub fn new(device: &Device, config: &SurfaceConfiguration) -> Self {
    let shader = super::shader::basic(device);

    let camera_layout = Camera::layout(device);

    let render_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
      label: Some("Basic Shading Layout"),
      bind_group_layouts: &[&camera_layout],
      push_constant_ranges: &[],
    });

    let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
      label: Some("Render Pipeline"),
      layout: Some(&render_pipeline_layout),
      vertex: wgpu::VertexState {
        module: &shader,
        entry_point: "main",
        buffers: &[],
      },
      fragment: Some(wgpu::FragmentState {
        module: &shader,
        entry_point: "main",
        targets: &[wgpu::ColorTargetState {
          format: config.format,
          blend: Some(wgpu::BlendState::REPLACE),
          write_mask: wgpu::ColorWrites::ALL,
        }],
      }),
      primitive: wgpu::PrimitiveState {
        topology: wgpu::PrimitiveTopology::TriangleList,
        strip_index_format: None,
        front_face: wgpu::FrontFace::Ccw,
        cull_mode: Some(wgpu::Face::Back),
        polygon_mode: wgpu::PolygonMode::Fill,
        clamp_depth: false,
        conservative: false,
      },
      depth_stencil: None,
      multisample: wgpu::MultisampleState {
        count: 1,
        mask: !0,
        alpha_to_coverage_enabled: false,
      },
    });

    Self { pipeline }
  }

  pub fn render<'a>(&'a self, render_pass: &mut RenderPass<'a>, camera: &'a Camera) {
    render_pass.set_pipeline(&self.pipeline);
    render_pass.set_bind_group(0, camera.bind_group(), &[]);
    render_pass.draw(0..3, 0..1);
  }
}

pub struct FeatureRenderer {
  pipeline: RenderPipeline,
  vertex_buffer: Buffer,
  #[allow(dead_code)]
  vertices: Vec<FeatureVertex>,
  index_buffer: Buffer,
  indices: Vec<u16>,
  instance_buffer: Buffer,
  instances: Vec<FeatureInstance>,
}

impl FeatureRenderer {
  pub fn new(geometry: Geometry, instances: Vec<Matrix4<f32>>, device: &Device, config: &SurfaceConfiguration) -> Self {
    let shader = super::shader::feature::compile(device);

    let camera_layout = Camera::layout(device);

    let render_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
      label: Some("Basic Shading Layout"),
      bind_group_layouts: &[&camera_layout],
      push_constant_ranges: &[],
    });

    let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
      label: Some("Render Pipeline"),
      layout: Some(&render_pipeline_layout),
      vertex: wgpu::VertexState {
        module: &shader,
        entry_point: "main",
        buffers: &[FeatureVertex::description(), FeatureInstance::description()],
      },
      fragment: Some(wgpu::FragmentState {
        module: &shader,
        entry_point: "main",
        targets: &[wgpu::ColorTargetState {
          format: config.format,
          blend: Some(wgpu::BlendState::REPLACE),
          write_mask: wgpu::ColorWrites::ALL,
        }],
      }),
      primitive: wgpu::PrimitiveState {
        topology: wgpu::PrimitiveTopology::TriangleList,
        strip_index_format: None,
        front_face: wgpu::FrontFace::Ccw,
        cull_mode: Some(wgpu::Face::Back),
        polygon_mode: wgpu::PolygonMode::Fill,
        clamp_depth: false,
        conservative: false,
      },
      depth_stencil: None,
      multisample: wgpu::MultisampleState {
        count: 1,
        mask: !0,
        alpha_to_coverage_enabled: false,
      },
    });

    let vertices: Vec<FeatureVertex> = geometry.vertices.iter().map(|x| FeatureVertex::from(*x)).collect();

    let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
      label: Some("Vertex Buffer"),
      contents: bytemuck::cast_slice(&vertices[..]),
      usage: wgpu::BufferUsages::VERTEX,
    });

    let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
      label: Some("Index Buffer"),
      contents: bytemuck::cast_slice(&geometry.indices[..]),
      usage: wgpu::BufferUsages::INDEX,
    });

    let instances: Vec<FeatureInstance> = instances.iter().map(|x| FeatureInstance::from(*x)).collect();

    let instance_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
      label: Some("Instance Buffer"),
      contents: bytemuck::cast_slice(&instances[..]),
      usage: wgpu::BufferUsages::VERTEX,
    });

    Self {
      pipeline,
      vertex_buffer,
      vertices,
      index_buffer,
      indices: geometry.indices,
      instance_buffer,
      instances,
    }
  }

  pub fn render<'a>(&'a self, render_pass: &mut RenderPass<'a>, camera: &'a Camera) {
    render_pass.set_pipeline(&self.pipeline);
    render_pass.set_bind_group(0, camera.bind_group(), &[]);
    render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
    render_pass.set_vertex_buffer(1, self.instance_buffer.slice(..));
    render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
    render_pass.draw_indexed(0..self.indices.len() as u32, 0, 0..self.instances.len() as u32);
  }
}
