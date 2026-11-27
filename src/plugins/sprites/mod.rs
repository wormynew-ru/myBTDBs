mod towers_px;
mod units_px;

use bevy::image::{Image, ImageSampler};
use bevy::prelude::*;
use bevy::render::render_asset::RenderAssetUsages;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};

use crate::core::types::{BloonLayer, TowerType};

pub struct SpritesPlugin;

impl Plugin for SpritesPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SpriteAssets>()
            .add_systems(Startup, generate_pixel_sprites);
    }
}

#[derive(Resource, Default)]
pub struct SpriteAssets {
    pub bloon_normal: Handle<Image>,
    pub bloon_lead: Handle<Image>,
    pub bloon_rainbow: Handle<Image>,
    pub bloon_moab: Handle<Image>,
    pub bloon_bfb: Handle<Image>,
    pub bloon_zomg: Handle<Image>,
    pub overlay_camo: Handle<Image>,

    pub tower_dart: Handle<Image>,
    pub tower_cannon: Handle<Image>,
    pub tower_ice: Handle<Image>,
    pub tower_ninja: Handle<Image>,
    pub tower_village: Handle<Image>,
    pub tower_spike_factory: Handle<Image>,
    pub tower_commander: Handle<Image>,
    pub tower_military_base: Handle<Image>,
    pub tower_crook_boss: Handle<Image>,
    pub tower_crook_boss_gold: Handle<Image>,

    pub unit_jeep: Handle<Image>,
    pub unit_tank: Handle<Image>,
    pub unit_railtank: Handle<Image>,
    pub unit_crook: Handle<Image>,
    pub unit_mafia_enforcer: Handle<Image>,

    pub chipper_base: Handle<Image>,
    pub chipper_p1_1: Handle<Image>,
    pub chipper_p1_2: Handle<Image>,
    pub chipper_p1_3: Handle<Image>,
    pub chipper_p1_4: Handle<Image>,
    pub chipper_p2_1: Handle<Image>,
    pub chipper_p2_2: Handle<Image>,
    pub chipper_p2_3: Handle<Image>,

    pub proj_dart: Handle<Image>,
    pub proj_bomb: Handle<Image>,
    pub proj_shuriken: Handle<Image>,
    pub spike_pile: Handle<Image>,
    pub shred_particle: Handle<Image>,
    pub portal_entry: Handle<Image>,
    pub portal_exit: Handle<Image>,
}

impl SpriteAssets {
    pub fn bloon_handle(&self, layer: BloonLayer, lead: bool) -> Handle<Image> {
        match layer {
            BloonLayer::Moab => self.bloon_moab.clone(),
            BloonLayer::Bfb => self.bloon_bfb.clone(),
            BloonLayer::Zomg => self.bloon_zomg.clone(),
            BloonLayer::Rainbow => self.bloon_rainbow.clone(),
            _ if lead => self.bloon_lead.clone(),
            _ => self.bloon_normal.clone(),
        }
    }

    pub fn tower_handle(&self, tower_type: TowerType, path1: u8, path2: u8) -> Handle<Image> {
        match tower_type {
            TowerType::DartMonkey => self.tower_dart.clone(),
            TowerType::Cannon => self.tower_cannon.clone(),
            TowerType::IceTower => self.tower_ice.clone(),
            TowerType::NinjaMonkey => self.tower_ninja.clone(),
            TowerType::Village => self.tower_village.clone(),
            TowerType::SpikeFactory => self.tower_spike_factory.clone(),
            TowerType::Commander => self.tower_commander.clone(),
            TowerType::MilitaryBase => self.tower_military_base.clone(),
            TowerType::CrookBoss => {
                if path1 >= 4 {
                    self.tower_crook_boss_gold.clone()
                } else {
                    self.tower_crook_boss.clone()
                }
            }
            TowerType::Bloonchipper => {
                if path1 >= 4 {
                    self.chipper_p1_4.clone()
                } else if path2 >= 3 {
                    self.chipper_p2_3.clone()
                } else if path1 >= path2 && path1 > 0 {
                    match path1 {
                        1 => self.chipper_p1_1.clone(),
                        2 => self.chipper_p1_2.clone(),
                        _ => self.chipper_p1_3.clone(),
                    }
                } else if path2 > 0 {
                    match path2 {
                        1 => self.chipper_p2_1.clone(),
                        _ => self.chipper_p2_2.clone(),
                    }
                } else {
                    self.chipper_base.clone()
                }
            }
        }
    }

    pub fn unit_handle(&self, unit_tier: u8) -> Handle<Image> {
        match unit_tier {
            0..=2 => self.unit_jeep.clone(),
            3 => self.unit_tank.clone(),
            _ => self.unit_railtank.clone(),
        }
    }

    pub fn crook_unit_handle(&self, is_enforcer: bool) -> Handle<Image> {
        if is_enforcer {
            self.unit_mafia_enforcer.clone()
        } else {
            self.unit_crook.clone()
        }
    }
}

pub fn make_tex(
    images: &mut Assets<Image>,
    rows: [&str; 16],
    palette: &[(char, [u8; 4])],
) -> Handle<Image> {
    let mut data = vec![0u8; 16 * 16 * 4];
    for (y, row) in rows.iter().enumerate() {
        for (x, ch) in row.chars().take(16).enumerate() {
            let rgba = palette
                .iter()
                .find(|(k, _)| *k == ch)
                .map(|(_, c)| *c)
                .unwrap_or([0, 0, 0, 0]);
            let idx = (y * 16 + x) * 4;
            data[idx..idx + 4].copy_from_slice(&rgba);
        }
    }
    let mut img = Image::new(
        Extent3d {
            width: 16,
            height: 16,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        data,
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::default(),
    );
    img.sampler = ImageSampler::nearest();
    images.add(img)
}

fn generate_pixel_sprites(mut images: ResMut<Assets<Image>>, mut assets: ResMut<SpriteAssets>) {
    let w = [255, 255, 255, 255];
    let d = [15, 23, 42, 255];
    let h = [255, 255, 255, 190];

    assets.bloon_normal = make_tex(&mut images, [
        ".....DDDDDD.....",
        "...DDWWWWWWDD...",
        "..DWWWHHWWWWWD..",
        ".DWWWHHHWWWWWWD.",
        ".DWWWWWWWWWWWWD.",
        "DWWWWWWWWWWWWWWD",
        "DWWWWWWWWWWWWWWD",
        "DWWWWWWWWWWWWWWD",
        "DWWWWWWWWWWWWWWD",
        "DWWWWWWWWWWWWWWD",
        ".DWWWWWWWWWWWWD.",
        ".DWWWWWWWWWWWWD.",
        "..DWWWWWWWWWWWD.",
        "...DDWWWWWWDD...",
        ".....DDDDDD.....",
        "......DDDD......",
    ], &[('D', d), ('W', w), ('H', h)]);

    assets.bloon_lead = make_tex(&mut images, [
        ".....DDDDDD.....",
        "...DDGGGGGGDD...",
        "..DGGGLLGGGGGGD.",
        ".DGGGLLLGGGGGGGD",
        ".DGGGGGGGGGGGGGD",
        "DGGLLGGGGGGLLGGD",
        "DGGGGGGGGGGGGGGD",
        "DGGGGGGGGGGGGGGD",
        "DGGGGGGGGGGGGGGD",
        "DGGLLGGGGGGLLGGD",
        ".DGGGGGGGGGGGGGD",
        ".DGGGGGGGGGGGGGD",
        "..DGGGGGGGGGGGD.",
        "...DDGGGGGGDD...",
        ".....DDDDDD.....",
        "......DDDD......",
    ], &[('D', d), ('G', [107, 120, 140, 255]), ('L', [210, 224, 238, 255])]);

    assets.bloon_rainbow = make_tex(&mut images, [
        ".....DDDDDD.....",
        "...DDRRRRRRDD...",
        "..DRRROOOORRRD..",
        ".DOOOOOOOOOOOOD.",
        ".DYYYYYYYYYYYYD.",
        "DYYYYYYYYYYYYYYD",
        "DGGGGGGGGGGGGGGD",
        "DGGGGGGGGGGGGGGD",
        "DBBBBBBBBBBBBBBD",
        "DBBBBBBBBBBBBBBD",
        ".DPPPPPPPPPPPPD.",
        ".DPPPPPPPPPPPPD.",
        "..DRRRRRRRRRRD..",
        "...DDYYYYYYDD...",
        ".....DDDDDD.....",
        "......DDDD......",
    ], &[
        ('D', d),
        ('R', [239, 68, 68, 255]),
        ('O', [249, 115, 22, 255]),
        ('Y', [250, 204, 21, 255]),
        ('G', [34, 197, 94, 255]),
        ('B', [59, 130, 246, 255]),
        ('P', [236, 72, 153, 255]),
    ]);

    assets.overlay_camo = make_tex(&mut images, [
        "................",
        "................",
        "....CC..........",
        "...CCCC...CC....",
        ".........CCCC...",
        "..CC............",
        "..CCC....CCC....",
        ".........CCC....",
        ".....CCC........",
        ".....CCC...CC...",
        "..CC.......CC...",
        "....CCC.........",
        "................",
        "................",
        "................",
        "................",
    ], &[('C', [20, 83, 45, 220])]);

    let make_blimp = |images: &mut Assets<Image>, body: [u8; 4], stripe: [u8; 4], fin: [u8; 4]| {
        make_tex(images, [
            "................",
            "..FFDDDDDDDD....",
            ".FFDBBBBBBBBDD..",
            "FFDBBSSSSSSBBBD.",
            "DDBBSSSSSSSSBBBD",
            "DBBBSSSSSSSSBBBD",
            "DBBBBBBBBBBBBBBD",
            "DBBSSSSSSSSSSBBD",
            "DBBSSSSSSSSSSBBD",
            "DBBBBBBBBBBBBBBD",
            "DBBBSSSSSSSSBBBD",
            "DDBBSSSSSSSSBBBD",
            "FFDBBSSSSSSBBBD.",
            ".FFDBBBBBBBBDD..",
            "..FFDDDDDDDD....",
            "................",
        ], &[('D', d), ('B', body), ('S', stripe), ('F', fin)])
    };

    assets.bloon_moab = make_blimp(&mut images, [56, 189, 248, 255], [224, 242, 254, 255], [248, 250, 252, 255]);
    assets.bloon_bfb = make_blimp(&mut images, [220, 38, 38, 255], [30, 41, 59, 255], [250, 204, 21, 255]);
    assets.bloon_zomg = make_blimp(&mut images, [22, 101, 52, 255], [250, 204, 21, 255], [15, 23, 42, 255]);

    towers_px::populate_tower_and_proj_sprites(&mut images, &mut assets);
    units_px::populate_commander_and_unit_sprites(&mut images, &mut assets);
}
