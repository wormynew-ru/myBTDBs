use bevy::prelude::*;

use super::{make_tex, SpriteAssets};

pub fn populate_new_tower_sprites(
    images: &mut Assets<Image>,
    assets: &mut SpriteAssets,
) {
    let d = [15, 23, 42, 255];

    // Glue Gunner (0-0..0-4 / 1-0..3-0): monkey with lime-yellow glue tank & nozzle (+X)
    assets.tower_glue = make_tex(
        images,
        [
            "................",
            "....DDDDDDDD....",
            "...DGGGGGGGGD...",
            "..DGBBFFFFBBGD..",
            "..DGFFFFFFFFGD..",
            "..DGFFDDDDFFGD..",
            "..DGGGFFFFGGMMMM",
            "..DYYYYGGGGGLLLL",
            "..DYYYYGGGGGLLLL",
            "..DGFFDDDDFFGD..",
            "..DGFFFFFFFFGD..",
            "..DGBBFFFFBBGD..",
            "...DGGGGGGGGD...",
            "....DDDDDDDD....",
            "................",
            "................",
        ],
        &[
            ('D', d),
            ('G', [101, 163, 13, 255]),
            ('B', [161, 82, 24, 255]),
            ('F', [232, 193, 144, 255]),
            ('Y', [250, 204, 21, 255]),
            ('L', [190, 242, 100, 255]),
            ('M', [148, 163, 184, 255]),
        ],
    );

    // 4-0 Bloon Liquefier: hazmat yellow-green suit & glowing acid nozzle (+X)
    assets.tower_glue_liquefier = make_tex(
        images,
        [
            "................",
            "....DDDDDDDD....",
            "...DYYYYYYYYD...",
            "..DYCCCCCCCCYD..",
            "..DYCCCCCCCCYD..",
            "..DYYDDDDDDYYD..",
            "..DYCCCCCCCCMMMM",
            "..DGGGGYYYYGLLLL",
            "..DGGGGYYYYGLLLL",
            "..DYYDDDDDDYYD..",
            "..DYCCCCCCCCYD..",
            "..DYCCCCCCCCYD..",
            "...DYYYYYYYYD...",
            "....DDDDDDDD....",
            "................",
            "................",
        ],
        &[
            ('D', d),
            ('Y', [234, 179, 8, 255]),
            ('C', [56, 189, 248, 255]),
            ('G', [22, 163, 74, 255]),
            ('L', [163, 230, 53, 255]),
            ('M', [71, 85, 105, 255]),
        ],
    );

    // Soul Eater: mystical obsidian-purple totem crystal with glowing magenta runes
    assets.tower_soul_eater = make_tex(
        images,
        [
            ".......DD.......",
            "......DPPD......",
            ".....DPVVPD.....",
            "....DPVMMVPD....",
            "...DPVVVVVVPD...",
            "..DPVMMMMMMVPD..",
            "..DPVMDDDDMVPD..",
            "..DPVMDCWDMVPD..",
            "..DPVMDDDDMVPD..",
            "..DPVMMMMMMVPD..",
            "...DPVVMMVVPD...",
            "...DDPVVVVPDD...",
            "....DPMMMMPD....",
            "....DDDDDDDD....",
            "................",
            "................",
        ],
        &[
            ('D', d),
            ('P', [59, 7, 100, 255]),
            ('V', [147, 51, 234, 255]),
            ('M', [232, 121, 249, 255]),
            ('C', [56, 189, 248, 255]),
            ('W', [255, 255, 255, 255]),
        ],
    );

    // Necromancer: hooded dark-violet mage with glowing cyan-purple orb staff (+X)
    assets.tower_necromancer = make_tex(
        images,
        [
            "................",
            "....DDDDDDDD....",
            "...DPPPPPPPPD...",
            "..DPPVVVVVVPPD..",
            "..DPVDDDDDDVPD..",
            "..DPVDCCCCDVPD..",
            "..DPVDDDDDDVPDCC",
            "..DPPPPPPPPPBBBB",
            "..DPPPPPPPPPBBBB",
            "..DPVDDDDDDVPDCC",
            "..DPVVVVVVVVPD..",
            "..DPPVVVVVVPPD..",
            "...DPPPPPPPPD...",
            "....DDDDDDDD....",
            "................",
            "................",
        ],
        &[
            ('D', d),
            ('P', [46, 16, 101, 255]),
            ('V', [107, 33, 168, 255]),
            ('C', [34, 211, 238, 255]),
            ('B', [120, 74, 36, 255]),
        ],
    );

    assets.proj_glue = make_tex(
        images,
        [
            "................",
            "................",
            "................",
            "......DDDD......",
            "....DDLLLLDD....",
            "...DLLYYYYLLD...",
            "..DLYYYYYYYYLD..",
            "..DLYYWWWWYYLD..",
            "..DLYYYYYYYYLD..",
            "...DLLYYYYLLD...",
            "....DDLLLLDD....",
            "......DDDD......",
            "................",
            "................",
            "................",
            "................",
        ],
        &[
            ('D', d),
            ('L', [163, 230, 53, 255]),
            ('Y', [234, 179, 8, 255]),
            ('W', [254, 249, 195, 255]),
        ],
    );

    assets.proj_dark_orb = make_tex(
        images,
        [
            "................",
            "................",
            "................",
            ".....DDDDDD.....",
            "....DPPPPPPD....",
            "...DPPVVVVPPD...",
            "..DPVVCCCCVVPD..",
            "..DPVCCWWCCVPD..",
            "..DPVVCCCCVVPD..",
            "...DPPVVVVPPD...",
            "....DPPPPPPD....",
            ".....DDDDDD.....",
            "................",
            "................",
            "................",
            "................",
        ],
        &[
            ('D', d),
            ('P', [88, 28, 135, 255]),
            ('V', [168, 85, 247, 255]),
            ('C', [34, 211, 238, 255]),
            ('W', [224, 242, 254, 255]),
        ],
    );

    assets.overlay_glue = make_tex(
        images,
        [
            "................",
            "................",
            "....LL..........",
            "...LLLL....YY...",
            "..........YYYY..",
            "..YY............",
            "..YYY.....LLL...",
            "..........LLL...",
            ".....LLL........",
            "....LLLL...YY...",
            "...........YY...",
            "...LLL..........",
            "................",
            "................",
            "................",
            "................",
        ],
        &[('L', [163, 230, 53, 235]), ('Y', [250, 204, 21, 235])],
    );

    assets.ghost_bloon = make_tex(
        images,
        [
            ".....DDDDDD.....",
            "...DDCCCCCCDD...",
            "..DCCCCCCCCCCD..",
            ".DCCVVCCCCVVCCD.",
            ".DCCVVCCCCVVCCD.",
            "DCCCCCCCCCCCCCCD",
            "DCCDDCCCCCCDDCCD",
            "DCCWCCCCCCCCWCCD",
            "DCCCCCCCCCCCCCCD",
            "DCCCCVVVVVVCCCCD",
            ".DCCCCCCCCCCCCD.",
            ".DCCDCCCCCCDCCD.",
            "..DCDDDCCDDDCD..",
            "...DD..DD..DD...",
            "................",
            "................",
        ],
        &[
            ('D', [30, 27, 75, 230]),
            ('C', [103, 232, 249, 210]),
            ('V', [192, 132, 252, 225]),
            ('W', [255, 255, 255, 245]),
        ],
    );

    assets.ghost_knight = make_tex(
        images,
        [
            "....VV....VV....",
            "...DVVDDDDVVD...",
            "..DPPPPPPPPPPD..",
            ".DPSSSSSSSSSSPD.",
            ".DPSCCCCCCCCSPD.",
            "DPSCCWWWWWWCCSPD",
            "DPSSSSSSSSSSSSPD",
            "DPSSDDDDDDDDSSPD",
            "DPSSDCCDDCCDSSPD",
            "DPSSDDDDDDDDSSPD",
            ".DPSSSSSSSSSSPD.",
            ".DPPPPPPPPPPPPD.",
            "..DPPVVVVVVPPD..",
            "...DDPPPPPPDD...",
            ".....DDDDDD.....",
            "................",
        ],
        &[
            ('D', d),
            ('P', [59, 7, 100, 245]),
            ('V', [192, 132, 252, 250]),
            ('S', [148, 163, 184, 245]),
            ('C', [34, 211, 238, 255]),
            ('W', [224, 242, 254, 255]),
        ],
    );

    assets.rift_portal = make_tex(
        images,
        [
            ".....DDDDDD.....",
            "...DDPPPPPPDD...",
            "..DPPVVVVVVPPD..",
            ".DPVVCCCCCCVVPD.",
            ".DPVCCWWWWCCVPD.",
            "DPVCWWDDDDWWCVPD",
            "DPVCWDDDDDDWCVPD",
            "DPVCWDDDDDDWCVPD",
            "DPVCWWDDDDWWCVPD",
            ".DPVCCWWWWCCVPD.",
            ".DPVVCCCCCCVVPD.",
            "..DPPVVVVVVPPD..",
            "...DDPPPPPPDD...",
            ".....DDDDDD.....",
            "................",
            "................",
        ],
        &[
            ('D', [15, 23, 42, 220]),
            ('P', [88, 28, 135, 230]),
            ('V', [168, 85, 247, 235]),
            ('C', [34, 211, 238, 240]),
            ('W', [224, 242, 254, 250]),
        ],
    );
}
