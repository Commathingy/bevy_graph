use std::usize;

use bevy::prelude::{Component, Entity, Query};

use crate::graph_vertex::GraphVertex;

use super::{GraphError, GraphPath, VisitedNodes};


/// Runs a depth-first search, starting at the start vertex and ending at the end vertex, returning the path in **reverse order**
/// 
/// Depth-first search on the directed graph with [vertices](GraphVertex) in the provided query. The resulting path will not necessarily be the shortest, 
/// however, this function may take less time than breadth first search to find a path depending on the graph's structure.
/// If a shortest path is required, consider using [breadth-first search](bfs) or [Dijkstra's algorithm](dijkstra_search).
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
) -> Result<GraphPath<()>, GraphError> {
    let start_vert = query.get(start_ent)?;

    if start_ent == end_ent {return Ok(GraphPath::single(start_ent, ()))}; //check for instant finish

    let mut search_queue: Vec<DepthNode<V>> = vec![DepthNode::new(start_ent, start_vert)];
    let mut visited = VisitedNodes::new_from_start(start_ent);

    while let Some(mut node) = search_queue.pop() {

        //check if we have any neighbours left to search from this vertex
        let Some(neighbour_ent) = node.get_next_neighbour() else {continue;};

        let previous = node.ent;
        search_queue.push(node); //push back onto queue to be checked again later

        if visited.is_visited(&neighbour_ent) {continue;}
        visited.insert(neighbour_ent, previous, 0, 0.0);

        if neighbour_ent == end_ent {return Ok(visited.determine_path(neighbour_ent).expect("The created path should be valid"))}

        let Ok(neighbour_vert) = query.get(neighbour_ent) else {continue;};
        search_queue.push(DepthNode::new(neighbour_ent, neighbour_vert));
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
pub fn dfs_computed_end<V, CE, FE> (
    query: &Query<(&V, &CE)>,
    start_ent: Entity,
    end_determiner: FE
) -> Result<GraphPath<()>, GraphError> 
where
    V: GraphVertex,
    CE: Component,
    FE: Fn(&CE) -> bool,
{
    let start_vert = query.get(start_ent)?;
    if end_determiner(start_vert.1) {return Ok(GraphPath::single(start_ent, ()))}; //check for instant finish

    let mut search_queue: Vec<DepthNode<V>> = vec![DepthNode::new(start_ent, start_vert.0)];
    let mut visited = VisitedNodes::new_from_start(start_ent);

    while let Some(mut node) = search_queue.pop() {

        //check if we have any neighbours left to search from this vertex
        let Some(neighbour_ent) = node.get_next_neighbour() else {continue;};

        let previous = node.ent;
        search_queue.push(node); //push back onto queue to be checked again later

        if visited.is_visited(&neighbour_ent) {continue;}

        let Ok((neighbour_vert, neighbour_data)) = query.get(neighbour_ent) else {continue;};
        visited.insert(neighbour_ent, previous, 0, 0.0);   
        if end_determiner(neighbour_data){return Ok(visited.determine_path(neighbour_ent).expect("The created path should be valid"));}
        search_queue.push(DepthNode::new(neighbour_ent, neighbour_vert));
    }

    //if we get to this point, then we must have found no path
    Err(GraphError::NoPath)
}


pub fn dfs_multiple_end<V, CE, FE> (
    query: &Query<(&V, &CE)>,
    start_ent: Entity,
    end_determiner: FE,
    max_ends: Option<usize>
) -> Result<Vec<GraphPath<()>>, GraphError> 
where
    V: GraphVertex,
    CE: Component,
    FE: Fn(&CE) -> bool,
{
    let max_paths = max_ends.unwrap_or(usize::MAX);
    let start_vert = query.get(start_ent)?;

    let mut found_paths = Vec::new();
    if end_determiner(start_vert.1) {found_paths.push(GraphPath::single(start_ent, ()))};

    let mut search_queue: Vec<DepthNode<V>> = vec![DepthNode::new(start_ent, start_vert.0)];
    let mut visited = VisitedNodes::new_from_start(start_ent);

    while let Some(mut node) = search_queue.pop() {

        if found_paths.len() == max_paths {break;} 

        //check if we have any neighbours left to search from this vertex
        let Some(neighbour_ent) = node.get_next_neighbour() else {continue;};

        let previous = node.ent;
        search_queue.push(node); //push back onto queue to be checked again later

        if visited.is_visited(&neighbour_ent) {continue;}

        let Ok((neighbour_vert, neighbour_data)) = query.get(neighbour_ent) else {continue;};
        visited.insert(neighbour_ent, previous, 0, 0.0);   
        if end_determiner(neighbour_data){found_paths.push(visited.determine_path(neighbour_ent).unwrap());}
        search_queue.push(DepthNode::new(neighbour_ent, neighbour_vert));
    }

    //if we get to this point, then we must have found no path
    Ok(found_paths)

}






struct DepthNode<'a, V>{
    pub ent: Entity,
    pub vertex: &'a V,
    pub neighbours_visited: usize,
}

impl<'a, V:GraphVertex> DepthNode<'a, V>{
    fn new(ent: Entity, vertex: &'a V) -> Self {
        Self {
            ent, 
            vertex, 
            neighbours_visited: 0
        }
    }

    fn get_next_neighbour(&mut self) -> Option<Entity>{
        let to_visit =  self.neighbours_visited;
        self.neighbours_visited += 1;
        self.vertex.get_neighbours().get(to_visit).map(|val| *val)
    }
}
