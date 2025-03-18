use core::{error::Error, fmt::Write};
use std::fs;

use bevy::{
    input::{ButtonState, common_conditions::*, keyboard::KeyboardInput, mouse::MouseButtonInput},
    prelude::*,
    ui::FocusPolicy,
};
use bevy_enhanced_input::prelude::*;
use serde::{Deserialize, Serialize};

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, EnhancedInputPlugin, KeybindingMenuPlugin))
        .run();
}

struct KeybindingMenuPlugin;

impl Plugin for KeybindingMenuPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(ClearColor(Color::srgb(0.9, 0.9, 0.9)))
            .add_systems(Startup, setup)
            .add_systems(
                Update,
                (
                    update_button_text,
                    (
                        cancel_binding
                            .never_param_warn()
                            .run_if(input_just_pressed(KeyCode::Escape)),
                        bind.never_param_warn(),
                    )
                        .chain(),
                ),
            )
            .add_systems(PostUpdate, update_button_background);
    }
}

const SETTINGS_PATH: &str = "target/settings.ron";
const GAP: Val = Val::Px(10.0);
const PADDING: UiRect = UiRect::all(Val::Px(15.0));
const PANEL_BACKGROUND: BackgroundColor = BackgroundColor(Color::srgb(0.8, 0.8, 0.8));
const DARK_TEXT: TextColor = TextColor(Color::srgb(0.1, 0.1, 0.1));

fn setup(mut commands: Commands) {
    let settings = match KeyboardSettings::read(SETTINGS_PATH) {
        Ok(settings) => {
            info!("loading settings from '{SETTINGS_PATH}'");
            settings
        }
        Err(e) => {
            info!(
                "switching unable to load settings from '{SETTINGS_PATH}', switching to defaults: {e}"
            );
            Default::default()
        }
    };

    commands.spawn(Camera2d);

    // We use separate root node to let dialogs cover the whole UI.
    commands
        .spawn((Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            ..Default::default()
        },))
        .with_children(|parent| {
            parent
                .spawn(Node {
                    flex_direction: FlexDirection::Column,
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    padding: PADDING,
                    row_gap: GAP,
                    ..Default::default()
                })
                .with_children(|parent| {
                    setup_actions(parent, &settings);

                    parent
                        .spawn(Node {
                            align_items: AlignItems::End,
                            width: Val::Percent(100.0),
                            height: Val::Percent(100.0),
                            justify_content: JustifyContent::End,
                            ..Default::default()
                        })
                        .with_children(|parent| {
                            parent
                                .spawn(SettingsButton)
                                .with_child(Text::new("Apply"))
                                .observe(apply);
                        });
                });
        });

    commands.insert_resource(settings);
}

/// Creates [`SettingsField`] from passed field.
///
/// Strips everything before first `.` in order to turn "settings.field_name" into just "field_name".
macro_rules! settings_field {
    ($path:expr) => {{
        let _validate_field = &$path;
        let full_path = stringify!($path);
        let field_name = full_path
            .split_once('.')
            .map(|(_, s)| s)
            .unwrap_or(full_path);
        SettingsField(field_name)
    }};
}

/// Stores name of the [`Settings`] field.
///
/// Used to utilize reflection when applying settings.
#[derive(Component, Clone, Copy)]
struct SettingsField(&'static str);

/// Number of input columns.
const INPUTS_PER_ACTION: usize = 3;

fn setup_actions(parent: &mut ChildBuilder, settings: &KeyboardSettings) -> Entity {
    parent
        .spawn(Node {
            display: Display::Grid,
            column_gap: GAP,
            row_gap: GAP,
            grid_template_columns: vec![GridTrack::auto(); INPUTS_PER_ACTION + 1],
            ..Default::default()
        })
        .with_children(|parent| {
            // We could utilzie reflection to iterate over fields,
            // but in real application you most likely want to have a nice and translatable text on buttons.
            setup_action_row(
                parent,
                "Forward",
                &settings.forward,
                settings_field!(settings.forward),
            );
            setup_action_row(
                parent,
                "Left",
                &settings.left,
                settings_field!(settings.left),
            );
            setup_action_row(
                parent,
                "Backward",
                &settings.backward,
                settings_field!(settings.backward),
            );
            setup_action_row(
                parent,
                "Right",
                &settings.right,
                settings_field!(settings.right),
            );
            setup_action_row(
                parent,
                "Jump",
                &settings.jump,
                settings_field!(settings.jump),
            );
            setup_action_row(parent, "Run", &settings.run, settings_field!(settings.run));
        })
        .id()
}

fn setup_action_row(
    parent: &mut ChildBuilder,
    name: &'static str,
    inputs: &[Input],
    field: SettingsField,
) {
    parent.spawn((Text::new(name), DARK_TEXT));
    for index in 0..INPUTS_PER_ACTION {
        parent
            .spawn(Node {
                column_gap: GAP,
                align_items: AlignItems::Center,
                ..Default::default()
            })
            .with_children(|parent| {
                let button_entity = parent
                    .spawn((
                        field,
                        Name::new(name),
                        InputButton {
                            input: inputs.get(index).copied(),
                        },
                    ))
                    .with_child(Text::default()) // Will be updated automatically on `InputButton` insertion
                    .observe(show_binding_dialog)
                    .id();
                parent
                    .spawn(DeleteButton { button_entity })
                    .with_child(Text::new("X"))
                    .observe(delete_binding);
            });
    }
}

fn delete_binding(
    trigger: Trigger<Pointer<Click>>,
    mut input_buttons: Query<(&Name, &mut InputButton)>,
    delete_buttons: Query<&DeleteButton>,
) {
    let delete_button = delete_buttons.get(trigger.entity()).unwrap();
    let (name, mut input_button) = input_buttons
        .get_mut(delete_button.button_entity)
        .expect("delete button should point to an input button");
    info!("deleting binding for '{name}'");
    input_button.input = None;
}

fn show_binding_dialog(
    trigger: Trigger<Pointer<Click>>,
    mut commands: Commands,
    root_entity: Single<Entity, (With<Node>, Without<Parent>)>,
    names: Query<&Name>,
) {
    let name = names.get(trigger.entity()).unwrap();
    info!("starting binding for '{name}'");

    commands.entity(*root_entity).with_children(|parent| {
        parent
            .spawn(BindingDialog {
                button_entity: trigger.entity(),
            })
            .with_children(|parent| {
                parent
                    .spawn((
                        Node {
                            flex_direction: FlexDirection::Column,
                            padding: PADDING,
                            row_gap: GAP,
                            ..Default::default()
                        },
                        PANEL_BACKGROUND,
                    ))
                    .with_children(|parent| {
                        parent.spawn((
                            TextLayout {
                                justify: JustifyText::Center,
                                ..Default::default()
                            },
                            DARK_TEXT,
                            Text::new(format!(
                                "Binding \"{name}\", \npress any key or Esc to cancel",
                            )),
                        ));
                    });
            });
    });
}

fn bind(
    mut commands: Commands,
    mut key_events: EventReader<KeyboardInput>,
    mut mouse_button_events: EventReader<MouseButtonInput>,
    dialog: Single<(Entity, &BindingDialog)>,
    root_entity: Single<Entity, (With<Node>, Without<Parent>)>,
    mut buttons: Query<(Entity, &Name, &mut InputButton)>,
) {
    let keys = key_events
        .read()
        .filter(|event| event.state == ButtonState::Pressed)
        .map(|event| event.key_code.into());
    let mouse_buttons = mouse_button_events
        .read()
        .filter(|event| event.state == ButtonState::Pressed)
        .map(|event| event.button.into());

    let Some(input) = keys.chain(mouse_buttons).next() else {
        return;
    };

    let (dialog_entity, dialog) = *dialog;

    if let Some((conflict_entity, name, _)) = buttons
        .iter()
        .find(|(.., button)| button.input == Some(input))
    {
        info!("found conflict with '{name}' for '{input}'");

        commands.entity(*root_entity).with_children(|parent| {
            parent
                .spawn(ConflictDialog {
                    button_entity: dialog.button_entity,
                    conflict_entity,
                })
                .with_children(|parent| {
                    parent
                        .spawn((
                            Node {
                                flex_direction: FlexDirection::Column,
                                align_items: AlignItems::Center,
                                padding: PADDING,
                                row_gap: GAP,
                                ..Default::default()
                            },
                            PANEL_BACKGROUND,
                        ))
                        .with_children(|parent| {
                            parent.spawn((
                                DARK_TEXT,
                                Text::new(format!("\"{input}\" is already used by \"{name}\"",)),
                            ));
                            parent
                                .spawn(Node {
                                    column_gap: GAP,
                                    ..Default::default()
                                })
                                .with_children(|parent| {
                                    parent
                                        .spawn(SettingsButton)
                                        .with_child(Text::new("Replace"))
                                        .observe(replace_binding);
                                    parent
                                        .spawn(SettingsButton)
                                        .with_child(Text::new("Cancel"))
                                        .observe(cancel_replace_binding);
                                });
                        });
                });
        });
    } else {
        let (_, name, mut button) = buttons
            .get_mut(dialog.button_entity)
            .expect("binding dialog should point to a button with input");
        info!("assigning '{input}' to '{name}'");
        button.input = Some(input);
    }

    commands.entity(dialog_entity).despawn_recursive();
}

fn cancel_binding(mut commands: Commands, dialog_entity: Single<Entity, With<BindingDialog>>) {
    info!("cancelling binding");
    commands.entity(*dialog_entity).despawn_recursive();
}

fn replace_binding(
    _trigger: Trigger<Pointer<Click>>,
    mut commands: Commands,
    dialog: Single<(Entity, &ConflictDialog)>,
    mut buttons: Query<(&Name, &mut InputButton)>,
) {
    let (dialog_entity, dialog) = *dialog;
    let (_, mut conflict_button) = buttons
        .get_mut(dialog.conflict_entity)
        .expect("binding conflict should point to a button");
    let input = conflict_button.input;
    conflict_button.input = None;

    let (name, mut button) = buttons
        .get_mut(dialog.button_entity)
        .expect("binding should point to a button");
    button.input = input;

    info!("reassigning binding to '{name}'");
    commands.entity(dialog_entity).despawn_recursive();
}

fn cancel_replace_binding(
    _trigger: Trigger<Pointer<Click>>,
    mut commands: Commands,
    dialog_entity: Single<Entity, With<ConflictDialog>>,
) {
    info!("cancelling replace binding");
    commands.entity(*dialog_entity).despawn_recursive();
}

fn apply(
    _trigger: Trigger<Pointer<Click>>,
    mut commands: Commands,
    mut settings: ResMut<KeyboardSettings>,
    buttons: Query<(&InputButton, &SettingsField)>,
) {
    settings.clear();
    for (button, field) in &buttons {
        if let Some(input) = button.input {
            // Utilize reflection to write by field name.
            let field_value = settings
                .path_mut::<Vec<Input>>(field.0)
                .expect("fields with mappings should be stored as Vec");
            field_value.push(input);
        }
    }

    commands.trigger(RebuildBindings);

    match settings.write(SETTINGS_PATH) {
        Ok(()) => info!("writing settings to '{SETTINGS_PATH}'"),
        Err(e) => error!("unable to write settings to '{SETTINGS_PATH}': {e}"),
    }
}

fn update_button_text(
    buttons: Query<(&InputButton, &Children), Changed<InputButton>>,
    mut text: Query<&mut Text>,
) {
    for (button, children) in &buttons {
        let mut iter = text.iter_many_mut(children);
        let mut text = iter.fetch_next().unwrap();
        text.clear();
        if let Some(input) = button.input {
            write!(text, "{input}").unwrap();
        } else {
            write!(text, "Empty").unwrap();
        };
    }
}

fn update_button_background(
    mut buttons: Query<(&Interaction, &mut BackgroundColor), (Changed<Interaction>, With<Button>)>,
) {
    for (&interaction, mut background) in &mut buttons {
        *background = match interaction {
            Interaction::Pressed => Color::srgb(0.35, 0.75, 0.35).into(),
            Interaction::Hovered => Color::srgb(0.25, 0.25, 0.25).into(),
            Interaction::None => Color::srgb(0.15, 0.15, 0.15).into(),
        };
    }
}

#[derive(Component, Default)]
#[require(
    Button,
    Node(|| Node {
        justify_content: JustifyContent::Center,
        align_items: AlignItems::Center,
        width: Val::Px(160.0),
        height: Val::Px(35.0),
        ..Default::default()
    }),
)]
struct SettingsButton;

/// Stores information about button mapping.
#[derive(Component)]
#[require(SettingsButton)]
struct InputButton {
    /// Assigned input.
    input: Option<Input>,
}

/// Stores assigned button with input.
#[derive(Component)]
#[require(
    Button,
    Node(|| Node {
        justify_content: JustifyContent::Center,
        align_items: AlignItems::Center,
        width: Val::Px(35.0),
        height: Val::Px(35.0),
        ..Default::default()
    }),
)]
struct DeleteButton {
    /// Entity with [`InputButton`].
    button_entity: Entity,
}

#[derive(Component, Default)]
#[require(
    Node(|| Node {
        position_type: PositionType::Absolute,
        width: Val::Percent(100.0),
        height: Val::Percent(100.0),
        align_items: AlignItems::Center,
        justify_content: JustifyContent::Center,
        ..Default::default()
    }),
    FocusPolicy(|| FocusPolicy::Block),
    BackgroundColor(|| BackgroundColor(Color::srgba(1.0, 1.0, 1.0, 0.3))),
)]
struct Dialog;

#[derive(Component)]
#[require(Dialog)]
struct BindingDialog {
    /// Entity with [`InputButton`].
    button_entity: Entity,
}

#[derive(Component)]
#[require(Dialog)]
struct ConflictDialog {
    /// Entity with [`InputButton`].
    button_entity: Entity,
    /// Entity with [`InputButton`] that conflicts with [`Self::button_entity`].
    conflict_entity: Entity,
}

/// Keyboard and mouse settings.
///
/// Most games assign bindings for different input sources (keyboard + mouse, gamepads, etc.) separately or
/// even only allow rebinding for keyboard and mouse.
/// For example, gamepads use sticks for movement, which are bidirectional, so it doesn't make sense to assign
/// actions like "forward" to [`GamepadAxis::LeftStickX`].
///
/// If you want to assign a specific part of the axis, such as the positive part of [`GamepadAxis::LeftStickX`],
/// you need to create your own input enum. However, this approach is mostly used in emulators rather than games.
#[derive(Resource, Reflect, Clone, Deserialize, Serialize)]
#[serde(default)]
pub struct KeyboardSettings {
    pub forward: Vec<Input>,
    pub left: Vec<Input>,
    pub backward: Vec<Input>,
    pub right: Vec<Input>,
    pub jump: Vec<Input>,
    pub run: Vec<Input>,
    pub fire: Vec<Input>,
}

impl KeyboardSettings {
    fn read(path: &str) -> Result<Self, Box<dyn Error>> {
        let content = fs::read_to_string(path)?;
        let settings = ron::from_str(&content)?;
        Ok(settings)
    }

    fn write(&self, path: &str) -> Result<(), Box<dyn Error>> {
        let content = ron::ser::to_string_pretty(self, Default::default())?;
        fs::write(path, content)?;
        Ok(())
    }

    fn clear(&mut self) {
        self.forward.clear();
        self.left.clear();
        self.backward.clear();
        self.right.clear();
        self.jump.clear();
        self.run.clear();
        self.fire.clear();
    }
}

impl Default for KeyboardSettings {
    fn default() -> Self {
        Self {
            forward: vec![KeyCode::KeyW.into(), KeyCode::ArrowUp.into()],
            left: vec![KeyCode::KeyA.into(), KeyCode::ArrowLeft.into()],
            backward: vec![KeyCode::KeyS.into(), KeyCode::ArrowDown.into()],
            right: vec![KeyCode::KeyD.into(), KeyCode::ArrowRight.into()],
            jump: vec![KeyCode::Space.into()],
            run: vec![KeyCode::ShiftLeft.into()],
            fire: vec![MouseButton::Left.into()],
        }
    }
}
