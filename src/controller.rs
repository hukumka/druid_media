use crate::{PipelineData, PipelineState, Timeline};
use druid::{
    lens::LensExt,
    widget::{Button, Flex, Slider, WidgetExt},
    BoxConstraints, Env, Event, EventCtx, LayoutCtx, LifeCycle, LifeCycleCtx, PaintCtx, Size,
    TimerToken, UpdateCtx, Widget,
};
use gstreamer::{prelude::*, ClockTime, Pipeline, SeekFlags};
use std::time::{Duration, Instant};

pub struct Controller<W> {
    pipeline: Pipeline,

    updated_time: Timeline,
    timer_id: TimerToken,

    inner: W,
}

impl Controller<()> {
    pub fn new(pipeline: Pipeline) -> impl Widget<PipelineData> {
        // Toggle play button
        let play_pause = Button::new(
            |data: &PipelineState, _env: &_| match data {
                PipelineState::Play => "Pause".into(),
                PipelineState::Pause => "Play".into(),
            },
            |_ctx, _data: &mut PipelineState, _env| {},
        )
        .fix_width(100.0)
        .lens(PipelineData::state);
        // Timeline splider
        let timeline_slider = Slider::new().lens(PipelineData::timeline.then(Timeline::frac));
        let controller = Controller {
            pipeline,
            timer_id: TimerToken::INVALID,
            updated_time: Timeline::new(),
            inner: play_pause,
        };
        Flex::row()
            .with_child(controller, 0.0)
            .with_child(timeline_slider, 1.0)
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
        println!("{:?}", self.pipeline.get_state(ClockTime::from_mseconds(10)));
        match (old_data.state, data.state) {
            (PipelineState::Pause, PipelineState::Play) => {
                let change = self.pipeline.set_state(gstreamer::State::Playing).unwrap();
                println!("Pause -> Play: {:?}", change);
            }
            (PipelineState::Play, PipelineState::Pause) => {
                let change = self.pipeline.set_state(gstreamer::State::Paused).unwrap();
                println!("Play -> Pause: {:?}", change);
            }
            _ => {}
        }
        if self.updated_time != data.timeline {
            println!("{:?} -> {:?}", self.updated_time, data.timeline);
            self.updated_time = data.timeline;
            let time = data.timeline.frac * data.timeline.duration;
            let position = ClockTime::from_nseconds(time as u64);
            let res = self.pipeline
                .seek_simple(SeekFlags::FLUSH, position)
                .unwrap();
            println!("Seeking: {:?}", res);
        }
        self.inner.update(ctx, old_data, data, env)
    }

    fn event(&mut self, ctx: &mut EventCtx, event: &Event, data: &mut PipelineData, _env: &Env) {
        match event {
            Event::Timer(id) if id == &self.timer_id && data.state == PipelineState::Play => {
                ctx.request_paint();
                data.timeline = get_timeline(&self.pipeline);
                self.updated_time = data.timeline.clone();

                let deadline = Instant::now() + Duration::from_millis(16);
                self.timer_id = ctx.request_timer(deadline);
            }
            Event::MouseDown(_) => {
                if data.state == PipelineState::Pause {
                    data.state = PipelineState::Play;
                    // Schedule timeline updates
                    let deadline = Instant::now() + Duration::from_millis(16);
                    self.timer_id = ctx.request_timer(deadline);
                }else{
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
    let pos: ClockTime = pipeline.query_position().unwrap_or(ClockTime::none());
    let dur: ClockTime = pipeline.query_duration().unwrap_or(ClockTime::none());
    let pos = pos.nanoseconds().unwrap_or(0) as f64;
    let mut duration = dur.nanoseconds().unwrap_or(1) as f64;
    if duration == 0.0 {
        duration = 1.0;
    }
    let frac = pos / duration;
    Timeline { frac, duration }
}
