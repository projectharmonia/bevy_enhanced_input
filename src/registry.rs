use alloc::vec::Vec;
use core::{
    any::{self, TypeId},
    cmp::Reverse,
    marker::PhantomData,
    mem,
};

use bevy::prelude::*;

use super::{
    input_reader::{InputReader, ResetInput},
    InputContext,
};

/// An extension trait for [`App`] to assign input to components.
pub trait InputContextAppExt {
    /// Registers an input context for component `C`.
    ///
    /// When the component is inserted, [`Binding`] event will be triggered to create [`InputContext`].
    /// To setup bindings, register an observer and mutate the trigger (the trigger dereferences to [`InputContext`]).
    ///
    /// The trigger data will be automatically consumed and stored in [`InputContextRegistry`].
    ///
    /// Until the component is exists on the entity, actions will be evaluated and trigger [`events`](super::events).
    ///
    /// Component removal deactivates [`InputContext`] for the entity and trigger transitions for all actions
    /// to [`ActionState::None`](super::ActionState::None).
    ///
    /// Component re-insertion re-triggers [`Binding`]. Use it if you want to reload bindings.
    /// You can also rebuild all component bindings by triggering [`RebuildBindings`].
    ///
    /// # Examples
    ///
    /// ```
    /// # use bevy::prelude::*;
    /// # use bevy_enhanced_input::prelude::*;
    /// # let mut app = App::new();
    /// app.add_input_context::<Player>().add_observer(player_binding);
    ///
    /// fn player_binding(mut trigger: Trigger<Binding<Player>>, settings: Res<AppSettings>) {
    ///     trigger.bind::<Jump>().to((settings.keyboard.jump, GamepadButton::South));
    /// }
    ///
    /// #[derive(Component)]
    /// struct Player;
    ///
    /// #[derive(Debug, InputAction)]
    /// #[input_action(output = bool)]
    /// struct Jump;
    ///
    /// #[derive(Resource)]
    /// struct AppSettings {
    ///     keyboard: KeyboardSettings,
    /// }
    ///
    /// struct KeyboardSettings {
    ///     jump: KeyCode,
    /// }
    /// ```
    fn add_input_context<C: Component>(&mut self) -> &mut Self;
}

impl InputContextAppExt for App {
    fn add_input_context<C: Component>(&mut self) -> &mut Self {
        debug!("registering context for `{}`", any::type_name::<C>());

        self.add_observer(add_consuming_observer::<C>)
            .add_observer(trigger_binding::<C>)
            .add_observer(rebuild_bindings::<C>)
            .add_observer(remove_input::<C>);

        self
    }
}

/// Spawns entity observer for [`Binding`] that consumes user-defined bindings.
///
/// Runs after user-defined bindings.
fn add_consuming_observer<C: Component>(trigger: Trigger<OnAdd, C>, mut commands: Commands) {
    debug!(
        "adding consuming observer for `{}` on `{}`",
        any::type_name::<C>(),
        trigger.entity()
    );

    commands
        .entity(trigger.entity())
        .observe(consume_context::<C>);
}

/// Triggers binding event.
fn trigger_binding<C: Component>(trigger: Trigger<OnInsert, C>, mut commands: Commands) {
    debug!(
        "triggering binding for `{}` on `{}`",
        any::type_name::<C>(),
        trigger.entity()
    );

    commands.trigger_targets(Binding::<C>::new(Default::default()), trigger.entity());
}

/// Moves [`InputContext`] out from the trigger into [`InputContextRegistry`].
fn consume_context<C: Component>(
    mut trigger: Trigger<Binding<C>>,
    mut registry: ResMut<InputContextRegistry>,
) {
    debug!(
        "consuming binding for `{}` on `{}`",
        any::type_name::<C>(),
        trigger.entity()
    );

    debug_assert!(
        registry.get_context::<C>(trigger.entity()).is_none(),
        "context for `{}` shouldn't already exist on `{}`",
        any::type_name::<C>(),
        trigger.entity()
    );

    let ctx = mem::take(&mut trigger.ctx);
    let priority = Reverse(ctx.priority());
    let index = match registry
        .0
        .binary_search_by_key(&priority, |entry| Reverse(entry.ctx.priority()))
    {
        Ok(index) => {
            // Insert last to preserve entry creation order.
            let last_priority_index = registry
                .0
                .iter()
                .skip(index + 1)
                .position(|entry| entry.ctx.priority() != ctx.priority())
                .unwrap_or_default();
            index + last_priority_index + 1
        }
        Err(index) => index,
    };

    let entry = ContextEntry::new::<C>(trigger.entity(), ctx);
    registry.0.insert(index, entry);
}

fn rebuild_bindings<C: Component>(
    _trigger: Trigger<RebuildBindings>,
    mut commands: Commands,
    mut reset_input: ResMut<ResetInput>,
    mut registry: ResMut<InputContextRegistry>,
    time: Res<Time<Virtual>>,
) {
    registry.0.retain_mut(|entry| {
        if entry.type_id != TypeId::of::<C>() {
            return true;
        }

        debug!(
            "triggering binding rebuild for `{}` on `{}`",
            any::type_name::<C>(),
            entry.entity
        );

        entry
            .ctx
            .reset(&mut commands, &mut reset_input, &time, entry.entity);
        commands.trigger_targets(Binding::<C>::new(mem::take(&mut entry.ctx)), entry.entity);

        false
    });
}

fn remove_input<C: Component>(
    trigger: Trigger<OnReplace, C>,
    mut commands: Commands,
    mut reset_input: ResMut<ResetInput>,
    mut registry: ResMut<InputContextRegistry>,
    time: Res<Time<Virtual>>,
) {
    debug!(
        "removing input context for `{}` from `{}`",
        any::type_name::<C>(),
        trigger.entity()
    );

    let index = registry
        .0
        .iter()
        .position(|entry| entry.entity == trigger.entity() && entry.type_id == TypeId::of::<C>())
        .expect("input entry should be created before removal");
    let mut entry = registry.0.remove(index);
    entry
        .ctx
        .reset(&mut commands, &mut reset_input, &time, trigger.entity());
}

/// Trigger that requests the creation of [`InputContext`] associated with component for an entity.
///
/// Can't be triggered by user. If you want to reload re-create bindings, just re-insert
/// the component or trigger [`RebuildBindings`].
#[derive(Event, Deref, DerefMut)]
pub struct Binding<C: Component> {
    #[deref]
    ctx: InputContext,
    _marker: PhantomData<C>,
}

impl<C: Component> Binding<C> {
    fn new(ctx: InputContext) -> Self {
        Self {
            ctx,
            _marker: PhantomData,
        }
    }
}

/// Stores instantiated [`InputContext`]s.
#[derive(Resource, Default)]
pub struct InputContextRegistry(Vec<ContextEntry>);

impl InputContextRegistry {
    pub(crate) fn update(
        &mut self,
        commands: &mut Commands,
        reader: &mut InputReader,
        time: &Time<Virtual>,
    ) {
        for entry in &mut self.0 {
            entry.ctx.update(commands, reader, time, entry.entity);
        }
    }

    /// Returns a context instance for a component on an entity, if it exists.
    ///
    /// For panicking version see [`Self::context`].
    pub fn get_context<C: Component>(&self, entity: Entity) -> Option<&InputContext> {
        self.0
            .iter()
            .find(|ctx| ctx.entity == entity && ctx.type_id == TypeId::of::<C>())
            .map(|entry| &entry.ctx)
    }

    /// Returns a context instance for a component on an entity.
    ///
    /// For a more ergonomic API, it's recommended to react on [`events`](super::events)
    /// with observers.
    ///
    /// The complexity is `O(n)`, where `n` is the number of input contexts,
    /// since the storage is optimized for iteration.
    /// However, there are usually only a few contexts that are instantiated.
    ///
    /// For non-panicking version see [`Self::get_context`].
    ///
    /// # Panics
    ///
    /// Panics if no input was registered for `C` or the entity doesn't have this component.
    ///
    /// # Examples
    ///
    /// ```
    /// # use bevy::prelude::*;
    /// # use bevy_enhanced_input::prelude::*;
    /// fn observer(trigger: Trigger<Attacked>, registry: Res<InputContextRegistry>) {
    ///     let ctx = registry.context::<Player>(trigger.entity());
    ///     let action = ctx.action::<Dodge>();
    ///     if action.events().contains(ActionEvents::FIRED) {
    ///         // ..
    ///     }
    /// }
    /// # #[derive(Event)]
    /// # struct Attacked;
    /// # #[derive(Component)]
    /// # struct Player;
    /// # #[derive(Debug, InputAction)]
    /// # #[input_action(output = bool)]
    /// # struct Dodge;
    /// ```
    pub fn context<C: Component>(&self, entity: Entity) -> &InputContext {
        self.get_context::<C>(entity).unwrap_or_else(|| {
            panic!(
                "`{}` should have registered input context and present on `{entity}`",
                any::type_name::<C>()
            )
        })
    }
}

/// Associates input with a component on an entity.
struct ContextEntry {
    entity: Entity,
    type_id: TypeId,
    ctx: InputContext,
}

impl ContextEntry {
    fn new<C: Component>(entity: Entity, ctx: InputContext) -> Self {
        Self {
            entity,
            type_id: TypeId::of::<C>(),
            ctx,
        }
    }
}

/// A trigger that causes the reconstruction of all active [`InputContext`]s by triggering [`Binding`].
///
/// Use it when you change your application settings and want to reload the mappings.
///
/// This will also reset all actions to [`ActionState::None`](super::ActionState::None)
/// and trigger the corresponding events.
#[derive(Event)]
pub struct RebuildBindings;
