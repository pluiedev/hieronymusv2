//! Some data types used by Minecraft.
use nom::combinator::{map_opt, map_res};
use nom_derive::{Nom, Parse};
use num_derive::FromPrimitive;
use num_traits::FromPrimitive;
use serde::Serialize;
use smol_str::SmolStr;
use thiserror::Error;

use crate::{net::TryToResponseField, nom::var_str, varint::varint};

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
        if x >= (1 << 25) {
            x -= 1 << 26;
        }
        x
    }
}

pub type Slot = Option<SlotData>;

#[derive(Clone, Debug)]
pub struct SlotData {
    id: u32,
    count: u8,
    nbt: Option<SlotNbt>,
}

impl TryToResponseField for SlotData {
    type Err = nbt::Error;
    fn try_to_request_field(
        &self,
        builder: &mut crate::net::ResponseBuilder,
    ) -> Result<(), Self::Err> {
        builder.add(self.id).add(self.count);
        match &self.nbt {
            Some(nbt) => builder.nbt(nbt)?,
            None => builder.add(0u8), // TAG_End
        };
        Ok(())
    }
}
impl Parse<&[u8]> for SlotData {
    fn parse(i: &[u8]) -> nom::IResult<&[u8], Self, nom::error::Error<&[u8]>> {
        use nom::{combinator::peek, number::streaming::be_u8};

        let (i, id) = varint::<u32>(i)?;
        let (i, count) = be_u8(i)?;
        let (i, test) = peek(be_u8)(i)?;
        let slot = SlotData {
            id,
            count,
            nbt: None,
        };

        Ok((i, slot))
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct SlotNbt {
    damage: i32,
    unbreakable: bool,
    // TODO
}

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

// TODO: implement some kind of intern system/arena memory management
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Identifier {
    pub namespace: SmolStr,
    pub path: SmolStr,
}

impl Identifier {
    pub fn as_ref<'a>(&'a self) -> IdentifierRef<'a> {
        IdentifierRef {
            namespace: &self.namespace,
            path: &self.path,
        }
    }
}
impl TryFrom<&str> for Identifier {
    type Error = ParseIdentifierError;

    #[inline]
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Ok(IdentifierRef::try_from(value)?.to_owned())
    }
}
impl Parse<&[u8]> for Identifier {
    fn parse(i: &[u8]) -> nom::IResult<&[u8], Self> {
        map_res(var_str, Self::try_from)(i)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IdentifierRef<'a> {
    pub namespace: &'a str,
    pub path: &'a str,
}
impl<'a> IdentifierRef<'a> {
    pub fn to_owned(&self) -> Identifier {
        Identifier {
            namespace: self.namespace.into(),
            path: self.path.into(),
        }
    }
}
impl<'a> TryFrom<&'a str> for IdentifierRef<'a> {
    type Error = ParseIdentifierError;

    #[inline]
    fn try_from(value: &'a str) -> Result<Self, Self::Error> {
        use ParseIdentifierError::*;
        if !value.is_ascii() {
            return Err(NotAscii);
        }

        let mut split = value.split(':');
        let namespace = split.next().ok_or(ExpectedSeparator)?;
        let path = split.next().ok_or(ExpectedSeparator)?;

        if let Some(c) = namespace.bytes().find(invalid_namespace_char) {
            return Err(InvalidCharacterInNamespace(char::from(c)));
        }
        if let Some(c) = path.bytes().find(invalid_path_char) {
            return Err(InvalidCharacterInPath(char::from(c)));
        }
        Ok(Self { namespace, path })
    }
}
impl<'a> Parse<&'a [u8]> for IdentifierRef<'a> {
    fn parse(i: &'a [u8]) -> nom::IResult<&'a [u8], Self> {
        map_res(var_str, Self::try_from)(i)
    }
}

fn invalid_namespace_char(c: &u8) -> bool {
    !matches!(*c, b'0'..=b'9' | b'a'..=b'z' | b'.' | b'_' | b'-')
}
fn invalid_path_char(c: &u8) -> bool {
    !matches!(*c, b'0'..=b'9' | b'a'..=b'z' | b'.' | b'_' | b'-' | b'/')
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, FromPrimitive)]
pub enum Arm {
    Left,
    Right,
}
impl Parse<&[u8]> for Arm {
    fn parse(i: &[u8]) -> nom::IResult<&[u8], Self> {
        map_opt(varint::<u32>, Self::from_u32)(i)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, FromPrimitive)]
pub enum Hand {
    Mainhand,
    Offhand,
}
impl Parse<&[u8]> for Hand {
    fn parse(i: &[u8]) -> nom::IResult<&[u8], Self> {
        map_opt(varint::<u32>, Self::from_u32)(i)
    }
}

#[derive(Debug, Nom)]
#[repr(u8)]
pub enum Direction {
    Bottom,
    Top,
    North,
    South,
    West,
    East,
}
