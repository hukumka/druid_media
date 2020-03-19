use druid::{Data, Lens};
use gstreamer::{ClockTime, Pipeline, prelude::*};

#[derive(Data, Copy, Clone, PartialEq, Eq, Debug, Lens)]
pub struct PipelineData{
    pub(crate) state: PipelineState,
    pub(crate) timeline: Timeline,
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub struct Timeline{
    pub(crate) duration: ClockTime,
    pub(crate) position: ClockTime,
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum PipelineState{
    Play,
    Pause,
}

impl PipelineData{
    pub fn new() -> Self{
        PipelineData{
            state: PipelineState::Pause,
            timeline: Timeline{
                duration: ClockTime::none(),
                position: ClockTime::none(),
            }
        }
    }

}

impl Timeline{
    pub(crate) fn new() -> Self{
        Self{
            duration: ClockTime::none(),
            position: ClockTime::none(),
        }
    }

    pub(crate) fn query(pipeline: &Pipeline) -> Self{
        Self{
            duration: pipeline.query_duration().unwrap_or(ClockTime::none()),
            position: pipeline.query_position().unwrap_or(ClockTime::none()),
        }
    }
}

impl Data for PipelineState{
    fn same(&self, other: &Self) -> bool {
        self == other
    }
}

impl Data for Timeline{
    fn same(&self, other: &Self) -> bool {
        self == other
    }
}

pub struct TimelineFractionLens;

impl Lens<Timeline, f64> for TimelineFractionLens {
    fn with<V, F: FnOnce(&f64) -> V>(&self, data: &Timeline, f: F) -> V {
        let pos = data.position.nanoseconds().unwrap_or(0) as f64;
        let dur = data.duration.nanoseconds().unwrap_or(1) as f64;
        f(&(pos / dur))
    }

    fn with_mut<V, F: FnOnce(&mut f64) -> V>(&self, data: &mut Timeline, f: F) -> V {
        let pos = data.position.nanoseconds().unwrap_or(0) as f64;
        let dur = data.duration.nanoseconds().unwrap_or(1) as f64;
        let mut frac = pos / dur;
        let res = f(&mut frac);
        data.position = ClockTime::from_nseconds((frac * dur) as u64);
        res
    }
}
