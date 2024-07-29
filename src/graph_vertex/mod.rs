use bevy::prelude::{Component, Entity};



pub trait GraphVertex : Component {
    fn get_neighbours(&self) -> Vec<Entity>;
    fn get_neighbours_with_weight(&self) -> Vec<(Entity, f32)>;
}

#[derive(Component)]
pub struct StandardGraphVertex {
    neighbours: Vec<(Entity, f32)>
}

impl StandardGraphVertex{
    pub fn new() -> Self{
        Self{neighbours: Vec::new()}
    }
    pub fn new_with_edges(edges: Vec<(Entity, f32)>) -> Self{
        Self{neighbours: edges}
    }
    pub fn add_edge(&mut self, other_vertex: Entity){
        todo!();
    }
    pub fn remove_edge(&mut self, other_vertex: Entity){
        todo!();
    }
    pub fn change_weight_of(&mut self, other_vertex: Entity, new_weight: f32){
        todo!()
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

