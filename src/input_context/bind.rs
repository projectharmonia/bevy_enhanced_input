use crate::prelude::*;

#[derive(Debug)]
pub struct InputBind {
    pub input: Input,
    pub modifiers: Vec<Box<dyn InputModifier>>,
    pub conditions: Vec<Box<dyn InputCondition>>,
    pub ignored: bool,
}

impl InputBind {
    pub fn new(input: impl Into<Input>) -> Self {
        Self {
            input: input.into(),
            modifiers: Vec::new(),
            conditions: Vec::new(),
            ignored: true,
        }
    }
}

#[derive(Debug, Default)]
pub struct InputBindSet {
    pub binds: Vec<InputBinds>,
    pub modifiers: Vec<Box<dyn InputModifier>>,
    pub conditions: Vec<Box<dyn InputCondition>>,
}

impl FromIterator<InputBinds> for InputBindSet {
    fn from_iter<T: IntoIterator<Item = InputBinds>>(iter: T) -> Self {
        Self {
            binds: iter.into_iter().collect(),
            modifiers: Vec::new(),
            conditions: Vec::new(),
        }
    }
}

impl Extend<InputBinds> for InputBindSet {
    fn extend<T: IntoIterator<Item = InputBinds>>(&mut self, iter: T) {
        self.binds.extend(iter)
    }
}

#[derive(Debug)]
pub enum InputBinds {
    One(InputBind),
    Set(InputBindSet),
}

pub trait IntoInputBinds
where
    Self: Sized,
{
    fn into_binds(self) -> InputBinds;

    fn with_modifier(self, modifier: impl InputModifier) -> InputBinds {
        self.into_binds().with_modifier(modifier)
    }

    fn with_condition(self, condition: impl InputCondition) -> InputBinds {
        self.into_binds().with_condition(condition)
    }
}

impl IntoInputBinds for InputBinds {
    fn into_binds(self) -> InputBinds {
        self
    }

    fn with_modifier(mut self, modifier: impl InputModifier) -> InputBinds {
        (match &mut self {
            Self::One(one) => &mut one.modifiers,
            Self::Set(set) => &mut set.modifiers,
        })
        .push(Box::new(modifier));
        self
    }

    fn with_condition(mut self, condition: impl InputCondition) -> InputBinds {
        (match &mut self {
            Self::One(one) => &mut one.conditions,
            Self::Set(set) => &mut set.conditions,
        })
        .push(Box::new(condition));
        self
    }
}

impl IntoInputBinds for InputBind {
    fn into_binds(self) -> InputBinds {
        InputBinds::One(self)
    }
}

impl<T: Into<Input>> IntoInputBinds for T {
    fn into_binds(self) -> InputBinds {
        InputBind::new(self).into_binds()
    }
}

impl IntoInputBinds for InputBindSet {
    fn into_binds(self) -> InputBinds {
        InputBinds::Set(self)
    }
}

// collections

impl<T: IntoInputBinds, const N: usize> IntoInputBinds for [T; N] {
    fn into_binds(self) -> InputBinds {
        InputBindSet::from_iter(self.into_iter().map(IntoInputBinds::into_binds)).into_binds()
    }
}

impl<T: IntoInputBinds> IntoInputBinds for Vec<T> {
    fn into_binds(self) -> InputBinds {
        InputBindSet::from_iter(self.into_iter().map(IntoInputBinds::into_binds)).into_binds()
    }
}

impl IntoInputBinds for () {
    fn into_binds(self) -> InputBinds {
        InputBindSet::default().into_binds()
    }
}

impl<T: IntoInputBinds> IntoInputBinds for (T,) {
    fn into_binds(self) -> InputBinds {
        self.0.into_binds()
    }
}

macro_rules! impl_tuple {
    ($(#[$meta:meta])* $($T:ident $i:tt),*) => {
        $(#[$meta])*
        impl<$($T: IntoInputBinds),*> IntoInputBinds for ($($T,)*) {
            fn into_binds(self) -> InputBinds {
                InputBindSet::from_iter([
                    $(self.$i.into_binds()),*
                ])
                .into_binds()
            }
        }
    };
}

impl_tuple!(A 0, B 1);
impl_tuple!(A 0, B 1, C 2);
impl_tuple!(A 0, B 1, C 2, D 3);
impl_tuple!(A 0, B 1, C 2, D 3, E 4);
impl_tuple!(A 0, B 1, C 2, D 3, E 4, F 5);
impl_tuple!(A 0, B 1, C 2, D 3, E 4, F 5, G 6);
impl_tuple!(A 0, B 1, C 2, D 3, E 4, F 5, G 6, H 7);
impl_tuple!(A 0, B 1, C 2, D 3, E 4, F 5, G 6, H 7, I 8);
impl_tuple!(A 0, B 1, C 2, D 3, E 4, F 5, G 6, H 7, I 8, J 9);
impl_tuple!(A 0, B 1, C 2, D 3, E 4, F 5, G 6, H 7, I 8, J 9, K 10);
impl_tuple!(A 0, B 1, C 2, D 3, E 4, F 5, G 6, H 7, I 8, J 9, K 10, L 11);
impl_tuple!(A 0, B 1, C 2, D 3, E 4, F 5, G 6, H 7, I 8, J 9, K 10, L 11, M 12);
impl_tuple!(A 0, B 1, C 2, D 3, E 4, F 5, G 6, H 7, I 8, J 9, K 10, L 11, M 12, N 13);
impl_tuple!(A 0, B 1, C 2, D 3, E 4, F 5, G 6, H 7, I 8, J 9, K 10, L 11, M 12, N 13, O 14);
impl_tuple!(A 0, B 1, C 2, D 3, E 4, F 5, G 6, H 7, I 8, J 9, K 10, L 11, M 12, N 13, O 14, P 15);
