use std::{collections::VecDeque, u64};

use bevy::prelude::{Component, Entity, Query};

use crate::graph_vertex::GraphVertex;

use super::{GraphError, GraphPath, VisitedNodes};



/// Runs a breadth-first search, starting at the start vertex and ending at the end vertex, returning the path in **reverse order**
/// 
/// Breadth-first search on the directed graph with [vertices](GraphVertex) in the provided [`Query`]. 
/// This is guaranteed to find a path with the shortest number of steps (edges traversed, not total edge weight) if it exists.
/// 
/// # Errors
/// 
/// [`GraphError::InvalidEntity`]: If the provided start vertex entity does not appear in the provided query.
/// 
/// [`GraphError::NoPath`]: If a path could not be found.
/// 
/// # Example
/// 
/// ```ignore
/// //A system that uses BFS to find the shortest path between a start and end tile, avoiding those marked as "Dangerous"
/// fn bfs_based_path_finder(
///     start_tile: Query<Entity, (With<VertexType>, With<StartMarker>),
///     end_tile: Query<Entity, (With<VertexType>, With<EndMarker>),
///     tiles: Query<&VertexType, Without<DangerousMarker>>
/// ) {
///     //get the entity of the starting point
///     let start_entity = start_tile.single();
///     //get the entity of the ending point
///     let end_entity = end_tile.single();
///     //run the algorithm
///     match bfs(start_entity, end_entity, tiles){
///         Ok(path) => {
///             //reverse the path to get it to start at the start entity
///             path.reverse();
///             println!("Path to safety found using these entities: {}", path)
///         },
///         Err(_) => {println!("No Path to safety found!")} 
///     }
/// }
/// ```
/// 
/// # See also
/// 
/// [`bfs_computed_end`]: For a breadth-first search where the endpoint is not known at the start, but satisfies some condition
/// 
/// [`dijkstra_search`]: For a search algorithm that minimises the path's total edge weight
pub fn bfs<V: GraphVertex>(
    query: &Query<&V>,
    start_ent: Entity,
    end_ent: Entity
) -> Result<GraphPath<()>, GraphError> {

    if start_ent == end_ent {return Ok(GraphPath::single(start_ent, ()))};

    let mut search_queue: VecDeque<Entity> = VecDeque::from([start_ent]);
    let mut visited: VisitedNodes = VisitedNodes::new_from_start(start_ent);

    //loop while still vertices to check
    while let Some(sv_ent) = search_queue.pop_front() {
        let Ok(sv_vert) = query.get(sv_ent) else {continue;};

        for neighbour_ent in sv_vert.get_neighbours(){
            
            if visited.is_visited(&neighbour_ent) {continue;}
            visited.insert(neighbour_ent, sv_ent, 0, 0.0);

            if neighbour_ent == end_ent {return Ok(visited.determine_path(neighbour_ent).expect("The created path should be valid"));}

            search_queue.push_back(neighbour_ent);
        }
    }
    //if we get to this point, then we must have found no path
    Err(GraphError::NoPath)
}

/// Runs a breadth-first search, starting at the start vertex and ending at the first vertex for which the provided function returns true, 
/// returning the path in **reverse order**
/// 
/// Breadth-first search on the directed graph with [vertices](GraphVertex) in the provided [`Query`], using a provided function to determine when to stop. 
/// This is guaranteed to find an endpoint and path with the shortest number of steps (edges traversed, not total edge weight) if it exists.
/// 
/// # Errors
/// 
/// [`GraphError::InvalidEntity`]: If the provided start vertex entity does not appear in the provided query.
/// 
/// [`GraphError::NoPath`]: If a path could not be found.
/// 
/// # Example
/// 
/// ```ignore
/// #[derive(Component)]
/// struct DataComponent(pub usize)
/// 
/// fn end_condition(data: &DataComponent) -> bool {
///     if data.0 > 10 {true} else {false}
/// }
/// 
/// //A system that uses BFS with a computed end to find the shortest path between the start tile and a tile with a DataComponent value over 10
/// fn computed_end_bfs_based_path_finder(
///     start_tile: Query<Entity, (With<VertexType>, With<StartMarker>),
///     tiles: Query<(&VertexType, &DataComponent)>
/// ) {
///     //get the entity of the starting point
///     let start_entity = start_tile.single();
///     //run the algorithm
///     match bfs_computed_end(start_entity, end_condition, tiles){
///         Ok(path) => {
///             //reverse the path to get it to start at the start entity
///             path.reverse();
///             println!("Path to big data found using these entities: {}", path)
///         },
///         Err(_) => {println!("No Path to big data found!")} 
///     }
/// }
/// ```
/// 
/// # See also
/// 
/// [`bfs`]: For a breadth-first search with a known endpoint
/// 
/// [`dijkstra_computed_end`]: For an edge weight-minimising path algorithm with an unknown endpoint
pub fn bfs_computed_end<V, C, F> (
    query: &Query<(&V, &C)>,
    start_ent: Entity,
    end_determiner: F
) -> Result<GraphPath<()>, GraphError> 
where
    V: GraphVertex,
    C: Component,
    F: Fn(&C) -> bool,
{
    let start_vert = query.get(start_ent)?;
    if end_determiner(start_vert.1) {return Ok(GraphPath::single(start_ent, ()))};

    let mut search_queue: VecDeque<BreadthNode<V>> = VecDeque::from([BreadthNode::new(start_ent, start_vert.0, 0)]);
    let mut visited: VisitedNodes = VisitedNodes::new_from_start(start_ent);

    //loop while still vertices to check
    while let Some(node) = search_queue.pop_front() {

        for neighbour_ent in node.vertex.get_neighbours(){

            if visited.is_visited(&neighbour_ent) {continue;}
            visited.insert(neighbour_ent, node.ent, node.step + 1, 0.0);

            let Ok((neighbour_vert, neighbour_data)) = query.get(start_ent) else {continue;};

            if end_determiner(neighbour_data) {return Ok(visited.determine_path(neighbour_ent).expect("The created path sould be valid"));}
            search_queue.push_back(BreadthNode::new(neighbour_ent, neighbour_vert, node.step + 1));
        }
    }

    Err(GraphError::NoPath)
}

pub fn dfs_multiple_end<V, CE, FE> (
    query: &Query<(&V, &CE)>,
    start_ent: Entity,
    end_determiner: FE,
    max_ends: Option<usize>,
    max_steps: Option<u64>
) -> Result<Vec<GraphPath<()>>, GraphError> 
where
    V: GraphVertex,
    CE: Component,
    FE: Fn(&CE) -> bool,
{
    let max_steps = max_steps.unwrap_or(u64::MAX);
    let max_paths = max_ends.unwrap_or(usize::MAX);

    let mut found_paths = Vec::new();

    let start_vert = query.get(start_ent)?;
    if end_determiner(start_vert.1) {found_paths.push(GraphPath::single(start_ent, ()))};

    let mut search_queue: VecDeque<BreadthNode<V>> = VecDeque::from([BreadthNode::new(start_ent, start_vert.0, 0)]);
    let mut visited: VisitedNodes = VisitedNodes::new_from_start(start_ent);

    //loop while still vertices to check
    while let Some(node) = search_queue.pop_front() {

        if node.step == max_steps {continue;}

        for neighbour_ent in node.vertex.get_neighbours(){

            if found_paths.len() == max_paths {return Ok(found_paths)}

            if visited.is_visited(&neighbour_ent) {continue;}
            visited.insert(neighbour_ent, node.ent, node.step + 1, 0.0);

            let Ok((neighbour_vert, neighbour_data)) = query.get(start_ent) else {continue;};

            if end_determiner(neighbour_data) {found_paths.push(visited.determine_path(neighbour_ent).expect("The created path sould be valid"));}
            search_queue.push_back(BreadthNode::new(neighbour_ent, neighbour_vert, node.step + 1));
        }
    }

    Ok(found_paths)

}



struct BreadthNode<'a, V>{
    pub ent: Entity,
    pub vertex: &'a V,
    pub step: u64,
}

impl<'a, V:GraphVertex> BreadthNode<'a, V>{
    fn new(ent: Entity, vertex: &'a V, step: u64) -> Self {
        Self {
            ent, 
            vertex, 
            step
        }
    }
}
