mod nbt;

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
        0x0a => PluginMessage
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

impl Connection {
    pub async fn join_game(&mut self, player: Player) -> eyre::Result<()> {
        self.server.join_game(player).await?;

        // Join game
        let dimension_type = nbt::DimensionType {
            piglin_safe: false,
            natural: true,
            ambient_light: 0.0,
            fixed_time: None,
            infiniburn: "hieronymus:infiniburn_wonderland".into(),
            respawn_anchor_works: false,
            has_skylight: true,
            bed_works: true,
            effects: "hieronymus:wonderland".into(),
            has_raids: false,
            min_y: 0,
            height: 256,
            logical_height: 256,
            coordinate_scale: 1.0,
            ultrawarm: false,
            has_ceiling: false,
        };
        let biome = nbt::BiomeProperties {
            precipitation: "rain".into(),
            depth: 0.125,
            temperature: 0.8,
            scale: 0.05,
            downfall: 0.4,
            category: "plains".into(),
            temperature_modifier: None,
            effects: None,
            particle: None,
        };
        let biome = nbt::BiomeEntry {
            name: "plains".into(),
            id: 1,
            element: biome,
        };
        let dimension_codec = nbt::DimensionCodec {
            dimension_type: vec![dimension_type.clone()],
            biome: vec![biome],
        };

        ResponseBuilder::new(0x26)
            .add(0) // EID
            .add(false) // not hardcore
            .add(0) // survival
            .add(-1) // no previous gamemode
            .add_many(&["hieronymus:wonderland"]) // world names
            .nbt(dimension_codec)? // dimension codec
            .nbt(dimension_type)? // dimension
            .add("hieronymus:wonderland") // current world name
            .add(rand::random::<u64>()) // hashed seed
            .varint(0u32) // max players (ignored)
            .varint(10u32) // view distance
            .varint(10u32) // simulation distance
            .add(false) // reduced debug info
            .add(true) // enable respawn screen
            .add(false) // is debug world
            .add(false) // is superflat
            .send(self)
            .await?;

        // prematurely kick
        self.kick(
            r#"{"text":"well... i haven't implemented like, the game yet lol. come back later XD"}"#
        ).await?;

        Ok(())
    }
}
