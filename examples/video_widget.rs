use druid_media::{VideoPlayer, PlayerData};

use druid::widget::{Align, Button, Flex, Label, Padding, WidgetExt};
use druid::{AppLauncher, LocalizedString, Widget, WindowDesc};
use gstreamer;

fn main() {
    gstreamer::init().unwrap();
    let main_window = WindowDesc::new(ui_builder);
    let data = PlayerData::Paused;
    AppLauncher::with_window(main_window)
        .use_simple_logger()
        .launch(data)
        .expect("launch failed");
}

fn ui_builder() -> impl Widget<PlayerData> {
    // The label text will be computed dynamically based on the current locale and count
    let button = Button::new("increment", |_ctx, data, _env| {
        if data == &PlayerData::Playing{
            *data = PlayerData::Paused;
        }else{
            *data = PlayerData::Playing;
        }
    })
        .padding(5.0);
    let player = VideoPlayer::new("https://www.freedesktop.org/software/gstreamer-sdk/data/media/sintel_trailer-480p.webm");

    Flex::column()
    .with_child(player, 1.0)
    .with_child(button, 1.0)
}