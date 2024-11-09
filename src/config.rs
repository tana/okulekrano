use serde::{Deserialize, Serialize};

#[derive(Clone, Default, Debug, Serialize, Deserialize)]
pub struct Config {
    pub capture: Capture,
    pub virtual_screen: VirtualScreen,
}

#[derive(Clone, Default, Debug, Serialize, Deserialize)]
pub struct Capture {
    pub output_name: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct VirtualScreen {
    #[serde(default = "default_height")]
    pub height: f32,
    #[serde(default = "default_distance")]
    pub distance: f32,
}

impl Default for VirtualScreen {
    fn default() -> Self {
        Self {
            height: default_height(),
            distance: default_distance(),
        }
    }
}

fn default_height() -> f32 {
    1.0
}

fn default_distance() -> f32 {
    1.0
}
