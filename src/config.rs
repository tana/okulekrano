// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use serde::{Deserialize, Serialize};

#[derive(Clone, Default, Debug, Serialize, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub capture: Capture,
    #[serde(default)]
    pub glasses: Glasses,
    #[serde(default)]
    pub virtual_screen: VirtualScreen,
}

#[derive(Clone, Default, Debug, Serialize, Deserialize)]
pub struct Capture {
    #[serde(default)]
    pub output_name: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Glasses {
    #[serde(default)]
    pub monitor_name: Option<String>,
    #[serde(default)]
    pub window_mode: bool,
}

impl Default for Glasses {
    fn default() -> Self {
        Self {
            monitor_name: None,
            window_mode: false,
        }
    }
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
