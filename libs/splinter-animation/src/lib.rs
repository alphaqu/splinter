//! # Pitaya Animation
//! The core of pitayas fluid design language.
//! This crate contains the Pitaya animation system that supports blending between two values in a certain amount of time with an optional ease curve.

pub mod config;
mod easing;
mod lerp;
mod manager;

pub use crate::easing::Easing;
pub use crate::lerp::{extend, Lerp};
pub use crate::manager::AnimationManager;
use std::marker::PhantomData;

use egui::{Id, Pos2, Rect};
use parking_lot::Mutex;
use std::sync::Arc;

pub struct AnimationRef<L> {
    pub id: Id,
    _d: PhantomData<L>,
}

impl<L> AnimationRef<L> {
    pub fn new(id: Id) -> AnimationRef<L> {
        AnimationRef {
            id,
            _d: Default::default(),
        }
    }
}

pub struct Animation<L: Lerp + Send + Sync> {
    pub(crate) time: f64,
    pub(crate) animation_time: f64,
    pub(crate) inner: AnimationImpl<L>,
    pub(crate) link: Arc<Mutex<AnimationImpl<L>>>,
}

impl<L: Lerp + Send + Sync> Animation<L> {
    /// Sets the from value to the current value.
    pub fn anchor_from(&mut self) -> &mut Self {
        self.inner.from = self.get();
        self
    }

    /// Sets the to value to the current value.
    pub fn anchor_to(&mut self) -> &mut Self {
        self.inner.to = self.get();
        self
    }

    pub fn redirect(&mut self, to: L) -> &mut Self {
        self.redirect_with_speed(to, 1.0)
    }

    /// If the to value is not the same as the parameter
    /// it will wait until the animation is finished and then "redirect" the animation to the new state.
    pub fn redirect_with_speed(&mut self, to: L, speed: f32) -> &mut Self {
        if self.get_to() != &to {
            self.when_done(|ani| ani.anchor_from().set_to(to).begin_with_speed(speed));
        }
        self
    }

    /// Removes any current animation and sets a static value.
    pub fn set_value(&mut self, value: L) -> &mut Self {
        self.inner.start = 0.0;
        self.inner.duration = 0.0;
        self.inner.from = value.clone();
        self.inner.to = value;
        self
    }

    /// Checks if the animation is currently moving
    pub fn is_active(&self) -> bool {
        let pos = self.get_pos();
        pos > 0.0 && pos < 1.0
    }

    pub fn has_started(&self) -> bool {
        let pos = self.get_pos();
        pos > 0.0
    }

    pub fn is_finished(&self) -> bool {
        let pos = self.get_pos();
        pos >= 1.0
    }

    pub fn when_done(&mut self, func: impl FnOnce(&mut Self)) {
        if self.is_finished() {
            func(self)
        }
    }

    /// Gets the current position of the animation
    pub fn get_pos(&self) -> f64 {
        if self.inner.duration == 0.0 {
            1.0
        } else {
            (self.time - self.inner.start) / self.inner.duration
        }
    }

    /// Gets the current value of the animation
    pub fn get(&self) -> L {
        let time_t = self.get_pos();
        let clamped_t = time_t.clamp(0.0, 1.0);
        let eased_t = self.inner.easing.apply(clamped_t);
        self.inner.from.lerp(&self.inner.to, eased_t as f32)
    }

    /// Starts a new animation to a new target.
    pub fn begin(&mut self) {
        self.begin_with_speed(1.0);
    }

    pub fn begin_with_speed(&mut self, speed: f32) {
        self.inner.start = self.time;
        self.inner.duration = speed as f64 * self.animation_time;
    }

    /// Overwrites the current source value
    pub fn set_from(&mut self, from: L) -> &mut Self {
        self.inner.from = from;
        self
    }

    /// Overwrites the current target value
    pub fn set_to(&mut self, to: L) -> &mut Self {
        self.inner.to = to;
        self
    }

    /// Overwrites the current easing
    pub fn set_easing(&mut self, easing: Easing) -> &mut Self {
        self.inner.easing = easing;
        self
    }

    pub fn get_to(&self) -> &L {
        &self.inner.to
    }

    pub fn get_from(&self) -> &L {
        &self.inner.from
    }
}

impl<L: Lerp + Send + Sync> Drop for Animation<L> {
    fn drop(&mut self) {
        *self.link.lock() = self.inner.clone();
    }
}

impl<L: Lerp + Send + Sync + Default> Animation<L> {
    /// Resets any current animation and sets it to the default value.
    pub fn reset(&mut self) -> &mut Self {
        self.set_value(L::default())
    }
}

#[derive(Copy, Clone)]
pub struct AnimationImpl<L: Lerp + Send + Sync> {
    pub from: L,
    pub to: L,
    pub easing: Easing,
    // seconds time
    pub(crate) start: f64,
    pub(crate) duration: f64,
}

impl<L: Lerp + Send + Sync> AnimationImpl<L> {
    pub fn new(from: L, to: L, easing: Easing) -> AnimationImpl<L> {
        AnimationImpl {
            from,
            to,
            easing,
            start: 0.0,
            duration: 0.0,
        }
    }

    pub fn simple(value: L) -> AnimationImpl<L> {
        AnimationImpl {
            from: value.clone(),
            to: value,
            easing: Easing::EaseInOut,
            start: 0.0,
            duration: 0.0,
        }
    }
}

impl AnimationImpl<Rect> {
    pub fn rect() -> AnimationImpl<Rect> {
        AnimationImpl {
            from: Rect::from_min_max(Pos2::ZERO, Pos2::ZERO),
            to: Rect::from_min_max(Pos2::ZERO, Pos2::ZERO),
            easing: Easing::EaseInOut,
            start: 0.0,
            duration: 0.0,
        }
    }
}
impl<L: Lerp + Send + Sync + Default> Default for AnimationImpl<L> {
    fn default() -> Self {
        Self::new(L::default(), L::default(), Easing::EaseInOut)
    }
}
