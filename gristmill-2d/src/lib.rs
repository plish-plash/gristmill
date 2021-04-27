pub mod renderer;

use slotmap::new_key_type;

use gristmill::geometry2d::*;
use gristmill::util::forest::Forest;

pub use renderer::Sprite;

new_key_type! {
    pub struct Entity;
}

#[derive(Default)]
struct WorldObject {
    position: Point,
    sprite: Option<Sprite>,
}

pub struct World {
    forest: Forest<Entity, WorldObject>,
    render_root: Entity,
}

impl World {
    pub fn new() -> World {
        let mut forest = Forest::new();
        let render_root = forest.add(WorldObject::default());
        World {
            forest,
            render_root,
        }
    }

    pub fn root(&self) -> Entity {
        self.render_root
    }
    pub fn get_children(&self, node: Entity) -> Vec<Entity> {
        self.forest.get_children(node)
    }
    pub fn iter_children(&self, node: Entity) -> std::slice::Iter<'_, Entity> {
        self.forest.iter_children(node)
    }

    pub fn add_sprite(&mut self, parent: Entity, position: Point, sprite: Sprite) -> Entity {
        let obj = WorldObject { position, sprite: Some(sprite) };
        self.forest.add_child(parent, obj)
    }
}