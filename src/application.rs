use super::featuredb::FeatureDB;
use super::net::Client;
use super::gfx::renderer::BasicRenderer;
use super::gfx::camera::Camera;

use winit::event::*;
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::{Window, WindowBuilder};
use wgpu::util::DeviceExt;

pub struct Application {
  _instance: wgpu::Instance,
  _adapter: wgpu::Adapter,
  surface: wgpu::Surface,
  device: wgpu::Device,
  queue: wgpu::Queue,
  config: wgpu::SurfaceConfiguration,
  size: winit::dpi::PhysicalSize<u32>,
  event_loop: Option<EventLoop<()>>,
  window: Window,

  camera: Camera,
  basic_renderer: BasicRenderer,
  _featuredb: FeatureDB,
  websocket: Option<Client>,
}

impl Application {
  pub async fn new() -> Self {
    env_logger::init();

    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
      .with_title("Lawny Simulator")
      .build(&event_loop)
      .unwrap();

    let size = window.inner_size();

    let instance = wgpu::Instance::new(wgpu::Backends::all());
    let surface = unsafe { instance.create_surface(&window) };

    let adapter = instance
      .request_adapter(&wgpu::RequestAdapterOptions {
        power_preference: wgpu::PowerPreference::default(),
        compatible_surface: Some(&surface),
      })
      .await
      .unwrap();

    let (device, queue) = adapter
      .request_device(
        &wgpu::DeviceDescriptor {
          features: wgpu::Features::empty(),
          limits: wgpu::Limits::default(),
          label: None,
        },
        None,
      )
      .await
      .unwrap();

    let config = wgpu::SurfaceConfiguration {
      usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
      format: surface.get_preferred_format(&adapter).unwrap(),
      width: size.width,
      height: size.height,
      present_mode: wgpu::PresentMode::Fifo,
    };
    surface.configure(&device, &config);

    let mut camera = Camera::new(&device);
    camera.aspect = size.width as f32 / size.height as f32;
    camera.up = (0.5, 0.5, 0.0).into();
    camera.eye = (0.0, 0.0, 1.0).into();
    camera.target = (0.0, 0.0, 0.0).into();
    camera.update(&device);

    let basic_renderer = BasicRenderer::new(&device, &config);

    Self {
      _instance: instance,
      _adapter: adapter,
      surface,
      device,
      queue,
      config,
      size,
      event_loop: Some(event_loop),
      window,
      camera,
      basic_renderer,
      _featuredb: FeatureDB::new(),
      websocket: Client::new().await.ok(),
    }
  }

  pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
    if new_size.width > 0 && new_size.height > 0 {
      self.size = new_size;
      self.config.width = new_size.width;
      self.config.height = new_size.height;
      self.surface.configure(&self.device, &self.config);
    }
  }

  pub fn _input(&mut self, _event: &WindowEvent) -> bool {
    unimplemented!();
  }

  pub fn update(&mut self) {
    self.camera.update(&self.device);
  }

  pub fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
    let output = self.surface.get_current_frame()?.output;
    let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());
    let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
      label: Some("Render Encoder"),
    });

    {
      let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
        label: Some("Render Pass"),
        color_attachments: &[wgpu::RenderPassColorAttachment {
          view: &view,
          resolve_target: None,
          ops: wgpu::Operations {
            load: wgpu::LoadOp::Clear(wgpu::Color {
              r: 0.1,
              g: 0.2,
              b: 0.3,
              a: 1.0,
            }),
            store: true,
          },
        }],
        depth_stencil_attachment: None,
      });

      self.basic_renderer.render(&mut render_pass, &self.camera);
    }

    // submit will accept anything that implements IntoIter
    self.queue.submit(std::iter::once(encoder.finish()));

    Ok(())
  }

  pub async fn run(mut self) {
    let event_loop = self.event_loop.take().unwrap();
    event_loop.run(move |event, _, control_flow| match event {
      Event::WindowEvent { ref event, window_id } if window_id == self.window.id() => match event {
        WindowEvent::CloseRequested
        | WindowEvent::KeyboardInput {
          input:
            KeyboardInput {
              state: ElementState::Pressed,
              virtual_keycode: Some(VirtualKeyCode::Escape),
              ..
            },
          ..
        } => *control_flow = ControlFlow::Exit,
        WindowEvent::Resized(physical_size) => {
          self.resize(*physical_size);
          self.camera.aspect = physical_size.width as f32 / physical_size.height as f32;
        }
        _ => {}
      },
      Event::RedrawRequested(_) => {
        if let Some(client) = &self.websocket {
          while let Ok(_msg) = client.stream().try_next() {
            println!("WS message received");
          }
        }
        self.update();
        match self.render() {
          Ok(_) => {}
          // Reconfigure the surface if lost
          Err(wgpu::SurfaceError::Lost) => self.resize(self.size),
          // The system is out of memory, we should probably quit
          Err(wgpu::SurfaceError::OutOfMemory) => *control_flow = ControlFlow::Exit,
          // All other errors (Outdated, Timeout) should be resolved by the next frame
          Err(e) => eprintln!("{:?}", e),
        }
      }
      Event::MainEventsCleared => {
        // RedrawRequested will only trigger once, unless we manually
        // request it.
        self.window.request_redraw();
      }
      _ => {}
    });
  }
}
