use crate::{PipelineData, PipelineState, Timeline};
use druid::{
    lens::LensExt,
    widget::{Button, Flex, Slider, WidgetExt},
    BoxConstraints, Env, Event, EventCtx, LayoutCtx, LifeCycle, LifeCycleCtx, PaintCtx, Size,
    TimerToken, UpdateCtx, Widget,
};
use gstreamer::{self as gst, prelude::*, ClockTime, Pipeline, SeekFlags};
use std::time::{Duration, Instant};
use thiserror::Error;

#[derive(Error, Debug)]
enum UpdateError {
    #[error("Failed to change pipeline state")]
    StateChange(#[from] gst::StateChangeError),
    #[error("{0}")]
    GlibError(#[from] gst::glib::BoolError),
}

/// Widget to control media playing
pub struct Controller<W> {
    /// Underlying pipeline
    pipeline: Pipeline,

    /// Time "expected" for pipeline to be in.
    /// Used to distinguish between cases of time
    /// being updated because media is played, or
    /// because of manual seeking.
    updated_time: Timeline,

    /// Timer to update timeline position
    timer_id: TimerToken,

    /// Inner button
    inner: W,
}

impl Controller<()> {
    /// Create new widget to controll pipeline playing.
    ///
    /// Contains Play/Pause button and timeline position slider.
    pub fn build_widget(pipeline: Pipeline) -> impl Widget<PipelineData> {
        // Toggle play button
        let play_pause = Button::new(
            |data: &PipelineState, _env: &_| match data {
                PipelineState::Play => "⏸️".into(),
                PipelineState::Pause => "▶".into(),
            }
        ).on_click(
            |_ctx, _data: &mut PipelineState, _env| {},
        )
        .fix_width(40.0)
        .lens(PipelineData::state);
        let timeline_slider = Slider::new()
            .lens(PipelineData::timeline.then(Timeline::frac))
            .expand_width();
        let muted_button = Button::new(
            |data: &PipelineData, _env: &_| {
                if data.muted {
                    "🔈x".to_string()
                } else if data.volume < 0.01 {
                    "🔈".to_string()
                } else if data.volume < 0.3 {
                    "🔉".to_string()
                } else {
                    "🔊".to_string()
                }
            }
        ).on_click(
            |_ctx, data: &mut PipelineData, _env| {
                data.muted = !data.muted;
            },
        )
        .fix_width(40.0);
        let volume_slider = Slider::new().lens(PipelineData::volume).fix_width(100.0);
        let controller = Controller {
            pipeline,
            timer_id: TimerToken::INVALID,
            updated_time: Timeline::new(),
            inner: play_pause,
        };
        Flex::row()
            .with_child(controller)
            .with_flex_child(timeline_slider, 1.0)
            .with_child(muted_button)
            .with_child(volume_slider)
    }
}

impl<W> Controller<W> {
    fn update_pipeline(
        &mut self,
        old_data: &PipelineData,
        data: &PipelineData,
    ) -> Result<(), UpdateError> {
        // Update pipeline state if needed
        match (old_data.state, data.state) {
            (PipelineState::Pause, PipelineState::Play) => {
                self.pipeline.set_state(gstreamer::State::Playing)?;
            }
            (PipelineState::Play, PipelineState::Pause) => {
                self.pipeline.set_state(gstreamer::State::Paused)?;
            }
            _ => {}
        }
        // Update timeline position if needed
        if self.updated_time != data.timeline {
            self.updated_time = data.timeline;
            let time = data.timeline.frac * data.timeline.duration;
            let position = ClockTime::from_nseconds(time as u64);
            self.pipeline.seek_simple(SeekFlags::FLUSH, position)?;
        }
        // Update volume
        if (old_data.volume - data.volume) > std::f64::EPSILON {
            self.pipeline.set_property("volume", &data.volume)?;
        }
        if old_data.muted != data.muted {
            self.pipeline.set_property("mute", &data.muted)?;
        }

        Ok(())
    }
}

impl<W: Widget<PipelineData>> Widget<PipelineData> for Controller<W> {
    fn lifecycle(
        &mut self,
        ctx: &mut LifeCycleCtx,
        event: &LifeCycle,
        data: &PipelineData,
        env: &Env,
    ) {
        self.inner.lifecycle(ctx, event, data, env);
    }

    fn layout(
        &mut self,
        ctx: &mut LayoutCtx,
        bc: &BoxConstraints,
        data: &PipelineData,
        env: &Env,
    ) -> Size {
        self.inner.layout(ctx, bc, data, env)
    }

    fn update(
        &mut self,
        ctx: &mut UpdateCtx,
        old_data: &PipelineData,
        data: &PipelineData,
        env: &Env,
    ) {
        self.update_pipeline(old_data, data).unwrap_or_else(|e| {
            log::error!(
                "Failed to update pipeline state:\n\
                    \told: {:?}\n\
                    \tnew: {:?}\n\
                    \tError: {}
                ",
                old_data,
                data,
                e
            )
        });
        self.inner.update(ctx, old_data, data, env)
    }

    fn event(&mut self, ctx: &mut EventCtx, event: &Event, data: &mut PipelineData, _env: &Env) {
        match event {
            Event::Timer(id) if id == &self.timer_id && data.state == PipelineState::Play => {
                ctx.request_paint();
                data.timeline = get_timeline(&self.pipeline);
                self.updated_time = data.timeline;

                let deadline = Instant::now() + Duration::from_millis(16);
                self.timer_id = ctx.request_timer(deadline);
            }
            Event::MouseDown(_) => {
                // Hack
                // State managed by handing event instead of using button this widget contains
                // to start timer events.
                if data.state == PipelineState::Pause {
                    data.state = PipelineState::Play;
                    // Schedule timeline updates
                    let deadline = Instant::now() + Duration::from_millis(16);
                    self.timer_id = ctx.request_timer(deadline);
                } else {
                    data.state = PipelineState::Pause;
                }
                ctx.request_paint();
            }
            _ => {}
        }
    }

    fn paint(&mut self, paint_ctx: &mut PaintCtx, data: &PipelineData, env: &Env) {
        self.inner.paint(paint_ctx, data, env);
    }
}

fn get_timeline(pipeline: &Pipeline) -> Timeline {
    let pos: ClockTime = pipeline.query_position().unwrap_or_else(ClockTime::none);
    let dur: ClockTime = pipeline.query_duration().unwrap_or_else(ClockTime::none);
    let pos = pos.nanoseconds().unwrap_or(0) as f64;
    let mut duration = dur.nanoseconds().unwrap_or(1) as f64;
    if duration == 0.0 {
        duration = 1.0;
    }
    let frac = pos / duration;
    Timeline { frac, duration }
}
