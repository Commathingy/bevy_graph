use bevy::{prelude::{Component, Entity, Query}, utils::HashMap};

use crate::graph_vertex::GraphVertex;

use super::{helper::determine_path, GraphError};


/// Runs a depth-first search, starting at the start vertex and ending at the end vertex, returning the path in **reverse order**
/// 
/// Depth-first search on the directed graph with [vertices](GraphVertex) in the provided query. The resulting path will not necessarily be the shortest,
/// if a shortest path is required, consider using [breadth-first search](bfs) or [Dijkstra's algorithm](dijkstra_search).
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
/// //A system that uses DFS to check if a start and end vertex are connected
/// fn dfs_based_connected_check(
///     start_tile: Query<Entity, (With<VertexType>, With<StartMarker>),
///     end_tile: Query<Entity, (With<VertexType>, With<EndMarker>),
///     tiles: Query<&VertexType>
/// ) {
///     //get the entity of the starting point
///     let start_entity = start_tile.single();
///     //get the entity of the ending point
///     let end_entity = end_tile.single();
///     //run the algorithm
///     match dfs(start_entity, end_entity, tiles){
///         Ok(path) => {println!("The start and end are connected!")},
///         Err(_) => {println!("No connection between the vertices")} 
///     }
/// }
/// ```
/// 
/// # See also
/// 
/// bfs
/// dfs computed end
/// 
/// TODO
pub fn dfs<V: GraphVertex>(
    query: &Query<&V>,
    start_ent: Entity,
    end_ent: Entity
) -> Result<Vec<Entity>, GraphError> {
    //test for invalid start or end
    query.get(start_ent)?;
    query.get(end_ent)?;

    if start_ent == end_ent {return Ok(vec![start_ent])};

    //create the search queue. the next element to be checked is the one at the end of the list
    //we will add to the end of the list to obtain the depth first behaviour
    //the usize represents how many neighbours of this vertex we have visited
    let mut search_queue: Vec<(Entity, &V, usize)> = vec![(start_ent, query.get(start_ent)?, 0)];

    //store the visited vertices, alongside the vertex that came before it
    let mut path_previous: HashMap<Entity, Option<Entity>> = HashMap::new();
    path_previous.insert(start_ent, None);

    //loop while still vertices to check
    while let Some((sv_ent, sv_vert, mut sv_neighbour_index)) = search_queue.pop() {
        //check if we have any neighbours left to search from this vertex
        if let Some(&neighbour_ent) = sv_vert.get_neighbours().get(sv_neighbour_index){
            //if there is a neighbour, we add one to this vertices "neighbour count (the .2)"
            //and then add it back onto the queue
            sv_neighbour_index += 1;
            search_queue.push((sv_ent, sv_vert, sv_neighbour_index));

            //check if we found the end vertex
            //if so, end the search and unravel the path
            if neighbour_ent == end_ent {
                path_previous.insert(neighbour_ent, Some(sv_ent));
                return Ok(determine_path(path_previous, neighbour_ent).unwrap());
            }

            //check if this neighbour vertex has been visited before
            //if it has, we ignore it
            if path_previous.contains_key(&neighbour_ent) {continue;}
            //otherwise we add it to our search queue, as the next vertex to be checked
            else if let Ok(neighbour_vert) = query.get(neighbour_ent){
                path_previous.insert(neighbour_ent, Some(sv_ent));
                search_queue.push((neighbour_ent, neighbour_vert, 0));
            }
        }
    }

    //if we get to this point, then we must have found no path
    Err(GraphError::NoPath)
}

/// Runs a depth-first search, starting at the start vertex and ending at the first vertex for which the provided function returns true, 
/// returning the path in **reverse order**
/// 
/// Depth-first search on the directed graph with [vertices](GraphVertex) in the provided query, using a provided function to determine when to stop. 
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
/// struct DataComponent(pub bool)
/// 
/// fn end_condition(data: &DataComponent) -> bool {
///     data.0
/// }
/// 
/// //A system that uses DFS with a computed end to check if the start vertex is connected to any DataComponents that contain a true
/// fn dfs_connectedness_tester(
///     start_tile: Query<Entity, (With<VertexType>, With<StartMarker>),
///     tiles: Query<(&VertexType, &DataComponent)>
/// ) {
///     //get the entity of the starting point
///     let start_entity = start_tile.single();
///     //run the algorithm
///     match dfs_computed_end(start_entity, end_condition, tiles){
///         Ok(_) => {println!("Connected to a suitable vertex!")},
///         Err(_) => {println!("No connected to a suitable vertex :(")} 
///     }
/// }
/// 
/// ```
/// 
/// # See also
/// 
/// [`dfs`]: For a depth-first search with a known endpoint
/// 
/// [`bfs_computed_end`]: For an algorithm that finds a shortest path with an unknown endpoint
pub fn dfs_computed_end<V, C, F> (
    query: &Query<(&V, &C)>,
    start_ent: Entity,
    end_determiner: F
) -> Result<Vec<Entity>, GraphError> 
where
    V: GraphVertex,
    C: Component,
    F: Fn(&C) -> bool,
{
    let start_vert = query.get(start_ent)?;
    if end_determiner(start_vert.1) {return Ok(vec![start_ent])};

    //create the search queue. the next element to be checked is the one at the end of the list
    //we will add to the end of the list to obtain the depth first behaviour
    //the usize represents how many neighbours of this vertex we have visited
    let mut search_queue: Vec<(Entity, &V, usize)> = vec![(start_ent, start_vert.0 , 0)];

    //store the visited vertices, alongside the vertex that came before it
    let mut path_previous: HashMap<Entity, Option<Entity>> = HashMap::new();
    path_previous.insert(start_ent, None);

    //loop while still vertices to check
    while let Some((sv_ent, sv_vert, mut sv_neighbour_index)) = search_queue.pop() {

        //check if we have any neighbours left to search from this vertex
        if let Some(&neighbour_ent) = sv_vert.get_neighbours().get(sv_neighbour_index){
            //if there is a neighbour, we add one to this vertices "neighbour count (the .2)"
            //and then add it back onto the queue
            sv_neighbour_index += 1;
            search_queue.push((sv_ent, sv_vert, sv_neighbour_index));

            //check if this neighbour vertex has been visited before
            //if it has, we ignore it
            if path_previous.contains_key(&neighbour_ent) {continue;}

            //otherwise we add it to our search queue, as the next vertex to be checked
            else if let Ok((neighbour_vert, neighbour_data)) = query.get(neighbour_ent){
                //add this vertex to the path_previous map
                path_previous.insert(neighbour_ent, Some(sv_ent));
                //check if we found the end vertex
                if end_determiner(neighbour_data){return Ok(determine_path(path_previous, neighbour_ent).unwrap());}
                //add to the search queue
                search_queue.push((neighbour_ent, neighbour_vert, 0));
            }
        }
    }

    //if we get to this point, then we must have found no path
    Err(GraphError::NoPath)
}

