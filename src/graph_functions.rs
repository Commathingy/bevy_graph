use bevy::{prelude::*, ecs::query::ReadOnlyWorldQuery, utils::{HashMap, HashSet}};
use priority_queue::PriorityQueue;
use std::{collections::VecDeque, cmp::Reverse, fs::File, io::{BufReader, BufRead}};
use crate::types::*;


//TODO:
//good docs!
//better tests
//a negative edge weight algo
//add a filter ability?






/// Takes the last vertex in the provided vector and follows it's [optional](Option) value to the previous entity in the path.
/// Terminates once it hits a [None]. Returns an [`InvalidPathError`] if there is a loop or a previous [entity](Entity) is not in the vector.
/// 
/// Returns the vector in reverse order
fn determine_path(visited: HashMap<Entity, Option<Entity>>, final_vert: Entity) -> Result<Vec<Entity>, InvalidPathError>{
    let Some(mut to_follow) = visited.get(&final_vert) else {return Err(InvalidPathError)};
    let mut path = vec![final_vert];
    let max_length = visited.len();
    while to_follow.is_some(){
        path.push(to_follow.unwrap());
        to_follow = match visited.get(&to_follow.unwrap()){
            Some(val) => val,
            None => return Err(InvalidPathError)
        };
        //check for a loop
        if path.len() > max_length {return Err(InvalidPathError)}
    }
    Ok(path)
}

#[allow(dead_code)]
/// Helper function to load a graph from a file into a world
/// 
/// Used for testing the graph algorithms, reads from a graph file and adds a [TestGraphVertex] and [GraphLabel] for each vertex in the file.
pub(crate) fn load_graph(world: &mut World, graph_file: &str) -> Vec<Entity>{
    let file = File::open(graph_file).unwrap();
    let reader = BufReader::new(file);
    let mut inter_graph_rep : Vec<Vec<(usize, f32)>> = Vec::new();
    let mut entity_vec : Vec<Entity> = Vec::new();
    for (pos, line_res) in reader.lines().enumerate(){
        entity_vec.push(world.spawn_empty().id());
        inter_graph_rep.push(Vec::new());
        let Ok(line) = line_res else {break;};
        let line_parts: Vec<&str> = line.split(":").collect();
        if line_parts[0].parse::<usize>().unwrap() != pos {break;}
        for pair in line_parts[1].split(","){
            let pair : Vec<&str> = pair.split("|").collect();
            let Ok(dest) = pair[0].parse::<usize>() else {break;};
            let Ok(weight) = pair[1].parse::<f32>() else {break;};
            inter_graph_rep[pos].push((dest, weight));
        }
        //TODO, we need to construct the graph from the inter_graph_rep
    }
    inter_graph_rep.into_iter().enumerate().for_each(|(pos,vec)| {
        //get the entitymut of the current vertex
        let mut current_ent = world.get_entity_mut(*entity_vec.get(pos).unwrap()).unwrap();
        //create the graphvert
        let edges = vec.into_iter().map(|(ind, weight)| (*entity_vec.get(ind).unwrap(), weight)).collect();
        let graph_vert = TestGraphVertex::new_with_edges(edges);
        current_ent.insert((graph_vert,GraphLabel{value: pos}));
    });
    entity_vec
}


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
pub fn bfs<V: GraphVertex, T:ReadOnlyWorldQuery>(
    start_ent: Entity,
    end_ent: Entity,
    query: &Query<&V, T>
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
pub fn bfs_computed_end<F, V, S, T> (
    start_ent: Entity,
    end_determiner: F,
    query: &Query<(&V, S), T>
) -> Result<Vec<Entity>, GraphError> 
where
    F: for<'a> Fn(S::Item<'a>) -> bool,
    V: GraphVertex,
    S: ReadOnlyWorldQuery,
    T: ReadOnlyWorldQuery
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
pub fn dfs<V: GraphVertex, T:ReadOnlyWorldQuery>(
    start_ent: Entity,
    end_ent: Entity,
    query: &Query<&V, T>
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
pub fn dfs_computed_end<F, V, S, T> (
    start_ent: Entity,
    end_determiner: F,
    query: &Query<(&V, S), T>
) -> Result<Vec<Entity>, GraphError> 
where
    F: for<'a> Fn(S::Item<'a>) -> bool,
    V: GraphVertex, 
    S: ReadOnlyWorldQuery,
    T: ReadOnlyWorldQuery
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
pub fn dijkstra_search<V: GraphVertex, T:ReadOnlyWorldQuery>(
    start_ent: Entity,
    end_ent: Entity,
    query: &Query<&V, T>,
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
pub fn dijkstra_computed_end<F, V, S, T>(
    start_ent: Entity,
    end_determiner: F,
    query: &Query<(&V, S), T>,
) -> Result<Vec<Entity>, GraphError> 
where
    F: for<'a> Fn(S::Item<'a>) -> bool,
    V: GraphVertex, 
    S: ReadOnlyWorldQuery,
    T: ReadOnlyWorldQuery
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

/// TODO
/// 
/// # Errors
/// 
/// TODO
/// 
/// # Example
/// 
/// TODO
/// 
/// # See also
/// 
/// TODO
pub fn a_star_search<F, V, S, T>(
    start_ent: Entity,
    end_ent: Entity,
    query: &Query<(&V, S), T>,
    heuristic_determiner: F
) -> Result<Vec<Entity>, GraphError> 
where
    F: for<'a> Fn(&S::Item<'a>, &S::Item<'a>) -> Heuristic,
    V: GraphVertex, 
    S: ReadOnlyWorldQuery,
    T: ReadOnlyWorldQuery
{
    query.get(start_ent)?;
    //get the data of the end vertex to be used for the heuristic calcs
    //the start vertex data will noyl be used before starting the loop
    let (_, end_data)= query.get(end_ent)?;

    //hashmap that stores the previous vertex of the path for a given vertex
    let mut path_previous: HashMap<Entity, Option<Entity>> = HashMap::new();
    path_previous.insert(start_ent, None);

    //The list of visited entities. stores a cardinality for current minimum found distance to the vertex
    //and a cardinality to represent the value of the heuristic
    let mut minimal_dist : HashMap<Entity, (PathWeight, Heuristic)> = HashMap::new();
    minimal_dist.insert(start_ent, (PathWeight{weight: 0.0}, Heuristic{value: 0.0}));

    //create the search queue, here the pathweight is the sum of the heuristic and actual minimum found distance
    let mut search_queue: PriorityQueue<Entity , Reverse<PathWeight>> = PriorityQueue::new();
    search_queue.push(start_ent, Reverse(PathWeight{weight: 0.0}));

    while let Some((sv_ent, _)) = search_queue.pop() {
        //check if we are currently searching the end vertex, as this implies we have already found the minimum path
        if sv_ent == end_ent {
            return Ok(determine_path(path_previous, sv_ent).unwrap());
        }

        //get the GraphVertex info of the search vertex
        let Ok((sv_vert, sv_data)) = query.get(sv_ent) else {continue;};

        //get the distance of the search vertex
        let sv_dist = minimal_dist.get(&sv_ent).unwrap().0;

        //loop over this vertex's neighbours
        for (neighbour_ent, edge_weight) in sv_vert.get_neighbours_with_weight(){

            //Determine the distance to this neighbour via the path to the search vertex
            let total_dist = sv_dist + edge_weight;
            
            //check if we have visited this vertex before
            //if so, compare the cardinalities to see if we should update
            if let Some((neighbour_dist, neighbour_heuristic)) = minimal_dist.get_mut(&neighbour_ent) {
                //check if the vertex was visited already at a closer distance
                //if so we ignore this vertex
                if total_dist > *neighbour_dist {continue;}
                //otherwise update the vertex's distance, previous vertex and priority in the queue
                //we do not need to recalculate the heuristic in this case
                *path_previous.get_mut(&neighbour_ent).unwrap() = Some(sv_ent);
                search_queue.change_priority(&neighbour_ent, Reverse(total_dist + *neighbour_heuristic));
                *neighbour_dist = total_dist;
            } else {
                //determine the heurstic of this new value
                let heuristic = heuristic_determiner(&sv_data, &end_data);
                //otherwise the vertex hasnt been visited before and so we add it to the queue, path_previous and the visited entities
                path_previous.insert(neighbour_ent, Some(sv_ent));
                search_queue.push(neighbour_ent, Reverse(total_dist + heuristic));
                minimal_dist.insert(neighbour_ent, (total_dist, heuristic));
                
            }
        }
    }

    //if we get to this point, then we must have found no path
    Err(GraphError::NoPath)
}

/// TODO
/// 
/// # Errors
/// 
/// [`GraphError::InvalidEntity`]: If the provided start vertex entity does not appear in the provided query.
/// 
/// # Example
/// 
/// TODO
/// 
/// # See also
/// 
/// [`at_step`]: For vertices that are only at the given step.
/// 
/// [`within_distance`]: For vertices that are within a given distance, by edge weight.
pub fn within_steps<V:GraphVertex, T:ReadOnlyWorldQuery>(
    start_ent: Entity,
    max_steps: usize,
    query: &Query<&V, T>
) -> Result<Vec<(Entity, usize)>, GraphError> {

    //vector of vertices that we want to check, alongside their distance from the start vertex
    let mut to_view: VecDeque<&V> = VecDeque::from([query.get(start_ent)?]);

    //hashset storing entities weve checked already
    let mut seen: HashSet<Entity> = HashSet::new();
    seen.insert(start_ent);

    //final output list
    let mut valid: Vec<(Entity, usize)> = vec![(start_ent, 0)];

    //the current step we are on
    let mut current_step = 0;
    //the number of vertices left to check at this distance
    let mut at_current_step = 1;
    
    while let Some(current_vert) = to_view.pop_front(){
        
        //decrement number left to check at this distance
        at_current_step -= 1;

        for neighbour in current_vert.get_neighbours(){
            //check if we have checked this entity before, skipping this iteration if so
            if !seen.insert(neighbour){continue;}
            //otherwise add it to the valid list and to_view queue
            if let Ok(vert) = query.get(start_ent){
                to_view.push_back(vert);
                valid.push((neighbour, current_step+1));
            }
        }

        //if we've viewed all at the current step, increment current_step and calculate how many at this step
        if at_current_step == 0 {
            current_step += 1;
            //check if we've checked far enough
            if current_step == max_steps {break;}
            at_current_step = to_view.len(); //if this is 0, we shouldnt run another loop iteration so should be ok
        }
    }
    Ok(valid)
}

/// TODO
/// 
/// [`GraphError::InvalidEntity`]: If the provided start vertex entity does not appear in the provided query.
/// 
/// [`GraphError::NegativeWeight`]: If a vertex returns a negative edge weight.
/// 
/// TODO
/// 
/// # Example
/// 
/// TODO
/// 
/// # See also
/// 
/// [`within_steps`]: For vertices that are within a given number of steps, rather than by distance.
pub fn within_distance<V:GraphVertex, T:ReadOnlyWorldQuery>(
    start_ent: Entity,
    max_distance: f32,
    query: &Query<&V, T>
) -> Result<Vec<(Entity, f32)>, GraphError> {

    //test for a valid start
    query.get(start_ent)?;

    //The list of visited entities. stores the cardinality (current minimum found distance to the vertex)
    let mut minimal_dist : HashMap<Entity, PathWeight> = HashMap::new();
    minimal_dist.insert(start_ent, PathWeight{weight: 0.0});

    //create the search queue
    let mut search_queue: PriorityQueue<Entity , Reverse<PathWeight>> = PriorityQueue::new();
    search_queue.push(start_ent, Reverse(PathWeight{weight: 0.0}));

    while let Some((sv_ent, Reverse(sv_dist))) = search_queue.pop() {

        //get the GraphVertex info of the search vertex
        let Ok(sv_vert) = query.get(sv_ent) else {continue;};

        //loop over this vertex's neighbours
        for (neighbour_ent, edge_weight) in sv_vert.get_neighbours_with_weight(){

            if edge_weight < 0.0 {return Err(GraphError::NegativeWeight);}

            //Determine the distance to this neighbour via the path to the search vertex
            let total_dist = sv_dist + edge_weight;
            if total_dist > (PathWeight{weight: max_distance}) {continue;}

            //check if we have visited this vertex before
            //if so, compare the cardinalities to see if we should update
            if let Some(dist) = minimal_dist.get_mut(&neighbour_ent) {
                //check if the vertex was visited already at a closer distance
                //if so we ignore this vertex
                if total_dist > *dist {continue;}
                //otherwise update the vertex's distance, previous vertex and priority in the queue
                *dist = total_dist;
                search_queue.change_priority(&neighbour_ent, Reverse(total_dist));
            } else {
                //otherwise the vertex hasnt been visited before and so we add it to the queue and the visited entities
                minimal_dist.insert(neighbour_ent, total_dist);
                search_queue.push(neighbour_ent, Reverse(total_dist));
            }
        }
    }

    Ok(minimal_dist.into_iter().map(|(ent, pathweight)| (ent, pathweight.weight)).collect())
}


/// Returns all vertices that are exactly the given number of steps away.
/// 
/// The returned set of vertices are those that are connected to the given start vertex, with the minimal number of steps in a path connecting them equal
/// to the given step. The returned vector may be empty if there are no such vertices. If vertices that are closer are also desired, use [`within_steps`] instead.
/// 
/// # Errors
/// 
/// [`GraphError::InvalidEntity`]: If the provided start vertex entity does not appear in the provided query.
/// 
/// # Example
/// 
/// TODO
/// 
/// # See also
/// 
/// [`within_steps`]: For a function to return vertices that are at most a certain number of steps away
pub fn at_step<V:GraphVertex, T:ReadOnlyWorldQuery>(
    start_ent: Entity,
    at_step: usize,
    query: &Query<&V, T>
) -> Result<Vec<Entity>, GraphError> {
    Ok(within_steps(start_ent, at_step, query)?.into_iter()
    .filter_map(|(ent, step)| if step == at_step {Some(ent)} else {None})
    .collect())
}