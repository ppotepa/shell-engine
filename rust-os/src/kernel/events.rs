use std::cmp::Ordering;
use std::collections::BinaryHeap;

pub struct ScheduledEvent {
    pub due_at_ms: u64,
    pub sequence: u64,
    pub action: Box<dyn FnOnce() + Send>,
    pub tag: Option<String>,
}

impl PartialEq for ScheduledEvent {
    fn eq(&self, other: &Self) -> bool {
        self.due_at_ms == other.due_at_ms && self.sequence == other.sequence
    }
}
impl Eq for ScheduledEvent {}

impl PartialOrd for ScheduledEvent {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for ScheduledEvent {
    fn cmp(&self, other: &Self) -> Ordering {
        // Min-heap: earlier events have higher priority
        other
            .due_at_ms
            .cmp(&self.due_at_ms)
            .then_with(|| other.sequence.cmp(&self.sequence))
    }
}

pub struct KernelEventQueue {
    queue: BinaryHeap<ScheduledEvent>,
    seq: u64,
}

impl KernelEventQueue {
    pub fn new() -> Self {
        Self {
            queue: BinaryHeap::new(),
            seq: 0,
        }
    }

    pub fn schedule_at(
        &mut self,
        due_at_ms: u64,
        action: impl FnOnce() + Send + 'static,
        tag: Option<&str>,
    ) {
        let seq = self.seq;
        self.seq += 1;
        self.queue.push(ScheduledEvent {
            due_at_ms,
            sequence: seq,
            action: Box::new(action),
            tag: tag.map(|s| s.to_string()),
        });
    }

    pub fn schedule_after(
        &mut self,
        now_ms: u64,
        delay_ms: u64,
        action: impl FnOnce() + Send + 'static,
        tag: Option<&str>,
    ) {
        self.schedule_at(now_ms + delay_ms, action, tag);
    }

    pub fn drain_ready(&mut self, now_ms: u64) -> Vec<ScheduledEvent> {
        let mut ready = Vec::new();
        while let Some(ev) = self.queue.peek() {
            if ev.due_at_ms <= now_ms {
                ready.push(self.queue.pop().unwrap());
            } else {
                break;
            }
        }
        ready
    }
}
