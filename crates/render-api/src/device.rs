// SPDX-License-Identifier: GPL-3.0-or-later
use crate::capabilities::{BufferUsage, DeviceLimits, RenderFeature, SurfaceFormat};
use topovium_foundation::StableHandle;

/// Marker type for buffer handles.
#[derive(Debug)]
pub enum BufferMarker {}
/// Marker type for texture handles.
#[derive(Debug)]
pub enum TextureMarker {}

/// A GPU buffer, opaque to everything above this crate.
pub type BufferId = StableHandle<BufferMarker>;
/// A GPU texture, opaque to everything above this crate.
pub type TextureId = StableHandle<TextureMarker>;

/// Why a rendering operation failed.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum RenderError {
    /// No adapter met the baseline. The message says which limit failed, because
    /// "graphics initialisation failed" tells a user nothing actionable.
    #[error("no suitable graphics adapter: {reason}")]
    NoAdapter { reason: String },
    #[error("device does not support required feature {0:?}")]
    MissingFeature(RenderFeature),
    #[error("requested {requested} bytes exceeds the device limit of {limit}")]
    AllocationTooLarge { requested: u64, limit: u64 },
    /// Allocation failed. Distinct from `AllocationTooLarge`: this one is recoverable
    /// by evicting caches and retrying, which is what the residency manager does
    /// before it ever reports a failure to the user.
    #[error("out of device memory allocating {requested} bytes")]
    OutOfMemory { requested: u64 },
    #[error("write of {size} bytes at offset {offset} exceeds buffer size {buffer_size}")]
    WriteOutOfBounds {
        offset: u64,
        size: u64,
        buffer_size: u64,
    },
    /// The surface was lost, typically by the window closing or the app backgrounding.
    /// Recoverable: recreate the surface and continue.
    #[error("surface lost and must be recreated")]
    SurfaceLost,
    #[error("{0}")]
    Backend(String),
}

impl RenderError {
    /// Whether retrying after freeing resources or recreating the surface may succeed.
    #[must_use]
    pub const fn is_recoverable(&self) -> bool {
        matches!(self, Self::OutOfMemory { .. } | Self::SurfaceLost)
    }
}

/// A contiguous update to part of a buffer.
///
/// Ranges, not whole buffers: uploading a whole object table because one transform
/// changed is the exact cost this architecture exists to avoid.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BufferWrite<'a> {
    pub buffer: BufferId,
    pub offset: u64,
    pub data: &'a [u8],
}

/// What a graphics backend must provide.
///
/// Implemented by `topovium-wgpu-common` today. A native backend implements the same
/// trait, and nothing above this crate can tell the difference.
pub trait RenderDevice: std::fmt::Debug {
    /// Hard limits of this device.
    fn limits(&self) -> DeviceLimits;

    /// Whether an optional feature is available. Callers must have a fallback.
    fn supports(&self, feature: RenderFeature) -> bool;

    /// Format of the presented surface.
    fn surface_format(&self) -> SurfaceFormat;

    /// Allocates a buffer.
    ///
    /// # Errors
    /// Fails if the size exceeds device limits or the allocation cannot be satisfied.
    fn create_buffer(
        &mut self,
        label: &str,
        size: u64,
        usage: BufferUsage,
    ) -> Result<BufferId, RenderError>;

    /// Applies a batch of buffer writes.
    ///
    /// Takes a slice rather than one write per call so the backend can coalesce them
    /// into a single staging copy. Per-write submission is the thing that makes delta
    /// updates slower than the full upload they replaced.
    ///
    /// # Errors
    /// Fails if any write falls outside its buffer.
    fn write_buffers(&mut self, writes: &[BufferWrite<'_>]) -> Result<(), RenderError>;

    /// Frees a buffer.
    ///
    /// # Errors
    /// Fails if the handle is stale or already destroyed.
    fn destroy_buffer(&mut self, buffer: BufferId) -> Result<(), RenderError>;

    /// Total bytes currently allocated through this device, for the memory budget.
    fn allocated_bytes(&self) -> u64;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn out_of_memory_is_recoverable_but_a_too_large_request_is_not() {
        // The residency manager retries the first after evicting; the second is a bug
        // in the caller and retrying would loop forever.
        assert!(RenderError::OutOfMemory { requested: 1024 }.is_recoverable());
        assert!(RenderError::SurfaceLost.is_recoverable());
        assert!(
            !RenderError::AllocationTooLarge {
                requested: 1 << 40,
                limit: 1 << 28
            }
            .is_recoverable()
        );
    }

    #[test]
    fn errors_say_what_failed_and_by_how_much() {
        let error = RenderError::AllocationTooLarge {
            requested: 5000,
            limit: 4096,
        };
        let message = error.to_string();
        assert!(message.contains("5000"));
        assert!(message.contains("4096"));
    }
}
