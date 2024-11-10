// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use ahrs::Ahrs;
use ar_drivers::{ARGlasses, GlassesEvent, Side};
use core::f32;
use na::{Matrix4, Perspective3, Rotation3, UnitQuaternion, Vector3};
use std::sync::mpsc;
use std::thread::{self, JoinHandle};

const FUSION_PERIOD_US: u64 = 10000;
const IPD: f32 = 0.07;

pub struct GlassesController {
    #[allow(dead_code)]
    join_handle: JoinHandle<()>,
    quat_receiver: mpsc::Receiver<UnitQuaternion<f32>>,
    stop_sender: mpsc::Sender<()>,
    pose: Rotation3<f32>,
    fov: f32,
    imu_to_display_l: Matrix4<f32>,
    imu_to_display_r: Matrix4<f32>,
}

impl GlassesController {
    pub fn new() -> Self {
        let mut glasses = ar_drivers::any_glasses()
            .expect("AR glasses not found. Maybe permission issues of hidraw device.");

        // Turn the glasses into 3D mode
        glasses
            .set_display_mode(ar_drivers::DisplayMode::Stereo)
            .unwrap();

        let fov = glasses.display_fov();

        let imu_to_display_l = Matrix4::from_column_slice(
            glasses
                .imu_to_display_matrix(Side::Left, IPD)
                .to_homogeneous()
                .as_slice(),
        )
        .cast();
        let imu_to_display_r = Matrix4::from_column_slice(
            glasses
                .imu_to_display_matrix(Side::Right, IPD)
                .to_homogeneous()
                .as_slice(),
        )
        .cast();

        let (quat_sender, quat_receiver) = mpsc::sync_channel(0);
        let (stop_sender, stop_receiver) = mpsc::channel();

        let join_handle = thread::spawn(move || {
            process_events(glasses, quat_sender, stop_receiver);
        });

        Self {
            join_handle,
            quat_receiver,
            stop_sender,
            pose: Rotation3::identity(),
            fov,
            imu_to_display_l,
            imu_to_display_r,
        }
    }

    pub fn update_pose(&mut self) {
        let Ok(quat) = self.quat_receiver.recv() else {
            return;
        };
        self.pose = quat.to_rotation_matrix();
    }

    pub fn camera_mat(&self, side: Side, aspect: f32) -> Matrix4<f32> {
        let proj = Perspective3::new(aspect, self.fov / aspect, 0.1, 10.0)
            .as_projective()
            .clone();
        let imu_to_display = match side {
            Side::Left => self.imu_to_display_l,
            Side::Right => self.imu_to_display_r,
        };
        proj.to_homogeneous() * imu_to_display * self.pose.inverse().to_homogeneous()
    }
}

impl Drop for GlassesController {
    fn drop(&mut self) {
        self.stop_sender.send(()).unwrap();
    }
}

fn process_events(
    mut glasses: Box<dyn ARGlasses>,
    quat_sender: mpsc::SyncSender<UnitQuaternion<f32>>,
    stop_receiver: mpsc::Receiver<()>,
) {
    let mut filter = ahrs::Madgwick::new(FUSION_PERIOD_US as f32 / 1e6, 0.1);

    let mut last_timestamp = 0;

    // Terminate when something is received from stop_receiver
    while stop_receiver.try_recv().is_err() {
        match glasses.read_event().unwrap() {
            GlassesEvent::AccGyro {
                accelerometer,
                gyroscope,
                timestamp,
            } if (timestamp - last_timestamp) >= FUSION_PERIOD_US => {
                // Because ahrs and ar_drivers use incompatible versions of nalgebra, conversions are needed.
                let quat = filter
                    .update_imu(
                        &Vector3::new(gyroscope.x, gyroscope.y, gyroscope.z),
                        &Vector3::new(accelerometer.x, accelerometer.y, accelerometer.z),
                    )
                    .unwrap();

                // Conversion from `ahrs` (z-down) to `ar-drivers` convention (y-up)
                let filter_to_glasses =
                    UnitQuaternion::from_axis_angle(&Vector3::x_axis(), -f32::consts::FRAC_PI_2);

                // Skip sending if the receiver is not received the old pose yet
                match quat_sender.try_send(filter_to_glasses * quat) {
                    Ok(_) => (),
                    Err(mpsc::TrySendError::Full(_)) => (),
                    Err(error) => panic!("{}", error),
                }

                last_timestamp = timestamp;
            }
            GlassesEvent::Magnetometer {
                magnetometer: _magnetometer,
                timestamp,
            } => {
                log::debug!("mag {}", timestamp);
            }
            _ => (),
        }
    }

    // Reset the glasses to 2D mode
    glasses
        .set_display_mode(ar_drivers::DisplayMode::SameOnBoth)
        .unwrap();
}
