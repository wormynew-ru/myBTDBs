use bevy::prelude::*;

use super::{make_tex, SpriteAssets};

pub fn populate_commander_and_unit_sprites(
    images: &mut Assets<Image>,
    assets: &mut SpriteAssets,
) {
    let d = [15, 23, 42, 255];

    assets.tower_commander = make_tex(
        images,
        [
            "................",
            "....DDDDDDDD....",
            "...DBBBBBBBBD...",
            "..DBBBBBBBBBBD..",
            "..DBBBGGGGGBBBD.",
            "..DBBDDDDDDBBMMM",
            "..DBBFFFFFFBBMMM",
            "..DBFFDDDDFFBD..",
            "..DBFFFFFFFFBD..",
            "..DBBFFFFFFBBD..",
            "...DBBBBBBBBD...",
            "...DDDDDDDDDD...",
            "................",
            "................",
            "................",
            "................",
        ],
        &[
            ('D', d),
            ('B', [29, 78, 216, 255]),
            ('G', [250, 204, 21, 255]),
            ('F', [232, 193, 144, 255]),
            ('M', [148, 163, 184, 255]),
        ],
    );

    assets.tower_military_base = make_tex(
        images,
        [
            "................",
            ".DDDDDDDDDDDDDD.",
            ".DGGGGGGGGGGGGD.",
            ".DGGLLLLLLLLGGD.",
            ".DGGDDDDDDDDGGD.",
            ".DGGDMMMMMMDGGD.",
            ".DGGDMMMMMMDGGD.",
            ".DGGDMMMMMMDGGD.",
            ".DGGDDDDDDDDGGD.",
            ".DGGGGGGGGGGGGD.",
            ".DGYYGGGGGGYYGD.",
            ".DGGGGGGGGGGGGD.",
            ".DDDDDDDDDDDDDD.",
            "................",
            "................",
            "................",
        ],
        &[
            ('D', d),
            ('G', [63, 98, 18, 255]),
            ('L', [101, 163, 13, 255]),
            ('M', [51, 65, 85, 255]),
            ('Y', [250, 204, 21, 255]),
        ],
    );

    assets.unit_jeep = make_tex(
        images,
        [
            "................",
            "..WWW......WWW..",
            "..DDD......DDD..",
            ".DGGGGGGGGGGGGD.",
            ".DGGGGGGGGGGGGD.",
            ".DGGWWCCCCWWGGD.",
            ".DGGWCCCCCCWGGD.",
            ".DGGGGGGGGGGGGD.",
            ".DGGGGGGGGGGGGD.",
            ".DGGWWCCCCWWGGD.",
            ".DGGWCCCCCCWGGD.",
            ".DGGGGGGGGGGGGD.",
            "..DDD......DDD..",
            "..WWW......WWW..",
            "................",
            "................",
        ],
        &[
            ('D', d),
            ('W', [30, 41, 59, 255]),
            ('G', [77, 124, 15, 255]),
            ('C', [148, 163, 184, 255]),
        ],
    );

    assets.unit_tank = make_tex(
        images,
        [
            "................",
            ".TTTTTTTTTTTT...",
            ".TDDDDDDDDDDT...",
            ".TDGGGGGGGGDT...",
            ".TDGGLLLLGGDT...",
            ".TDGLGGGGGLDTTTT",
            ".TDGLGGGGGLDDDDD",
            ".TDGLGGGGGLDDDDD",
            ".TDGLGGGGGLDTTTT",
            ".TDGGLLLLGGDT...",
            ".TDGGGGGGGGDT...",
            ".TDDDDDDDDDDT...",
            ".TTTTTTTTTTTT...",
            "................",
            "................",
            "................",
        ],
        &[
            ('D', d),
            ('T', [30, 41, 59, 255]),
            ('G', [63, 98, 18, 255]),
            ('L', [132, 204, 22, 255]),
        ],
    );

    assets.unit_railtank = make_tex(
        images,
        [
            "................",
            ".TTTTTTTTTTTT...",
            ".TDDDDDDDDDDT...",
            ".TDSSSSSSSSDT...",
            ".TDSSCCCCSSDT...",
            ".TDSCBBBBCCDDDDD",
            ".TDSCBCCCCBBBBCC",
            ".TDSCBCCCCBBBBCC",
            ".TDSCBBBBCCDDDDD",
            ".TDSSCCCCSSDT...",
            ".TDSSSSSSSSDT...",
            ".TDDDDDDDDDDT...",
            ".TTTTTTTTTTTT...",
            "................",
            "................",
            "................",
        ],
        &[
            ('D', d),
            ('T', [15, 23, 42, 255]),
            ('S', [71, 85, 105, 255]),
            ('B', [30, 64, 175, 255]),
            ('C', [56, 189, 248, 255]),
        ],
    );

    let make_crook_boss = |images: &mut Assets<Image>, suit: [u8; 4], band: [u8; 4], gun: [u8; 4]| {
        make_tex(
            images,
            [
                "................",
                "....DDDDDDDD....",
                "...DSSSSSSSSDD..",
                "..DSHHHHHHHHSD..",
                "..DSHBBBBBBHSD..",
                "..DSHHHHHHHHSD..",
                "..DSHHDDDDHHSMMM",
                "..DSHHFFFFHHSMMM",
                "..DSHHDDDDHHSD..",
                "..DSHHHHHHHHSD..",
                "..DSHBBBBBBHSD..",
                "..DSHHHHHHHHSD..",
                "...DSSSSSSSSDD..",
                "....DDDDDDDD....",
                "................",
                "................",
            ],
            &[
                ('D', d),
                ('S', suit),
                ('H', [24, 24, 27, 255]),
                ('B', band),
                ('F', [232, 193, 144, 255]),
                ('M', gun),
            ],
        )
    };

    assets.tower_crook_boss = make_crook_boss(
        images,
        [39, 39, 42, 255],
        [248, 250, 252, 255],
        [148, 163, 184, 255],
    );
    assets.tower_crook_boss_gold = make_crook_boss(
        images,
        [202, 138, 4, 255],
        [250, 204, 21, 255],
        [250, 204, 21, 255],
    );

    assets.unit_crook = make_tex(
        images,
        [
            "................",
            ".....DDDDDD.....",
            "....DSSSSSSD....",
            "...DSHHHHHHSD...",
            "...DSHWWWWSHD...",
            "...DSHHHHHHSD...",
            "...DSHDDDDHSMMM.",
            "...DSHFFFFHSMMM.",
            "...DSHHHHHHSD...",
            "...DSHWWWWSHD...",
            "...DSHHHHHHSD...",
            "....DSSSSSSD....",
            ".....DDDDDD.....",
            "................",
            "................",
            "................",
        ],
        &[
            ('D', d),
            ('S', [39, 39, 42, 255]),
            ('H', [82, 82, 91, 255]),
            ('W', [226, 232, 240, 255]),
            ('F', [232, 193, 144, 255]),
            ('M', [148, 163, 184, 255]),
        ],
    );

    assets.unit_mafia_enforcer = make_tex(
        images,
        [
            "................",
            "....DDDDDDDD....",
            "...DGGGGGGGGD...",
            "..DGGHHHHHHGGD..",
            "..DGHGGGGGGHGD..",
            "..DGHHHHHHHHGD..",
            "..DGHDDDDDDHGMMM",
            "..DGHFFFFFFHGMMM",
            "..DGHHHHHHHHGD..",
            "..DGHGGGGGGHGD..",
            "..DGGHHHHHHGGD..",
            "...DGGGGGGGGD...",
            "....DDDDDDDD....",
            "................",
            "................",
            "................",
        ],
        &[
            ('D', d),
            ('H', [24, 24, 27, 255]),
            ('G', [250, 204, 21, 255]),
            ('F', [232, 193, 144, 255]),
            ('M', [203, 213, 225, 255]),
        ],
    );
}
