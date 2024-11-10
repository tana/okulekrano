// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use std::sync::Arc;

use glium::Texture2d;

pub mod fake;
mod texture;
pub mod wayland;

pub trait Capturer {
    fn capture(&mut self) -> Arc<Texture2d>;

    fn resolution(&self) -> (u32, u32);
}
