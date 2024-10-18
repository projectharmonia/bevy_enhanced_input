//! Assert action events in tests.

use std::any::{self, TypeId};

use bevy::{ecs::entity::EntityHashMap, prelude::*, utils::HashMap};
use bevy_enhanced_input::{prelude::*, EnhancedInputSystem};

pub trait AppTriggeredExt {
    /// Observes for [`ActionEvent`] and stores them inside [`RecordedActions`].
    fn record_action<A: InputAction>(&mut self) -> &mut Self;
}

impl AppTriggeredExt for App {
    fn record_action<A: InputAction>(&mut self) -> &mut Self {
        self.world_mut()
            .resource_mut::<RecordedActions>()
            .register::<A>();
        self.observe(read::<A>)
    }
}

fn read<A: InputAction>(trigger: Trigger<ActionEvent<A>>, mut triggered: ResMut<RecordedActions>) {
    triggered.insert::<A>(trigger.entity(), *trigger.event());
}

pub struct ActionRecorderPlugin;

impl Plugin for ActionRecorderPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<RecordedActions>()
            .add_systems(PreUpdate, Self::clear.before(EnhancedInputSystem));
    }
}

impl ActionRecorderPlugin {
    fn clear(mut triggered: ResMut<RecordedActions>) {
        triggered.clear();
    }
}

#[derive(Default, Resource)]
pub struct RecordedActions(HashMap<TypeId, EntityHashMap<Vec<UntypedActionEvent>>>);

impl RecordedActions {
    fn insert<A: InputAction>(&mut self, entity: Entity, event: ActionEvent<A>) {
        let event_group = self.0.entry(TypeId::of::<A>()).or_default();
        let events = event_group.entry(entity).or_default();
        events.push(event.into());
    }

    #[allow(dead_code)]
    pub fn assert_array<A: InputAction, const SIZE: usize>(
        &self,
        entity: Entity,
    ) -> [UntypedActionEvent; SIZE] {
        let events = self.get::<A>(entity);
        events.try_into().unwrap_or_else(|_| {
            panic!(
                "expected {SIZE} events of type `{}`, but got {}",
                any::type_name::<A>(),
                events.len()
            );
        })
    }

    #[allow(dead_code)]
    pub fn is_empty<A: InputAction>(&self, entity: Entity) -> bool {
        self.get::<A>(entity).is_empty()
    }

    #[allow(dead_code)]
    pub fn last<A: InputAction>(&self, entity: Entity) -> &UntypedActionEvent {
        self.get::<A>(entity).last().unwrap_or_else(|| {
            panic!(
                "expected at least one action event of type `{}`",
                any::type_name::<A>()
            )
        })
    }

    #[allow(dead_code)]
    fn get<A: InputAction>(&self, entity: Entity) -> &[UntypedActionEvent] {
        let event_group = self.0.get(&TypeId::of::<A>()).unwrap_or_else(|| {
            panic!(
                "action event of type `{}` is not registered",
                any::type_name::<A>()
            )
        });

        event_group
            .get(&entity)
            .map(|events| &events[..])
            .unwrap_or(&[])
    }

    fn register<A: InputAction>(&mut self) {
        self.0.insert(TypeId::of::<A>(), Default::default());
    }

    fn clear(&mut self) {
        for event_group in self.0.values_mut() {
            event_group.clear();
        }
    }
}

/// Untyped version of [`ActionEvent`].
#[derive(Clone, Copy)]
#[allow(dead_code)]
pub struct UntypedActionEvent {
    pub transition: ActionTransition,
    pub value: ActionValue,
    pub state: ActionState,
}

impl<A: InputAction> From<ActionEvent<A>> for UntypedActionEvent {
    fn from(value: ActionEvent<A>) -> Self {
        Self {
            transition: value.transition,
            value: value.value,
            state: value.state,
        }
    }
}
