use wgpu::{util::DeviceExt, BufferSlice, Device, Queue};

pub struct BufferPool {
    pool: Vec<wgpu::Buffer>,
    current: usize,
}
impl BufferPool {
    pub fn new(device: &Device, size: usize, buffer_size: usize) -> Self {
        Self {
            pool: (0..size)
                .map(|i| {
                    device.create_buffer(&wgpu::BufferDescriptor {
                        label: Some(&format!("Buffer {}", i)),
                        size: buffer_size as u64,
                        usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                        mapped_at_creation: false,
                    })
                })
                .collect(),
            current: 0,
        }
    }
    pub fn new_session(&mut self) {
        self.current = 0
    }
    pub fn use_buffer(&mut self, device: &Device, queue: &Queue, data: &[u8]) -> BufferSlice<'_> {
        if self.current >= self.pool.len() {
            self.pool.push(
                device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some(&format!("Buffer {}", self.current)),
                    contents: data,
                    usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                }),
            );
        } else if self.pool[self.current].size() < data.len() as u64 {
            self.pool[self.current] =
                device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some(&format!("Buffer {}", self.current)),
                    contents: data,
                    usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                });
        } else {
            queue.write_buffer(&self.pool[self.current], 0, data);
        }
        let slice = self.pool[self.current].slice(..);
        self.current += 1;
        slice
    }
}
