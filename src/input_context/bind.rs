use crate::prelude::*;

#[derive(Debug, Default)]
pub struct BindConfigInfo {
    pub modifiers: Vec<Box<dyn InputModifier>>,
    pub conditions: Vec<Box<dyn InputCondition>>,
}

#[derive(Debug, Default)]
pub struct BindConfigSet {
    pub binds: Vec<BindConfigs>,
    pub info: BindConfigInfo,
}

impl FromIterator<BindConfigs> for BindConfigSet {
    fn from_iter<T: IntoIterator<Item = BindConfigs>>(iter: T) -> Self {
        Self {
            binds: iter.into_iter().collect(),
            info: BindConfigInfo::default(),
        }
    }
}

impl Extend<BindConfigs> for BindConfigSet {
    fn extend<T: IntoIterator<Item = BindConfigs>>(&mut self, iter: T) {
        self.binds.extend(iter)
    }
}

#[derive(Debug)]
pub struct BindConfig {
    pub input: Input,
    pub info: BindConfigInfo,
    pub ignored: bool,
}

impl BindConfig {
    pub fn new(input: impl Into<Input>) -> Self {
        Self {
            input: input.into(),
            ignored: true,
            info: BindConfigInfo::default(),
        }
    }
}

#[derive(Debug)]
pub enum BindConfigs {
    One(BindConfig),
    Set(BindConfigSet),
}

impl BindConfigs {
    pub fn info(&self) -> &BindConfigInfo {
        match self {
            Self::One(config) => &config.info,
            Self::Set(set) => &set.info,
        }
    }

    pub fn info_mut(&mut self) -> &mut BindConfigInfo {
        match self {
            Self::One(config) => &mut config.info,
            Self::Set(set) => &mut set.info,
        }
    }
}

pub trait IntoBindConfigs
where
    Self: Sized,
{
    fn into_configs(self) -> BindConfigs;

    fn with_modifier(self, modifier: impl InputModifier) -> BindConfigs {
        self.into_configs().with_modifier(modifier)
    }

    fn with_condition(self, condition: impl InputCondition) -> BindConfigs {
        self.into_configs().with_condition(condition)
    }
}

impl IntoBindConfigs for BindConfigs {
    fn into_configs(self) -> BindConfigs {
        self
    }

    fn with_modifier(mut self, modifier: impl InputModifier) -> BindConfigs {
        self.info_mut().modifiers.push(Box::new(modifier));
        self
    }

    fn with_condition(mut self, condition: impl InputCondition) -> BindConfigs {
        self.info_mut().conditions.push(Box::new(condition));
        self
    }
}

impl IntoBindConfigs for BindConfig {
    fn into_configs(self) -> BindConfigs {
        BindConfigs::One(self)
    }
}

impl<T: Into<Input>> IntoBindConfigs for T {
    fn into_configs(self) -> BindConfigs {
        BindConfig::new(self).into_configs()
    }
}

impl IntoBindConfigs for BindConfigSet {
    fn into_configs(self) -> BindConfigs {
        BindConfigs::Set(self)
    }
}

impl<T: IntoBindConfigs, const N: usize> IntoBindConfigs for [T; N] {
    fn into_configs(self) -> BindConfigs {
        BindConfigSet::from_iter(self.into_iter().map(IntoBindConfigs::into_configs)).into_configs()
    }
}

impl IntoBindConfigs for () {
    fn into_configs(self) -> BindConfigs {
        BindConfigSet::default().into_configs()
    }
}

impl<T: IntoBindConfigs> IntoBindConfigs for (T,) {
    fn into_configs(self) -> BindConfigs {
        self.0.into_configs()
    }
}

macro_rules! impl_tuple {
    ($(#[$meta:meta])* $($T:ident),*) => {
        $(#[$meta])*
        impl<$($T),*> IntoBindConfigs for ($($T,)*) {
            fn into_configs(self) -> BindConfigs {
                BindConfigSet::from_iter([
                    // TODO!!!
                ])
                .into_configs()
            }
        }
    };
}

impl_tuple!(T0, T1);
impl_tuple!(T0, T1, T2);
impl_tuple!(T0, T1, T2, T3);
