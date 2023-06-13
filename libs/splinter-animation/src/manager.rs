use crate::config::AnimationConfig;
use crate::lerp::Lerp;
use crate::{Animation, AnimationImpl};
use ahash::AHashMap;
use egui::{Context, Id};
use log::{info, trace};
use parking_lot::Mutex;
use std::any::{type_name, Any};
use std::sync::Arc;

pub struct AnimationManager {
    inner: Mutex<AnimationManagerInner>,
}

impl AnimationManager {
    pub fn new(config: AnimationConfig) -> AnimationManager {
        info!("Created animation manager");

        AnimationManager {
            inner: Mutex::new(AnimationManagerInner {
                config,
                animations: Default::default(),
                any_active: false,
                time: 0.0,
            }),
        }
    }

    pub fn get<L: Lerp + Send + Sync + Default>(&self, id: Id) -> Animation<L> {
        self.get_or(id, AnimationImpl::default)
    }

    pub fn get_or<L: Lerp + Send + Sync>(
        &self,
        id: Id,
        default: impl FnOnce() -> AnimationImpl<L>,
    ) -> Animation<L> {
        let mut inner = self.inner.lock();
        let arc = inner.animations.entry(id).or_insert_with(|| {
            let inner = default();
            trace!("Added animation <{}> to {id:?}", type_name::<L>());
            Box::new(Arc::new(Mutex::new(inner)))
        });
        let any = &(**arc);
        let link = any
            .downcast_ref::<Arc<Mutex<AnimationImpl<L>>>>()
            .unwrap_or_else(|| panic!("Wrong type <{}> for animation at {id:?}", type_name::<L>()))
            .clone();

        let animation = (link.lock()).clone();
        let animation = Animation {
            time: inner.time,
            animation_time: inner.config.animation_speed as f64,
            inner: animation,
            link,
        };

        if !inner.any_active {
            inner.any_active = animation.is_active();
        }

        animation
    }

    pub fn tick(&self, ctx: &Context) {
        let mut inner = self.inner.lock();
        inner.time = ctx.input(|i| i.time);
        if inner.any_active {
            ctx.request_repaint();
            inner.any_active = false;
        }
    }

    pub fn end_tick(&mut self, ctx: &Context) {
        let mut inner = self.inner.lock();
        if inner.any_active {
            ctx.request_repaint();
        }
        inner.any_active = false;
    }
}

struct AnimationManagerInner {
    config: AnimationConfig,
    animations: AHashMap<Id, Box<dyn Any + Send + Sync>>,
    any_active: bool,
    time: f64,
}
