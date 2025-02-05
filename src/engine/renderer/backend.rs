use crate::engine::core::window::Window;
use std::sync::{Arc, Mutex};
use wgpu::{
    Backends, Device, Instance, InstanceDescriptor, Queue, Surface, SurfaceConfiguration,
    TextureFormat,
};

pub struct Backend<'a> {
    surface: Surface<'a>,
    device: Arc<Device>,
    queue: Arc<Queue>,
    config: Mutex<SurfaceConfiguration>,
    format: TextureFormat,
}

impl Backend<'_> {
    pub async fn new(window: &Window) -> Self {
        let instance = Instance::new(&InstanceDescriptor {
            backends: Backends::PRIMARY,
            ..Default::default()
        });

        let surface = instance.create_surface(window.window().clone()).unwrap();

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
                    required_features: wgpu::Features::PUSH_CONSTANTS,
                    // WebGL doesn't support all of wgpu's features, so if
                    // we're building for the web, we'll have to disable some.
                    required_limits: wgpu::Limits {
                        max_push_constant_size: 128, // Mindestens 12 Bytes, typischerweise 128 oder 256
                        ..Default::default()
                    },

                    label: None,
                    memory_hints: Default::default(),
                },
                None, // Trace path
            )
            .await
            .unwrap();

        let surface_caps = surface.get_capabilities(&adapter);

        let surface_format = surface_caps
            .formats
            .iter()
            .copied()
            .find(|f| f.is_srgb())
            .unwrap_or(surface_caps.formats[0]);

        let (width, height) = window.dimension();

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width,
            height,
            present_mode: surface_caps.present_modes[0],
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        surface.configure(&device, &config);

        let queue = Arc::new(queue);

        Self {
            device: Arc::new(device),
            queue,
            surface,
            config: Mutex::new(config),
            format: surface_format,
        }
    }

    pub fn device(&self) -> &Arc<Device> {
        &self.device
    }

    pub fn queue(&self) -> &Arc<Queue> {
        &self.queue
    }

    pub fn surface(&self) -> &Surface {
        &self.surface
    }

    pub fn surface_configuration(&self) -> &Mutex<SurfaceConfiguration> {
        &self.config
    }

    pub fn surface_format(&self) -> &TextureFormat {
        &self.format
    }
}
