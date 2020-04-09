use druid_media::{AudioPlayer, PipelineData};

use druid::{AppLauncher, Widget, WindowDesc};
use gstreamer;

const URI_EXAMPLES: [&'static str; 1] = ["file:///F:/Music/benn_beauty_of_annihilation_2017.mp3"];

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
    AudioPlayer::build_widget(URI_EXAMPLES[0]).unwrap()
}
