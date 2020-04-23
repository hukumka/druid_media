use crate::PipelineCreationError;
use crate::{Controller, PipelineData};
use druid::Widget;
use gstreamer::{prelude::*, Pipeline};

pub struct AudioPlayer;

impl AudioPlayer {
    /// Create new audio player widget
    pub fn build_widget(uri: &str) -> Result<impl Widget<PipelineData>, PipelineCreationError> {
        let pipeline = gstreamer::ElementFactory::make("playbin", None)?;
        let pipeline = pipeline
            .dynamic_cast::<Pipeline>()
            .expect("Created element should always be a pipeline.");
        pipeline.set_property("uri", &Some(uri))?;

        Ok(Controller::build_widget(pipeline))
    }
}