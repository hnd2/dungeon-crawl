use crate::ldtk::Ldtk;
use crate::pos::Direction;
use bevy::prelude::*;
use std::collections::HashSet;
use std::convert::From;
use std::iter::FromIterator;
use std::str::FromStr;

pub struct Dungeon {
    pub levels: Vec<Level>,
}
#[derive(Resource, Clone)]
pub struct Level {
    pub width: i32,
    pub length: i32,
    pub tiles: Vec<Tile>,
    pub entities: Vec<Entity>,
}
impl Level {
    pub fn get_tile(&self, x: i32, z: i32) -> Option<&Tile> {
        self.tiles.iter().find(|t| t.x == x && t.z == z)
    }

    pub fn get_entity(&self, x: i32, z: i32) -> Option<&Entity> {
        let entity = self.entities.iter().find(|e| e.x == x && e.z == z);

        return if entity.is_none() {
            None
        } else {
            match entity.unwrap().entity_type {
                EntityType::PlayerStart => None,
                EntityType::Cat => entity,
            }
        };
    }
}
#[derive(Clone)]
pub struct Tile {
    pub x: i32,
    pub z: i32,
    pub walls: HashSet<Direction>,
}
impl Tile {
    pub fn has_wall(&self, direction: &Direction) -> bool {
        self.walls.contains(direction)
    }
}
#[derive(Clone)]
pub struct Entity {
    pub x: i32,
    pub z: i32,
    pub entity_type: EntityType,
    pub direction: Direction,
    pub message: Option<String>,
}
#[derive(Clone)]
pub enum EntityType {
    PlayerStart,
    Cat,
}
impl FromStr for EntityType {
    type Err = ();
    fn from_str(input: &str) -> Result<EntityType, Self::Err> {
        return match input.to_lowercase().as_str() {
            "playerstart" => Ok(EntityType::PlayerStart),
            "cat" => Ok(EntityType::Cat),
            _ => Err(()),
        };
    }
}

impl From<&Ldtk> for Dungeon {
    fn from(ldtk: &Ldtk) -> Self {
        let default_grid_size = ldtk.default_grid_size;
        Dungeon {
            levels: ldtk
                .levels
                .iter()
                .map(|level| {
                    let mut tiles: Vec<Tile> = vec![];
                    let mut entities: Vec<Entity> = vec![];
                    let width = (level.px_wid / default_grid_size) as i32;
                    let length = (level.px_hei / default_grid_size) as i32;

                    if let Some(layer_instances) =
                        Option::Some(level).and_then(|level| level.layer_instances.as_ref())
                    {
                        for layer_instance in layer_instances.iter() {
                            let grid_size = (layer_instance.c_wid, layer_instance.c_wid);
                            if layer_instance.identifier == "Entities" {
                                entities = layer_instance
                                    .entity_instances
                                    .iter()
                                    .map(|entity| {
                                        let direction = entity
                                            .field_instances
                                            .iter()
                                            .find(|field_instance| {
                                                field_instance.identifier == "Direction"
                                            })
                                            .and_then(|field_instance| {
                                                match field_instance.value.as_ref() {
                                                    Some(serde_json::Value::String(s)) => Some(s),
                                                    _ => None,
                                                }
                                            })
                                            .and_then(|s| {
                                                Some(
                                                    s.as_str()
                                                        .parse::<Direction>()
                                                        .unwrap_or(Direction::Right),
                                                )
                                            });
                                        let identifier =
                                            entity.identifier.parse::<EntityType>().unwrap();
                                        let x = entity.grid[0] as i32;
                                        let z = entity.grid[1] as i32;

                                        let message = entity
                                            .field_instances
                                            .iter()
                                            .find(|field_instance| {
                                                field_instance.identifier == "Message"
                                            })
                                            .and_then(|field_instance| {
                                                match field_instance.value.as_ref() {
                                                    Some(serde_json::Value::String(s)) => {
                                                        Some(s.to_owned())
                                                    }
                                                    _ => None,
                                                }
                                            });
                                        Entity {
                                            x: x,
                                            z: z,
                                            entity_type: identifier,
                                            direction: direction.unwrap_or(Direction::Right),
                                            message: message,
                                        }
                                    })
                                    .collect();
                            } else if layer_instance.identifier == "Tiles" {
                                let tileset = layer_instance
                                    .tileset_def_uid
                                    .and_then(|uid| {
                                        ldtk.defs.tilesets.iter().find(|tileset| tileset.uid == uid)
                                    })
                                    .unwrap();
                                tiles = layer_instance
                                    .grid_tiles
                                    .iter()
                                    .map(|tile| {
                                        let x = (tile.px[0] / grid_size.0) as i32;
                                        let z = (tile.px[1] / grid_size.1) as i32;
                                        let directions = tileset
                                            .custom_data
                                            .iter()
                                            .find(|d| d.tile_id == tile.t)
                                            .map(|d| d.data.as_str())
                                            .and_then(|s| {
                                                if s.is_empty() {
                                                    None
                                                } else {
                                                    Some(
                                                        s.split(',')
                                                            .map(|s| s.parse::<Direction>())
                                                            .flat_map(|d| d)
                                                            .collect(),
                                                    )
                                                }
                                            })
                                            .unwrap_or(vec![]);
                                        Tile {
                                            x: x,
                                            z: z,
                                            walls: HashSet::from_iter(directions),
                                        }
                                    })
                                    .collect()
                            }
                        }
                    }
                    Level {
                        width: width,
                        length: length,
                        tiles: tiles,
                        entities: entities,
                    }
                })
                .collect(),
        }
    }
}