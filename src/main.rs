//! Implementation of common graph functions in terms of Bevy's ECS system.
//! 
//! Focuses on path algorithms, such as:
//! 
//! 
//! 
//!
//! 


pub mod graph_functions;
mod types;
mod filteredgraphquery;

#[cfg(test)]
mod tests;

use bevy::ecs::{world::World, system::{SystemState, Query}};
use filteredgraphquery::FilteredGraphQuery;
pub use types::*; 

fn main(){
    let mut world = World::new();
    let mut state: SystemState<Query<(&TestGraphVertex, &GraphLabel)>> = SystemState::new(&mut world);
    let query = state.get(&world);

    let mut filtered: FilteredGraphQuery<'_, '_, '_, _, _, TestGraphVertex> = FilteredGraphQuery::new(&query);
    filtered.add_filter(|label: &GraphLabel| {label.value < 20});

}






