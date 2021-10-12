use super::camera::Camera;

use wgpu::{Device, SurfaceConfiguration, RenderPipeline, RenderPass};

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

    Self {
      pipeline,
    }
  }

  pub fn render<'a>(&'a self, render_pass: &mut RenderPass<'a>, camera: &'a Camera) {
    render_pass.set_pipeline(&self.pipeline);
    render_pass.set_bind_group(0, camera.bind_group(), &[]);
    render_pass.draw(0..3, 0..1);
  }
}
