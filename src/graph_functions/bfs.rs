use std::collections::VecDeque;

use bevy::{prelude::{Component, Entity, Query}, utils::HashMap};

use crate::graph_vertex::GraphVertex;

use super::{helper::determine_path, GraphError};







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
) -> Result<Vec<Entity>, GraphError> {

    //test for invalid start or end
    query.get(start_ent)?;
    query.get(end_ent)?;

    if start_ent == end_ent {return Ok(vec![start_ent])};

    //create the search queue. the next element to be checked is the one at the start of the list
    //we will add to the end of the list to obtain the breadth first behaviour
    let mut search_queue: VecDeque<Entity> = VecDeque::from([start_ent]);

    //store the visited vertices, alongside the vertex that came before it
    let mut path_previous: HashMap<Entity, Option<Entity>> = HashMap::new();
    path_previous.insert(start_ent, None);

    //loop while still vertices to check
    while let Some(sv_ent) = search_queue.pop_front() {
        let Ok(sv_vert) = query.get(sv_ent) else {continue;};
        //loop over this vertex's neighbours
        for neighbour_ent in sv_vert.get_neighbours(){
            //check if the neighbour is the final vertex, in which case we compute the path and return it
            if neighbour_ent == end_ent {
                path_previous.insert(neighbour_ent, Some(sv_ent));
                return Ok(determine_path(path_previous, neighbour_ent)?);
            }

            //otherwise check if this vertex is already searched, adding it to the search if not
            if !path_previous.contains_key(&neighbour_ent) {
                path_previous.insert(neighbour_ent, Some(sv_ent));
                search_queue.push_back(neighbour_ent);
            }
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
) -> Result<Vec<Entity>, GraphError> 
where
    V: GraphVertex,
    C: Component,
    F: Fn(&C) -> bool,
{
    //check if the start vertex is a valid end vertex
    let start_vert = query.get(start_ent)?;
    if end_determiner(start_vert.1) {return Ok(vec![start_ent])};
    //create the search queue. the next element to be checked is the one at the start of the list
    //we will add to the end of the list to obtain the breadth first behaviour
    let mut search_queue: VecDeque<(Entity, &V)> = VecDeque::from([(start_ent, start_vert.0)]);

    //store the visited vertices, alongside the vertex that came before it
    let mut path_previous: HashMap<Entity, Option<Entity>> = HashMap::new();
    path_previous.insert(start_ent, None);

    //loop while still vertices to check
    while let Some((sv_ent, sv_vert)) = search_queue.pop_front() {
        //loop over this vertex's neighbours
        for neighbour_ent in sv_vert.get_neighbours(){

            //check if this vertex is already searched, if so we ignore it
            if path_previous.contains_key(&neighbour_ent) {continue;}

            else if let Ok((neighbour_vert, neighbour_data)) = query.get(start_ent){
                path_previous.insert(neighbour_ent, Some(sv_ent));
                //check if this neighbour is a valid end vertex
                if end_determiner(neighbour_data) {return Ok(determine_path(path_previous, neighbour_ent).unwrap());}
                search_queue.push_back((neighbour_ent, neighbour_vert));
            }
        }
    }

    //if we get to this point, then we must have found no path
    Err(GraphError::NoPath)
}
