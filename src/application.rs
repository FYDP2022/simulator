use super::featuredb::FeatureDB;
use super::net::Client;

use std::sync::{Arc, Mutex};

use winit::event::*;
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::{Window, WindowBuilder};

pub struct Application {
  instance: wgpu::Instance,
  adapter: wgpu::Adapter,
  surface: wgpu::Surface,
  device: wgpu::Device,
  queue: wgpu::Queue,
  config: wgpu::SurfaceConfiguration,
  size: winit::dpi::PhysicalSize<u32>,
  event_loop: Mutex<Option<EventLoop<()>>>,
  window: Window,
  featuredb: FeatureDB,
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

    // The instance is a handle to our GPU
    // BackendBit::PRIMARY => Vulkan + Metal + DX12 + Browser WebGPU
    let instance = wgpu::Instance::new(wgpu::Backends::all());
    let surface = unsafe { instance.create_surface(&window) };
    let _adapter = instance
      .request_adapter(&wgpu::RequestAdapterOptions {
        power_preference: wgpu::PowerPreference::default(),
        compatible_surface: Some(&surface),
      })
      .await
      .unwrap();

    let adapter = instance
      .enumerate_adapters(wgpu::Backends::all())
      .filter(|adapter| {
        // Check if this adapter supports our surface
        surface.get_preferred_format(&adapter).is_some()
      })
      .next()
      .unwrap();

    let (device, queue) = adapter
      .request_device(
        &wgpu::DeviceDescriptor {
          features: wgpu::Features::empty(),
          limits: wgpu::Limits::default(),
          label: None,
        },
        None, // Trace path
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

    Self {
      instance,
      adapter,
      surface,
      device,
      queue,
      config,
      size,
      event_loop: Mutex::new(Some(event_loop)),
      window,
      featuredb: FeatureDB::new(),
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

  pub fn input(&mut self, event: &WindowEvent) -> bool {
    false
  }

  pub fn update(&mut self) {}

  pub fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
    let output = self.surface.get_current_frame()?.output;
    let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());
    let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
      label: Some("Render Encoder"),
    });

    {
      let _render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
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
    }

    // submit will accept anything that implements IntoIter
    self.queue.submit(std::iter::once(encoder.finish()));

    Ok(())
  }

  pub async fn run(mut self) {
    let event_loop = self.event_loop.lock().unwrap().take().unwrap();
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
