use std::{env, fs, path::PathBuf};

use bevy::{
    app::AppExit,
    prelude::*,
    render::view::screenshot::{Screenshot, save_to_disk},
};
use ndoto_framework::dimension::{DimensionState, SpatialMode};

pub fn configure(app: &mut App) {
    let Some(output_dir) = env::var_os("NDOTO_CAPTURE_DIR").map(PathBuf::from) else {
        return;
    };

    fs::create_dir_all(&output_dir).expect("failed to create NDOTO_CAPTURE_DIR");

    app.insert_resource(AutoCaptureState::new(output_dir))
        .add_systems(Update, run_auto_capture);
}

#[derive(Resource)]
struct AutoCaptureState {
    output_dir: PathBuf,
    step_index: usize,
    stage: CaptureStage,
    timer: Timer,
}

impl AutoCaptureState {
    fn new(output_dir: PathBuf) -> Self {
        Self {
            output_dir,
            step_index: 0,
            stage: CaptureStage::ApplyMode,
            timer: Timer::from_seconds(0.5, TimerMode::Once),
        }
    }
}

#[derive(Clone, Copy)]
enum CaptureStage {
    ApplyMode,
    CaptureFrame,
    Advance,
    Exit,
}

#[derive(Clone, Copy)]
struct CaptureStep {
    label: &'static str,
    mode: SpatialMode,
    four_d: bool,
}

const CAPTURE_STEPS: [CaptureStep; 4] = [
    CaptureStep {
        label: "3d",
        mode: SpatialMode::ThreeD,
        four_d: false,
    },
    CaptureStep {
        label: "2d",
        mode: SpatialMode::TwoD,
        four_d: false,
    },
    CaptureStep {
        label: "1d",
        mode: SpatialMode::OneD,
        four_d: false,
    },
    CaptureStep {
        label: "4d",
        mode: SpatialMode::ThreeD,
        four_d: true,
    },
];

fn run_auto_capture(
    mut commands: Commands,
    time: Res<Time>,
    mut state: ResMut<AutoCaptureState>,
    mut dimension_state: ResMut<DimensionState>,
    mut app_exit: MessageWriter<AppExit>,
) {
    if !state.timer.tick(time.delta()).just_finished() {
        return;
    }

    match state.stage {
        CaptureStage::ApplyMode => {
            let Some(step) = CAPTURE_STEPS.get(state.step_index).copied() else {
                state.stage = CaptureStage::Exit;
                state.timer = Timer::from_seconds(1.5, TimerMode::Once);
                return;
            };

            dimension_state.set_spatial_mode(step.mode);
            if dimension_state.is_four_d() != step.four_d {
                dimension_state.toggle_four_d();
            }

            state.stage = CaptureStage::CaptureFrame;
            let capture_delay = if step.mode == SpatialMode::TwoD {
                6.0
            } else {
                1.0
            };
            state.timer = Timer::from_seconds(capture_delay, TimerMode::Once);
        }
        CaptureStage::CaptureFrame => {
            let step = CAPTURE_STEPS[state.step_index];
            let path =
                state
                    .output_dir
                    .join(format!("{:02}-{}.png", state.step_index + 1, step.label));

            commands
                .spawn(Screenshot::primary_window())
                .observe(save_to_disk(path));

            state.stage = CaptureStage::Advance;
            state.timer = Timer::from_seconds(0.8, TimerMode::Once);
        }
        CaptureStage::Advance => {
            state.step_index += 1;
            state.stage = CaptureStage::ApplyMode;
            state.timer = Timer::from_seconds(0.35, TimerMode::Once);
        }
        CaptureStage::Exit => {
            app_exit.write(AppExit::Success);
        }
    }
}
