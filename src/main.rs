// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

mod capturer;
mod config;
mod glasses;
mod mode_refresh;
mod renderer;
mod winit_app;

extern crate nalgebra as na;

fn main() {
    env_logger::init();

    winit_app::run();
}
