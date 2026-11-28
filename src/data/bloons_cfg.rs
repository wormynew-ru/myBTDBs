use bevy::prelude::*;

use crate::core::types::BloonLayer;

pub const WAVE_DURATION_SECS: f32 = 16.0;

#[derive(Debug, Clone, Copy)]
pub struct BloonCfg {
    pub hp: i32,
    pub speed: f32,
    pub size: f32,
    pub weight: u32,
    pub color: Color,
    pub rbe: i32,
    pub child: Option<(BloonLayer, u8)>,
}

pub fn bloon_cfg(layer: BloonLayer) -> BloonCfg {
    match layer {
        BloonLayer::Red => BloonCfg { hp: 1, speed: 64.0, size: 20.0, weight: 1, color: Color::srgb(0.94, 0.27, 0.27), rbe: 1, child: None },
        BloonLayer::Blue => BloonCfg { hp: 1, speed: 80.0, size: 22.0, weight: 1, color: Color::srgb(0.23, 0.51, 0.96), rbe: 2, child: Some((BloonLayer::Red, 1)) },
        BloonLayer::Green => BloonCfg { hp: 1, speed: 98.0, size: 24.0, weight: 1, color: Color::srgb(0.13, 0.77, 0.37), rbe: 3, child: Some((BloonLayer::Blue, 1)) },
        BloonLayer::Yellow => BloonCfg { hp: 1, speed: 126.0, size: 25.0, weight: 1, color: Color::srgb(0.92, 0.70, 0.05), rbe: 4, child: Some((BloonLayer::Green, 1)) },
        BloonLayer::Pink => BloonCfg { hp: 1, speed: 152.0, size: 26.0, weight: 1, color: Color::srgb(0.93, 0.28, 0.60), rbe: 5, child: Some((BloonLayer::Yellow, 1)) },
        BloonLayer::Rainbow => BloonCfg { hp: 1, speed: 136.0, size: 27.0, weight: 1, color: Color::srgb(0.95, 0.45, 0.85), rbe: 11, child: Some((BloonLayer::Pink, 2)) },
        BloonLayer::Ceramic => BloonCfg { hp: 10, speed: 104.0, size: 30.0, weight: 3, color: Color::srgb(0.80, 0.48, 0.12), rbe: 32, child: Some((BloonLayer::Rainbow, 2)) },
        BloonLayer::Moab => BloonCfg { hp: 200, speed: 48.0, size: 40.0, weight: 3, color: Color::srgb(0.22, 0.74, 0.97), rbe: 180, child: Some((BloonLayer::Ceramic, 4)) },
        BloonLayer::Bfb => BloonCfg { hp: 700, speed: 34.0, size: 46.0, weight: 3, color: Color::srgb(0.86, 0.15, 0.15), rbe: 600, child: Some((BloonLayer::Moab, 4)) },
        BloonLayer::Zomg => BloonCfg { hp: 4000, speed: 24.0, size: 52.0, weight: 99, color: Color::srgb(0.10, 0.42, 0.20), rbe: 1800, child: Some((BloonLayer::Bfb, 4)) },
    }
}

#[derive(Debug, Clone, Copy)]
pub struct SendPackCfg {
    pub short_label: &'static str,
    pub layer: BloonLayer,
    pub camo: bool,
    pub lead: bool,
    pub count: u32,
    pub spacing: f32,
    pub cost: i32,
    pub eco_gain: i32,
    pub unlock_wave: u32,
    pub badge_color: Color,
}

pub fn wave_ramp_multipliers(wave: u32) -> (f32, f32) {
    if wave < 30 {
        (1.0, 1.0)
    } else {
        let extra = (wave - 29) as f32;
        (1.0 + 0.22 * extra, 1.0 + 0.03 * extra)
    }
}

pub const SEND_PACKS: &[SendPackCfg] = &[
    SendPackCfg { short_label: "6x Red", layer: BloonLayer::Red, camo: false, lead: false, count: 6, spacing: 0.14, cost: 45, eco_gain: 3, unlock_wave: 1, badge_color: Color::srgb(0.88, 0.25, 0.25) },
    SendPackCfg { short_label: "5x Blue", layer: BloonLayer::Blue, camo: false, lead: false, count: 5, spacing: 0.22, cost: 70, eco_gain: 5, unlock_wave: 1, badge_color: Color::srgb(0.22, 0.48, 0.90) },
    SendPackCfg { short_label: "5x Grn", layer: BloonLayer::Green, camo: false, lead: false, count: 5, spacing: 0.16, cost: 115, eco_gain: 8, unlock_wave: 2, badge_color: Color::srgb(0.14, 0.72, 0.35) },
    SendPackCfg { short_label: "5x Ylw", layer: BloonLayer::Yellow, camo: false, lead: false, count: 5, spacing: 0.18, cost: 165, eco_gain: 11, unlock_wave: 3, badge_color: Color::srgb(0.85, 0.65, 0.08) },
    SendPackCfg { short_label: "Camo Y", layer: BloonLayer::Yellow, camo: true, lead: false, count: 4, spacing: 0.18, cost: 230, eco_gain: 14, unlock_wave: 4, badge_color: Color::srgb(0.45, 0.65, 0.15) },
    SendPackCfg { short_label: "4x Lead", layer: BloonLayer::Green, camo: false, lead: true, count: 4, spacing: 0.26, cost: 290, eco_gain: 16, unlock_wave: 5, badge_color: Color::srgb(0.42, 0.48, 0.56) },
    SendPackCfg { short_label: "7x Pink", layer: BloonLayer::Pink, camo: false, lead: false, count: 7, spacing: 0.11, cost: 360, eco_gain: 16, unlock_wave: 6, badge_color: Color::srgb(0.88, 0.25, 0.55) },
    SendPackCfg { short_label: "3x Ceram", layer: BloonLayer::Ceramic, camo: false, lead: false, count: 3, spacing: 0.35, cost: 620, eco_gain: 22, unlock_wave: 8, badge_color: Color::srgb(0.75, 0.45, 0.12) },
    SendPackCfg { short_label: "6x Rain", layer: BloonLayer::Rainbow, camo: false, lead: false, count: 6, spacing: 0.22, cost: 750, eco_gain: 30, unlock_wave: 9, badge_color: Color::srgb(0.92, 0.42, 0.78) },
    SendPackCfg { short_label: "4x C.Cer", layer: BloonLayer::Ceramic, camo: true, lead: false, count: 4, spacing: 0.30, cost: 1100, eco_gain: 25, unlock_wave: 10, badge_color: Color::srgb(0.55, 0.52, 0.16) },
    SendPackCfg { short_label: "1x MOAB", layer: BloonLayer::Moab, camo: false, lead: false, count: 1, spacing: 0.50, cost: 1200, eco_gain: -50, unlock_wave: 11, badge_color: Color::srgb(0.22, 0.74, 0.97) },
    SendPackCfg { short_label: "3x MOAB", layer: BloonLayer::Moab, camo: false, lead: false, count: 3, spacing: 0.75, cost: 3400, eco_gain: -180, unlock_wave: 13, badge_color: Color::srgb(0.14, 0.56, 0.85) },
    SendPackCfg { short_label: "1x BFB", layer: BloonLayer::Bfb, camo: false, lead: false, count: 1, spacing: 0.80, cost: 5500, eco_gain: -350, unlock_wave: 15, badge_color: Color::srgb(0.86, 0.15, 0.15) },
    SendPackCfg { short_label: "2x BFB", layer: BloonLayer::Bfb, camo: false, lead: false, count: 2, spacing: 1.10, cost: 10500, eco_gain: -750, unlock_wave: 17, badge_color: Color::srgb(0.68, 0.10, 0.15) },
    SendPackCfg { short_label: "1x ZOMG", layer: BloonLayer::Zomg, camo: false, lead: false, count: 1, spacing: 1.00, cost: 13000, eco_gain: -900, unlock_wave: 19, badge_color: Color::srgb(0.12, 0.52, 0.25) },
    SendPackCfg { short_label: "3x ZOMG", layer: BloonLayer::Zomg, camo: false, lead: false, count: 3, spacing: 1.40, cost: 36000, eco_gain: -2500, unlock_wave: 22, badge_color: Color::srgb(0.08, 0.36, 0.16) },
];

pub struct WaveGroupCfg {
    pub layer: BloonLayer,
    pub count: u32,
    pub spacing: f32,
    pub camo: bool,
    pub lead: bool,
}

pub fn wave_groups(wave: u32) -> Vec<WaveGroupCfg> {
    match wave {
        1 => vec![WaveGroupCfg {
            layer: BloonLayer::Red,
            count: 10,
            spacing: 0.55,
            camo: false,
            lead: false,
        }],
        2 => vec![
            WaveGroupCfg { layer: BloonLayer::Red, count: 8, spacing: 0.45, camo: false, lead: false },
            WaveGroupCfg { layer: BloonLayer::Blue, count: 6, spacing: 0.5, camo: false, lead: false },
        ],
        3 => vec![
            WaveGroupCfg { layer: BloonLayer::Blue, count: 10, spacing: 0.36, camo: false, lead: false },
            WaveGroupCfg { layer: BloonLayer::Green, count: 5, spacing: 0.45, camo: false, lead: false },
        ],
        4 => vec![
            WaveGroupCfg { layer: BloonLayer::Green, count: 10, spacing: 0.34, camo: false, lead: false },
            WaveGroupCfg { layer: BloonLayer::Yellow, count: 5, spacing: 0.4, camo: false, lead: false },
        ],
        5 => vec![
            WaveGroupCfg { layer: BloonLayer::Green, count: 8, spacing: 0.32, camo: false, lead: false },
            WaveGroupCfg { layer: BloonLayer::Yellow, count: 4, spacing: 0.35, camo: true, lead: false },
        ],
        6 => vec![
            WaveGroupCfg { layer: BloonLayer::Yellow, count: 10, spacing: 0.28, camo: false, lead: false },
            WaveGroupCfg { layer: BloonLayer::Green, count: 4, spacing: 0.45, camo: false, lead: true },
        ],
        7 => vec![
            WaveGroupCfg { layer: BloonLayer::Yellow, count: 8, spacing: 0.24, camo: false, lead: false },
            WaveGroupCfg { layer: BloonLayer::Pink, count: 7, spacing: 0.28, camo: false, lead: false },
        ],
        _ => {
            let scale = (wave - 6).min(12);
            let mut groups = vec![
                WaveGroupCfg {
                    layer: BloonLayer::Yellow,
                    count: 8 + scale,
                    spacing: 0.2,
                    camo: false,
                    lead: false,
                },
                WaveGroupCfg {
                    layer: BloonLayer::Pink,
                    count: 6 + scale,
                    spacing: 0.18,
                    camo: wave % 2 == 0,
                    lead: false,
                },
                WaveGroupCfg {
                    layer: BloonLayer::Green,
                    count: 4 + scale / 2,
                    spacing: 0.28,
                    camo: false,
                    lead: true,
                },
            ];
            if wave >= 9 {
                groups.push(WaveGroupCfg {
                    layer: BloonLayer::Rainbow,
                    count: (wave - 7).min(8),
                    spacing: 0.32,
                    camo: false,
                    lead: false,
                });
            }
            if wave >= 10 {
                groups.push(WaveGroupCfg {
                    layer: BloonLayer::Ceramic,
                    count: (wave - 8).min(8),
                    spacing: 0.42,
                    camo: false,
                    lead: false,
                });
            }
            if wave >= 13 {
                groups.push(WaveGroupCfg {
                    layer: BloonLayer::Moab,
                    count: ((wave - 11) / 2).clamp(1, 3),
                    spacing: 1.6,
                    camo: false,
                    lead: false,
                });
            }
            if wave >= 17 {
                groups.push(WaveGroupCfg {
                    layer: BloonLayer::Bfb,
                    count: 1,
                    spacing: 2.0,
                    camo: false,
                    lead: false,
                });
            }
            if wave >= 21 {
                groups.push(WaveGroupCfg {
                    layer: BloonLayer::Zomg,
                    count: 1,
                    spacing: 2.5,
                    camo: false,
                    lead: false,
                });
            }
            groups
        }
    }
}
