pub mod history;
pub mod systems;

use bevy::prelude::*;

use crate::rendering::RenderingUpdateSet;

use self::{
    history::TimeHistoryState,
    systems::{
        playback_state, record_state, setup_time_feedback, setup_time_indicator,
        update_time_indicator, update_time_mode, update_time_trails,
    },
};

pub struct TimeReversalPlugin;

impl Plugin for TimeReversalPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<TimeHistoryState>()
            .add_systems(Startup, (setup_time_feedback, setup_time_indicator))
            .add_systems(
                Update,
                (
                    update_time_mode,
                    playback_state,
                    record_state,
                    update_time_indicator,
                    update_time_trails,
                )
                    .chain()
                    .after(RenderingUpdateSet),
            );
    }
}
