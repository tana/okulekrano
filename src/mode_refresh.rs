// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use std::collections::HashMap;

use wayland_client::{
    protocol::{
        wl_output::{self, WlOutput},
        wl_registry::{self, WlRegistry},
    },
    Connection, Dispatch, EventQueue, Proxy,
};

#[derive(Default)]
struct State {
    outputs: HashMap<WlOutput, IncompleteMonitorInfo>,
}

#[derive(Default)]
struct IncompleteMonitorInfo {
    name: Option<String>,
    width: Option<u32>,
    height: Option<u32>,
    reported: bool,
}

#[derive(Clone, Debug)]
pub struct MonitorInfo {
    pub name: String,
    pub width: u32,
    #[allow(dead_code)]
    pub height: u32,
}

pub struct QueryMonitors {
    queue: EventQueue<State>,
    state: State,
}

impl Iterator for QueryMonitors {
    type Item = MonitorInfo;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            self.queue.blocking_dispatch(&mut self.state).unwrap();
            for info in self.state.outputs.values_mut() {
                if info.name.is_some()
                    && info.width.is_some()
                    && info.height.is_some()
                    && !info.reported
                {
                    info.reported = true;

                    return Some(MonitorInfo {
                        name: info.name.clone().unwrap(),
                        width: info.width.clone().unwrap(),
                        height: info.height.clone().unwrap(),
                    });
                }
            }
        }
    }
}

pub fn query_monitors() -> QueryMonitors {
    let conn = Connection::connect_to_env().unwrap();
    let display = conn.display();

    let mut queue = conn.new_event_queue();

    let mut state = State::default();

    // (1) Retrieve global objects such as wl_display.
    // It will fire `wl_registry::global` event.
    let _registry = display.get_registry(&queue.handle(), ());
    queue.roundtrip(&mut state).unwrap();

    QueryMonitors { queue, state }
}

impl Dispatch<WlRegistry, (), Self> for State {
    fn event(
        state: &mut Self,
        proxy: &WlRegistry,
        event: <WlRegistry as wayland_client::Proxy>::Event,
        _data: &(),
        _conn: &Connection,
        qhandle: &wayland_client::QueueHandle<Self>,
    ) {
        // (2) Store received global objects.
        match event {
            wl_registry::Event::Global {
                name,
                interface,
                version,
            } => {
                if interface == WlOutput::interface().name {
                    let output: WlOutput = proxy.bind(name, version, qhandle, ());
                    state
                        .outputs
                        .insert(output, IncompleteMonitorInfo::default());
                }
            }
            _ => (),
        }
    }
}

impl Dispatch<WlOutput, (), Self> for State {
    fn event(
        state: &mut Self,
        proxy: &WlOutput,
        event: <WlOutput as wayland_client::Proxy>::Event,
        _data: &(),
        _conn: &Connection,
        _qhandle: &wayland_client::QueueHandle<Self>,
    ) {
        let info = state.outputs.get_mut(proxy).unwrap();
        match event {
            wl_output::Event::Name { name } => {
                (*info).name = Some(name);
            }
            wl_output::Event::Mode { width, height, .. } => {
                (*info).width = Some(width as u32);
                (*info).height = Some(height as u32);
            }
            _ => (),
        }
    }
}
