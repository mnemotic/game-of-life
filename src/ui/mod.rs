//
// Copyright (c) 2023 Martin Green. All rights reserved.
//

use bevy::prelude::*;
use bevy_egui::egui::Layout;
use bevy_egui::{egui, EguiContexts, EguiPlugin};
use egui_extras::{Size, StripBuilder};

use crate::game::{Life, SimulationConfig, SimulationUpdateTimer};
use crate::input::InputAction;
use crate::{ui, GameState};


pub mod widgets;


pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(EguiPlugin)
            .add_systems(
                PreUpdate,
                absorb_egui_inputs
                    .after(bevy_egui::systems::process_input_system)
                    .before(bevy_egui::EguiSet::BeginFrame),
            )
            .add_systems(Update, draw_controls_ui);
    }
}


fn draw_controls_ui(
    state: Res<'_, State<GameState>>,
    life: Res<'_, Life>,
    mut config: ResMut<'_, SimulationConfig>,
    mut timer: ResMut<'_, SimulationUpdateTimer>,
    mut contexts: EguiContexts<'_, '_>,
    mut actions: EventWriter<'_, InputAction>,
) {
    let mut paused = *state.get() == GameState::Paused;
    egui::Window::new("Controls")
        .resizable(false)
        .collapsible(true)
        .movable(true)
        .show(contexts.ctx_mut(), |ui| {
            egui::Grid::new("controls")
                .num_columns(2)
                .spacing([20.0, 4.0])
                .striped(true)
                .show(ui, |ui| {
                    ui.label("Pause");
                    ui.add(ui::widgets::toggle(&mut paused));
                    ui.end_row();

                    let mut tps = config.ticks_per_second;
                    ui.label("Speed (tps)")
                        .on_hover_text_at_pointer("Ticks per second.");
                    if ui.add(egui::Slider::new(&mut tps, 1..=64)).changed() {
                        config.ticks_per_second = tps;
                        *timer = SimulationUpdateTimer(Timer::from_seconds(
                            1.0 / tps as f32,
                            TimerMode::Repeating,
                        ));
                    }
                    ui.end_row();

                    let gen = life.generation;
                    ui.label("Generation");
                    ui.label(format!("{gen}"));
                    ui.end_row();
                });

            ui.separator();

            let vh = ui.spacing().interact_size.y;
            StripBuilder::new(ui)
                .cell_layout(Layout::centered_and_justified(egui::Direction::LeftToRight))
                .sizes(Size::exact(vh), 1)
                .vertical(|mut strip| {
                    strip.strip(|builder| {
                        builder.sizes(Size::remainder(), 2).horizontal(|mut strip| {
                            strip.cell(|ui| {
                                if ui.button("Rewind").clicked() {
                                    actions.send(InputAction::PauseSimulation);
                                    actions.send(InputAction::RewindSimulation);
                                }
                            });

                            strip.cell(|ui| {
                                if ui.button("Advance").clicked() {
                                    actions.send(InputAction::PauseSimulation);
                                    actions.send(InputAction::AdvanceSimulation);
                                }
                            });
                        });
                    });
                })
        });
    match state.get() {
        GameState::Paused if !paused => actions.send(InputAction::UnpauseSimulation),
        GameState::Running if paused => actions.send(InputAction::PauseSimulation),
        _ => {}
    };
}


// @CREDIT: <https://github.com/mvlabat/bevy_egui/issues/47#issuecomment-1703964969>
fn absorb_egui_inputs(
    mut mouse: ResMut<'_, Input<MouseButton>>,
    mut contexts: EguiContexts<'_, '_>,
) {
    if contexts.ctx_mut().is_pointer_over_area() {
        mouse.reset_all();
    }
}
