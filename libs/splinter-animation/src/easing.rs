#[derive(Copy, Clone)]
pub enum Easing {
    Linear,
    // Quart (its a touch more aggressive than cubic)
    EaseIn,
    EaseOut,
    EaseInOut,
}

impl Easing {
    #[inline(always)]
    pub fn apply(&self, x: f64) -> f64 {
        assert!((0.0..=1.0).contains(&x));
        match self {
            Easing::Linear => x,
            Easing::EaseIn => x * x * x * x,
            Easing::EaseOut => 1.0 - (1.0 - x).powf(4.0),
            Easing::EaseInOut => {
                if x < 0.5 {
                    8.0 * x * x * x * x
                } else {
                    1.0 - (-2.0 * x + 2.0).powf(4.0) / 2.0
                }
            }
        }
    }
}
