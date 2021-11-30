use nom::IResult;

use crate::match_id_and_forward;

use super::Connection;

pub fn read_packet<'data>(
    conn: &mut Connection,
) -> impl FnMut(&'data [u8]) -> IResult<&'data [u8], ()> + '_ {
    match_id_and_forward! {
    }
}