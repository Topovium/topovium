// SPDX-License-Identifier: GPL-3.0-or-later
use crate::capability::{map_features, map_limits};
use std::collections::HashMap;
use topovium_foundation::{Arena, StableHandle};
use topovium_render_api::{
    BufferId, BufferUsage, BufferWrite, DeviceLimits, RenderDevice, RenderError, RenderFeature,
    SurfaceFormat,
};

/// A buffer allocated through this device.
#[derive(Debug)]
struct TrackedBuffer {
    buffer: wgpu::Buffer,
    size: u64,
}

/// [`RenderDevice`] backed by `wgpu`.
///
/// Owns the buffers it hands out and returns opaque handles, so nothing above
/// `render-api` can hold a `wgpu::Buffer` even accidentally.
#[derive(Debug)]
pub struct WgpuDevice {
    device: wgpu::Device,
    queue: wgpu::Queue,
    features: wgpu::Features,
    limits: DeviceLimits,
    surface_format: SurfaceFormat,
    buffers: Arena<TrackedBuffer>,
    /// Kept separately so `allocated_bytes` is O(1): the memory HUD reads it every
    /// frame and must not walk the arena to do so.
    allocated_bytes: u64,
    labels: HashMap<u32, String>,
}

impl WgpuDevice {
    /// Wraps an already-created `wgpu` device and queue.
    ///
    /// Adapter selection lives in the platform shell, which knows about surfaces and
    /// windows; this type only needs the result.
    #[must_use]
    pub fn new(
        device: wgpu::Device,
        queue: wgpu::Queue,
        features: wgpu::Features,
        wgpu_limits: &wgpu::Limits,
        surface_format: SurfaceFormat,
    ) -> Self {
        Self {
            device,
            queue,
            features,
            limits: map_limits(wgpu_limits),
            surface_format,
            buffers: Arena::new(),
            allocated_bytes: 0,
            labels: HashMap::new(),
        }
    }

    /// The underlying `wgpu` device.
    ///
    /// Available only inside this crate's own module tree and to backend-internal code
    /// during migration; it is not re-exported through `render-api`.
    #[must_use]
    pub const fn raw_device(&self) -> &wgpu::Device {
        &self.device
    }

    /// The underlying `wgpu` queue.
    #[must_use]
    pub const fn raw_queue(&self) -> &wgpu::Queue {
        &self.queue
    }

    fn usage_flags(usage: BufferUsage) -> wgpu::BufferUsages {
        let base = wgpu::BufferUsages::COPY_DST;
        base | match usage {
            BufferUsage::Uniform => wgpu::BufferUsages::UNIFORM,
            BufferUsage::Storage => wgpu::BufferUsages::STORAGE,
            BufferUsage::Vertex => wgpu::BufferUsages::VERTEX,
            BufferUsage::Index => wgpu::BufferUsages::INDEX,
            BufferUsage::Indirect => wgpu::BufferUsages::INDIRECT,
        }
    }
}

impl RenderDevice for WgpuDevice {
    fn limits(&self) -> DeviceLimits {
        self.limits
    }

    fn supports(&self, feature: RenderFeature) -> bool {
        map_features(self.features, feature)
    }

    fn surface_format(&self) -> SurfaceFormat {
        self.surface_format
    }

    fn create_buffer(
        &mut self,
        label: &str,
        size: u64,
        usage: BufferUsage,
    ) -> Result<BufferId, RenderError> {
        if size > self.limits.max_buffer_size {
            return Err(RenderError::AllocationTooLarge {
                requested: size,
                limit: self.limits.max_buffer_size,
            });
        }

        let buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some(label),
            size,
            usage: Self::usage_flags(usage),
            mapped_at_creation: false,
        });

        let handle = self
            .buffers
            .insert(TrackedBuffer { buffer, size })
            .map_err(|error| RenderError::Backend(error.to_string()))?;

        self.allocated_bytes = self.allocated_bytes.saturating_add(size);
        self.labels.insert(handle.index(), label.to_owned());
        Ok(StableHandle::new(handle.index(), handle.generation()))
    }

    fn write_buffers(&mut self, writes: &[BufferWrite<'_>]) -> Result<(), RenderError> {
        // Validate every write before performing any of them. A partially applied
        // batch would leave the GPU scene describing a state that never existed.
        for write in writes {
            let handle = StableHandle::new(write.buffer.index(), write.buffer.generation());
            let tracked = self
                .buffers
                .get(handle)
                .map_err(|error| RenderError::Backend(error.to_string()))?;
            let end = write.offset.saturating_add(write.data.len() as u64);
            if end > tracked.size {
                return Err(RenderError::WriteOutOfBounds {
                    offset: write.offset,
                    size: write.data.len() as u64,
                    buffer_size: tracked.size,
                });
            }
        }

        for write in writes {
            let handle = StableHandle::new(write.buffer.index(), write.buffer.generation());
            if let Ok(tracked) = self.buffers.get(handle) {
                self.queue
                    .write_buffer(&tracked.buffer, write.offset, write.data);
            }
        }
        Ok(())
    }

    fn destroy_buffer(&mut self, buffer: BufferId) -> Result<(), RenderError> {
        let handle = StableHandle::new(buffer.index(), buffer.generation());
        let tracked = self
            .buffers
            .remove(handle)
            .map_err(|error| RenderError::Backend(error.to_string()))?;
        self.allocated_bytes = self.allocated_bytes.saturating_sub(tracked.size);
        self.labels.remove(&buffer.index());
        tracked.buffer.destroy();
        Ok(())
    }

    fn allocated_bytes(&self) -> u64 {
        self.allocated_bytes
    }
}
