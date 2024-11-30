//! Inputs consumed by UI and not propagated to actions.
//! In order to run this example pass `--features egui_priority,bevy_egui/render,bevy_egui/default_fonts,bevy/default_font` to cargo.

mod player_box;

use bevy::{color::palettes::tailwind::NEUTRAL_900, prelude::*};
use bevy_egui::{egui::Window, EguiContexts, EguiPlugin};
use bevy_enhanced_input::prelude::*;

use player_box::{PlayerBox, PlayerBoxBundle, PlayerBoxPlugin, DEFAULT_SPEED};

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
            .add_systems(Startup, Self::spawn)
            .add_systems(Update, Self::draw_egui)
            .observe(Self::apply_movement)
            .observe(Self::zoom);
    }
}

impl GamePlugin {
    fn spawn(mut commands: Commands) {
        commands.spawn(Camera2dBundle::default());
        commands.spawn(PlayerBoxBundle::default());

        // Setup simple node with text using Bevy UI.
        commands
            .spawn(NodeBundle {
                style: Style {
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    align_items: AlignItems::Start,
                    justify_content: JustifyContent::End,
                    ..Default::default()
                },
                ..Default::default()
            })
            .with_children(|parent| {
                parent
                    .spawn(NodeBundle {
                        style: Style {
                            margin: UiRect::all(Val::Px(15.0)),
                            padding: UiRect::all(Val::Px(15.0)),
                            ..Default::default()
                        },
                        background_color: NEUTRAL_900.into(),
                        ..Default::default()
                    })
                    .with_children(|parent| {
                        parent.spawn((
                            Interaction::default(), // All UI nodes with `Interaction` component will intercept all mouse input.
                            TextBundle::from_section(
                                "Bevy UI",
                                TextStyle {
                                    font_size: 30.0,
                                    color: Color::WHITE,
                                    ..default()
                                },
                            ),
                        ));
                    });
            });
    }

    fn draw_egui(mut text_edit: Local<String>, mut contexts: EguiContexts) {
        Window::new("Egui").show(contexts.ctx_mut(), |ui| {
            ui.label("Type text:");
            ui.text_edit_singleline(&mut *text_edit);
        });
    }

    fn apply_movement(trigger: Trigger<ActionEvent<Move>>, mut players: Query<&mut Transform>) {
        let event = trigger.event();
        if event.is_fired() {
            let mut transform = players.get_mut(trigger.entity()).unwrap();
            transform.translation += event.value.as_axis3d();
        }
    }

    fn zoom(trigger: Trigger<ActionEvent<Zoom>>, mut players: Query<&mut Transform>) {
        /// Scale entity to fake zoom.
        let event = trigger.event();
        if trigger.event().is_fired() {
            let mut transform = players.get_mut(trigger.entity()).unwrap();
            transform.scale += Vec3::splat(event.value.as_axis1d());
        }
    }
}

impl InputContext for PlayerBox {
    fn context_instance(_world: &World, _entity: Entity) -> ContextInstance {
        let mut ctx = ContextInstance::default();

        ctx.bind::<Move>()
            .with_wasd()
            .with_modifier(DeadZone::default())
            .with_modifier(DeltaLerp::default())
            .with_modifier(Scale::splat(DEFAULT_SPEED));
        ctx.bind::<Zoom>()
            .to(Input::mouse_wheel().with_modifier(SwizzleAxis::YXZ))
            .with_modifier(Scale::splat(3.0));

        ctx
    }
}

#[derive(Debug, InputAction)]
#[input_action(output = Vec2)]
struct Move;

#[derive(Debug, InputAction)]
#[input_action(output = f32)]
struct Zoom;
