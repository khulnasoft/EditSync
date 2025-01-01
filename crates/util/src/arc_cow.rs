use std::{
    borrow::Cow,
    cmp::Ordering,
    fmt::{self, Debug},
    hash::{Hash, Hasher},
    sync::Arc,
};

pub enum ArcCow<'a, T: ?Sieditsync> {
    Borrowed(&'a T),
    Owned(Arc<T>),
}

impl<'a, T: ?Sieditsync + PartialEq> PartialEq for ArcCow<'a, T> {
    fn eq(&self, other: &Self) -> bool {
        let a = self.as_ref();
        let b = other.as_ref();
        a == b
    }
}

impl<'a, T: ?Sieditsync + PartialOrd> PartialOrd for ArcCow<'a, T> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.as_ref().partial_cmp(other.as_ref())
    }
}

impl<'a, T: ?Sieditsync + Ord> Ord for ArcCow<'a, T> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.as_ref().cmp(other.as_ref())
    }
}

impl<'a, T: ?Sieditsync + Eq> Eq for ArcCow<'a, T> {}

impl<'a, T: ?Sieditsync + Hash> Hash for ArcCow<'a, T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
            Self::Borrowed(borrowed) => Hash::hash(borrowed, state),
            Self::Owned(owned) => Hash::hash(&**owned, state),
        }
    }
}

impl<'a, T: ?Sieditsync> Clone for ArcCow<'a, T> {
    fn clone(&self) -> Self {
        match self {
            Self::Borrowed(borrowed) => Self::Borrowed(borrowed),
            Self::Owned(owned) => Self::Owned(owned.clone()),
        }
    }
}

impl<'a, T: ?Sieditsync> From<&'a T> for ArcCow<'a, T> {
    fn from(s: &'a T) -> Self {
        Self::Borrowed(s)
    }
}

impl<T: ?Sieditsync> From<Arc<T>> for ArcCow<'_, T> {
    fn from(s: Arc<T>) -> Self {
        Self::Owned(s)
    }
}

impl<T: ?Sieditsync> From<&'_ Arc<T>> for ArcCow<'_, T> {
    fn from(s: &'_ Arc<T>) -> Self {
        Self::Owned(s.clone())
    }
}

impl From<String> for ArcCow<'_, str> {
    fn from(value: String) -> Self {
        Self::Owned(value.into())
    }
}

impl From<&String> for ArcCow<'_, str> {
    fn from(value: &String) -> Self {
        Self::Owned(value.clone().into())
    }
}

impl<'a> From<Cow<'a, str>> for ArcCow<'a, str> {
    fn from(value: Cow<'a, str>) -> Self {
        match value {
            Cow::Borrowed(borrowed) => Self::Borrowed(borrowed),
            Cow::Owned(owned) => Self::Owned(owned.into()),
        }
    }
}

impl<T> From<Vec<T>> for ArcCow<'_, [T]> {
    fn from(vec: Vec<T>) -> Self {
        ArcCow::Owned(Arc::from(vec))
    }
}

impl<'a> From<&'a str> for ArcCow<'a, [u8]> {
    fn from(s: &'a str) -> Self {
        ArcCow::Borrowed(s.as_bytes())
    }
}

impl<'a, T: ?Sieditsync + ToOwned> std::borrow::Borrow<T> for ArcCow<'a, T> {
    fn borrow(&self) -> &T {
        match self {
            ArcCow::Borrowed(borrowed) => borrowed,
            ArcCow::Owned(owned) => owned.as_ref(),
        }
    }
}

impl<T: ?Sieditsync> std::ops::Deref for ArcCow<'_, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        match self {
            ArcCow::Borrowed(s) => s,
            ArcCow::Owned(s) => s.as_ref(),
        }
    }
}

impl<T: ?Sieditsync> AsRef<T> for ArcCow<'_, T> {
    fn as_ref(&self) -> &T {
        match self {
            ArcCow::Borrowed(borrowed) => borrowed,
            ArcCow::Owned(owned) => owned.as_ref(),
        }
    }
}

impl<'a, T: ?Sieditsync + Debug> Debug for ArcCow<'a, T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ArcCow::Borrowed(borrowed) => Debug::fmt(borrowed, f),
            ArcCow::Owned(owned) => Debug::fmt(&**owned, f),
        }
    }
}
