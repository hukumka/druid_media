use druid::{
    Widget, Data, Event, LifeCycle,
    UpdateCtx, EventCtx, PaintCtx, LayoutCtx, LifeCycleCtx, Env,
    Size, BoxConstraints, Rect, Point,
    piet::{RenderContext, ImageFormat, InterpolationMode},
    Selector,
};
use std::time::{Duration, Instant};
use gstreamer::prelude::*;
use gstreamer_app::AppSink;
use gstreamer_video::prelude::*;

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum PlayerData{
    Paused,
    Playing
}

pub struct VideoPlayer{
    pipeline: gstreamer::Pipeline,
    appsink: AppSink,
    last_sample: Option<gstreamer::Sample>,
}

impl Data for PlayerData{
    fn same(&self, other: &Self) -> bool {
        self == other
    }
}

impl VideoPlayer{
    pub fn new(uri: &'static str) -> VideoPlayer{
        let pipeline = gstreamer::ElementFactory::make("playbin", None).unwrap();
        let appsink = gstreamer::ElementFactory::make("appsink", Some("sink")).unwrap();
        let appsink = appsink.dynamic_cast::<AppSink>().unwrap();
        let caps = gstreamer::Caps::builder("video/x-raw")
                .field("format", &gstreamer_video::VideoFormat::Rgba.to_string())
                .field("pixel-aspect-ratio", &gstreamer::Fraction::from((1, 1)))
                .build();
        appsink
            .set_caps(Some(&caps));
        pipeline.set_property("video-sink", &appsink).unwrap();

        let pipeline = pipeline.dynamic_cast::<gstreamer::Pipeline>().unwrap();
        pipeline.set_property("uri", &Some(uri)).unwrap();
        pipeline.set_state(gstreamer::State::Paused).unwrap();

        VideoPlayer{
            pipeline,
            appsink,
            last_sample: None,
        }
    }
}

impl Widget<PlayerData> for VideoPlayer{
    fn lifecycle(&mut self, ctx: &mut LifeCycleCtx, event: &LifeCycle, data: &PlayerData, env: &Env) {
        if data == &PlayerData::Playing{
            ctx.request_anim_frame()
        }
    }

    fn paint(&mut self, paint_ctx: &mut PaintCtx, data: &PlayerData, _env: &Env) {
        if let PlayerData::Playing = data{
            self.last_sample = self.appsink.pull_sample().ok();
        }
        if let Some(sample) = &self.last_sample{
            let caps = sample.get_caps().unwrap();
            let width: i32 = caps_property(caps, "width");
            let height: i32 = caps_property(caps, "height");
            let data = sample.get_buffer().unwrap();
            let map = data.map_readable().unwrap();

            let image = paint_ctx.render_ctx.make_image(
                width as usize, 
                height as usize, 
                map.as_slice(), 
                ImageFormat::RgbaPremul
            ).unwrap();
            let widget_rect = Rect::from_origin_size(Point::ORIGIN, paint_ctx.size());
            let video_rect = fit_video_rect(&widget_rect, width, height);
            paint_ctx.render_ctx.draw_image(&image, video_rect, InterpolationMode::NearestNeighbor);
        }
    }

    fn update(&mut self, _ctx: &mut UpdateCtx, old_data: &PlayerData, data: &PlayerData, _env: &Env) {
        match (old_data, data) {
            (PlayerData::Paused, PlayerData::Playing) => {
                self.pipeline.set_state(gstreamer::State::Playing).unwrap();
                _ctx.children_changed();
                _ctx.request_paint();
            },
            (PlayerData::Playing, PlayerData::Paused) => {
                self.pipeline.set_state(gstreamer::State::Paused).unwrap();
            },
            _ => {}
        }
    }

    fn layout(&mut self, _ctx: &mut LayoutCtx, bc: &BoxConstraints, _data: &PlayerData, _env: &Env) -> Size {
        bc.max()
    }

    fn event(&mut self, _ctx: &mut EventCtx, event: &Event, _data: &mut PlayerData, _env: &Env) {
    }
}

fn caps_property<'a, T: gstreamer::glib::value::FromValueOptional<'a>>(caps: &'a gstreamer::CapsRef, name: &str) -> T {
    caps.iter().next().unwrap().get(name).unwrap().unwrap()
}

fn fit_video_rect(base_rect: &Rect, width: i32, height: i32) -> Rect{
    let w = width as f64;
    let h = height as f64;
    if base_rect.width() /  base_rect.height() > w / h {
        // Box to wide
        let new_w = base_rect.height() * w / h;
        let center = (base_rect.x0 + base_rect.x1) * 0.5;
        let x0 = center - new_w * 0.5;
        let x1 = center + new_w * 0.5;
        Rect::new(x0, base_rect.y0, x1, base_rect.y1)
    }else{
        // Box to high
        let new_h = base_rect.width() * h / w;
        let center = (base_rect.y0 + base_rect.y1) * 0.5;
        let y0 = center - new_h * 0.5;
        let y1 = center + new_h * 0.5;
        Rect::new(base_rect.x0, y0, base_rect.x1, y1)
    }
}