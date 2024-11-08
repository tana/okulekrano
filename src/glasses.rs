use ahrs::Ahrs;
use ar_drivers::{ARGlasses, GlassesEvent};
use na::{UnitQuaternion, Vector3};
use std::sync::mpsc;
use std::thread::{self, JoinHandle};

const FUSION_PERIOD_US: u64 = 10000;

pub struct GlassesController {
    #[allow(dead_code)]
    join_handle: JoinHandle<()>,
    quat_receiver: mpsc::Receiver<UnitQuaternion<f32>>,
}

impl GlassesController {
    pub fn new() -> Self {
        let glasses = ar_drivers::any_glasses().expect("AR glasses not found. Maybe permission issues of hidraw device.");

        let (quat_sender, quat_receiver) = mpsc::sync_channel(0);

        let join_handle = thread::spawn(move || {
            process_events(glasses, quat_sender);
        });

        Self {
            join_handle,
            quat_receiver,
        }
    }

    pub fn get_pose(&self) -> UnitQuaternion<f32> {
        self.quat_receiver.recv().unwrap()
    }
}

fn process_events(mut glasses: Box<dyn ARGlasses>, quat_sender: mpsc::SyncSender<UnitQuaternion<f32>>) {
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
                let quat = filter.update_imu(
                    &Vector3::new(gyroscope.x, gyroscope.y, gyroscope.z),
                    &Vector3::new(accelerometer.x, accelerometer.y, accelerometer.z),
                ).unwrap();
                // Skip sending if the receiver is not received the old pose yet
                match quat_sender.try_send(quat.clone()) {
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
