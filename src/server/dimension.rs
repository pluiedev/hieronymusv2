use eyre::ContextCompat;
use serde::{ser::SerializeStruct, Serialize};

pub struct DimensionManager {
    pub dimension_types: Vec<Entry<DimensionType>>,
    biomes: Vec<Entry<BiomeProperties>>,

    current_dimension: i32,
}
impl DimensionManager {
    pub fn new() -> Self {
        let dimension_type = DimensionType {
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
        let dimension_types = vec![Entry {
            name: "hieronymus:wonderland".into(),
            id: 0,
            element: dimension_type.clone(),
        }];

        let biome = BiomeProperties {
            precipitation: "rain".into(),
            depth: 0.125,
            temperature: 0.8,
            scale: 0.05,
            downfall: 0.4,
            category: "plains".into(),
            temperature_modifier: None,
            effects: BiomeEffects {
                sky_color: 0x7fa1ff,
                water_fog_color: 0x7fa1ff,
                fog_color: 0x7fa1ff,
                water_color: 0x7fa1ff,
                foliage_color: None,
                grass_color: None,
                grass_color_modifier: None,
                music: None,
                ambient_sound: None,
                additions_sound: None,
                mood_sound: None,
            },
            particle: None,
        };
        let biomes = vec![Entry {
            name: "minecraft:plains".into(),
            id: 1,
            element: biome,
        }];
        Self {
            dimension_types,
            biomes,
            current_dimension: 0,
        }
    }

    pub fn current_dimension(&self) -> &DimensionType {
        self.dimension_types
            .iter()
            .find(|e| e.id == self.current_dimension)
            .map(|e| &e.element)
            .wrap_err("current dimension not found! what??")
            .unwrap()
    }
}
impl Serialize for DimensionManager {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut codec = serializer.serialize_struct("DimensionCodec", 2)?;
        codec.serialize_field(
            "minecraft:dimension_type",
            &Registry {
                registry_type: "minecraft:dimension_type",
                value: &self.dimension_types,
            },
        )?;
        codec.serialize_field(
            "minecraft:worldgen/biome",
            &Registry {
                registry_type: "minecraft:worldgen/biome",
                value: &self.biomes,
            },
        )?;
        codec.end()
    }
}

#[derive(Debug, Serialize)]
struct Registry<'a, T> {
    #[serde(rename = "type")]
    registry_type: &'a str,
    value: &'a [T],
}

#[derive(Debug, Serialize, Clone)]
pub struct Entry<T: Clone> {
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

#[derive(Debug, Serialize, Clone)]
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

#[derive(Debug, Serialize, Clone)]
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

#[derive(Debug, Serialize, Clone)]
pub struct BiomeMusic {
    pub replace_current_music: bool,
    pub sound: String,
    pub max_delay: i32,
    pub min_delay: i32,
}
#[derive(Debug, Serialize, Clone)]
pub struct BiomeAdditionsSound {
    pub sound: String,
    pub tick_chance: f64,
}
#[derive(Debug, Serialize, Clone)]
pub struct BiomeMoodSound {
    pub sound: String,
    pub tick_delay: i32,
    pub offset: f64,
    pub block_search_extent: i32,
}
#[derive(Debug, Serialize, Clone)]
pub struct BiomeParticle {
    pub probability: f32,
    pub options: Option<BiomeParticleOptions>,
}
#[derive(Debug, Serialize, Clone)]
pub struct BiomeParticleOptions {
    #[serde(rename = "type")]
    pub particle_type: String,
}
