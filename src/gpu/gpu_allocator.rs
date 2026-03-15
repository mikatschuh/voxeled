use std::collections::HashMap;

use wgpu::{Device, Queue};

pub const MAX_BUFFER_SIZE: usize = 268_435_456;

/// (size_class_id, buffer_id, slot_id)
pub type SlotID = (usize, usize, usize);

pub struct GPUSlotAllocatorStats {
    pub reserved_bytes: usize,
    pub payload_bytes: usize,
    pub allocated_slots: usize,
    pub free_slots: usize,
    pub size_classes: usize,
}

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
    slot_usage: HashMap<SlotID, usize>,
}

impl GPUSlotAllocator {
    pub fn new(slot_size: usize, cap: usize) -> Self {
        let min_slot_size = slot_size.max(1).next_power_of_two();
        let mut allocator = Self {
            min_slot_size,
            default_cap: cap,
            size_class_lookup: HashMap::new(),
            size_classes: Vec::new(),
            slot_usage: HashMap::with_capacity(cap),
        };

        allocator.ensure_size_class(min_slot_size);
        allocator
    }

    pub fn reserved_size(&self) -> usize {
        self.size_classes
            .iter()
            .map(|class| {
                class
                    .buffer_pool
                    .iter()
                    .map(|buf| buf.size() as usize)
                    .sum::<usize>()
            })
            .sum()
    }

    pub fn stats(&self) -> GPUSlotAllocatorStats {
        let reserved_bytes = self.reserved_size();
        let payload_bytes = self.slot_usage.values().sum();
        let free_slots = self
            .size_classes
            .iter()
            .map(|class| class.free_slots.len())
            .sum();

        GPUSlotAllocatorStats {
            reserved_bytes,
            payload_bytes,
            allocated_slots: self.slot_usage.len(),
            free_slots,
            size_classes: self.size_classes.len(),
        }
    }

    pub fn deallocate_slot(&mut self, slot_id: SlotID) {
        self.slot_usage.remove(&slot_id);
        self.size_classes[slot_id.0]
            .free_slots
            .push((slot_id.1, slot_id.2));
    }

    /// Writes `data` into `slot_id`.
    /// If the slot is too small, data is moved to a new fitting slot and the old slot is freed.
    /// Returns the slot that now owns the data.
    pub fn write_slot(
        &mut self,
        device: &Device,
        queue: &Queue,
        slot_id: SlotID,
        data: &[u8],
    ) -> SlotID {
        let current_slot_size = self.slot_size(slot_id);
        if data.len() > current_slot_size {
            let new_slot = self.allocate_slot(device, data.len());
            self.write_to_slot(queue, new_slot, data);
            self.slot_usage.insert(new_slot, data.len());
            self.deallocate_slot(slot_id);
            return new_slot;
        }

        self.write_to_slot(queue, slot_id, data);
        self.slot_usage.insert(slot_id, data.len());
        slot_id
    }

    /// Allocates a slot for at least `required_size` bytes.
    /// Slot sizes are grouped into power-of-two classes.
    pub fn allocate_slot(&mut self, device: &Device, required_size: usize) -> SlotID {
        let slot_size = self.slot_size_for(required_size);
        let class_id = self.ensure_size_class(slot_size);
        let class = &mut self.size_classes[class_id];

        if let Some((buffer_id, slot_id)) = class.free_slots.pop() {
            let slot = (class_id, buffer_id, slot_id);
            self.slot_usage.insert(slot, 0);
            return slot;
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

        let slot = (class_id, new_buffer_id, 0);
        self.slot_usage.insert(slot, 0);
        slot
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

    fn ensure_size_class(&mut self, slot_size: usize) -> usize {
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

        let size_class = SizeClass {
            slot_size,
            slots_per_buffer,
            buffer_pool: Vec::with_capacity(self.default_cap.div_ceil(slots_per_buffer).max(1)),
            free_slots: Vec::with_capacity(slots_per_buffer.min(self.default_cap.max(1))),
        };

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

    fn write_to_slot(&self, queue: &Queue, slot_id: SlotID, data: &[u8]) {
        let class = &self.size_classes[slot_id.0];
        debug_assert!(data.len() <= class.slot_size);

        queue.write_buffer(
            &class.buffer_pool[slot_id.1],
            (slot_id.2 * class.slot_size) as u64,
            data,
        );
    }
}
