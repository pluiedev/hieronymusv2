//! Utilities for reading [Minecraft's variable-length integers](https://wiki.vg/Protocol#VarInt_and_VarLong).
//!
//! Minecraft's network protocol uses a variation of
//! [ProtoBuf's `VarInt`s](`https://developers.google.com/protocol-buffers/docs/encoding#varints`),
//! to encode integers space-efficiently, and there's a [*ton*](https://docs.rs/integer-encoding)
//! [of](https://docs.rs/varint) [crates](https://docs.rs/mc-varint) that do this.
//!
//! The problem is... those are all ill-suited for my purposes.
//! What I would like, is a [generic trait](VarInt) that can be implemented on
//! *any* multi-byte integer, a [generic algorithm](varint) to read those `VarInt`s
//! from a byte stream that also works in synergy with [`nom`](https://docs.rs/nom).
//!
//! And I think I've made just that. Enjoy.
use eyre::eyre;
use std::ops::{BitAnd, BitOr, Shl, ShrAssign};

use nom::{
    bytes::streaming::take_while_m_n,
    combinator::{map, recognize},
    number::streaming::be_u8,
    sequence::pair,
    IResult,
};

/// A trait offering all the necessary information [`varint`] needs to deserialize
/// the implementor successfully as a variable-length integer.
///
/// Implemented on all integer types except for [`u8`], [`i8`], [`usize`] and
/// [`isize`] by default.

// who needs `num-traits` when you can have `num-traits` at home
// `num-traits` at home:
pub trait VarInt:
    BitAnd<Output = Self>
    + BitOr<Output = Self>
    + Shl<Output = Self>
    + ShrAssign
    + PartialEq
    + Copy
    + From<u8>
    + Sized
{
    /// The maximum number of bytes a variable-length integer of this type can occupy.
    const MAX_SIZE: usize;
    /// The zero (0) value of this type.
    const ZERO: Self;
    /// The amount the integer must shift left or right for each byte. Currently
    /// seven (7) for all types.
    ///
    /// Alternatively, think of this as how many significant bits (i.e. bits not
    /// used as markers) each byte encodes.
    const SHIFT_CONSTANT: Self;
    /// The bitmask used to determine if the varint is done writing.
    /// Equals to `!0x7f` for all integers by default.
    const END_MASK: Self;

    fn bottom_u8(self) -> u8;
}

pub fn varint<V: VarInt>(i: &[u8]) -> IResult<&[u8], V> {
    // thanks Nemo157#0157 on Discord for optimizing this to this level.
    // you're a true nom wizard.
    map(
        // nom can't grab one more byte after it's done reading, so this is
        // needed, though IMO it kinda sucks
        recognize(pair(
            take_while_m_n(0, V::MAX_SIZE, |v| v & 0x80 == 0x80),
            be_u8,
        )),
        |bytes: &[u8]| {
            bytes.iter().rev().fold(V::ZERO, |acc, &b| {
                acc << V::SHIFT_CONSTANT | V::from(b & 0x7f)
            })
        },
    )(i)
}

pub fn serialize_and_append<V: VarInt>(mut v: V, buf: &mut Vec<u8>) -> eyre::Result<()> {
    for _ in 0..V::MAX_SIZE {
        if v & V::END_MASK == V::ZERO {
            buf.push(v.bottom_u8());
            return Ok(());
        }
        buf.push(v.bottom_u8() | 0x80);
        v >>= V::SHIFT_CONSTANT;
    }
    Err(eyre!("overflow when converting varint to bytes"))
}

#[inline]
pub fn serialize_to_bytes<V: VarInt>(v: V) -> eyre::Result<Vec<u8>> {
    let mut buf = vec![];
    serialize_and_append(v, &mut buf)?;
    Ok(buf)
}

#[cfg(test)]
mod tests {
    use nom::Finish;

    use super::VarInt;

    #[test]
    fn it_works() {
        // u16
        verify(0u16, &[0x00]);
        verify(1u16, &[0x01]);
        verify(2u16, &[0x02]);
        verify(3u16, &[0x03]);
        verify(127u16, &[0x7f]);
        verify(128u16, &[0x80, 0x01]);
        verify(255u16, &[0xff, 0x01]);
        verify(0x3fffu16, &[0xff, 0x7f]);
        verify(0xffffu16, &[0xff, 0xff, 0x03]);
        // u32
        verify(0x0fff_ffffu32, &[0xff, 0xff, 0xff, 0x7f]);
        verify(0xffff_ffffu32, &[0xff, 0xff, 0xff, 0xff, 0x0f]);
        // u64
        verify(
            0x7fff_ffff_ffff_ffffu64,
            &[0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0x7f],
        );
        verify(
            0xffff_ffff_ffff_ffffu64,
            &[0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0x01],
        );
        // u128. a bit ridiculous
        verify(
            0x3fff_ffff_ffff_ffff_ffff_ffff_ffff_ffffu128,
            &[
                0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
                0xff, 0xff, 0xff, 0x7f,
            ],
        );
        verify(
            0xffff_ffff_ffff_ffff_ffff_ffff_ffff_ffffu128,
            &[
                0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
                0xff, 0xff, 0xff, 0xff, 0x03,
            ],
        );
    }

    fn verify<V: VarInt + std::fmt::Debug>(expected: V, data: &[u8]) {
        let (rest, actual): (&[u8], V) = super::varint(data).finish().unwrap();
        assert_eq!(expected, actual);
        assert!(rest.is_empty());
    }
}

macro_rules! varint_impl {
    ($($ty:ty => $max:expr),+) => {
        $(
            impl VarInt for $ty {
                const MAX_SIZE: usize = $max;
                const ZERO: Self = 0;
                const SHIFT_CONSTANT: Self = 7;
                const END_MASK: Self = !0x7f;

                fn bottom_u8(self) -> u8 {
                    (self & 0xff) as u8
                }
            }
        )+
    };
}
varint_impl!(
    u16 => 3,
    i16 => 3,
    u32 => 5,
    i32 => 5,
    u64 => 10,
    i64 => 10,
    u128 => 19,
    i128 => 19
);
