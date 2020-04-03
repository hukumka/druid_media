use druid_media::{PipelineData, VideoPlayer};

use druid::{AppLauncher, Widget, WindowDesc};
use gstreamer;

const URI_EXAMPLES: [&'static str; 3] = [
    "https://matrix-client.matrix.org/_matrix/media/r0/download/matrix.org/IYfzAwoWvuyEnVTBvKZMcOCh",
    "file:///F:/Downloads/Pictures/2019-12/DASH_720.mp4",
    "https://www.freedesktop.org/software/gstreamer-sdk/data/media/sintel_cropped_multilingual.webm",
];

fn main() {
    gstreamer::init().unwrap();
    let main_window = WindowDesc::new(ui_builder);
    let data = PipelineData::new();
    AppLauncher::with_window(main_window)
        .use_simple_logger()
        .launch(data)
        .expect("launch failed");
}

fn ui_builder() -> impl Widget<PipelineData> {
    VideoPlayer::new(URI_EXAMPLES[0])
}