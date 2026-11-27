use bevy::prelude::*;

use super::{make_tex, SpriteAssets};

fn make_chipper(
    images: &mut Assets<Image>,
    body: [u8; 4],
    accent: [u8; 4],
    triple: bool,
    flame: bool,
) -> Handle<Image> {
    let dark = [15, 23, 42, 255];
    let wheel = [30, 41, 59, 255];
    let metal = [148, 163, 184, 255];
    let pipe = [71, 85, 105, 255];
    let fire1 = [249, 115, 22, 255];
    let fire2 = [250, 204, 21, 255];

    let rows: [&str; 16] = if flame {
        [
            "....WW..........",
            "...WWWW.........",
            "..PPBBBB..MMMMFF",
            "..PPBBBBBBBBBBFF",
            "..BBBBBBBBBBBBYY",
            "..BBBABBBBBBBBFF",
            "..BBBBBBBBBBBBYY",
            "..BBBBBBBBBBBBYY",
            "..BBBBBBBBBBBBYY",
            "..BBBBBBBBBBBBYY",
            "..BBBABBBBBBBBFF",
            "..BBBBBBBBBBBBYY",
            "..PPBBBBBBBBBBFF",
            "...WWWW...MMMMFF",
            "....WW..........",
            "................",
        ]
    } else if triple {
        [
            "....WW..........",
            "...WWWW.........",
            "..PPBBBB..MMMMMM",
            "..PPBBBBBBBBMM..",
            "..BBBBBBBBBBBBMM",
            "..BBBABBBBBBBBM.",
            "..BBBBBBBBBBBBMM",
            "..BBBBBBBBBBMM..",
            "..BBBBBBBBBBMM..",
            "..BBBBBBBBBBBBMM",
            "..BBBABBBBBBBBM.",
            "..BBBBBBBBBBBBMM",
            "..PPBBBBBBBBMM..",
            "...WWWW...MMMMMM",
            "....WW..........",
            "................",
        ]
    } else {
        [
            "....WW..........",
            "...WWWW.........",
            "..PPBBBB........",
            "..PPBBBBBBBB....",
            "...BBBBBBBBBBMMM",
            "...BBBABBBBBBMMM",
            "...BBBBBBBBBBMMM",
            "...BBBBBBBBBBMMM",
            "...BBBBBBBBBBMMM",
            "...BBBBBBBBBBMMM",
            "...BBBABBBBBBMMM",
            "...BBBBBBBBBBMMM",
            "..PPBBBBBBBB....",
            "...WWWW.........",
            "....WW..........",
            "................",
        ]
    };

    make_tex(
        images,
        rows,
        &[
            ('B', body),
            ('A', accent),
            ('W', wheel),
            ('M', metal),
            ('P', pipe),
            ('F', fire1),
            ('Y', fire2),
            ('D', dark),
        ],
    )
}

pub fn populate_tower_and_proj_sprites(
    images: &mut Assets<Image>,
    assets: &mut SpriteAssets,
) {
    let w = [255, 255, 255, 255];
    let d = [15, 23, 42, 255];

    assets.tower_dart = make_tex(images, [
        "................",
        "....DDDDDDDD....",
        "...DBBBBBBBBD...",
        "..DBBFFFFFFBBD..",
        "..DBFFFFFFFFBD..",
        "..DBFFDDDDFFBD..",
        "..DBFFFFFFFFMMMM",
        "..DBFFFFFFFFMMMM",
        "..DBFFFFFFFFMMMM",
        "..DBFFDDDDFFBD..",
        "..DBFFFFFFFFBD..",
        "..DBBFFFFFFBBD..",
        "...DBBBBBBBBD...",
        "....DDDDDDDD....",
        "................",
        "................",
    ], &[('D', d), ('B', [161, 82, 24, 255]), ('F', [232, 193, 144, 255]), ('M', [226, 232, 240, 255])]);

    assets.tower_cannon = make_tex(images, [
        "....WWWW........",
        "...WWWWWW.......",
        "..DDDDDDDD......",
        "..DGGGGGGDDDDDD.",
        "..DGGGGGGGGGGGGD",
        "..DGGGGGGGGGGGGD",
        "..DGGGGGGGGGGGGD",
        "..DGGGGGGGGGGGGD",
        "..DGGGGGGGGGGGGD",
        "..DGGGGGGGGGGGGD",
        "..DGGGGGGDDDDDD.",
        "..DDDDDDDD......",
        "...WWWWWW.......",
        "....WWWW........",
        "................",
        "................",
    ], &[('D', d), ('W', [30, 41, 59, 255]), ('G', [71, 85, 105, 255])]);

    assets.tower_ice = make_tex(images, [
        ".......DD.......",
        "......DIID......",
        ".....DIIIID.....",
        "...DDIILLIIID...",
        "..DIILLLLLLII...",
        ".DIILLWWWWLLII..",
        "DIILWWWWWWWWLID.",
        "DIILWWWWWWWWLID.",
        ".DIILLWWWWLLII..",
        "..DIILLLLLLII...",
        "...DDIILLIIID...",
        ".....DIIIID.....",
        "......DIID......",
        ".......DD.......",
        "................",
        "................",
    ], &[('D', d), ('I', [14, 165, 233, 255]), ('L', [125, 211, 252, 255]), ('W', [224, 242, 254, 255])]);

    assets.tower_ninja = make_tex(images, [
        "................",
        "....DDDDDDDD....",
        "...DNNNNNNNND...",
        "..DNNRRRRRRNND..",
        "..DNRRFFFFRRND..",
        "..DNNFFFFFFNND..",
        "..DNNNNNNNNNNMSS",
        "..DNNNNNNNNNNMSS",
        "..DNNNNNNNNNND..",
        "..DNNFFFFFFNND..",
        "..DNRRFFFFRRND..",
        "..DNNRRRRRRNND..",
        "...DNNNNNNNND...",
        "....DDDDDDDD....",
        "................",
        "................",
    ], &[
        ('D', d),
        ('N', [30, 41, 59, 255]),
        ('R', [220, 38, 38, 255]),
        ('F', [232, 193, 144, 255]),
        ('M', [226, 232, 240, 255]),
        ('S', [248, 250, 252, 255]),
    ]);

    assets.tower_village = make_tex(images, [
        ".......YY.......",
        "......YYYY......",
        "....DDRRRRDD....",
        "...DRRRRRRRRD...",
        "..DRRRRRRRRRRD..",
        ".DRRWWWWWWWWWRD.",
        ".DWWWWWWWWWWWWD.",
        ".DWWDDDDDDDDWWD.",
        ".DWWDDDDDDDDWWD.",
        ".DWWWWWWWWWWWWD.",
        ".DWWBBWWWWBBWWD.",
        ".DWWBBWWWWBBWWD.",
        "..DDDDDDDDDDDD..",
        "................",
        "................",
        "................",
    ], &[
        ('D', d),
        ('Y', [250, 204, 21, 255]),
        ('R', [220, 38, 38, 255]),
        ('W', [226, 207, 168, 255]),
        ('B', [120, 74, 36, 255]),
    ]);

    assets.chipper_base = make_chipper(images, [37, 99, 235, 255], [147, 197, 253, 255], false, false);
    assets.chipper_p1_1 = make_chipper(images, [234, 179, 8, 255], [254, 240, 138, 255], false, false);
    assets.chipper_p1_2 = make_chipper(images, [249, 115, 22, 255], [254, 215, 170, 255], false, false);
    assets.chipper_p1_3 = make_chipper(images, [220, 38, 38, 255], [254, 202, 202, 255], false, false);
    assets.chipper_p1_4 = make_chipper(images, [185, 28, 28, 255], [250, 204, 21, 255], false, true);
    assets.chipper_p2_1 = make_chipper(images, [132, 204, 22, 255], [217, 249, 157, 255], false, false);
    assets.chipper_p2_2 = make_chipper(images, [22, 163, 74, 255], [134, 239, 172, 255], false, false);
    assets.chipper_p2_3 = make_chipper(images, [126, 34, 206, 255], [250, 204, 21, 255], true, false);

    assets.proj_dart = make_tex(images, [
        "................", "................", "................", "................",
        "................", "....RR..........", "..RRRRBBBBBBMMM.", "..RRRRBBBBBBMMMM",
        "..RRRRBBBBBBMMM.", "....RR..........", "................", "................",
        "................", "................", "................", "................",
    ], &[('R', [239, 68, 68, 255]), ('B', [180, 120, 60, 255]), ('M', [226, 232, 240, 255])]);

    assets.proj_bomb = make_tex(images, [
        "................", "................", "..........YY....", ".........YOO....",
        ".....DDDDD......", "...DDGGGGGD.....", "..DGGGGGGGGD....", "..DGGLLGGGGD....",
        "..DGGGGGGGGD....", "..DGGGGGGGGD....", "...DDGGGGDD.....", ".....DDDD.......",
        "................", "................", "................", "................",
    ], &[('D', d), ('G', [30, 41, 59, 255]), ('L', [148, 163, 184, 255]), ('Y', [250, 204, 21, 255]), ('O', [249, 115, 22, 255])]);

    assets.proj_shuriken = make_tex(images, [
        ".......MM.......", "......MMMM......", "......MMMM......", ".......MM.......",
        ".......MM.......", ".......MM.......", ".MMMMMMDDMMMMMM.", "MMMMMMMDDMMMMMMM",
        ".MMMMMMDDMMMMMM.", ".......MM.......", ".......MM.......", ".......MM.......",
        "......MMMM......", "......MMMM......", ".......MM.......", "................",
    ], &[('M', [226, 232, 240, 255]), ('D', d)]);

    assets.shred_particle = make_tex(images, [
        "................", "................", "....WWWW........", "...WWWWWW.......",
        "...WWWWWW..WWW..", "....WWWW..WWWW..", "..........WWWW..", ".....WWW........",
        "....WWWWW.......", "....WWWWW.......", ".....WWW........", "................",
        "................", "................", "................", "................",
    ], &[('W', w)]);

    assets.portal_entry = make_tex(images, [
        "..DDDDDDDDDDDD..", ".DGGGGGGGGGGGGD.", ".DGLLLLLLLLLLGD.", ".DGLWWWWWWWWLGD.",
        ".DGLWWGGGGWWLGD.", ".DGLWGGGGGGWLGD.", ".DGLWGGGGGGWLGD.", ".DGLWWGGGGWWLGD.",
        ".DGLWWWWWWWWLGD.", ".DGLLLLLLLLLLGD.", ".DGGGGGGGGGGGGD.", "..DDDDDDDDDDDD..",
        "................", "................", "................", "................",
    ], &[('D', d), ('G', [22, 163, 74, 255]), ('L', [74, 222, 128, 255]), ('W', w)]);

    assets.portal_exit = make_tex(images, [
        "..DDDDDDDDDDDD..", ".DRRRRRRRRRRRRD.", ".DRLLLLLLLLLLRD.", ".DRLWWWWWWWWLRD.",
        ".DRLWWDDDDWWLRD.", ".DRLWDDDDDDWLRD.", ".DRLWDDDDDDWLRD.", ".DRLWWDDDDWWLRD.",
        ".DRLWWWWWWWWLRD.", ".DRLLLLLLLLLLRD.", ".DRRRRRRRRRRRRD.", "..DDDDDDDDDDDD..",
        "................", "................", "................", "................",
    ], &[('D', d), ('R', [220, 38, 38, 255]), ('L', [248, 113, 113, 255]), ('W', w)]);

    assets.tower_spike_factory = make_tex(images, [
        "................",
        "..DDDDDDDDDDDD..",
        ".DYYBBYYBBYYBBD.",
        ".DBGGGGGGGGGGBD.",
        ".DYGLLRRRRLLGYD.",
        ".DBGGRDDDDDRGBD.",
        ".DYGRDDDDDDDRYD.",
        ".DBGRDDDDDDDRBD.",
        ".DYGRDDDDDDDRYD.",
        ".DBGGRDDDDDRGBD.",
        ".DYGLLRRRRLLGYD.",
        ".DBGGGGGGGGGGBD.",
        ".DYYBBYYBBYYBBD.",
        "..DDDDDDDDDDDD..",
        "................",
        "................",
    ], &[
        ('D', d),
        ('B', [30, 41, 59, 255]),
        ('Y', [250, 204, 21, 255]),
        ('G', [100, 116, 139, 255]),
        ('L', [148, 163, 184, 255]),
        ('R', [203, 213, 225, 255]),
    ]);

    assets.spike_pile = make_tex(images, [
        "................",
        "....D......D....",
        "...DWD....DMD...",
        "..DWWWD..DMMMD..",
        ".DWWLWWDDMMLMMD.",
        "..DWWWD..DMMMD..",
        "...DWD....DMD...",
        "....D..D...D....",
        "......DWD.......",
        ".....DWWWD......",
        "...DDWWLWWDD....",
        "..DWWWWLWWWWD...",
        "...DDWWLWWDD....",
        ".....DWWWD......",
        "......DWD.......",
        ".......D........",
    ], &[
        ('D', d),
        ('W', [226, 232, 240, 255]),
        ('L', [255, 255, 255, 255]),
        ('M', [148, 163, 184, 255]),
    ]);

    assets.tower_farm = make_tex(images, [
        "................",
        ".....DDDDDD.....",
        "...DDGGGGGGDD...",
        "..DGGYYGGYYGGD..",
        ".DGGYYYGGYYYGGD.",
        ".DGGGGSSSSGGGGD.",
        ".DGGSSGGGGSSGGD.",
        ".DGYYGGGGGGYYGD.",
        ".DGGGSSGGSSGGGD.",
        "..DGGSSSSSSGGD..",
        "..DBBBBBBBBBBD..",
        "..DBBBBBBBBBBD..",
        "..DB.DBBBB.DBD..",
        "..DB.DBBBB.DBD..",
        "..DDDDDDDDDDDD..",
        "................",
    ], &[
        ('D', d),
        ('G', [34, 197, 94, 255]),
        ('S', [21, 128, 61, 255]),
        ('Y', [250, 204, 21, 255]),
        ('B', [133, 77, 14, 255]),
    ]);
}
