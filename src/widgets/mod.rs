pub mod cpu_panel;
pub mod gpu_panel;
pub mod header;
pub mod mem_panel;
pub mod power_bar;

/// Ring buffer for sparkline history.
#[derive(Clone)]
pub struct History {
    data: Vec<u64>,
    capacity: usize,
}

impl History {
    pub fn new(capacity: usize) -> Self {
        Self {
            data: Vec::with_capacity(capacity),
            capacity,
        }
    }

    pub fn push(&mut self, val: u64) {
        if self.data.len() >= self.capacity {
            self.data.remove(0);
        }
        self.data.push(val);
    }

    pub fn data(&self) -> &[u64] {
        &self.data
    }
}
