use bevy::prelude::{Component, Entity};


pub trait GraphVertex : Component {
    fn get_neighbours(&self) -> Vec<Entity>;
    fn get_neighbours_with_weight(&self) -> Vec<(Entity, f32)>;
}

#[derive(Component)]
pub struct StandardGraphVertex {
    neighbours: Vec<(Entity, f32)>
}

#[allow(dead_code)]
impl StandardGraphVertex{
    pub fn new() -> Self{
        Self{neighbours: Vec::new()}
    }
    pub fn new_with_edges(edges: Vec<(Entity, f32)>) -> Self{
        Self{neighbours: edges}
    }
    pub fn add_edge(&mut self, other_vertex: Entity, weight: f32) -> bool{
        let exists = self.neighbours.iter()
        .any(|(ent, _)| *ent == other_vertex);

        if !exists {
            self.neighbours.push((other_vertex, weight));
        }

        exists
    }
    pub fn remove_edge(&mut self, other_vertex: Entity) -> bool{
        self.neighbours.iter()
        .position(|(ent,_)| *ent == other_vertex)
        .map(|pos| self.neighbours.swap_remove(pos))
        .is_some()
    }
    pub fn change_weight_of(&mut self, other_vertex: Entity, new_weight: f32) -> bool{
        self.neighbours.iter()
        .position(|(ent,_)| *ent == other_vertex)
        .map(|pos| self.neighbours[pos].1 = new_weight)
        .is_some()
    }
}

impl GraphVertex for StandardGraphVertex {
    fn get_neighbours(&self) -> Vec<Entity>{
        self.neighbours.iter().map(|(ent, _)| *ent).collect()
    }
    fn get_neighbours_with_weight(&self) -> Vec<(Entity, f32)> {
        self.neighbours.clone()
    }
}

