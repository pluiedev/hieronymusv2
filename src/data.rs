//! Some data types used by Minecraft.

use std::{ops::Deref, str::FromStr, borrow::Borrow};

use nom::character::is_alphanumeric;
use nom_derive::Nom;
use thiserror::Error;

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default, Nom)]
pub struct Position(u64);

impl Position {
    pub fn x(self) -> i32 {
        let mut x = (self.0 & 0x3ffffff) as i32;
        if x >= (1 << 25) {
            x -= 1 << 26;
        }
        x
    }
    pub fn y(self) -> i16 {
        let mut x = (self.0 << 26 & 0xfff) as i16;
        if x >= (1 << 11) {
            x -= 1 << 12;
        }
        x
    }
    pub fn z(self) -> i32 {
        let mut x = (self.0 << 38 & 0x3ffffff) as i32;
        if x >= (1 << 25){
            x -= 1 << 26;
        }
        x
    }
}

#[derive(Clone, Debug, Nom)]
pub struct Slot;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug, Default, Nom)]
pub struct Angle(pub u8);

pub trait Angular {
    fn from_angle_degrees(angle: Angle) -> Self;
    fn from_angle_radians(angle: Angle) -> Self;
    fn into_angle_degrees_rounded(self) -> Angle;
    fn into_angle_radians_rounded(self) -> Angle;
}
impl Angle {
    pub fn from_degrees_rounded<F: Angular>(f: F) -> Self {
        F::into_angle_degrees_rounded(f)
    }
    pub fn from_radians_rounded<F: Angular>(f: F) -> Self {
        F::into_angle_radians_rounded(f)
    }
    pub fn to_degrees<F: Angular>(self) -> F {
        F::from_angle_degrees(self)
    }
    pub fn to_radians<F: Angular>(self) -> F {
        F::from_angle_radians(self)
    }
}
macro_rules! angular_impl {
    ($($ty:ty,$tau:expr);+) => {
        $(
            impl Angular for $ty {
                fn from_angle_degrees(angle: Angle) -> Self {
                    <$ty as std::convert::From<u8>>::from(angle.0) / 256.0 * 360.0
                }
                fn from_angle_radians(angle: Angle) -> Self {
                    <$ty as std::convert::From<u8>>::from(angle.0) / 256.0 * $tau
                }
                fn into_angle_degrees_rounded(self) -> Angle {
                    Angle((self / 360.0 * 256.0) as u8)
                }
                fn into_angle_radians_rounded(self) -> Angle {
                    Angle((self / 360.0 * $tau) as u8)
                }
            }
        )+
    };
}
angular_impl!(
    f32, std::f32::consts::TAU;
    f64, std::f64::consts::TAU
);

pub struct OwnedIdentifier {
    pub namespace: String,
    pub path: String,
}
impl Borrow<Identifier<'_>> for OwnedIdentifier {
    fn borrow(&self) -> &Identifier<'_> {
        &Identifier {
            namespace: &self.namespace,
            path: &self.path,
        }
    }
}
impl AsRef<Identifier<'_>> for OwnedIdentifier {
    fn as_ref(&self) -> &Identifier<'_> {
        &Identifier {
            namespace: &self.namespace,
            path: &self.path,
        }
    }
}

pub struct Identifier<'a> {
    pub namespace: &'a str,
    pub path: &'a str,
}
impl FromStr for Identifier<'_> {
    type Err = ParseIdentifierError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        use ParseIdentifierError::*;
        if !s.is_ascii() {
            return Err(NotAscii);
        }
        
        let s = s.split(':');
        let namespace = s.next().ok_or(ExpectedSeparator)?;
        if let Some(c) = namespace.bytes().find(invalid_namespace_char) {
            return Err(InvalidCharacterInNamespace(char::from(c)))
        }
        let path = s.next().ok_or(ExpectedSeparator)?;
        if let Some(c) = path.bytes().find(invalid_path_char) {
            return Err(InvalidCharacterInPath(char::from(c)))
        }
        Ok(Identifier { namespace, path })
        
    }
}
fn invalid_namespace_char(c: &u8) -> bool {
    match *c {
        b'0'..=b'9' | b'a'..=b'z' | b'.' | b'_' | b'-' => false,
        _ => true
    }
}
fn invalid_path_char(c: &u8) -> bool {
    match *c {
        b'0'..=b'9' | b'a'..=b'z' | b'.' | b'_' | b'-' | b'/' => false,
        _ => true
    }
}
#[derive(Debug, Error)]
pub enum ParseIdentifierError {
    #[error("Invalid character in namespace: {0}")]
    InvalidCharacterInNamespace(char),
    #[error("Invalid character in path: {0}")]
    InvalidCharacterInPath(char),
    #[error("Identifier must be encoded in ASCII")]
    NotAscii,
    #[error("Expected separator `:`")]
    ExpectedSeparator,
}

impl ToOwned for Identifier<'_> {
    type Owned = OwnedIdentifier;

    fn to_owned(&self) -> Self::Owned {
        OwnedIdentifier {
            namespace: self.namespace.into(),
            path: self.path.into(),
        }
    }
}