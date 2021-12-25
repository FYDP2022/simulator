use super::featuredb::{Feature, FeatureDB};
use super::gfx::camera::Camera;
use super::gfx::renderer::{BasicRenderer, FeatureRenderer};
use super::gfx::shader::feature::FeatureInstance;
use super::gfx::texture::Texture;
use super::net::Client;
use super::ui::{KeyEvent, MouseEvent, UIEvent, UserInterface};

use winit::event::*;
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::{Window, WindowBuilder};

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
  feature_renderer: FeatureRenderer,
  _database: FeatureDB,
  websocket: Option<Client>,
  user_interface: UserInterface,
  depth_texture: Texture,
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
        force_fallback_adapter: false,
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
    camera.up = (0.0, 1.0, 0.0).into();
    camera.eye = (0.0, 0.0, 5.0).into();
    camera.target = (0.0, 0.0, 0.0).into();
    camera.update(&device);

    let database = FeatureDB::new().unwrap();
    let instances = {
      let mut stmt = database.current_frame().unwrap();
      stmt
        .query_map([], Feature::from_row)
        .unwrap()
        .map(|result| result.unwrap())
        .map(|feature| FeatureInstance {
          model: feature.transform().into(),
          color: feature.color.map(|x| x as f32 / 255.0).into(),
        })
        .collect()
    };

    use super::gfx::renderer;

    let basic_renderer = BasicRenderer::new(renderer::BasicRendererConfiguration {
      device: &device,
      surface_config: &config,
    });

    let feature_renderer = FeatureRenderer::new(renderer::FeatureRendererConfiguration {
      geometry: super::gfx::geometry::uv_sphere(100),
      instances,
      device: &device,
      surface_config: &config,
    });

    let depth_texture = Texture::create_depth_texture(&device, &config, "depth_texture");

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
      feature_renderer,
      _database: database,
      websocket: Client::new().await.ok(),
      user_interface: UserInterface::new(size),
      depth_texture,
    }
  }

  pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
    if new_size.width > 0 && new_size.height > 0 {
      self.size = new_size;
      self.config.width = new_size.width;
      self.config.height = new_size.height;
      self.surface.configure(&self.device, &self.config);
      self.depth_texture = Texture::create_depth_texture(&self.device, &self.config, "depth_texture");
    }
  }

  pub fn input(&mut self, event: &WindowEvent) -> bool {
    let current = &mut self.user_interface.current_state;
    match event {
      WindowEvent::MouseInput { button, state, .. } => {
        match button {
          MouseButton::Left => current.left = MouseEvent::from(*state),
          MouseButton::Middle => current.middle = MouseEvent::from(*state),
          MouseButton::Right => current.right = MouseEvent::from(*state),
          _ => (),
        };
        true
      }
      WindowEvent::CursorMoved { position, .. } => {
        current.position = *position;
        true
      }
      WindowEvent::KeyboardInput { input, .. } => {
        if let Some(key) = input.virtual_keycode {
          current.keys.insert(key, KeyEvent::from(input.state));
          true
        } else {
          false
        }
      }
      _ => false,
    }
  }

  pub fn update(&mut self) {
    let current = self.user_interface.current_state.clone();
    // let last = self.user_interface.last_state.clone();
    let mut next = current.clone();

    // let last_ray = last.ray(&self.camera, self.window.inner_size());
    // let current_ray = current.ray(&self.camera, self.window.inner_size());

    match current.left {
      MouseEvent::Click => {
        next.left = MouseEvent::Move;
      }
      MouseEvent::Release => {
        next.left = MouseEvent::None;
      }
      _ => (),
    };

    match current.middle {
      MouseEvent::Click => {
        next.middle = MouseEvent::Move;
      }
      MouseEvent::Release => {
        next.middle = MouseEvent::None;
      }
      _ => (),
    };

    match current.right {
      MouseEvent::Click => {
        next.event = UIEvent::FreeMoveCamera;
        next.right = MouseEvent::Move;
      }
      MouseEvent::Move => {
        if let UIEvent::FreeMoveCamera = current.event {
          self.user_interface.free_move(&mut self.camera);
        }
      }
      MouseEvent::Release => {
        next.event = UIEvent::None;
        next.right = MouseEvent::None;
      }
      _ => (),
    }

    for (_, event) in next.keys.iter_mut() {
      match event {
        KeyEvent::Press => *event = KeyEvent::Hold,
        KeyEvent::Release => *event = KeyEvent::None,
        _ => (),
      }
    }

    self.user_interface.last_state = current;
    self.user_interface.current_state = next;
    self.camera.update(&self.device);
  }

  pub fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
    let output = self.surface.get_current_texture()?;
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
        depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
          view: &self.depth_texture.view,
          depth_ops: Some(wgpu::Operations {
            load: wgpu::LoadOp::Clear(1.0),
            store: true,
          }),
          stencil_ops: None,
        }),
      });

      self.basic_renderer.render(&mut render_pass, &self.camera);
      self.feature_renderer.render(&mut render_pass, &self.camera);
    }

    self.queue.submit(std::iter::once(encoder.finish()));
    output.present();

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
        _ => {
          self.input(event);
        }
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
