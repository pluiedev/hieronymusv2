use nom::{combinator::map, number::streaming::be_u64, IResult};

use crate::{match_id_and_forward, nom::nop};

use super::{Connection, PacketHandler};

pub fn read_packet<'data>(
    conn: &mut Connection,
) -> impl FnMut(&'data [u8]) -> IResult<&'data [u8], ()> + '_ {
    match_id_and_forward! {
        0 => map(nop, |_| conn.handle(Status)),
        1 => map(be_u64, |payload| conn.handle(Ping(payload)))
    }
}

struct Status;
impl PacketHandler<Status> for Connection {
    fn handle(&mut self, packet: Status) {
        todo!()
    }
}

struct Ping(u64);
impl PacketHandler<Ping> for Connection {
    fn handle(&mut self, packet: Ping) {
        todo!()
    }
}
