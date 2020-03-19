use druid_media::{VideoPlayer, PipelineData};

use druid::widget::{Align, Button, Flex, Label, Padding, WidgetExt};
use druid::{AppLauncher, LocalizedString, Widget, WindowDesc};
use gstreamer;

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
    VideoPlayer::new("file:///F:/Movies/Frozen II.mkv")
}