//! `nom` utilities.

use std::ops::RangeFrom;

use nom::{
    combinator::{map, map_opt, map_res},
    error::ParseError,
    multi::length_data,
    number::streaming::be_u8,
    IResult, InputIter, InputLength, Parser, Slice, ToUsize,
};
use nom_derive::Nom;

use crate::{
    net::ConnectionState,
    varint::{varint, VarInt},
};

#[macro_export]
macro_rules! match_id_and_forward {
    {$input:expr; $($id:expr => $ty:ty),*} => {{
        use nom::{Err::Failure, error::{ErrorKind, make_error}};
        use tracing::trace;
        let input = $input;
        trace!(?input);
        let (input, id) = crate::varint::varint::<u32>(input)?;
        trace!(?input, ?id);
        Ok(match id {
            $(
                $id => {
                    let (input, output) = <$ty as nom_derive::Parse<&[u8]>>::parse(input)?;
                    (input, Box::new(output))
                }
            )*
            _ => return Err(Failure(make_error($input, ErrorKind::Alt))),
        })
    }};
}

//region Generalized idioms

/// A parser that tries to parse a [`bool`] value from a read [`u8`].
///
/// Zero (0) maps to `false`, one (1) maps to `true`, and any other value
/// triggers an error in the parser.
pub fn boolean<I, E: ParseError<I>>(i: I) -> IResult<I, bool, E>
where
    I: Slice<RangeFrom<usize>> + InputIter<Item = u8> + InputLength + Clone,
{
    map_opt(be_u8, |v| match v {
        0 => Some(false),
        1 => Some(true),
        _ => None,
    })(i)
}

/// A parser that may or may not parse additional data, based on a read
/// [boolean flag](boolean).
pub fn maybe<I, T, E, P>(mut parser: P) -> impl FnMut(I) -> IResult<I, Option<T>, E>
where
    I: Slice<RangeFrom<usize>> + InputIter<Item = u8> + InputLength + Clone,
    P: Parser<I, T, E>,
    E: ParseError<I>,
{
    move |i| {
        let (i, b) = boolean(i)?;
        match b {
            true => parser.parse(i).map(|(i, v)| (i, Some(v))),
            false => Ok((i, None)),
        }
    }
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

//endregion
//region Byte slice-specific operations

/// A parser that reads a variable-length byte slice.
///
/// Length is read as a [variable-length integer](varint) prefixed before the
/// actual data.
pub fn var_bytes(i: &[u8]) -> IResult<&[u8], &[u8]> {
    length_data(varint::<u32>)(i)
}

/// Returns a parser that reads a variable-length UTF-8 string with a maximum
/// length of 32767 bytes.
///
/// Length is read as a [variable-length integer](varint) prefixed before the
/// actual data.
///
/// The parser will read at most (*size of length field* + 32767) bytes of data.
/// Use [`var_str_with_max_length`] if customizing the maximum length is needed.
#[inline]
pub fn var_str(i: &[u8]) -> IResult<&[u8], &str> {
    var_str_with_max_length(32767u32)(i)
}

/// Returns a parser that reads a variable-length UTF-8 string with a maximum
/// length in bytes.
///
/// Length is read as a [variable-length integer](varint) prefixed before the
/// actual data.
///
/// The parser will read at most (*size of length field* + `max_length`) bytes
/// of data.
pub fn var_str_with_max_length<V>(max_length: V) -> impl Fn(&[u8]) -> IResult<&[u8], &str>
where
    V: VarInt + Ord + ToUsize,
{
    move |i| {
        map_res(
            length_data(map(varint::<V>, |len| len.min(max_length))),
            std::str::from_utf8,
        )(i)
    }
}

/// Reads a [`ConnectionState`].
///
/// Only used with handshake packets to determine the state to progress to.
pub fn connection_state(i: &[u8]) -> IResult<&[u8], ConnectionState> {
    map_opt(varint::<u32>, |v| match v {
        1 => Some(ConnectionState::Status),
        2 => Some(ConnectionState::Login),
        _ => None,
    })(i)
}
//endregion
//region Misc

#[cfg(test)]
mod tests {
    use crate::nom::var_str;

    #[test]
    fn test_read_var_str() {
        fn test(input: &[u8], expected: &str) {
            let (input, actual) = var_str(input).unwrap();
            assert!(input.is_empty());
            assert_eq!(actual, expected);
        }
        test(b"\x00", "");
        test(b"\x01!", "!");
        test(b"\x05hello", "hello");
        test(
            b"\x19a slightly longer example",
            "a slightly longer example",
        );
        test(
            b"\x11UTF-8 \xe6\xb5\x8b\xe8\xaf\x95 \xf0\x9f\x99\x8b",
            "UTF-8 测试 🙋",
        );
        test(
            b"\xa5\x03Testing the limits here!!! :DD
I'd just like to interject for a moment.  What you're referring to as Linux,
is in fact, GNU/Linux, or as I've recently taken to calling it, GNU plus Linux.
Linux is not an operating system unto itself, but rather another free component
of a fully functioning GNU system made useful by the GNU corelibs, shell
utilities and vital system components comprising a full OS as defined by POSIX.
",
            "Testing the limits here!!! :DD
I'd just like to interject for a moment.  What you're referring to as Linux,
is in fact, GNU/Linux, or as I've recently taken to calling it, GNU plus Linux.
Linux is not an operating system unto itself, but rather another free component
of a fully functioning GNU system made useful by the GNU corelibs, shell
utilities and vital system components comprising a full OS as defined by POSIX.
",
        )
    }
}
