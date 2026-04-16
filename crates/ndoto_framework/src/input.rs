use bevy::prelude::*;
use ndoto_engine::EngineFixedSet;

#[derive(Resource, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct FixedPlayerInput {
    pub move_axis: IVec2,
    pub switch_to_3d: bool,
    pub switch_to_2d: bool,
    pub switch_to_1d: bool,
    pub toggle_four_d: bool,
    pub jump: bool,
    pub sprint: bool,
    pub rewind: bool,
    pub fast_forward: bool,
}

#[derive(Resource, Clone, Copy, Debug, Default, PartialEq, Eq)]
struct PendingPlayerInput {
    move_axis: IVec2,
    switch_to_3d: bool,
    switch_to_2d: bool,
    switch_to_1d: bool,
    toggle_four_d: bool,
    jump: bool,
    sprint: bool,
    rewind: bool,
    fast_forward: bool,
}

impl PendingPlayerInput {
    fn clear_edges(&mut self) {
        self.switch_to_3d = false;
        self.switch_to_2d = false;
        self.switch_to_1d = false;
        self.toggle_four_d = false;
        self.jump = false;
    }
}

impl From<PendingPlayerInput> for FixedPlayerInput {
    fn from(value: PendingPlayerInput) -> Self {
        Self {
            move_axis: value.move_axis,
            switch_to_3d: value.switch_to_3d,
            switch_to_2d: value.switch_to_2d,
            switch_to_1d: value.switch_to_1d,
            toggle_four_d: value.toggle_four_d,
            jump: value.jump,
            sprint: value.sprint,
            rewind: value.rewind,
            fast_forward: value.fast_forward,
        }
    }
}

pub struct PlayerInputPlugin;

impl Plugin for PlayerInputPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PendingPlayerInput>()
            .init_resource::<FixedPlayerInput>()
            .add_systems(PreUpdate, sample_player_input)
            .add_systems(
                FixedUpdate,
                latch_fixed_player_input.in_set(EngineFixedSet::Input),
            );
    }
}

fn sample_player_input(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut pending: ResMut<PendingPlayerInput>,
) {
    pending.move_axis = IVec2::new(
        axis_value(
            &keyboard_input,
            [KeyCode::KeyA, KeyCode::ArrowLeft],
            [KeyCode::KeyD, KeyCode::ArrowRight],
        ),
        axis_value(
            &keyboard_input,
            [KeyCode::KeyS, KeyCode::ArrowDown],
            [KeyCode::KeyW, KeyCode::ArrowUp],
        ),
    );
    pending.switch_to_3d |= keyboard_input.just_pressed(KeyCode::Digit1);
    pending.switch_to_2d |= keyboard_input.just_pressed(KeyCode::Digit2);
    pending.switch_to_1d |= keyboard_input.just_pressed(KeyCode::Digit3);
    pending.toggle_four_d |= keyboard_input.just_pressed(KeyCode::Digit4);
    pending.jump |= keyboard_input.just_pressed(KeyCode::Space);
    pending.sprint =
        keyboard_input.pressed(KeyCode::ShiftLeft) || keyboard_input.pressed(KeyCode::ShiftRight);
    pending.rewind =
        keyboard_input.pressed(KeyCode::Space) || keyboard_input.pressed(KeyCode::KeyR);
    pending.fast_forward = keyboard_input.pressed(KeyCode::KeyF);
}

fn axis_value(
    keyboard_input: &ButtonInput<KeyCode>,
    negative: [KeyCode; 2],
    positive: [KeyCode; 2],
) -> i32 {
    let negative_pressed = negative.into_iter().any(|key| keyboard_input.pressed(key));
    let positive_pressed = positive.into_iter().any(|key| keyboard_input.pressed(key));

    i32::from(positive_pressed) - i32::from(negative_pressed)
}

fn latch_fixed_player_input(
    mut pending: ResMut<PendingPlayerInput>,
    mut fixed_input: ResMut<FixedPlayerInput>,
) {
    *fixed_input = (*pending).into();
    pending.clear_edges();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clearing_edges_preserves_held_input() {
        let mut pending = PendingPlayerInput {
            move_axis: IVec2::new(1, 0),
            switch_to_3d: true,
            switch_to_2d: false,
            switch_to_1d: false,
            toggle_four_d: true,
            jump: true,
            sprint: true,
            rewind: true,
            fast_forward: false,
        };

        pending.clear_edges();

        assert!(!pending.switch_to_3d);
        assert!(!pending.toggle_four_d);
        assert!(!pending.jump);
        assert_eq!(pending.move_axis, IVec2::new(1, 0));
        assert!(pending.sprint);
        assert!(pending.rewind);
    }

    #[test]
    fn axis_value_balances_opposing_keys() {
        let mut input = ButtonInput::<KeyCode>::default();
        input.press(KeyCode::KeyA);
        input.press(KeyCode::KeyD);

        assert_eq!(
            axis_value(
                &input,
                [KeyCode::KeyA, KeyCode::ArrowLeft],
                [KeyCode::KeyD, KeyCode::ArrowRight]
            ),
            0
        );
    }
}
