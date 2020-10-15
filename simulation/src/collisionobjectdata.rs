use ncollide2d::pipeline::CollisionObjectSlabHandle;
use std::cell::Cell;

use crate::entity::Entity;

#[derive(Clone)]
pub struct CollisionObjectData {
    pub entity_type: Entity,
    pub id: i32,
    pub env_handle: Option<CollisionObjectSlabHandle>,
    pub fitness: Cell<i32>,
    pub eaten: Cell<bool>,
    pub energy: Cell<i32>,
    pub score: Cell<i32>,
}

impl CollisionObjectData {
    pub fn new(entity_type: Entity, id: i32, env_handle: Option<CollisionObjectSlabHandle>) -> CollisionObjectData {
        CollisionObjectData {
            entity_type: entity_type,
            id: id,
            env_handle: env_handle,
            fitness: Cell::new(0),
            eaten: Cell::new(false),
            energy: Cell::new(400),
            score: Cell::new(0),
        }
    }
}