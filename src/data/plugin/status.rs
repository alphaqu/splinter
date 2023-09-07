#[derive(Ord, PartialOrd, Eq, PartialEq, Hash, Copy, Clone)]
pub enum PluginStatus {
    Enabled = 0,
    Disabled = 1,
    NotTheProblem = 2,
}

impl PluginStatus {
    pub fn enabled(&self) -> bool {
        match self {
            PluginStatus::Disabled | PluginStatus::NotTheProblem => false,
            PluginStatus::Enabled => true,
        }
    }

    pub fn iter() -> [PluginStatus; 3] {
        [
            PluginStatus::Enabled,
            PluginStatus::Disabled,
            PluginStatus::NotTheProblem,
        ]
    }
}
