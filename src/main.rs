pub mod graph_functions;
pub mod types;

use bevy::ecs::{world::World, system::{SystemState, Query}, entity::Entity};
pub use types::*;
pub use graph_functions::*;

fn main(){
    let mut world = World::new();
    load_graph(&mut world, "./assets/test_graph.graph");
    let mut system_state_1: SystemState<Query<&GraphVertex>> = SystemState::new(&mut world);
    let mut system_state_2: SystemState<Query<(Entity, &GraphLabel)>> = SystemState::new(&mut world);

    let vert_query = system_state_1.get(&world);
    let label_vert_query = system_state_2.get(&world);

    let mut entity_start = Entity::PLACEHOLDER;
    let mut entity_end = Entity::PLACEHOLDER;
    for (ent, label) in label_vert_query.iter(){
        if label.value == 19 {entity_start = ent}
        else if label.value == 3 {entity_end = ent}
    }
    
    let path1 = dfs(entity_start, entity_end, &vert_query).unwrap();
    let path2 = bfs(entity_start, entity_end, &vert_query).unwrap();
    let path3 = dijkstra_search(entity_start, entity_end, &vert_query, NoneValueResponse::Impassable).unwrap();
    let path4 = dijkstra_search(entity_start, entity_end, &vert_query, NoneValueResponse::Value(0.0)).unwrap();

    let labels1 : Vec<usize> = path1.into_iter().map(|ent| label_vert_query.get(ent).unwrap().1.value).collect();
    let labels2 : Vec<usize> = path2.into_iter().map(|ent| label_vert_query.get(ent).unwrap().1.value).collect();
    let labels3 : Vec<usize> = path3.into_iter().map(|ent| label_vert_query.get(ent).unwrap().1.value).collect();
    let labels4 : Vec<usize> = path4.into_iter().map(|ent| label_vert_query.get(ent).unwrap().1.value).collect();

    println!("Depth first:");
    println!("{:?}\n",labels1);
    println!("Breadth first:");
    println!("{:?}\n",labels2);
    println!("Dijkstra impassable:");
    println!("{:?}\n",labels3);
    println!("Dijkstra 0:");
    println!("{:?}\n",labels4);
}