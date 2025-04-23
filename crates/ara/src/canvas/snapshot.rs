use anyhow::{ bail, Result };

use futures::channel::oneshot::{ self };
use ara_math::Size;
use wgpu::{ BufferAsyncError, Maintain, TextureUsages };

use crate::GpuContext;

use super::Canvas;

pub type SnapshotReceiver = oneshot::Receiver<CanvasSnapshotResult>;

pub type CanvasSnapshotResult = anyhow::Result<CanvasSnapshot>;

// This will only work with textures with usage COPY_SRC. Surface textures in some platform does
// not allow to add that flag; we need to render it to seperate texture instead
// TODO: config
pub trait CanvasSnapshotSource {
    fn get_source_texture(&self) -> wgpu::Texture;

    fn read_texture_data_async(&self, canvas: &Canvas) -> Result<SnapshotReceiver> {
        let source_texture = self.get_source_texture();

        if !source_texture.usage().contains(TextureUsages::COPY_SRC) {
            bail!("required TextureUsages::COPY_SRC in source texture");
        }

        let size = Size {
            width: source_texture.width(),
            height: source_texture.height(),
        };

        let gpu = canvas.renderer.gpu();

        let (sender, receiver) = oneshot::channel::<CanvasSnapshotResult>();

        read_texels_async(gpu, &source_texture, move |res| {
            let res = match res {
                Ok(data) => anyhow::Result::Ok(CanvasSnapshot { data, size }),
                Err(err) => anyhow::Result::Err(anyhow::anyhow!("Error reading texels {:#?}", err)),
            };

            if sender.send(res).is_err() {
                log::error!("Error reading texels: failed at sending async data");
            }
        })?;

        Ok(receiver)
    }
}

pub struct CanvasSnapshot {
    pub size: Size<u32>,
    pub data: Vec<u8>,
}

impl Canvas {
    pub fn snapshot_sync<Source: CanvasSnapshotSource>(
        &self,
        source: &Source
    ) -> CanvasSnapshotResult {
        let receiver = source.read_texture_data_async(self)?;

        self.renderer.gpu().device.poll(Maintain::Wait);

        futures::executor::block_on(receiver)?
    }

    // asyncronously receive a snapshot
    pub async fn snapshot<Source: CanvasSnapshotSource>(
        &self,
        source: &Source
    ) -> CanvasSnapshotResult {
        let gpu = self.renderer.gpu();

        let receiver = source.read_texture_data_async(self)?;

        while !gpu.device.poll(wgpu::Maintain::Poll).is_queue_empty() {}

        receiver.await?
    }
}

// FIXME: Alignment for copy buffer
pub fn read_texels_async(
    gpu: &GpuContext,
    src: &wgpu::Texture,
    read: impl FnOnce(Result<Vec<u8>, BufferAsyncError>) + Send + 'static
) -> Result<()> {
    let bytes_per_texel = src
        .format()
        .block_copy_size(None /* Sorry I wont read any depth or stencil textures */)
        .ok_or(anyhow::anyhow!("Invalid format unable to get texel size"))?;

    let buffer_size = (src.width() * src.height() * bytes_per_texel) as u64;

    let output_buffer = gpu.device.create_buffer(
        &(wgpu::BufferDescriptor {
            label: Some("Output Buffer"),
            size: buffer_size,
            usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        })
    );

    let mut encoder = gpu.create_command_encoder(Some("Command Encoder"));

    encoder.copy_texture_to_buffer(
        wgpu::TexelCopyTextureInfo {
            texture: src,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        wgpu::TexelCopyBufferInfo {
            buffer: &output_buffer,
            layout: wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(src.width() * bytes_per_texel),
                rows_per_image: Some(src.height()),
            },
        },
        wgpu::Extent3d {
            width: src.width(),
            height: src.height(),
            depth_or_array_layers: 1,
        }
    );

    // Submit the commands
    gpu.queue.submit(Some(encoder.finish()));
    let buffer_slice = output_buffer.slice(..);

    buffer_slice.map_async(wgpu::MapMode::Read, {
        let buffer = output_buffer.clone();
        move |res| {
            let data = buffer.slice(..).get_mapped_range();
            let res = res.map(|_| data.to_vec());
            read(res)
        }
    });

    Ok(())
}
