#![allow(dead_code)] // ponytail: unused until Task 7 wires the module; remove then

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Monitor {
    /// Device path from IDesktopWallpaper, stable across reboots.
    pub id: String,
    /// 1-based display number shown in the UI.
    pub number: u32,
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
}
