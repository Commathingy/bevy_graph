



fn breadth_first_search_test() {
    //load the test graph
    let mut world = World::new();
    load_graph(&mut world, "./assets/test_graph.graph");

    //get the entity for the start and end vertex
    let mut entity_start = Entity::PLACEHOLDER;
    let mut entity_end = Entity::PLACEHOLDER;
    for (ent, label) in label_vert_query.iter(){
        if label.value == 19 {entity_start = ent}
        else if label.value == 3 {entity_end = ent}
    }
}