use std::collections::HashMap;

use wgpu::{Device, Queue};

pub const MAX_BUFFER_SIZE: usize = 268_435_456;

/// (size_class_id, buffer_id, slot_id)
pub type SlotID = (usize, usize, usize);

struct SizeClass {
    slot_size: usize,
    slots_per_buffer: usize,
    buffer_pool: Vec<wgpu::Buffer>,
    free_slots: Vec<(usize, usize)>,
}

pub struct GPUSlotAllocator {
    min_slot_size: usize,
    default_cap: usize,
    size_class_lookup: HashMap<usize, usize>,
    size_classes: Vec<SizeClass>,
}

impl GPUSlotAllocator {
    pub fn new(device: &Device, slot_size: usize, cap: usize) -> Self {
        let min_slot_size = slot_size.max(1).next_power_of_two();
        let mut allocator = Self {
            min_slot_size,
            default_cap: cap,
            size_class_lookup: HashMap::new(),
            size_classes: Vec::new(),
        };

        allocator.ensure_size_class(device, min_slot_size);
        allocator
    }

    pub fn deallocate_slot(&mut self, slot_id: SlotID) {
        self.size_classes[slot_id.0]
            .free_slots
            .push((slot_id.1, slot_id.2));
    }

    pub fn write_slot(&mut self, queue: &Queue, slot_id: SlotID, data: &[u8]) {
        let class = &self.size_classes[slot_id.0];

        assert!(
            data.len() <= class.slot_size,
            "slot overflow: data size {} > slot size {}",
            data.len(),
            class.slot_size
        );

        queue.write_buffer(
            &class.buffer_pool[slot_id.1],
            (slot_id.2 * class.slot_size) as u64,
            data,
        );
    }

    /// Allocates a slot for at least `required_size` bytes.
    /// Slot sizes are grouped into power-of-two classes.
    pub fn allocate_slot(&mut self, device: &Device, required_size: usize) -> SlotID {
        let slot_size = self.slot_size_for(required_size);
        let class_id = self.ensure_size_class(device, slot_size);
        let class = &mut self.size_classes[class_id];

        if let Some((buffer_id, slot_id)) = class.free_slots.pop() {
            return (class_id, buffer_id, slot_id);
        }

        class.buffer_pool.push(Self::create_buffer(
            device,
            slot_size,
            class.buffer_pool.len(),
        ));

        let new_buffer_id = class.buffer_pool.len() - 1;
        for s in 1..class.slots_per_buffer {
            class.free_slots.push((new_buffer_id, s));
        }

        (class_id, new_buffer_id, 0)
    }

    pub fn slot_size(&self, slot_id: SlotID) -> usize {
        self.size_classes[slot_id.0].slot_size
    }

    pub fn buffer_and_offset(&self, slot_id: SlotID) -> (&wgpu::Buffer, u64) {
        let class = &self.size_classes[slot_id.0];
        (
            &class.buffer_pool[slot_id.1],
            (slot_id.2 * class.slot_size) as u64,
        )
    }

    fn slot_size_for(&self, required_size: usize) -> usize {
        let requested = required_size.max(1).next_power_of_two();
        requested.max(self.min_slot_size)
    }

    fn ensure_size_class(&mut self, device: &Device, slot_size: usize) -> usize {
        if let Some(&class_id) = self.size_class_lookup.get(&slot_size) {
            return class_id;
        }

        assert!(
            slot_size <= MAX_BUFFER_SIZE,
            "slot size {} exceeds max buffer size {}",
            slot_size,
            MAX_BUFFER_SIZE
        );

        let slots_per_buffer = (MAX_BUFFER_SIZE / slot_size).max(1);
        let num_of_bufs = self.default_cap.div_ceil(slots_per_buffer).max(1);

        let mut size_class = SizeClass {
            slot_size,
            slots_per_buffer,
            buffer_pool: Vec::with_capacity(num_of_bufs),
            free_slots: Vec::with_capacity(num_of_bufs * slots_per_buffer),
        };

        for b in 0..num_of_bufs {
            size_class
                .buffer_pool
                .push(Self::create_buffer(device, slot_size, b));
            for s in 0..slots_per_buffer {
                size_class.free_slots.push((b, s));
            }
        }

        let class_id = self.size_classes.len();
        self.size_classes.push(size_class);
        self.size_class_lookup.insert(slot_size, class_id);
        class_id
    }

    fn create_buffer(device: &Device, slot_size: usize, buffer_id: usize) -> wgpu::Buffer {
        device.create_buffer(&wgpu::BufferDescriptor {
            label: Some(&format!("slot-size={}, buffer-id={}", slot_size, buffer_id)),
            size: MAX_BUFFER_SIZE as u64,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        })
    }
}
