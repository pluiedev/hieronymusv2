use nom::{combinator::map, sequence::tuple, IResult};

use crate::{
    match_id_and_forward,
    nom::{maybe, var_bytes, var_str_with_max_length},
};

use super::{Connection, PacketHandler};

pub fn read_packet<'data>(
    conn: &mut Connection,
) -> impl FnMut(&'data [u8]) -> IResult<&'data [u8], ()> + '_ {
    match_id_and_forward! {
        0 => map(var_str_with_max_length(16u32), |username| conn.handle(LoginStart { username })),
        1 => map(
            tuple((var_bytes, var_bytes)),
            |(shared_secret, verify_token)| conn.handle(EncryptionResponse {
                shared_secret,
                verify_token
            })
        ),
        2 => map(
            tuple((
                varint,
                maybe(var_bytes),
            )),
            |(message_id, data)| conn.handle(LoginPluginResponse {
                message_id,
                data
            })
        )
    }
}

struct LoginStart<'a> {
    username: &'a str,
}
impl PacketHandler<LoginStart<'_>> for Connection {
    fn handle(&mut self, packet: LoginStart<'_>) {
        todo!()
    }
}

struct EncryptionResponse<'a> {
    shared_secret: &'a [u8],
    verify_token: &'a [u8],
}
impl PacketHandler<EncryptionResponse<'_>> for Connection {
    fn handle(&mut self, packet: EncryptionResponse<'_>) {
        todo!()
    }
}

struct LoginPluginResponse<'a> {
    message_id: u32,
    data: Option<&'a [u8]>,
}
impl PacketHandler<LoginPluginResponse<'_>> for Connection {
    fn handle(&mut self, packet: LoginPluginResponse<'_>) {
        todo!()
    }
}
