use ahrs::Ahrs;
use ar_drivers::{ARGlasses, GlassesEvent};
use core::f32;
use na::{Matrix4, Perspective3, Projective3, Rotation3, UnitQuaternion, Vector3};
use std::sync::mpsc;
use std::thread::{self, JoinHandle};

const FUSION_PERIOD_US: u64 = 10000;

pub struct GlassesController {
    #[allow(dead_code)]
    join_handle: JoinHandle<()>,
    quat_receiver: mpsc::Receiver<UnitQuaternion<f32>>,
    pose: Rotation3<f32>,
    proj_left: Projective3<f32>,
}

impl GlassesController {
    pub fn new() -> Self {
        let glasses = ar_drivers::any_glasses()
            .expect("AR glasses not found. Maybe permission issues of hidraw device.");

        let aspect = 16.0 / 9.0;
        let fovy = glasses.display_fov() / aspect;

        let (quat_sender, quat_receiver) = mpsc::sync_channel(0);

        let join_handle = thread::spawn(move || {
            process_events(glasses, quat_sender);
        });

        Self {
            join_handle,
            quat_receiver,
            pose: Rotation3::identity(),
            proj_left: Perspective3::new(aspect, fovy, 0.1, 10.0)
                .as_projective()
                .clone(),
        }
    }

    pub fn update_pose(&mut self) {
        let quat = self.quat_receiver.recv().unwrap();
        self.pose = quat.to_rotation_matrix();
    }

    pub fn camera_mat_left(&self) -> Matrix4<f32> {
        self.proj_left.to_homogeneous() * self.pose.inverse().to_homogeneous()
    }
}

fn process_events(
    mut glasses: Box<dyn ARGlasses>,
    quat_sender: mpsc::SyncSender<UnitQuaternion<f32>>,
) {
    let mut filter = ahrs::Madgwick::new(FUSION_PERIOD_US as f32 / 1e6, 0.1);

    let mut last_timestamp = 0;

    loop {
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
                println!("mag {}", timestamp);
            }
            _ => (),
        }
    }
}
