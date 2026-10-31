use std::{collections::BTreeMap, fmt};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Type {
    Number,
    Bool,
    String,
    List(Box<Type>),
    Record(BTreeMap<String, Type>),
    Enum(String),
    Optional(Box<Type>),
    // Inference variables for absence and empty collections. Neither may escape
    // a completed binding without a concrete type supplied by its context.
    None,
    Unknown,
}

impl Type {
    pub fn optional(self) -> Self {
        if matches!(self, Self::Optional(_) | Self::None) {
            self
        } else {
            Self::Optional(Box::new(self))
        }
    }

    pub fn concrete(&self) -> bool {
        match self {
            Self::None | Self::Unknown => false,
            Self::List(ty) | Self::Optional(ty) => ty.concrete(),
            Self::Record(fields) => fields.values().all(Self::concrete),
            _ => true,
        }
    }

    pub fn accepts(&self, actual: &Self) -> bool {
        if self == actual || *actual == Self::Unknown {
            return true;
        }
        match (self, actual) {
            (Self::Optional(_), Self::None) => true,
            (Self::Optional(expected), Self::Optional(actual)) => expected.accepts(actual),
            (Self::Optional(expected), actual) => expected.accepts(actual),
            (Self::List(expected), Self::List(actual)) => expected.accepts(actual),
            (Self::Record(expected), Self::Record(actual)) => {
                actual.keys().eq(expected.keys())
                    && expected.iter().all(|(name, ty)| {
                        actual
                            .get(name)
                            .map_or(matches!(ty, Self::Optional(_)), |value| ty.accepts(value))
                    })
            }
            _ => false,
        }
    }

    pub fn comparable(&self, other: &Self) -> bool {
        if self == other {
            return true;
        }
        match (self, other) {
            (Self::Optional(_), Self::None) | (Self::None, Self::Optional(_)) => true,
            (Self::Optional(a), Self::Optional(b)) => a == b,
            (Self::Optional(a), b) | (b, Self::Optional(a)) => a.as_ref() == b,
            _ => false,
        }
    }

    pub fn join(&self, other: &Self) -> Option<Self> {
        if self == other || *other == Self::Unknown {
            return Some(self.clone());
        }
        if *self == Self::Unknown {
            return Some(other.clone());
        }
        match (self, other) {
            (Self::None, ty) | (ty, Self::None) => Some(ty.clone().optional()),
            (Self::Optional(a), Self::Optional(b)) => Some(a.join(b)?.optional()),
            (Self::Optional(a), b) | (b, Self::Optional(a)) => Some(a.join(b)?.optional()),
            (Self::List(a), Self::List(b)) => Some(Self::List(Box::new(a.join(b)?))),
            (Self::Record(a), Self::Record(b)) if a.keys().eq(b.keys()) => Some(Self::Record(
                a.iter()
                    .map(|(name, ty)| Some((name.clone(), ty.join(&b[name])?)))
                    .collect::<Option<_>>()?,
            )),
            _ => None,
        }
    }
}

impl fmt::Display for Type {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Number => f.write_str("number"),
            Self::Bool => f.write_str("bool"),
            Self::String => f.write_str("string"),
            Self::Enum(name) => f.write_str(name),
            Self::None => f.write_str("none"),
            Self::Unknown => f.write_str("unknown"),
            Self::List(ty) => write!(f, "{ty}[]"),
            Self::Optional(ty) => write!(f, "{ty}?"),
            Self::Record(fields) => {
                f.write_str("{ ")?;
                for (index, (name, ty)) in fields.iter().enumerate() {
                    if index > 0 {
                        f.write_str(", ")?;
                    }
                    write!(f, "{name}: {ty}")?;
                }
                f.write_str(" }")
            }
        }
    }
}
