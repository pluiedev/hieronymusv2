use serde::{Serialize, ser::SerializeStruct};

#[derive(Debug)]
pub struct DimensionCodec {
    pub dimension_type: Vec<Entry<DimensionType>>,
    pub biome: Vec<Entry<BiomeProperties>>,
}
impl Serialize for DimensionCodec {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut codec = serializer.serialize_struct("DimensionCodec", 2)?;
        codec.serialize_field("minecraft:dimension_type", &Registry {
            registry_type: "minecraft:dimension_type",
            value: &self.dimension_type
        })?;
        codec.serialize_field("minecraft:worldgen/biome", &Registry {
            registry_type: "minecraft:worldgen/biome",
            value: &self.biome
        })?;
        codec.end()
    }
}

#[derive(Debug, Serialize)]
struct Registry<'a, T> {
    #[serde(rename = "type")]
    registry_type: &'a str,
    value: &'a [T],
}

#[derive(Debug, Serialize)]
pub struct Entry<T> {
    pub name: String,
    pub id: i32,
    pub element: T,
}

#[derive(Debug, Clone, Serialize)]
pub struct DimensionType {
    pub piglin_safe: bool,
    pub natural: bool,
    pub ambient_light: f32,
    pub fixed_time: Option<i64>,
    pub infiniburn: String,
    pub respawn_anchor_works: bool,
    pub has_skylight: bool,
    pub bed_works: bool,
    pub effects: String,
    pub has_raids: bool,
    pub min_y: i32,
    pub height: i32,
    pub logical_height: i32,
    pub coordinate_scale: f32,
    pub ultrawarm: bool,
    pub has_ceiling: bool,
}

#[derive(Debug, Serialize)]
pub struct BiomeProperties {
    pub precipitation: String,
    pub depth: f32,
    pub temperature: f32,
    pub scale: f32,
    pub downfall: f32,
    pub category: String,
    pub temperature_modifier: Option<String>,
    pub effects: BiomeEffects,
    pub particle: Option<BiomeParticle>,
}

#[derive(Debug, Serialize)]
pub struct BiomeEffects {
    pub sky_color: i32,
    pub water_fog_color: i32,
    pub fog_color: i32,
    pub water_color: i32,
    pub foliage_color: Option<i32>,
    pub grass_color: Option<i32>,
    pub grass_color_modifier: Option<String>,
    pub music: Option<BiomeMusic>,
    pub ambient_sound: Option<String>,
    pub additions_sound: Option<BiomeAdditionsSound>,
    pub mood_sound: Option<BiomeMoodSound>,
}

#[derive(Debug, Serialize)]
pub struct BiomeMusic {
    pub replace_current_music: bool,
    pub sound: String,
    pub max_delay: i32,
    pub min_delay: i32,
}
#[derive(Debug, Serialize)]
pub struct BiomeAdditionsSound {
    pub sound: String,
    pub tick_chance: f64,
}
#[derive(Debug, Serialize)]
pub struct BiomeMoodSound {
    pub sound: String,
    pub tick_delay: i32,
    pub offset: f64,
    pub block_search_extent: i32,
}
#[derive(Debug, Serialize)]
pub struct BiomeParticle {
    pub probability: f32,
    pub options: Option<BiomeParticleOptions>,
}
#[derive(Debug, Serialize)]
pub struct BiomeParticleOptions {
    #[serde(rename = "type")]
    pub particle_type: String,
}