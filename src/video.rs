use crate::PipelineCreationError;
use crate::{Controller, PipelineData};
use druid::{
    piet::Piet,
    piet::{ImageFormat, InterpolationMode, RenderContext},
    widget::Flex,
    BoxConstraints, Env, Event, EventCtx, LayoutCtx, LifeCycle, LifeCycleCtx, PaintCtx, Point,
    Rect, Size, UpdateCtx, Widget,
};
use gstreamer::{self as gst, prelude::*, Pipeline};
use gstreamer_app::AppSink;
use std::sync::{Arc, Mutex};
use thiserror::Error;

/// Video playing widget
pub struct VideoPlayer {
    /// Current sample to display. Updated via pipeline callbacks
    sample: Arc<Mutex<Option<gstreamer::Sample>>>,
}

#[derive(Error, Debug)]
enum PaintError {
    /// Error originated from glib.
    #[error("{0}")]
    Glib(#[from] gst::glib::BoolError),
    /// Sample was not set.
    #[error("No sample available")]
    NoSample,
    /// Unable to read sample capabilities.
    #[error("No capabilities set for sample.")]
    NoCapsSet,
    /// Error then trying to get sample buffer.
    #[error("Unable to get sample buffer.")]
    BufferError,
    /// Error obtaining property from capabilities.
    #[error("{0}")]
    GetProperty(#[from] GetPropertyError),
    /// Error originated in piet.
    #[error("piet: {0}")]
    Piet(#[from] druid::piet::Error),
}

#[derive(Error, Debug)]
enum GetPropertyError {
    #[error("No capabilities set for Caps.")]
    NoCaps,
    #[error("Property {0} not found.")]
    NoProperty(&'static str),
    #[error("{0}")]
    Get(#[from] gst::structure::GetError<'static>),
}

type PietImage = <Piet<'static> as RenderContext>::Image;

impl VideoPlayer {
    /// Create new video player with controller below.
    pub fn build_widget(uri: &str) -> Result<impl Widget<PipelineData>, PipelineCreationError> {
        let (pipeline, player) = Self::build_player(uri)?;
        let controller = Controller::build_widget(pipeline);

        let res = Flex::column()
            .with_flex_child(player, 1.0)
            .with_child(controller);
        Ok(res)
    }

    fn build_player(uri: &str) -> Result<(Pipeline, VideoPlayer), PipelineCreationError> {
        // Create shared storage for samples used in painting
        // as well as video receiving callbacks
        let sample = Arc::new(Mutex::new(None));
        let shared_sample = Arc::clone(&sample);
        let shared_sample_preroll = Arc::clone(&sample);

        let pipeline = gstreamer::ElementFactory::make("playbin", None)?;
        let appsink = gstreamer::ElementFactory::make("appsink", Some("sink"))?;

        let pipeline = pipeline
            .dynamic_cast::<Pipeline>()
            .expect("Playbin should always be a pipeline.");

        let appsink: AppSink = appsink
            .dynamic_cast()
            .expect("Appsink should be instance of `AppSink`.");
        // Force RGBA pixel format, since it's only one druid supports
        // (it does support Rgb, but it being converted into rgba under the hood, so no point in using it)
        let caps = gstreamer::Caps::builder("video/x-raw")
            .field("format", &gstreamer_video::VideoFormat::Rgba.to_string())
            .field("pixel-aspect-ratio", &gstreamer::Fraction::from((1, 1)))
            .build();
        appsink.set_caps(Some(&caps));
        // Set callback to update current sample.
        // Superiour to pulling in gui thread, since it avoids blocking gui thread until next frame arrives
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
                .eos(move |_x| {
                    println!("End of stream.");
                })
                .build(),
        );
        pipeline.set_property("uri", &Some(uri))?;
        pipeline.set_property("video-sink", &appsink)?;

        pipeline.set_state(gstreamer::State::Paused)?;

        let player = VideoPlayer { sample };

        Ok((pipeline, player))
    }

    fn get_sample_image(&self, paint_ctx: &mut PaintCtx) -> Result<(PietImage, Size), PaintError> {
        let lock = self.sample.lock().expect("Sample mutex got poisoned.");
        let sample = lock.as_ref().ok_or(PaintError::NoSample)?;
        let caps = sample.get_caps().ok_or(PaintError::NoCapsSet)?;
        let width: i32 = caps_property(caps, "width")?;
        let height: i32 = caps_property(caps, "height")?;
        let data = sample.get_buffer().ok_or(PaintError::BufferError)?;
        let map = data.map_readable()?;

        let image = paint_ctx.render_ctx.make_image(
            width as usize,
            height as usize,
            map.as_slice(),
            ImageFormat::RgbaPremul,
        )?;
        Ok((image, Size::new(width as f64, height as f64)))
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
        match self.get_sample_image(paint_ctx) {
            Ok((image, size)) => {
                let widget_rect = Rect::from_origin_size(Point::ORIGIN, paint_ctx.size());
                let video_rect = fit_video_rect(&widget_rect, &size);
                paint_ctx
                    .render_ctx
                    .draw_image(&image, video_rect, InterpolationMode::Bilinear);
            }
            Err(PaintError::NoSample) => {
                log::debug!("No video sample to paint.");
            }
            Err(e) => {
                log::error!("{}", e);
            }
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
    name: &'static str,
) -> Result<T, GetPropertyError> {
    let cap = caps.iter().next().ok_or_else(|| GetPropertyError::NoCaps)?;
    let property = cap
        .get(name)?
        .ok_or_else(|| GetPropertyError::NoProperty(name))?;
    Ok(property)
}

/// Calculate maximum rect that fit `base_rect` and has aspect ration of `width/height`
fn fit_video_rect(base_rect: &Rect, size: &Size) -> Rect {
    let w = size.width;
    let h = size.height;
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
