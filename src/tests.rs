
use bevy::ecs::{
    world::World, 
    entity::Entity, 
    system::{
        SystemState, 
        Query
    }
};

use crate::{
    graph_functions::{
        load_graph, 
        bfs, 
        dfs, 
        dijkstra_search
    }, 
    GraphLabel, 
    TestGraphVertex
};




#[test]
fn breadth_first_search_test() {
    //load the test graph
    let mut world = World::new();
    load_graph(&mut world, "./assets/test_graph.graph");

    //determine the entity ids for the desired vertices so they can be used to test the algorithm
    let entity_start = get_entity_with_label(&mut world, 1).expect("The given label should exist");
    let entity_end = get_entity_with_label(&mut world, 5).expect("The given label should exist");

    //create system states so we can get the queries for testing
    let mut vertex_sys_state: SystemState<Query<&TestGraphVertex>> = SystemState::new(&mut world);
    let mut label_sys_state: SystemState<Query<&GraphLabel>> = SystemState::new(&mut world);

    //get a query for the vertices and one for the labels as if inside a system
    let vert_query = vertex_sys_state.get(&world);
    let label_query = label_sys_state.get(&world);

    //run the algorithm and turn the result from Entities to Label values
    let result = bfs(entity_start, entity_end, &vert_query).expect("Test graph should have a valid path between the test vertices");
    let labels: Vec<usize> = result.into_iter()
    .filter_map(|ent| label_query.get(ent).ok().and_then(|b| Some(b.value))).collect();

    assert_eq!(labels, vec![5, 10, 15, 14, 19, 13, 12, 3, 2, 1]);
}

#[test]
fn depth_first_search_test() {
    //load the test graph
    let mut world = World::new();
    load_graph(&mut world, "./assets/test_graph.graph");

    //determine the entity ids for the desired vertices so they can be used to test the algorithm
    let entity_start = get_entity_with_label(&mut world, 1).expect("The given label should exist");
    let entity_end = get_entity_with_label(&mut world, 5).expect("The given label should exist");

    //create system states so we can get the queries for testing
    let mut vertex_sys_state: SystemState<Query<&TestGraphVertex>> = SystemState::new(&mut world);
    let mut label_sys_state: SystemState<Query<&GraphLabel>> = SystemState::new(&mut world);

    //get a query for the vertices and one for the labels as if inside a system
    let vert_query = vertex_sys_state.get(&world);
    let label_query = label_sys_state.get(&world);

    //run the algorithm and turn the result from Entities to Label values
    let result = dfs(entity_start, entity_end, &vert_query).expect("Test graph should have a valid path between the test vertices");
    let labels: Vec<usize> = result.into_iter()
    .filter_map(|ent| label_query.get(ent).ok().and_then(|b| Some(b.value))).collect();

    assert_eq!(labels, vec![5, 10, 15, 14, 19, 13, 12, 3, 2, 1]);
}

#[test]
fn dijkstra_search_test() {
    //load the test graph
    let mut world = World::new();
    load_graph(&mut world, "./assets/test_graph.graph");

    //determine the entity ids for the desired vertices so they can be used to test the algorithm
    let entity_start = get_entity_with_label(&mut world, 1).expect("The given label should exist");
    let entity_end = get_entity_with_label(&mut world, 5).expect("The given label should exist");

    //create system states so we can get the queries for testing
    let mut vertex_sys_state: SystemState<Query<&TestGraphVertex>> = SystemState::new(&mut world);
    let mut label_sys_state: SystemState<Query<&GraphLabel>> = SystemState::new(&mut world);

    //get a query for the vertices and one for the labels as if inside a system
    let vert_query = vertex_sys_state.get(&world);
    let label_query = label_sys_state.get(&world);

    //run the algorithm and turn the result from Entities to Label values
    let result = dijkstra_search(entity_start, entity_end, &vert_query).expect("Test graph should have a valid path between the test vertices");
    let labels: Vec<usize> = result.into_iter()
    .filter_map(|ent| label_query.get(ent).ok().and_then(|b| Some(b.value))).collect();

    assert_eq!(labels, vec![5, 10, 15, 14, 19, 13, 12, 7, 2, 1]);
}




/// Helper function that returns the Entity with corresponding GraphLabel value
fn get_entity_with_label(mut world: &mut World, label: usize) -> Option<Entity> {
    let mut label_sys_state: SystemState<Query<(Entity, &GraphLabel)>> = SystemState::new(&mut world);
    let label_query = label_sys_state.get(&world);
    label_query.iter().find_map(|(ent, lab)| if lab.value == label {Some(ent)} else {None})
}

