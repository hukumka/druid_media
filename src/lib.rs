mod audio;
mod controller;
mod pipeline_data;
mod video;

pub use audio::AudioPlayer;
pub(crate) use controller::Controller;
pub use pipeline_data::{PipelineData, PipelineState, Timeline};
pub use video::VideoPlayer;

pub use gstreamer;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum PipelineCreationError {
    #[error("{0}")]
    Glib(#[from] gstreamer::glib::BoolError),
    #[error("Failed to change pipeline state")]
    StateChange(#[from] gstreamer::StateChangeError),
}
