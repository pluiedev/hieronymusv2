use async_trait::async_trait;
use bitflags::bitflags;
use nom::{
    combinator::{map_opt, rest},
    number::streaming::be_u8,
    IResult,
};
use nom_derive::Nom;
use tracing::instrument;

use crate::{
    data::{Identifier, Position, Slot},
    match_id_and_forward,
    nom::{boolean, var_str_with_max_length},
    server::Player,
    varint::varint,
};

use super::{BoxedPacket, Connection, Packet, ResponseBuilder};

pub fn read_packet(input: &[u8]) -> IResult<&[u8], BoxedPacket<'_>> {
    match_id_and_forward! {
        input;
        0x00 => TeleportConfirm,
        0x01 => QueryBlockNbt,
        0x02 => SetDifficulty,
        0x03 => ChatMessage,
        0x04 => ClientStatus,
        0x05 => ClientSettings,
        0x06 => TabComplete,
        0x07 => ClickWindowButton,
        0x08 => ClickWindow,
        0x09 => CloseWindow,
        0x0a => PluginMessage,
        0x11 => PlayerPosition,
        0x12 => PlayerPositionAndRotation,
        0x13 => PlayerRotation
    }
}
#[derive(Debug, Nom)]
struct TeleportConfirm {
    #[nom(Parse = "varint")]
    teleport_id: u32,
}
#[async_trait]
impl Packet for TeleportConfirm {
    #[instrument(skip(conn))]
    async fn handle(&self, conn: &mut Connection) -> eyre::Result<()> {
        Ok(())
    }
}

#[derive(Debug, Nom)]
struct QueryBlockNbt {
    #[nom(Parse = "varint")]
    transaction_id: u32,
    location: Position,
}
#[async_trait]
impl Packet for QueryBlockNbt {
    #[instrument(skip(conn))]
    async fn handle(&self, conn: &mut Connection) -> eyre::Result<()> {
        Ok(())
    }
}

#[derive(Debug, Nom)]
struct SetDifficulty {
    new_difficulty: u8,
}
#[async_trait]
impl Packet for SetDifficulty {
    #[instrument(skip(conn))]
    async fn handle(&self, conn: &mut Connection) -> eyre::Result<()> {
        Ok(())
    }
}

#[derive(Debug, Nom)]
struct ChatMessage<'a> {
    #[nom(Parse = "var_str_with_max_length(256u32)")]
    message: &'a str,
}
#[async_trait]
impl Packet for ChatMessage<'_> {
    #[instrument(skip(conn))]
    async fn handle(&self, conn: &mut Connection) -> eyre::Result<()> {
        Ok(())
    }
}

#[derive(Debug, Nom)]
struct ClientStatus {
    #[nom(Parse = "varint")]
    action_id: u32, // todo
}
#[async_trait]
impl Packet for ClientStatus {
    #[instrument(skip(conn))]
    async fn handle(&self, conn: &mut Connection) -> eyre::Result<()> {
        Ok(())
    }
}

#[derive(Debug, Nom)]
struct ClientSettings<'a> {
    #[nom(Parse = "var_str_with_max_length(16u32)")]
    locale: &'a str,
    view_distance: u8,
    #[nom(Parse = "varint")]
    chat_mode: u32, //TODO
    #[nom(Parse = "boolean")]
    chat_colors: bool,
    #[nom(Parse = "map_opt(be_u8, DisplayedSkinParts::from_bits)")]
    displayed_skin_parts: DisplayedSkinParts,
    #[nom(Parse = "varint")]
    main_hand: u32,
    #[nom(Parse = "boolean")]
    enable_text_filtering: bool,
    #[nom(Parse = "boolean")]
    allow_server_listings: bool,
}
#[async_trait]
impl Packet for ClientSettings<'_> {
    #[instrument(skip(conn))]
    async fn handle(&self, conn: &mut Connection) -> eyre::Result<()> {
        Ok(())
    }
}
bitflags! {
    struct DisplayedSkinParts: u8 {
        const CAPE = 0x01;
        const JACKET = 0x02;
        const LEFT_SLEEVE = 0x04;
        const RIGHT_SLEEVE = 0x08;
        const LEFT_PANTS_LEG = 0x10;
        const RIGHT_PANTS_LEG = 0x20;
        const HAT = 0x40;
    }
}

#[derive(Debug, Nom)]
struct TabComplete<'a> {
    #[nom(Parse = "varint")]
    transaction_id: u32,
    #[nom(Parse = "var_str_with_max_length(32500u32)")]
    text: &'a str,
}
#[async_trait]
impl Packet for TabComplete<'_> {
    #[instrument(skip(conn))]
    async fn handle(&self, conn: &mut Connection) -> eyre::Result<()> {
        Ok(())
    }
}

#[derive(Debug, Nom)]
struct ClickWindowButton {
    window_id: u8,
    button_id: u8,
}
#[async_trait]
impl Packet for ClickWindowButton {
    #[instrument(skip(conn))]
    async fn handle(&self, conn: &mut Connection) -> eyre::Result<()> {
        Ok(())
    }
}

#[derive(Debug, Nom)]
struct ClickWindow {
    window_id: u8,
    #[nom(Parse = "varint")]
    state_id: u32,
    slot: i16,
    button: u8,
    #[nom(Parse = "varint")]
    mode: u32, // todo,
    slots: Vec<(i16, Slot)>,
    clicked_item: Slot,
}
#[async_trait]
impl Packet for ClickWindow {
    #[instrument(skip(conn))]
    async fn handle(&self, conn: &mut Connection) -> eyre::Result<()> {
        Ok(())
    }
}

#[derive(Debug, Nom)]
struct CloseWindow {
    window_id: u8,
}
#[async_trait]
impl Packet for CloseWindow {
    #[instrument(skip(conn))]
    async fn handle(&self, conn: &mut Connection) -> eyre::Result<()> {
        Ok(())
    }
}

#[derive(Debug, Nom)]
struct PluginMessage<'a> {
    channel: Identifier,
    #[nom(Parse = "rest")]
    data: &'a [u8],
}
#[async_trait]
impl Packet for PluginMessage<'_> {
    #[instrument(skip(conn))]
    async fn handle(&self, conn: &mut Connection) -> eyre::Result<()> {
        Ok(())
    }
}

#[derive(Debug, Nom)]
struct PlayerPosition {
    x: f64,
    feet_y: f64,
    z: f64,
    #[nom(Parse = "boolean")]
    on_ground: bool,
}
#[async_trait]
impl Packet for PlayerPosition {
    #[instrument(skip(conn))]
    async fn handle(&self, conn: &mut Connection) -> eyre::Result<()> {
        Ok(())
    }
}

#[derive(Debug, Nom)]
struct PlayerPositionAndRotation {
    x: f64,
    feet_y: f64,
    z: f64,
    yaw: f32,
    pitch: f32,
    #[nom(Parse = "boolean")]
    on_ground: bool,
}
#[async_trait]
impl Packet for PlayerPositionAndRotation {
    #[instrument(skip(conn))]
    async fn handle(&self, conn: &mut Connection) -> eyre::Result<()> {
        Ok(())
    }
}

#[derive(Debug, Nom)]
struct PlayerRotation {
    yaw: f32,
    pitch: f32,
    #[nom(Parse = "boolean")]
    on_ground: bool,
}

#[async_trait]
impl Packet for PlayerRotation {
    #[instrument(skip(conn))]
    async fn handle(&self, conn: &mut Connection) -> eyre::Result<()> {
        Ok(())
    }
}

impl Connection {
    #[instrument(skip(self))]
    pub async fn join_game(&mut self, player: Player) -> eyre::Result<()> {
        self.server.join_game(player).await?;

        //TODO
        let dimension_info = self.server.get_dimension_info().await?;
        // Join game

        ResponseBuilder::new(0x26)
            .add(0u32) // EID
            .add(false) // not hardcore
            .add(0u8) // survival
            .add(-1i8) // no previous gamemode
            .add_many(&["hieronymus:wonderland"]) // world names
            .raw_data(dimension_info) // dimension codec and current dimension
            .add("hieronymus:wonderland") // current world name
            .add(rand::random::<u64>()) // hashed seed
            .varint(0u32) // max players (ignored)
            .varint(10u32) // view distance
            .add(false) // reduced debug info
            .add(true) // enable respawn screen
            .add(false) // is debug world
            .add(false) // is superflat
            .send(self)
            .await?;

        use AbsOrRel::*;
        self.player_position_and_look(
            Absolute(69.0),
            Relative(0.0),
            Absolute(420.0),
            Absolute(0.0),
            Absolute(0.0),
            false,
        )
        .await?;
        // prematurely kick
        // self.kick(
        //     r#"{"text":"well... i haven't implemented like, the game yet lol. come back later XD"}"#
        // ).await?;

        Ok(())
    }

    #[instrument(skip(self))]
    pub async fn player_position_and_look(
        &mut self,
        x: AbsOrRel<f64>,
        y: AbsOrRel<f64>,
        z: AbsOrRel<f64>,
        yaw: AbsOrRel<f32>,
        pitch: AbsOrRel<f32>,
        dismount_vehicle: bool,
    ) -> eyre::Result<()> {
        let mut flags = 0;
        let x = x.unwrap_and_set_flag(0b00001, &mut flags);
        let y = y.unwrap_and_set_flag(0b00010, &mut flags);
        let z = z.unwrap_and_set_flag(0b00100, &mut flags);
        let yaw = yaw.unwrap_and_set_flag(0b01000, &mut flags);
        let pitch = pitch.unwrap_and_set_flag(0b10000, &mut flags);
        let teleport_id = rand::random::<u32>();

        ResponseBuilder::new(0x38)
            .add(x)
            .add(y)
            .add(z)
            .add(yaw)
            .add(pitch)
            .add(flags)
            .varint(teleport_id)
            .add(dismount_vehicle)
            .send(self)
            .await
    }
}

#[derive(Debug)]
pub enum AbsOrRel<T> {
    Absolute(T),
    Relative(T),
}
impl<T> AbsOrRel<T> {
    pub fn unwrap_and_set_flag(self, flag: u8, flags: &mut u8) -> T {
        match self {
            Self::Absolute(t) => t,
            Self::Relative(t) => {
                *flags |= flag;
                t
            }
        }
    }
}
