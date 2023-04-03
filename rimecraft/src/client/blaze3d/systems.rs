use std::collections::VecDeque;

pub struct RenderCall {
    executor: Box<dyn Fn()>,
}

impl RenderCall {
    pub fn new(executor: Box<impl Fn() + 'static>) -> Self {
        Self { executor }
    }

    pub fn execute(&self) {
        (self.executor)()
    }
}

pub struct RenderCallStorage {
    recording_queues: Vec<VecDeque<RenderCall>>,
    recording: bool,
    recording_index: usize,
    processing: bool,
    processing_index: usize,
    last_processed_index: usize,
}

impl RenderCallStorage {
    pub fn can_record(&self) -> bool {
        !self.recording && self.recording_index == self.processing_index
    }

    pub fn start_recording(&mut self) -> bool {
        if !self.recording && self.can_record() {
            self.recording_index = (self.processing_index + 1) % self.recording_queues.len();
            self.recording = true;
            true
        } else {
            false
        }
    }

    pub fn record(&mut self, call: RenderCall) {
        if self.recording {
            self.get_recording_queue_mut().push_front(call)
        }
    }

    pub fn can_process(&self) -> bool {
        !self.processing && self.recording_index != self.processing_index
    }

    pub fn start_processing(&mut self) -> bool {
        if !self.processing && self.can_process() {
            self.processing = true;
            true
        } else {
            false
        }
    }

    pub fn stop_processing(&mut self) {
        if self.processing {
            self.processing = false;
            self.last_processed_index = self.processing_index;
            self.processing_index = self.recording_index;
        }
    }

    pub fn get_recording_queue(&self) -> &VecDeque<RenderCall> {
        self.recording_queues.get(self.recording_index).unwrap()
    }

    pub fn get_recording_queue_mut(&mut self) -> &mut VecDeque<RenderCall> {
        self.recording_queues.get_mut(self.recording_index).unwrap()
    }
}

impl Default for RenderCallStorage {
    fn default() -> Self {
        Self {
            recording_queues: vec![
                VecDeque::new(),
                VecDeque::new(),
                VecDeque::new(),
                VecDeque::new(),
            ],
            recording: Default::default(),
            recording_index: usize::default() + 1,
            processing: Default::default(),
            processing_index: usize::default() + 1,
            last_processed_index: Default::default(),
        }
    }
}
