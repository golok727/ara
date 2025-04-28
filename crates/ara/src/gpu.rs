pub mod error;

use std::ops::Deref;

pub use error::*;

pub use wgpu::*;

#[derive(Debug, Clone)]
pub struct GpuContext {
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub instance: wgpu::Instance,
    pub adapter: wgpu::Adapter,
}

impl Deref for GpuContext {
    type Target = wgpu::Device;

    fn deref(&self) -> &Self::Target {
        &self.device
    }
}

#[derive(Debug, Default)]
pub struct GpuContextCreateOptions<'a, 'window> {
    pub backends: wgpu::Backends,
    pub power_preference: wgpu::PowerPreference,
    pub compatible_surface: Option<&'a wgpu::Surface<'window>>,
}

impl GpuContext {
    pub fn create_instance(desc: &wgpu::InstanceDescriptor) -> wgpu::Instance {
        wgpu::Instance::new(desc)
    }

    pub async fn create_instance_with_wgpu_detection(
        desc: &wgpu::InstanceDescriptor
    ) -> wgpu::Instance {
        wgpu::util::new_instance_with_webgpu_detection(desc).await
    }

    async fn create_gpu_context<'a, 'window>(
        instance: wgpu::Instance,
        options: &GpuContextCreateOptions<'a, 'window>
    ) -> Result<Self, error::GpuContextCreateError> {
        let adapter = instance
            .request_adapter(
                &(wgpu::RequestAdapterOptions {
                    power_preference: options.power_preference,
                    force_fallback_adapter: false,
                    compatible_surface: options.compatible_surface,
                })
            ).await
            .map_err(|err| error::GpuContextCreateError::RequestAdapterError(err))?;

        let adapter_info = adapter.get_info();
        log::info!("Adapter: {:?}", adapter_info);

        let (device, queue) = adapter
            .request_device(
                &(wgpu::DeviceDescriptor {
                    label: Some("ara device"),
                    required_features: wgpu::Features::empty(),
                    required_limits: (
                        if cfg!(target_arch = "wasm32") {
                            wgpu::Limits::downlevel_webgl2_defaults()
                        } else {
                            wgpu::Limits::default()
                        }
                    ).using_resolution(adapter.limits()),
                    memory_hints: wgpu::MemoryHints::MemoryUsage,
                    ..Default::default()
                })
            ).await
            .map_err(error::GpuContextCreateError::RequestDeviceError)?;

        Ok(Self {
            device,
            queue,
            instance,
            adapter,
        })
    }

    pub async fn new<'a, 'window>(
        instance: wgpu::Instance,
        options: &GpuContextCreateOptions<'a, 'window>
    ) -> Result<Self, error::GpuContextCreateError> {
        Self::create_gpu_context(instance, options).await
    }

    pub fn create_command_encoder(&self, label: Option<&str>) -> wgpu::CommandEncoder {
        self.device.create_command_encoder(&(wgpu::CommandEncoderDescriptor { label }))
    }

    pub fn create_shader(&self, source: &str) -> wgpu::ShaderModule {
        self.device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: None,
            source: wgpu::ShaderSource::Wgsl(source.into()),
        })
    }

    pub fn create_shader_labeled(&self, source: &str, label: &str) -> wgpu::ShaderModule {
        self.device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some(label),
            source: wgpu::ShaderSource::Wgsl(source.into()),
        })
    }

    pub fn create_texture_init(
        &self,
        format: wgpu::TextureFormat,
        width: u32,
        height: u32,
        data: &[u8]
    ) -> wgpu::Texture {
        Self::create_texture_init_impl(&self.device, &self.queue, format, width, height, data)
    }

    pub fn create_vertex_buffer(&self, size: u64) -> wgpu::Buffer {
        self.device.create_buffer(
            &(wgpu::BufferDescriptor {
                label: Some("ara_draw_vertex_buffer"),
                mapped_at_creation: false,
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                size,
            })
        )
    }

    pub fn create_index_buffer(&self, size: u64) -> wgpu::Buffer {
        self.device.create_buffer(
            &(wgpu::BufferDescriptor {
                label: Some("ara_draw_index_buffer"),
                mapped_at_creation: false,
                usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
                size,
            })
        )
    }

    #[inline]
    pub fn create_texture_init_impl(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        format: wgpu::TextureFormat,
        width: u32,
        height: u32,
        data: &[u8]
    ) -> wgpu::Texture {
        let texture_size = wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        };

        let texture = device.create_texture(
            &(wgpu::TextureDescriptor {
                label: Some("Check Texture"),
                size: texture_size,
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format,
                usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
                view_formats: &[],
            })
        );

        queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &texture,
                aspect: wgpu::TextureAspect::All,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
            },
            data,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(4 * width),
                rows_per_image: None,
            },
            texture_size
        );

        texture
    }
}
