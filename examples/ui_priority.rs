//! Inputs consumed by UI and not propagated to actions.
//! In order to run this example pass `--features egui_priority,bevy_egui/render,bevy_egui/default_fonts` to cargo.

mod player_box;

use bevy::{color::palettes::tailwind::NEUTRAL_900, prelude::*};
use bevy_egui::{EguiContexts, EguiPlugin, egui::Window};
use bevy_enhanced_input::prelude::*;

use player_box::{DEFAULT_SPEED, PlayerBox, PlayerBoxPlugin};

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            EguiPlugin,
            EnhancedInputPlugin,
            PlayerBoxPlugin,
            GamePlugin,
        ))
        .run();
}

struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.add_input_context::<PlayerBox>()
            .add_observer(binding)
            .add_observer(apply_movement)
            .add_observer(zoom)
            .add_systems(Startup, spawn)
            .add_systems(Update, draw_egui);
    }
}

fn spawn(mut commands: Commands) {
    commands.spawn(Camera2d);
    commands.spawn(PlayerBox);

    // Setup simple node with text using Bevy UI.
    commands
        .spawn(Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            align_items: AlignItems::Start,
            justify_content: JustifyContent::End,
            ..Default::default()
        })
        .with_children(|parent| {
            parent
                .spawn((
                    Node {
                        margin: UiRect::all(Val::Px(15.0)),
                        padding: UiRect::all(Val::Px(15.0)),
                        ..Default::default()
                    },
                    BackgroundColor(NEUTRAL_900.into()),
                ))
                .with_children(|parent| {
                    parent.spawn((
                        Interaction::default(), // All UI nodes with `Interaction` component will intercept all mouse input.
                        Text::new("Bevy UI"),
                        TextColor(Color::WHITE),
                        TextFont {
                            font_size: 30.0,
                            ..Default::default()
                        },
                    ));
                });
        });
}

fn binding(mut trigger: Trigger<Binding<PlayerBox>>) {
    trigger
        .bind::<Move>()
        .to(Cardinal::wasd_keys())
        .with_modifiers((
            DeadZone::default(),
            SmoothNudge::default(),
            Scale::splat(DEFAULT_SPEED),
        ));
    trigger
        .bind::<Zoom>()
        .to(Input::mouse_wheel().with_modifiers(SwizzleAxis::YXZ))
        .with_modifiers(Scale::splat(3.0));
}

fn draw_egui(mut text_edit: Local<String>, mut contexts: EguiContexts) {
    Window::new("Egui").show(contexts.ctx_mut(), |ui| {
        ui.label("Type text:");
        ui.text_edit_singleline(&mut *text_edit);
    });
}

fn apply_movement(trigger: Trigger<Fired<Move>>, mut players: Query<&mut Transform>) {
    let mut transform = players.get_mut(trigger.entity()).unwrap();
    transform.translation += trigger.value.extend(0.0);
}

fn zoom(trigger: Trigger<Fired<Zoom>>, mut players: Query<&mut Transform>) {
    // Scale entity to fake zoom.
    let mut transform = players.get_mut(trigger.entity()).unwrap();
    transform.scale += Vec3::splat(trigger.value);
}

#[derive(Debug, InputAction)]
#[input_action(output = Vec2)]
struct Move;

#[derive(Debug, InputAction)]
#[input_action(output = f32)]
struct Zoom;
