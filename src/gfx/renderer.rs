use super::camera::Camera;
use super::geometry::Geometry;
use super::shader::feature::{FeatureInstance, FeatureVertex};
use super::texture::Texture;

use wgpu::util::DeviceExt;
use wgpu::{Buffer, Device, RenderPass, RenderPipeline, SurfaceConfiguration};

pub struct BasicRendererConfiguration<'a> {
  pub device: &'a Device,
  pub surface_config: &'a SurfaceConfiguration,
}

pub struct BasicRenderer {
  pipeline: RenderPipeline,
}

impl BasicRenderer {
  pub fn new(config: BasicRendererConfiguration) -> Self {
    let shader = super::shader::basic(config.device);

    let camera_layout = Camera::layout(config.device);

    let render_pipeline_layout = config.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
      label: Some("Basic Shading Layout"),
      bind_group_layouts: &[&camera_layout],
      push_constant_ranges: &[],
    });

    let pipeline = config.device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
      label: Some("Render Pipeline"),
      layout: Some(&render_pipeline_layout),
      vertex: wgpu::VertexState {
        module: &shader,
        entry_point: "vertex",
        buffers: &[],
      },
      fragment: Some(wgpu::FragmentState {
        module: &shader,
        entry_point: "fragment",
        targets: &[wgpu::ColorTargetState {
          format: config.surface_config.format,
          blend: Some(wgpu::BlendState::REPLACE),
          write_mask: wgpu::ColorWrites::ALL,
        }],
      }),
      primitive: wgpu::PrimitiveState {
        topology: wgpu::PrimitiveTopology::TriangleList,
        strip_index_format: None,
        front_face: wgpu::FrontFace::Ccw,
        cull_mode: None,
        polygon_mode: wgpu::PolygonMode::Fill,
        unclipped_depth: false,
        conservative: false,
      },
      depth_stencil: Some(wgpu::DepthStencilState {
        format: Texture::DEPTH_FORMAT,
        depth_write_enabled: true,
        depth_compare: wgpu::CompareFunction::Less,
        stencil: wgpu::StencilState::default(),
        bias: wgpu::DepthBiasState::default(),
      }),
      multisample: wgpu::MultisampleState {
        count: 1,
        mask: !0,
        alpha_to_coverage_enabled: false,
      },
      multiview: None,
    });

    Self { pipeline }
  }

  pub fn render<'a>(&'a self, render_pass: &mut RenderPass<'a>, camera: &'a Camera) {
    render_pass.set_pipeline(&self.pipeline);
    render_pass.set_bind_group(0, camera.bind_group(), &[]);
    render_pass.draw(0..3, 0..1);
  }
}

pub struct FeatureRendererConfiguration<'a> {
  pub geometry: Geometry,
  pub instances: Vec<FeatureInstance>,
  pub device: &'a Device,
  pub surface_config: &'a SurfaceConfiguration,
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
  pub fn new(config: FeatureRendererConfiguration) -> Self {
    let shader = super::shader::feature::compile(config.device);

    let camera_layout = Camera::layout(config.device);

    let render_pipeline_layout = config.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
      label: Some("Basic Shading Layout"),
      bind_group_layouts: &[&camera_layout],
      push_constant_ranges: &[],
    });

    let pipeline = config.device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
      label: Some("Render Pipeline"),
      layout: Some(&render_pipeline_layout),
      vertex: wgpu::VertexState {
        module: &shader,
        entry_point: "vertex",
        buffers: &[FeatureVertex::description(), FeatureInstance::description()],
      },
      fragment: Some(wgpu::FragmentState {
        module: &shader,
        entry_point: "fragment",
        targets: &[wgpu::ColorTargetState {
          format: config.surface_config.format,
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
        unclipped_depth: false,
        conservative: false,
      },
      depth_stencil: Some(wgpu::DepthStencilState {
        format: Texture::DEPTH_FORMAT,
        depth_write_enabled: true,
        depth_compare: wgpu::CompareFunction::Less,
        stencil: wgpu::StencilState::default(),
        bias: wgpu::DepthBiasState::default(),
      }),
      multisample: wgpu::MultisampleState {
        count: 1,
        mask: !0,
        alpha_to_coverage_enabled: false,
      },
      multiview: None,
    });

    let vertices: Vec<FeatureVertex> = config
      .geometry
      .vertices
      .iter()
      .zip(config.geometry.normals.iter())
      .map(FeatureVertex::from)
      .collect();

    let vertex_buffer = config.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
      label: Some("Vertex Buffer"),
      contents: bytemuck::cast_slice(&vertices[..]),
      usage: wgpu::BufferUsages::VERTEX,
    });

    let index_buffer = config.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
      label: Some("Index Buffer"),
      contents: bytemuck::cast_slice(&config.geometry.indices[..]),
      usage: wgpu::BufferUsages::INDEX,
    });

    let instance_buffer = config.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
      label: Some("Instance Buffer"),
      contents: bytemuck::cast_slice(&config.instances[..]),
      usage: wgpu::BufferUsages::VERTEX,
    });

    Self {
      pipeline,
      vertex_buffer,
      vertices,
      index_buffer,
      indices: config.geometry.indices,
      instance_buffer,
      instances: config.instances,
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
