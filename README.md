# okulekrano
**okulekrano** is a program for using augmented reality (AR) glasses as a virtual screen for Linux desktop.

## Features
- 3DoF head tracking using accelerometer and gyroscope
- Screen capture and rendering is all done in GPU
- Supports multiple brands of AR glasses such as XREAL or Rokid
  - All glasses supported by [ar-driver-rs](https://github.com/badicsalex/ar-drivers-rs)
  - Although, only tested on XREAL Air

## Requirements
- Linux
- [wlroots](https://gitlab.freedesktop.org/wlroots/wlroots/)-based Wayland compositor
  - Such as [labwc](https://labwc.github.io/) or [Sway](https://swaywm.org/)
  - Includes the standard desktop environment of Raspberry Pi OS

## Usage
1. Configure a virtual screen output in addition to the AR glasses (cf. [VirtualOutputAdd](https://labwc.github.io/labwc-actions.5.html#entry_action_name=virtualoutputadd_output_name=value) in labwc or [swaymsg create_output](https://wiki.archlinux.org/title/Sway#Create_headless_outputs))
3. Create config file like this and save to `~/.config/okulekrano/default-config.toml`
```toml
[capture]
output_name = "Virtual-1"

[glasses]
monitor_name = "HDMI-A-2"
```
4. Just launch the `okulekrano` executable

## Notes
The name *okulekrano* means *eye screen* in Esperanto.
