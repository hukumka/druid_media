use crate::{Controller, PipelineData};
use druid::Widget;
use gstreamer::{prelude::*, Pipeline, State};

pub struct AudioPlayer;

impl AudioPlayer{
    pub fn new(uri: &str) -> impl Widget<PipelineData> {
        let pipeline = gstreamer::ElementFactory::make("playbin", None).unwrap();
        let pipeline = pipeline.dynamic_cast::<Pipeline>().unwrap();
        pipeline.set_property("uri", &Some(uri)).unwrap();
        pipeline.set_state(State::Paused).unwrap();

        Controller::new(pipeline)
    }
}