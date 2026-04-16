use bevy::prelude::*;

#[derive(Resource, Clone, Debug, PartialEq, Eq)]
pub struct DimensionState {
    pub spatial_mode: SpatialMode,
    pub temporal_mode: TemporalMode,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum SpatialMode {
    OneD,
    TwoD,
    #[default]
    ThreeD,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum TemporalMode {
    #[default]
    Live,
    FourD,
}

impl Default for DimensionState {
    fn default() -> Self {
        Self {
            spatial_mode: SpatialMode::ThreeD,
            temporal_mode: TemporalMode::Live,
        }
    }
}

impl DimensionState {
    pub fn is_four_d(&self) -> bool {
        self.temporal_mode == TemporalMode::FourD
    }

    pub fn set_spatial_mode(&mut self, mode: SpatialMode) {
        self.spatial_mode = mode;
    }

    pub fn toggle_four_d(&mut self) {
        self.temporal_mode = if self.is_four_d() {
            TemporalMode::Live
        } else {
            TemporalMode::FourD
        };
    }
}

pub struct DimensionPlugin;

impl Plugin for DimensionPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<DimensionState>();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dimension_state_defaults_to_3d_live() {
        let state = DimensionState::default();
        assert_eq!(state.spatial_mode, SpatialMode::ThreeD);
        assert_eq!(state.temporal_mode, TemporalMode::Live);
    }

    #[test]
    fn toggling_four_d_preserves_spatial_mode() {
        let mut state = DimensionState::default();
        state.set_spatial_mode(SpatialMode::TwoD);
        state.toggle_four_d();

        assert!(state.is_four_d());
        assert_eq!(state.spatial_mode, SpatialMode::TwoD);
    }
}
