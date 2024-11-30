use super::{input_condition::InputCondition, input_modifier::InputModifier};
use crate::input::Input;

/// Associated input for [`ActionBind`].
#[derive(Debug)]
pub struct InputBind {
    pub input: Input,
    pub modifiers: Vec<Box<dyn InputModifier>>,
    pub conditions: Vec<Box<dyn InputCondition>>,

    /// Newly created mappings are ignored by default until until a zero
    /// value is read for them.
    ///
    /// This prevents newly created contexts from reacting to currently
    /// held inputs until they are released.
    pub(super) ignored: bool,
}

impl InputBind {
    /// Creates a new instance without modifiers and conditions.
    pub fn new(input: impl Into<Input>) -> Self {
        Self {
            input: input.into(),
            modifiers: Default::default(),
            conditions: Default::default(),
            ignored: true,
        }
    }
}

impl<I: Into<Input>> From<I> for InputBind {
    fn from(input: I) -> Self {
        Self::new(input)
    }
}

/// A trait to ergonomically add a modifier or condition to any type that can be converted into a binding.
pub trait InputBindModCond {
    /// Adds modifier.
    #[must_use]
    fn with_modifier(self, modifier: impl InputModifier) -> InputBind;

    /// Adds condition.
    #[must_use]
    fn with_condition(self, condition: impl InputCondition) -> InputBind;
}

impl<T: Into<InputBind>> InputBindModCond for T {
    fn with_modifier(self, modifier: impl InputModifier) -> InputBind {
        let mut binding = self.into();
        binding.modifiers.push(Box::new(modifier));
        binding
    }

    fn with_condition(self, condition: impl InputCondition) -> InputBind {
        let mut binding = self.into();
        binding.conditions.push(Box::new(condition));
        binding
    }
}
