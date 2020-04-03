use crate::{Controller, PipelineData};
use druid::{
    piet::{ImageFormat, InterpolationMode, RenderContext},
    widget::Flex,
    BoxConstraints, Env, Event, EventCtx, LayoutCtx, LifeCycle, LifeCycleCtx, PaintCtx, Point,
    Rect, Size, UpdateCtx, Widget,
};
use gstreamer::{prelude::*, Pipeline};
use gstreamer_app::AppSink;
use std::sync::{Arc, Mutex};

pub struct VideoPlayer {
    sample: Arc<Mutex<Option<gstreamer::Sample>>>,
}

impl VideoPlayer {
    pub fn new(uri: &str) -> impl Widget<PipelineData> {
        let (pipeline, player) = Self::build_player(uri);
        let controller = Controller::new(pipeline);

        Flex::column()
            .with_child(player, 1.0)
            .with_child(controller, 0.0)
    }

    fn build_player(uri: &str) -> (Pipeline, VideoPlayer) {
        let sample = Arc::new(Mutex::new(None));
        let shared_sample = Arc::clone(&sample);
        let shared_sample_preroll = Arc::clone(&sample);

        let pipeline = gstreamer::ElementFactory::make("playbin", None).unwrap();
        let appsink = gstreamer::ElementFactory::make("appsink", Some("sink")).unwrap();

        let pipeline = pipeline.dynamic_cast::<gstreamer::Pipeline>().unwrap();
       
        let appsink: AppSink = appsink.dynamic_cast().unwrap();
        // Force RGBA pixel format, since it's only one druid supports
        // (Rgb being converted into rgba under the hood)
        let caps = gstreamer::Caps::builder("video/x-raw")
            .field("format", &gstreamer_video::VideoFormat::Rgba.to_string())
            .field("pixel-aspect-ratio", &gstreamer::Fraction::from((1, 1)))
            .build();
        appsink.set_caps(Some(&caps));
        // Set callback to update current sample.
        // Superiour to pulling in gui thread, since it avoids back pressure if
        // gui thread lags for some reason
        appsink.set_callbacks(
            gstreamer_app::AppSinkCallbacks::new()
                .new_sample(move |x| {
                    if let Ok(sample) = x.pull_sample() {
                        let mut guard = shared_sample.lock().unwrap();
                        *guard = Some(sample);
                    }
                    Ok(gstreamer::FlowSuccess::Ok)
                })
                .new_preroll(move |x| {
                    if let Ok(sample) = x.pull_preroll() {
                        let mut guard = shared_sample_preroll.lock().unwrap();
                        *guard = Some(sample);
                    }
                    Ok(gstreamer::FlowSuccess::Ok)
                })
                .eos(move |x| {
                    println!("EOS");
                })
                .build(),
        );
        pipeline.set_property("uri", &Some(uri)).unwrap();
        pipeline.set_property("video-sink", &appsink).unwrap();

        pipeline.set_state(gstreamer::State::Paused).unwrap();

        let player = VideoPlayer { sample };

        (pipeline, player)
    }
}

impl Widget<PipelineData> for VideoPlayer {
    fn lifecycle(
        &mut self,
        _ctx: &mut LifeCycleCtx,
        _event: &LifeCycle,
        _data: &PipelineData,
        _env: &Env,
    ) {
    }

    fn paint(&mut self, paint_ctx: &mut PaintCtx, _data: &PipelineData, _env: &Env) {
        let sample = {
            let guard = self.sample.lock().unwrap();
            guard.clone()
        };
        if let Some(sample) = sample {
            let caps = sample.get_caps().unwrap();
            let width: i32 = caps_property(caps, "width");
            let height: i32 = caps_property(caps, "height");
            let data = sample.get_buffer().unwrap();
            let map = data.map_readable().unwrap();

            let image = paint_ctx
                .render_ctx
                .make_image(
                    width as usize,
                    height as usize,
                    map.as_slice(),
                    ImageFormat::RgbaPremul,
                )
                .unwrap();
            let widget_rect = Rect::from_origin_size(Point::ORIGIN, paint_ctx.size());
            let video_rect = fit_video_rect(&widget_rect, width, height);
            paint_ctx
                .render_ctx
                .draw_image(&image, video_rect, InterpolationMode::Bilinear);
        }
    }

    fn update(
        &mut self,
        _ctx: &mut UpdateCtx,
        _old_data: &PipelineData,
        _data: &PipelineData,
        _env: &Env,
    ) {
    }

    fn layout(
        &mut self,
        _ctx: &mut LayoutCtx,
        bc: &BoxConstraints,
        _data: &PipelineData,
        _env: &Env,
    ) -> Size {
        bc.max()
    }

    fn event(&mut self, _ctx: &mut EventCtx, _event: &Event, _data: &mut PipelineData, _env: &Env) {
    }
}

/// Get property from capabilities structure. Panics if such property does not exist
fn caps_property<'a, T: gstreamer::glib::value::FromValueOptional<'a>>(
    caps: &'a gstreamer::CapsRef,
    name: &str,
) -> T {
    caps.iter().next().unwrap().get(name).unwrap().unwrap()
}

/// Calculate maximum rect that fit `base_rect` and has aspect ration of `width/height`
fn fit_video_rect(base_rect: &Rect, width: i32, height: i32) -> Rect {
    let w = width as f64;
    let h = height as f64;
    if base_rect.width() / base_rect.height() > w / h {
        // Box to wide
        let new_w = base_rect.height() * w / h;
        let center = (base_rect.x0 + base_rect.x1) * 0.5;
        let x0 = center - new_w * 0.5;
        let x1 = center + new_w * 0.5;
        Rect::new(x0, base_rect.y0, x1, base_rect.y1)
    } else {
        // Box to high
        let new_h = base_rect.width() * h / w;
        let center = (base_rect.y0 + base_rect.y1) * 0.5;
        let y0 = center - new_h * 0.5;
        let y1 = center + new_h * 0.5;
        Rect::new(base_rect.x0, y0, base_rect.x1, y1)
    }
}
