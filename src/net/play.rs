use nom::IResult;

use crate::{match_id_and_forward, server::Player};

use super::{BoxedPacket, Connection};

pub fn read_packet(input: &[u8]) -> IResult<&[u8], BoxedPacket<'_>> {
    match_id_and_forward! {
        input;
    }
}

impl Connection {
    pub async fn join_game(&mut self, player: Player) -> eyre::Result<()> {
        self.server.join_game(player).await?;

        // Join game
        // let dim_codec = nbt::Blob::new();

        // RequestBuilder::new(0x26)
        //     .add(0)         // EID
        //     .add(false)    // not hardcore
        //     .add(0)          // survival
        //     .add(-1)         // no previous gamemode
        //     .add_many(&["minecraft:overworld"])

        //     .send(conn)
        //     .await?;

        // prematurely kick
        self.kick(
            r#"{"text":"well... i haven't implemented like, the game yet lol. come back later XD"}"#
        ).await?;

        Ok(())
    }
}
