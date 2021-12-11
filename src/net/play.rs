use async_trait::async_trait;
use bitflags::bitflags;
use nom::{
    combinator::{map_opt, rest},
    IResult,
};
use nom_derive::{Nom, Parse};
use num_derive::FromPrimitive;
use num_traits::FromPrimitive;
use tracing::instrument;

use crate::{
    data::{Direction, Hand, Identifier, Position, Slot, Arm},
    match_id_and_forward,
    nom::{boolean, maybe, var_str, var_str_with_max_length},
    parse_impl_for_bitflags,
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
        0x0b => EditBook,
        0x0c => QueryEntityNbt,
        0x0d => InteractEntity,
        0x0e => GenerateStructure,
        0x0f => KeepAlive,
        0x10 => LockDifficulty,
        0x11 => PlayerPosition,
        0x12 => PlayerPositionAndRotation,
        0x13 => PlayerRotation,
        0x14 => PlayerMovement,
        0x15 => VehicleMove,
        0x16 => SteerBoat,
        0x17 => PickItem,
        0x18 => CraftRecipeRequest,
        0x19 => PlayerAbilities,
        0x1a => PlayerDigging,
        0x1b => EntityAction,
        0x1c => SteerVehicle,
        0x1d => Pong,
        0x1e => SetRecipeBookState,
        0x1f => SetDisplayedRecipe
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
    displayed_skin_parts: DisplayedSkinParts,
    main_arm: Arm,
    #[nom(Parse = "boolean")]
    enable_text_filtering: bool,
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
parse_impl_for_bitflags!(DisplayedSkinParts);

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
struct EditBook<'a> {
    hand: Hand,
    #[nom(Parse = "varint")]
    count: u32,
    #[nom(LengthCount = "varint::<u32>", Parse = "var_str")]
    entries: Vec<&'a str>,
    #[nom(Parse = "maybe(var_str)")]
    title: Option<&'a str>,
}
#[async_trait]
impl Packet for EditBook<'_> {
    #[instrument(skip(conn))]
    async fn handle(&self, conn: &mut Connection) -> eyre::Result<()> {
        Ok(())
    }
}

#[derive(Debug, Nom)]
struct QueryEntityNbt {
    #[nom(Parse = "varint")]
    transaction_id: u32,
    #[nom(Parse = "varint")]
    entity_id: u32,
}
#[async_trait]
impl Packet for QueryEntityNbt {
    #[instrument(skip(conn))]
    async fn handle(&self, conn: &mut Connection) -> eyre::Result<()> {
        Ok(())
    }
}

#[derive(Debug, Nom)]
struct InteractEntity {
    #[nom(Parse = "varint")]
    entity_id: u32,
    #[nom(Parse = "varint")]
    interaction_type: u32,
    #[nom(Parse = "{|i| EntityInteraction::parse(i, interaction_type)}")]
    interaction: EntityInteraction,
    #[nom(Parse = "boolean")]
    is_sneaking: bool,
}
#[async_trait]
impl Packet for InteractEntity {
    #[instrument(skip(conn))]
    async fn handle(&self, conn: &mut Connection) -> eyre::Result<()> {
        Ok(())
    }
}

#[derive(Debug, Nom)]
#[nom(Selector = "u32")]
enum EntityInteraction {
    #[nom(Selector = "0")]
    Interact(Hand),
    #[nom(Selector = "1")]
    Attack,
    #[nom(Selector = "2")]
    InteractAt(f32, f32, f32, Hand),
}

#[derive(Debug, Nom)]
struct GenerateStructure {
    location: Position,
    #[nom(Parse = "varint")]
    levels: u32,
    #[nom(Parse = "boolean")]
    keep_jigsaws: bool,
}
#[async_trait]
impl Packet for GenerateStructure {
    #[instrument(skip(conn))]
    async fn handle(&self, conn: &mut Connection) -> eyre::Result<()> {
        Ok(())
    }
}

#[derive(Debug, Nom)]
struct KeepAlive(u64);
#[async_trait]
impl Packet for KeepAlive {
    #[instrument(skip(conn))]
    async fn handle(&self, conn: &mut Connection) -> eyre::Result<()> {
        Ok(())
    }
}

#[derive(Debug, Nom)]
struct LockDifficulty {
    #[nom(Parse = "boolean")]
    locked: bool,
}
#[async_trait]
impl Packet for LockDifficulty {
    #[instrument(skip(conn))]
    async fn handle(&self, conn: &mut Connection) -> eyre::Result<()> {
        Ok(())
    }
}

#[derive(Debug, Nom)]
struct PacketPos {
    x: f64,
    y: f64,
    z: f64,
}
#[derive(Debug, Nom)]
struct PacketRot {
    yaw: f32,
    pitch: f32,
}

#[derive(Debug, Nom)]
struct PlayerPosition {
    pos: PacketPos,
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
    pos: PacketPos,
    rot: PacketRot,
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
    rot: PacketRot,
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

#[derive(Debug, Nom)]
struct PlayerMovement {
    #[nom(Parse = "boolean")]
    on_ground: bool,
}
#[async_trait]
impl Packet for PlayerMovement {
    #[instrument(skip(conn))]
    async fn handle(&self, conn: &mut Connection) -> eyre::Result<()> {
        Ok(())
    }
}

#[derive(Debug, Nom)]
struct VehicleMove {
    pos: PacketPos,
    rot: PacketRot,
}
#[async_trait]
impl Packet for VehicleMove {
    #[instrument(skip(conn))]
    async fn handle(&self, conn: &mut Connection) -> eyre::Result<()> {
        Ok(())
    }
}

#[derive(Debug, Nom)]
struct SteerBoat {
    #[nom(Parse = "boolean")]
    left_paddle_turning: bool,
    #[nom(Parse = "boolean")]
    right_paddle_turning: bool,
}
#[async_trait]
impl Packet for SteerBoat {
    #[instrument(skip(conn))]
    async fn handle(&self, conn: &mut Connection) -> eyre::Result<()> {
        Ok(())
    }
}

#[derive(Debug, Nom)]
struct PickItem {
    #[nom(Parse = "varint")]
    slot_id: u32,
}
#[async_trait]
impl Packet for PickItem {
    #[instrument(skip(conn))]
    async fn handle(&self, conn: &mut Connection) -> eyre::Result<()> {
        Ok(())
    }
}

#[derive(Debug, Nom)]
struct CraftRecipeRequest {
    window_id: u8,
    recipe: Identifier,
    #[nom(Parse = "boolean")]
    make_all: bool,
}
#[async_trait]
impl Packet for CraftRecipeRequest {
    #[instrument(skip(conn))]
    async fn handle(&self, conn: &mut Connection) -> eyre::Result<()> {
        Ok(())
    }
}

bitflags! {
    struct PlayerAbilities: u8 {
        const FLYING = 0x02;
    }
}
parse_impl_for_bitflags!(PlayerAbilities);
#[async_trait]
impl Packet for PlayerAbilities {
    #[instrument(skip(conn))]
    async fn handle(&self, conn: &mut Connection) -> eyre::Result<()> {
        Ok(())
    }
}

#[derive(Debug, Nom)]
struct PlayerDigging {
    #[nom(Parse = "map_opt(varint::<u32>, DiggingStatus::from_u32)")]
    status: DiggingStatus,
    location: Position,
    face: Direction,
}
#[async_trait]
impl Packet for PlayerDigging {
    #[instrument(skip(conn))]
    async fn handle(&self, conn: &mut Connection) -> eyre::Result<()> {
        Ok(())
    }
}

#[derive(Debug, FromPrimitive)]
enum DiggingStatus {
    StartedDigging,
    CancelledDigging,
    FinishedDigging,
    DropItemStack,
    DropItem,
    FinishUsing,
    SwapItem,
}

#[derive(Debug, Nom)]
struct EntityAction {
    #[nom(Parse = "varint")]
    entity_id: u32,
    #[nom(Parse = "map_opt(varint::<u32>, EntityActionVariant::from_u32)")]
    action_id: EntityActionVariant,
    #[nom(Parse = "varint")]
    jump_boost: u32,
}
#[async_trait]
impl Packet for EntityAction {
    #[instrument(skip(conn))]
    async fn handle(&self, conn: &mut Connection) -> eyre::Result<()> {
        Ok(())
    }
}

#[derive(Debug, FromPrimitive)]
enum EntityActionVariant {
    StartSneaking,
    StopSneaking,
    LeaveBed,
    StartSprinting,
    StopSprinting,
    StartJumpingWithHorse,
    StopJumpingWithHorse,
    OpenHorseInventory,
    StartFlyingWithElytra,
}

#[derive(Debug, Nom)]
struct SteerVehicle {
    sideways: f32,
    forward: f32,
    flags: SteerVehicleFlags,
}
#[async_trait]
impl Packet for SteerVehicle {
    #[instrument(skip(conn))]
    async fn handle(&self, conn: &mut Connection) -> eyre::Result<()> {
        Ok(())
    }
}
bitflags! {
    struct SteerVehicleFlags: u8 {
        const JUMP = 0x1;
        const UNMOUNT = 0x2;
    }
}
parse_impl_for_bitflags!(SteerVehicleFlags);

#[derive(Debug, Nom)]
struct Pong(u32);
#[async_trait]
impl Packet for Pong {
    #[instrument(skip(conn))]
    async fn handle(&self, conn: &mut Connection) -> eyre::Result<()> {
        Ok(())
    }
}

#[derive(Debug, Nom)]
struct SetRecipeBookState {
    #[nom(Parse = "varint")]
    book_id: u32,
    #[nom(Parse = "boolean")]
    book_open: bool,
    #[nom(Parse = "boolean")]
    filter_active: bool,
}
#[async_trait]
impl Packet for SetRecipeBookState {
    #[instrument(skip(conn))]
    async fn handle(&self, conn: &mut Connection) -> eyre::Result<()> {
        Ok(())
    }
}

#[derive(Debug, Nom)]
struct SetDisplayedRecipe {
    recipe_id: Identifier,
}
#[async_trait]
impl Packet for SetDisplayedRecipe {
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
