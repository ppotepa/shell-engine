//! Memory pool for buffer reuse — reduces per-frame allocations during rendering.
//!
//! This module pre-allocates scratch buffers at startup and reuses them frame-to-frame,
//! reducing GC pressure and allocation overhead. Common scratch buffers include:
//! - Layer temporary buffers (for effect composition)
//! - Sprite scratch space (for intermediate rasterization)
//! - Halfblock virtual buffers (for 2× scale rendering)

use engine_core::buffer::Buffer;
use std::cell::RefCell;
use std::sync::Arc;

/// Configuration for buffer pool sizing.
#[derive(Debug, Clone, Copy)]
pub struct BufferPoolConfig {
    /// Maximum width for pooled buffers
    pub max_width: u16,
    /// Maximum height for pooled buffers
    pub max_height: u16,
    /// Number of buffers to pre-allocate
    pub pool_size: usize,
}

impl Default for BufferPoolConfig {
    fn default() -> Self {
        // Default pool: 512×256 cells max, 8 buffers
        // Total memory: ~8 × 512 × 256 × 6 bytes (Cell size) ≈ 6 MB
        Self {
            max_width: 512,
            max_height: 256,
            pool_size: 8,
        }
    }
}

/// Thread-local buffer pool for efficient buffer reuse.
pub struct BufferPool {
    /// Stack of available buffers
    available: RefCell<Vec<Buffer>>,
    /// Configuration
    config: BufferPoolConfig,
}

impl BufferPool {
    /// Create a new buffer pool with the given configuration.
    pub fn new(config: BufferPoolConfig) -> Self {
        let mut available = Vec::with_capacity(config.pool_size);
        // Pre-allocate buffers at maximum size
        for _ in 0..config.pool_size {
            available.push(Buffer::new(config.max_width, config.max_height));
        }
        Self {
            available: RefCell::new(available),
            config,
        }
    }

    /// Acquire a buffer from the pool, resized to (width, height).
    /// If no buffer is available, allocates a new one.
    pub fn acquire(&self, width: u16, height: u16) -> PooledBuffer {
        let mut available = self.available.borrow_mut();
        let mut buf = if let Some(mut b) = available.pop() {
            b.resize(width.min(self.config.max_width), height.min(self.config.max_height));
            b
        } else {
            Buffer::new(
                width.min(self.config.max_width),
                height.min(self.config.max_height),
            )
        };
        buf.fill(engine_core::color::Color::Reset);
        PooledBuffer {
            buffer: Some(buf),
            pool: Arc::new(self as *const _ as usize),
        }
    }

    /// Return a buffer to the pool.
    fn release(&self, buf: Buffer) {
        let mut available = self.available.borrow_mut();
        if available.len() < self.config.pool_size {
            available.push(buf);
        }
    }

    /// Return statistics about pool usage.
    pub fn stats(&self) -> PoolStats {
        let available = self.available.borrow();
        PoolStats {
            available_count: available.len(),
            pool_size: self.config.pool_size,
            max_buffer_cells: (self.config.max_width as usize) * (self.config.max_height as usize),
        }
    }
}

/// Statistics about buffer pool usage.
#[derive(Debug, Clone, Copy)]
pub struct PoolStats {
    pub available_count: usize,
    pub pool_size: usize,
    pub max_buffer_cells: usize,
}

/// RAII guard for pooled buffers — automatically returns buffer to pool on drop.
pub struct PooledBuffer {
    buffer: Option<Buffer>,
    pool: Arc<usize>,
}

impl PooledBuffer {
    /// Get mutable reference to the buffered buffer.
    pub fn as_mut(&mut self) -> &mut Buffer {
        self.buffer.as_mut().expect("pooled buffer")
    }

    /// Get immutable reference to the buffered buffer.
    pub fn as_ref(&self) -> &Buffer {
        self.buffer.as_ref().expect("pooled buffer")
    }

    /// Extract the buffer from the pool guard without returning it.
    /// **Warning**: Manually manage buffer lifetime after this call.
    pub fn take(mut self) -> Buffer {
        self.buffer.take().expect("pooled buffer")
    }
}

impl std::ops::Deref for PooledBuffer {
    type Target = Buffer;
    fn deref(&self) -> &Self::Target {
        self.as_ref()
    }
}

impl std::ops::DerefMut for PooledBuffer {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.as_mut()
    }
}

impl Drop for PooledBuffer {
    fn drop(&mut self) {
        if let Some(buf) = self.buffer.take() {
            // Unsafely recover the pool pointer to return the buffer
            // This is safe because the pool outlives the PooledBuffer
            unsafe {
                let pool_ptr = *self.pool as *const BufferPool;
                if !pool_ptr.is_null() {
                    (*pool_ptr).release(buf);
                }
            }
        }
    }
}

thread_local! {
    /// Global thread-local buffer pool for compositor use.
    static BUFFER_POOL: BufferPool = BufferPool::new(BufferPoolConfig::default());
}

/// Acquire a buffer from the thread-local pool.
pub fn acquire_buffer(width: u16, height: u16) -> PooledBuffer {
    BUFFER_POOL.with(|pool| pool.acquire(width, height))
}

/// Query thread-local pool statistics.
pub fn pool_stats() -> PoolStats {
    BUFFER_POOL.with(|pool| pool.stats())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn buffer_pool_creates_initial_buffers() {
        let config = BufferPoolConfig {
            max_width: 100,
            max_height: 50,
            pool_size: 4,
        };
        let pool = BufferPool::new(config);
        let stats = pool.stats();
        assert_eq!(stats.available_count, 4);
        assert_eq!(stats.pool_size, 4);
    }

    #[test]
    fn acquire_reduces_available_count() {
        let config = BufferPoolConfig {
            max_width: 100,
            max_height: 50,
            pool_size: 4,
        };
        let pool = BufferPool::new(config);
        let _buf = pool.acquire(50, 50);
        let stats = pool.stats();
        assert_eq!(stats.available_count, 3);
    }

    #[test]
    fn buffer_reuse_on_drop() {
        let config = BufferPoolConfig {
            max_width: 100,
            max_height: 50,
            pool_size: 4,
        };
        let pool = BufferPool::new(config);
        {
            let _buf = pool.acquire(50, 50);
            let stats = pool.stats();
            assert_eq!(stats.available_count, 3);
        }
        let stats = pool.stats();
        assert_eq!(stats.available_count, 4);
    }

    #[test]
    fn pooled_buffer_derefs_to_buffer() {
        let config = BufferPoolConfig {
            max_width: 100,
            max_height: 50,
            pool_size: 2,
        };
        let pool = BufferPool::new(config);
        let mut buf = pool.acquire(50, 50);
        buf.fill(engine_core::color::Color::White);
        let cell = buf.get(0, 0);
        assert!(cell.is_some());
    }
}
