use std::ops::Range;

use crate::{paint::Vertex, render::ItemContext, DrawList, Mesh};

use super::System;

pub struct GeometrySystem {
    drawlist: DrawList,
    device: wgpu::Device,
    queue: wgpu::Queue,
    store: ahash::HashMap<GeometryHandle, Option<RenderBuffer>>,
    next_handle: usize,
}

struct RenderBuffer {
    vertices: Vec<Vertex>,
    indices: Vec<u32>,
    pub vb: GpuBuffer,
    pub ib: GpuBuffer,
    pub synced: bool,
}

pub struct RenderBufferSlice<'a> {
    pub vertex_buffer: wgpu::BufferSlice<'a>,
    pub index_buffer: wgpu::BufferSlice<'a>,
    pub index_count: u32,
}

impl<'a> RenderBufferSlice<'a> {
    fn new(buffer: &'a RenderBuffer, slice: &RenderBufferRange) -> Self {
        Self {
            index_count: slice.index_count as u32,
            vertex_buffer: buffer.vb.buffer.slice(slice.vertex_slice.clone()),
            index_buffer: buffer.ib.buffer.slice(slice.index_slice.clone()),
        }
    }
}

impl RenderBuffer {
    pub fn clear(&mut self) {
        self.synced = false;
        self.indices.clear();
        self.vertices.clear();
    }

    pub fn append_from_mesh(&mut self, mesh: &Mesh) {
        self.synced = false;
        self.indices.extend(mesh.indices.iter());
        self.vertices.extend(mesh.vertices.iter());
    }

    pub fn sync(&mut self, device: &wgpu::Device, queue: &wgpu::Queue) {
        if self.synced {
            return;
        }

        let vertex_size = std::mem::size_of::<Vertex>();
        let index_size = std::mem::size_of::<u32>();

        let vertex_buffer_size = (self.vertices.len() * vertex_size) as wgpu::BufferAddress;
        let index_buffer_size = (self.indices.len() * index_size) as wgpu::BufferAddress;

        if vertex_buffer_size > self.vb.capacity {
            self.vb = GpuBuffer::new(
                device,
                vertex_buffer_size,
                wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            );
        }

        if index_buffer_size > self.ib.capacity {
            self.ib = GpuBuffer::new(
                device,
                index_buffer_size,
                wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
            );
        }

        let vertex_data = bytemuck::cast_slice(&self.vertices);
        let index_data = bytemuck::cast_slice(&self.indices);

        queue.write_buffer(&self.vb.buffer, 0, vertex_data);
        queue.write_buffer(&self.ib.buffer, 0, index_data);

        self.synced = true;
    }
}

struct GpuBuffer {
    pub buffer: wgpu::Buffer,
    pub capacity: wgpu::BufferAddress,
}

impl GpuBuffer {
    pub fn new(
        device: &wgpu::Device,
        size: wgpu::BufferAddress,
        usage: wgpu::BufferUsages,
    ) -> Self {
        let buffer = device.create_buffer(
            &(wgpu::BufferDescriptor {
                label: Some("GPU Buffer"),
                size,
                usage,
                mapped_at_creation: false,
            }),
        );

        Self {
            buffer,
            capacity: size,
        }
    }
}

static INITIAL_VERTEX_BUFFER_SIZE: u64 = (std::mem::size_of::<Vertex>() * 1024) as u64;
static INITIAL_INDEX_BUFFER_SIZE: u64 = (std::mem::size_of::<u32>() * 1024 * 3) as u64;

impl RenderBuffer {
    pub fn new(device: &wgpu::Device) -> Self {
        let vb = GpuBuffer::new(
            device,
            INITIAL_VERTEX_BUFFER_SIZE,
            wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
        );
        let ib = GpuBuffer::new(
            device,
            INITIAL_INDEX_BUFFER_SIZE,
            wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
        );

        Self {
            vertices: Vec::new(),
            indices: Vec::new(),
            vb,
            ib,
            synced: false,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct GeometryHandle(pub(crate) usize);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RenderBufferRange {
    pub(crate) vertex_slice: Range<wgpu::BufferAddress>,
    pub(crate) index_slice: Range<wgpu::BufferAddress>,
    pub(crate) vertex_count: usize,
    pub(crate) index_count: usize,
}

impl RenderBufferRange {
    pub fn is_empty(&self) -> bool {
        self.vertex_count == 0 && self.index_count == 0
    }
}

impl GeometrySystem {
    pub fn new(cx: &mut ItemContext<Self>) -> Self {
        Self {
            drawlist: DrawList::default(),
            device: cx.gpu.device.clone(),
            queue: cx.gpu.queue.clone(),
            store: Default::default(),
            next_handle: 1,
        }
    }
}

impl System for GeometrySystem {
    fn init(&mut self, _cx: &mut crate::render::RenderContext) {}
}

impl GeometrySystem {
    pub fn reserve(&mut self) -> GeometryHandle {
        let handle = GeometryHandle(self.next_handle);
        self.next_handle += 1;
        self.store.insert(handle, None);
        handle
    }

    pub fn clear_data(&mut self, handle: GeometryHandle) {
        if let Some(Some(data)) = self.store.get_mut(&handle) {
            data.vertices.clear();
            data.indices.clear();
        }
    }

    pub fn get<'a>(
        &'a self,
        handle: GeometryHandle,
        slice: &RenderBufferRange,
    ) -> Option<RenderBufferSlice<'a>> {
        let mesh = self.store.get(&handle)?;
        let mesh = mesh.as_ref()?;
        let view = RenderBufferSlice::new(mesh, slice);
        Some(view)
    }

    #[allow(unused)]
    pub fn set_data(
        &mut self,
        handle: GeometryHandle,
        builder: &mut dyn GeometryBuilder,
    ) -> RenderBufferRange {
        self.update_impl(handle, builder, true)
    }

    pub fn append_data(
        &mut self,
        handle: GeometryHandle,
        builder: &mut dyn GeometryBuilder,
    ) -> RenderBufferRange {
        self.update_impl(handle, builder, false)
    }

    pub fn sync(&mut self, handle: GeometryHandle) {
        if let Some(Some(entry)) = self.store.get_mut(&handle) {
            entry.sync(&self.device, &self.queue);
        }
    }

    fn update_impl(
        &mut self,
        handle: GeometryHandle,
        builder: &mut dyn GeometryBuilder,
        clear: bool,
    ) -> RenderBufferRange {
        // Get or insert a buffer for this handle
        let buffer = self
            .store
            .entry(handle)
            .or_insert_with(|| Some(RenderBuffer::new(&self.device)))
            .get_or_insert_with(|| RenderBuffer::new(&self.device));

        if clear {
            buffer.clear();
        }

        // Rest of implementation...
        let vertex_start = buffer.vertices.len();
        let index_start = buffer.indices.len();

        self.drawlist.clear();

        builder.build(&mut self.drawlist);
        buffer.append_from_mesh(&self.drawlist.mesh);

        let vertex_end = buffer.vertices.len();
        let index_end = buffer.indices.len();

        let v_size = std::mem::size_of::<Vertex>() as wgpu::BufferAddress;
        let i_size = std::mem::size_of::<u32>() as wgpu::BufferAddress;

        RenderBufferRange {
            vertex_slice: (vertex_start as wgpu::BufferAddress) * v_size
                ..((vertex_end as wgpu::BufferAddress) * v_size) as wgpu::BufferAddress,
            index_slice: (index_start as u64) * i_size..(index_end as u64) * i_size,
            vertex_count: vertex_end - vertex_start,
            index_count: index_end - index_start,
        }
    }
}

pub trait GeometryBuilder {
    fn build(&mut self, drawlist: &mut DrawList);
}
