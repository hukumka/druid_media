use gstreamer::{Pipeline, SeekFlags, prelude::*};
use druid::{
    Widget, Event, LifeCycle,
    UpdateCtx, EventCtx, PaintCtx, LayoutCtx, LifeCycleCtx, Env,
    Size, BoxConstraints, 
    TimerToken,
    widget::{Flex, WidgetExt, Button, Slider},
    lens::LensExt,
};
use crate::{PipelineState, PipelineData, Timeline, TimelineFractionLens};
use std::time::{Duration, Instant};

pub struct Controller<W>{
    pipeline: Pipeline,
    predicted_time: Timeline,
    timer_id: Option<TimerToken>,

    inner: W,
}

impl Controller<()>{
    pub fn new(pipeline: Pipeline) -> impl Widget<PipelineData> {
        // Toggle play button
        let play_pause = Button::new(
            |data: &PipelineState, _env: &_| {
                match data{
                    PipelineState::Play => "Pause".into(),
                    PipelineState::Pause => "Play".into(),
                }
            },
            |_ctx, data: &mut PipelineState, _env| {
                match data{
                    PipelineState::Play => {
                        *data = PipelineState::Pause;
                    },
                    PipelineState::Pause => {
                        *data = PipelineState::Play;
                    },
                }
            }
        ).fix_width(100.0);
        // Timeline splider
        let timeline_slider = Slider::new();

        let inner = Flex::row()
            .with_child(play_pause.lens(PipelineData::state), 0.0)
            .with_child(timeline_slider.lens(PipelineData::timeline.then(TimelineFractionLens)), 1.0);
        
        Controller{
            pipeline,
            timer_id: None,
            predicted_time: Timeline::new(),
            inner,
        }
    }
}

impl<W: Widget<PipelineData>> Widget<PipelineData> for Controller<W>{
    fn lifecycle(&mut self, ctx: &mut LifeCycleCtx, event: &LifeCycle, data: &PipelineData, env: &Env) {
        self.inner.lifecycle(ctx, event, data, env);
    }

    fn layout(&mut self, ctx: &mut LayoutCtx, bc: &BoxConstraints, data: &PipelineData, env: &Env) -> Size {
        self.inner.layout(ctx, bc, data, env)
    }

    fn update(&mut self, ctx: &mut UpdateCtx, old_data: &PipelineData, data: &PipelineData, env: &Env) {
        match (old_data.state, data.state) {
            (PipelineState::Pause, PipelineState::Play) => {
                self.pipeline.set_state(gstreamer::State::Playing).unwrap();
            },
            (PipelineState::Play, PipelineState::Pause) => {
                self.pipeline.set_state(gstreamer::State::Paused).unwrap();
            },
            _ => {}
        }
        if self.predicted_time != data.timeline && data.timeline.duration.is_some(){
            self.pipeline.seek_simple(SeekFlags::FLUSH, data.timeline.position).unwrap();
        }
        self.inner.update(ctx, old_data, data, env)
    }

    fn event(&mut self, ctx: &mut EventCtx, event: &Event, data: &mut PipelineData, env: &Env) {
        self.inner.event(ctx, event, data, env);
        match event{
            Event::Timer(id) if Some(id) == self.timer_id.as_ref() => {
                data.timeline = Timeline::query(&self.pipeline);
                self.predicted_time = data.timeline.clone();
                let deadline = Instant::now() + Duration::from_millis(100);
                self.timer_id = Some(ctx.request_timer(deadline));
            },
            Event::MouseUp(_) if data.state == PipelineState::Play => {
                println!("start new timer");
                let deadline = Instant::now() + Duration::from_millis(100);
                self.timer_id = Some(ctx.request_timer(deadline));
            },
            _ => {}
        }
        
    }

    fn paint(&mut self, paint_ctx: &mut PaintCtx, data: &PipelineData, env: &Env) {
        self.inner.paint(paint_ctx, data, env);
    }
}