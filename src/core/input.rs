use bevy::prelude::*;

#[derive(Resource, Debug, Default)]
pub struct PointerInput {
    pub just_pressed: bool,
    pub world_pos: Option<Vec2>,
    pub is_touch: bool,
}

pub fn update_pointer_input(
    mouse: Res<ButtonInput<MouseButton>>,
    touches: Res<Touches>,
    windows: Query<&Window>,
    cameras: Query<(&Camera, &GlobalTransform), With<Camera2d>>,
    ui_interactions: Query<&Interaction, With<Button>>,
    mut pointer: ResMut<PointerInput>,
) {
    pointer.just_pressed = false;
    pointer.world_pos = None;
    pointer.is_touch = false;

    // Ignore world clicks when interacting with any UI button
    let ui_busy = ui_interactions
        .iter()
        .any(|i| *i == Interaction::Pressed || *i == Interaction::Hovered);

    let mut screen_pos: Option<Vec2> = None;

    if let Some(touch) = touches.iter_just_pressed().next() {
        screen_pos = Some(touch.position());
        pointer.just_pressed = !ui_busy;
        pointer.is_touch = true;
    } else if mouse.just_pressed(MouseButton::Left) {
        if let Ok(win) = windows.get_single() {
            screen_pos = win.cursor_position();
            pointer.just_pressed = !ui_busy;
        }
    } else if let Ok(win) = windows.get_single() {
        screen_pos = win.cursor_position();
    }

    if let (Some(sp), Ok((camera, cam_tf))) = (screen_pos, cameras.get_single()) {
        if let Ok(wp) = camera.viewport_to_world_2d(cam_tf, sp) {
            pointer.world_pos = Some(wp);
        }
    }
}
