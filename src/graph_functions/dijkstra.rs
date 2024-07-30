use std::cmp::Reverse;

use bevy::{prelude::{Component, Entity, Query}, utils::HashMap};
use priority_queue::PriorityQueue;

use crate::graph_vertex::GraphVertex;

use super::{helper::determine_path, GraphError, PathWeight};


/// Runs Dijkstra's algorithm to find the path minimising total edge weight between two vertices, returning the path in **reverse order**
/// 
/// Dijkstra's algorithm on the directed graph with vertices in the provided query, this will result in a path that minimises distance,
/// not one that minimises distance. This algorithm does not work for graphs with negative weight edges.
/// 
/// # Errors
/// 
/// [`GraphError::InvalidEntity`]: If the provided start or end vertex entity does not appear in the provided query.
/// 
/// [`GraphError::NoPath`]: If a path could not be found.
/// 
/// [`GraphError::NegativeWeight`]: If a vertex provides an edge with a negative weight
/// 
/// # Example
/// 
///```ignore
/// //A system that uses Dijkstra's algorithm to find the fastest route home
/// fn shortest_route_home(
///     start_tile: Query<Entity, (With<VertexType>, With<StartMarker>),
///     end_tile: Query<Entity, (With<VertexType>, With<Home>),
///     tiles: Query<&VertexType, Without<DangerousMarker>>
/// ) {
///     //get the entity of the starting point
///     let start_entity = start_tile.single();
///     //get the entity of the ending point
///     let end_entity = end_tile.single();
///     //run the algorithm
///     match dijkstra_search(start_entity, end_entity, tiles){
///         Ok(path) => {
///             //reverse the path to get it to start at the start entity
///             path.reverse();
///             println!("Shortest route home via these entities: {}", path)
///         },
///         Err(_) => {println!("No Path to safety found!")} 
///     }
/// }
/// ```
/// 
/// # See also
/// 
/// [`bfs`]: For a breadth-first search with a known endpoint
/// 
/// [`a_star_search`]: For an A* approach to finding the shortest path
pub fn dijkstra_search<V: GraphVertex>(
    query: &Query<&V>,
    start_ent: Entity,
    end_ent: Entity
) -> Result<Vec<Entity>, GraphError> {
    //test for invalid start or end
    query.get(start_ent)?;
    query.get(end_ent)?;

    //hashmap that stores the previous vertex of the path for a given vertex
    let mut path_previous: HashMap<Entity, Option<Entity>> = HashMap::new();
    path_previous.insert(start_ent, None);

    //The list of visited entities. stores the cardinality (current minimum found distance to the vertex)
    let mut minimal_dist : HashMap<Entity, PathWeight> = HashMap::new();
    minimal_dist.insert(start_ent, PathWeight{weight: 0.0});

    //create the search queue
    let mut search_queue: PriorityQueue<Entity , Reverse<PathWeight>> = PriorityQueue::new();
    search_queue.push(start_ent, Reverse(PathWeight{weight: 0.0}));

    while let Some((sv_ent, Reverse(sv_dist))) = search_queue.pop() {
        //check if we are currently searching the end vertex, as this implies we have already found the minimum path
        if sv_ent == end_ent {
            return Ok(determine_path(path_previous, sv_ent).unwrap());
        }

        //get the GraphVertex info of the search vertex
        let Ok(sv_vert) = query.get(sv_ent) else {continue;};

        //loop over this vertex's neighbours
        for (neighbour_ent, edge_weight) in sv_vert.get_neighbours_with_weight(){

            if edge_weight < 0.0 {return Err(GraphError::NegativeWeight);}

            //Determine the distance to this neighbour via the path to the searvh vertex
            let total_dist = sv_dist + edge_weight;

            //check if we have visited this vertex before
            //if so, compare the cardinalities to see if we should update
            if let Some(dist) = minimal_dist.get_mut(&neighbour_ent) {
                //check if the vertex was visited already at a closer distance
                //if so we ignore this vertex
                if total_dist > *dist {continue;}
                //otherwise update the vertex's distance, previous vertex and priority in the queue
                *path_previous.get_mut(&neighbour_ent).unwrap() = Some(sv_ent);
                *dist = total_dist;
                search_queue.change_priority(&neighbour_ent, Reverse(total_dist));
            } else {
                //otherwise the vertex hasnt been visited before and so we add it to the queue, path_previous and the visited entities
                path_previous.insert(neighbour_ent, Some(sv_ent));
                minimal_dist.insert(neighbour_ent, total_dist);
                search_queue.push(neighbour_ent, Reverse(total_dist));
            }
        }
    }

    //if we get to this point, then we must have found no path
    Err(GraphError::NoPath)
}

/// Run's Dijkstra's algorithm until an endpoint, which satisfies the end determiner returns true, is found at a minimal distance from the start vertex, 
/// returning the path in **reverse order**
/// 
/// Dijkstra's algorithm on the directed graph with vertices in the provided query, this will result in a path to the closest valid endpoint to the start vertex
/// with the minimal distance achieved by said path. This algorithm does not work for graphs with negative weight edges.
/// 
/// # Errors
/// 
/// [`GraphError::InvalidEntity`]: If the provided start or end vertex entity does not appear in the provided query.
/// 
/// [`GraphError::NoPath`]: If a path could not be found.
/// 
/// [`GraphError::NegativeWeight`]: If a vertex provides an edge with a negative weight
/// 
/// # Example
/// 
///```ignore
/// //A system that uses Dijkstra's algorithm to find the fastest route to the nearest grocery store from home
/// fn closest_grocery_store(
///     start_tile: Query<Entity, (With<VertexType>, With<Home>),
///     tiles: Query<&VertexType, Without<DangerousMarker>>
/// ) {
///     //get the entity of the starting point
///     let start_entity = start_tile.single();
///     //run the algorithm
///     match dijkstra_computed_end(
///         start_entity, 
///         |building: &Building|{building.is_store()}, 
///         tiles
///     ){
///         Ok(path) => {
///             //reverse the path to get it to start at the start entity
///             path.reverse();
///             println!("Shortest route to a store via these entities: {}", path)
///         },
///         Err(_) => {println!("No Path to safety found!")} 
///     }
/// }
/// ```
/// 
/// # See also
/// 
/// [`dijkstra_search`]: For Dijkstra's algorithm with a known endpoint
/// 
/// [`bfs_computed_end`]: For a breadth-first search with an unknown endpoint
pub fn dijkstra_computed_end<V, C, F>(
    query: &Query<(&V, &C)>,
    start_ent: Entity,
    end_determiner: F,
) -> Result<Vec<Entity>, GraphError> 
where
    V: GraphVertex, 
    C: Component,
    F: Fn(&C) -> bool,
{
    query.get(start_ent)?;

    //hashmap that stores the previous vertex of the path for a given vertex
    let mut path_previous: HashMap<Entity, Option<Entity>> = HashMap::new();
    path_previous.insert(start_ent, None);

    //The list of visited entities. stores the cardinality (current minimum found distance to the vertex)
    let mut minimal_dist : HashMap<Entity, PathWeight> = HashMap::new();
    minimal_dist.insert(start_ent, PathWeight{weight: 0.0});

    //create the search queue
    let mut search_queue: PriorityQueue<Entity , Reverse<PathWeight>> = PriorityQueue::new();
    search_queue.push(start_ent, Reverse(PathWeight{weight: 0.0}));

    while let Some((sv_ent, Reverse(sv_dist))) = search_queue.pop() {
        //get the info of the search vertex
        let Ok((sv_vert, sv_data)) = query.get(sv_ent) else {continue;};

        //check if we are currently searching a valid end vertex, as this implies we have already found a minimum path
        if end_determiner(sv_data) {return Ok(determine_path(path_previous, sv_ent).unwrap());}

        //loop over this vertex's neighbours
        for (neighbour_ent, edge_weight) in sv_vert.get_neighbours_with_weight(){
            if edge_weight < 0.0 {return Err(GraphError::NegativeWeight);}

            //Determine the distance to this neighbour via the path to the searvh vertex
            let total_dist = sv_dist + edge_weight;

            //check if we have visited this vertex before
            //if so, compare the cardinalities to see if we should update
            if let Some(dist) = minimal_dist.get_mut(&neighbour_ent) {
                //check if the vertex was visited already at a closer distance
                //if so we ignore this vertex
                if total_dist > *dist {continue;}
                //otherwise update the vertex's distance, previous vertex and priority in the queue
                *path_previous.get_mut(&neighbour_ent).unwrap() = Some(sv_ent);
                *dist = total_dist;
                search_queue.change_priority(&neighbour_ent, Reverse(total_dist));
            } else {
                //otherwise the vertex hasnt been visited before and so we add it to the queue, path_previous and the visited entities
                path_previous.insert(neighbour_ent, Some(sv_ent));
                minimal_dist.insert(neighbour_ent, total_dist);
                search_queue.push(neighbour_ent, Reverse(total_dist));
            }
        }
    }

    //if we get to this point, then we must have found no path
    Err(GraphError::NoPath)
}
