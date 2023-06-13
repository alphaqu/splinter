use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Default)]
pub struct AnimationConfig {
	#[serde(default = "AnimationConfig::default_animation_speed")]
	pub animation_speed: f32
}

impl AnimationConfig {
	pub fn default_animation_speed() -> f32 {
		0.25
	}
}