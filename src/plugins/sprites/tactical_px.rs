use bevy::prelude::*;

use super::{make_tex, SpriteAssets};

pub fn populate_tactical_tower_sprites(
    images: &mut Assets<Image>,
    assets: &mut SpriteAssets,
) {
    let d = [15, 23, 42, 255];

    // Saboteur: cyan-black cyber jammer dish with glowing signal rings
    assets.tower_saboteur = make_tex(
        images,
        [
            "................",
            "....DDDDDDDD....",
            "...DCCCCCCCCD...",
            "..DCBBBBBBBBCD..",
            "..DCBMWWWWMBCD..",
            "..DCBWCCCCWBCD..",
            "..DCBMCWWCMBCD..",
            "..DCBMWWWWMBCD..",
            "..DCBWCCCCWBCD..",
            "..DCBMMMMMMBCD..",
            "..DCBBBBBBBBCD..",
            "...DCCCCCCCCD...",
            "....DDDDDDDD....",
            "................",
            "................",
            "................",
        ],
        &[
            ('D', d),
            ('C', [6, 182, 212, 255]),
            ('B', [30, 41, 59, 255]),
            ('M', [236, 72, 153, 255]),
            ('W', [224, 242, 254, 255]),
        ],
    );

    // Scrap Collector: industrial orange-steel crusher hopper with gear center
    assets.tower_scrap_collector = make_tex(
        images,
        [
            "................",
            "..DDDDDDDDDDDD..",
            ".DOOOOOOOOOOOOD.",
            ".DOSSSSSSSSSSOD.",
            ".DOSDDDDDDDDOSD.",
            ".DOSDGGGGGGDSOD.",
            ".DOSDGYDDYGDSOD.",
            ".DOSDGDDDDGDSOD.",
            ".DOSDGYDDYGDSOD.",
            ".DOSDGGGGGGDSOD.",
            ".DOSDDDDDDDDOSD.",
            ".DOSSSSSSSSSSOD.",
            ".DOOOOOOOOOOOOD.",
            "..DDDDDDDDDDDD..",
            "................",
            "................",
        ],
        &[
            ('D', d),
            ('O', [234, 88, 12, 255]),
            ('S', [100, 116, 139, 255]),
            ('G', [71, 85, 105, 255]),
            ('Y', [250, 204, 21, 255]),
        ],
    );

    // Mirage Generator: holographic prism projector with triple cyan emitters
    assets.tower_mirage_generator = make_tex(
        images,
        [
            ".......CC.......",
            "......CCCC......",
            "....DDPPPPDD....",
            "...DPBBBBBBPD...",
            "..DPBCCCCCCBPD..",
            "..DPBCWWWWCBPD..",
            "CCDPBCWWWWCBPDCC",
            "CCDPBCCCCCCBPDCC",
            "..DPBCWWWWCBPD..",
            "..DPBCCCCCCBPD..",
            "...DPBBBBBBPD...",
            "....DDPPPPDD....",
            "......CCCC......",
            ".......CC.......",
            "................",
            "................",
        ],
        &[
            ('D', d),
            ('P', [30, 58, 138, 255]),
            ('B', [15, 23, 42, 255]),
            ('C', [56, 189, 248, 255]),
            ('W', [224, 242, 254, 255]),
        ],
    );

    // Jam Static Overlay: jagged interference bars shown over jammed towers
    assets.jam_static = make_tex(
        images,
        [
            "................",
            "..CCCC....MMMM..",
            "..CC....MMMM....",
            "....WWWW..CC....",
            "..MMMM......CCCC",
            "....CC..WWWW....",
            "................",
            "..WWWW....MMMM..",
            "....CCCC....CC..",
            "..MMMM..WWWW....",
            "......CCCC......",
            "..CC......MMMM..",
            "....WWWW........",
            "................",
            "................",
            "................",
        ],
        &[
            ('C', [34, 211, 238, 235]),
            ('M', [244, 114, 182, 235]),
            ('W', [255, 255, 255, 240]),
        ],
    );
}
