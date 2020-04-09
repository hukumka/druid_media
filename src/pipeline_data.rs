use druid::{Data, Lens};

#[derive(Data, Copy, Clone, PartialEq, Debug, Lens)]
pub struct PipelineData {
    pub(crate) state: PipelineState,
    pub(crate) timeline: Timeline,
    pub(crate) volume: f64,
    pub(crate) muted: bool,
}

#[derive(Lens, Copy, Clone, PartialEq, Debug)]
pub struct Timeline {
    pub frac: f64,
    pub duration: f64,
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum PipelineState {
    Play,
    Pause,
}

impl PipelineData {
    pub fn new() -> Self {
        PipelineData {
            state: PipelineState::Pause,
            timeline: Timeline::new(),
            volume: 1.0,
            muted: false,
        }
    }
}

impl Default for PipelineData {
    fn default() -> Self {
        PipelineData::new()
    }
}

impl Timeline {
    pub fn new() -> Self {
        Self {
            frac: 0.0,
            duration: 0.0,
        }
    }
}
impl Default for Timeline {
    fn default() -> Self {
        Self::new()
    }
}

impl Data for PipelineState {
    fn same(&self, other: &Self) -> bool {
        self == other
    }
}

impl Data for Timeline {
    fn same(&self, other: &Self) -> bool {
        self == other
    }
}
