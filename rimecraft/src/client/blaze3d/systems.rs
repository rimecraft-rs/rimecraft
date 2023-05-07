use glam::Vec3;
use std::{collections::VecDeque, rc::Rc};

pub struct RenderCall {
    executor: Box<dyn Fn()>,
}

impl RenderCall {
    pub fn new(executor: impl Fn() + 'static) -> Self {
        Self {
            executor: Box::new(executor),
        }
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

    pub fn get_last_processed_queue(&self) -> &VecDeque<RenderCall> {
        self.recording_queues
            .get(self.last_processed_index)
            .unwrap()
    }

    pub fn get_last_processed_queue_mut(&mut self) -> &mut VecDeque<RenderCall> {
        self.recording_queues
            .get_mut(self.last_processed_index)
            .unwrap()
    }

    pub fn get_recording_queue(&self) -> &VecDeque<RenderCall> {
        self.recording_queues.get(self.recording_index).unwrap()
    }

    pub fn get_recording_queue_mut(&mut self) -> &mut VecDeque<RenderCall> {
        self.recording_queues.get_mut(self.recording_index).unwrap()
    }

    pub fn get_processing_queue(&self) -> &VecDeque<RenderCall> {
        self.recording_queues.get(self.processing_index).unwrap()
    }

    pub fn get_processing_queue_mut(&mut self) -> &mut VecDeque<RenderCall> {
        self.recording_queues
            .get_mut(self.processing_index)
            .unwrap()
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

#[derive(Clone)]
pub struct VertexSorter {
    inner: Rc<Box<dyn Fn(Vec<Vec3>) -> Vec<usize>>>,
}

impl VertexSorter {
    pub fn by_distance_default() -> Self {
        Self::by_distance_xyz(0.0, 0.0, 0.0)
    }

    pub fn by_z_default() -> Self {
        Self::of(|vec| -vec.z)
    }

    pub fn new(f: impl Fn(Vec<Vec3>) -> Vec<usize> + 'static) -> Self {
        Self {
            inner: Rc::new(Box::new(f)),
        }
    }

    pub fn by_distance_xyz(origin_x: f32, origin_y: f32, origin_z: f32) -> Self {
        Self::by_distance(Vec3::new(origin_x, origin_y, origin_z))
    }

    pub fn by_distance(origin: Vec3) -> Self {
        Self::of(move |vec| origin.distance_squared(vec))
    }

    pub fn of(mapper: impl Fn(Vec3) -> f32 + 'static) -> Self {
        Self {
            inner: Rc::new(Box::new(move |vec| {
                let mut fs = Vec::with_capacity(vec.len());
                let mut us: Vec<usize> = Vec::with_capacity(vec.len());
                let mut ii: usize = 0;
                for i in vec {
                    fs.push(mapper(i));
                    us.push(ii);
                    ii += 1;
                }
                us.sort_by(|a, b| {
                    fs.get(*b)
                        .unwrap()
                        .partial_cmp(fs.get(*a).unwrap())
                        .unwrap()
                });
                us
            })),
        }
    }

    pub fn sort(&self, var1: Vec<Vec3>) -> Vec<usize> {
        (self.inner)(var1)
    }
}
