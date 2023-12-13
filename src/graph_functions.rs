use bevy::{prelude::*, ecs::query::ReadOnlyWorldQuery};
use priority_queue::PriorityQueue;
use std::{collections::{HashMap, VecDeque}, cmp::Reverse, fs::File, io::{BufReader, BufRead}};
use crate::types::*;


/// Takes the last vertex in the provided vector and follows it's option to the previous entity in the path.
/// Terminates once it hits a None. Returns an InvalidPathError if there is a loop or an entity not in the vector.
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


pub(crate) fn load_graph(world: &mut World, graph_file: &str) -> Vec<Entity>{
    let file = File::open(graph_file).unwrap();
    let reader = BufReader::new(file);
    let mut inter_graph_rep : Vec<Vec<(usize, Option<f32>)>> = Vec::new();
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
            let weight = pair[1].parse::<f32>().ok();
            inter_graph_rep[pos].push((dest, weight));
        }
        //TODO, we need to construct the graph from the inter_graph_rep
    }
    inter_graph_rep.into_iter().enumerate().for_each(|(pos,vec)| {
        //get the entitymut of the current vertex
        let mut current_ent = world.get_entity_mut(*entity_vec.get(pos).unwrap()).unwrap();
        //create the graphvert
        let edges = vec.into_iter().map(|(ind, weight)| (*entity_vec.get(ind).unwrap(), weight)).collect();
        let graph_vert = GraphVertex::new_with_edges(edges);
        current_ent.insert((graph_vert,GraphLabel{value: pos}));
    });
    entity_vec
}


/// Note:returns in reverse order
pub fn bfs<T:ReadOnlyWorldQuery>(
    start_ent: Entity,
    end_ent: Entity,
    query: &Query<&GraphVertex, T>
) -> Result<Vec<Entity>, NoPathError> {
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
                return Ok(determine_path(path_previous, neighbour_ent).unwrap());
            }

            //otherwise check if this vertex is already searched, adding it to the search if not
            if !path_previous.contains_key(&neighbour_ent) {
                path_previous.insert(neighbour_ent, Some(sv_ent));
                search_queue.push_back(neighbour_ent);
            }
        }
    }
    //if we get to this point, then we must have found no path
    Err(NoPathError)
}

/// Note:returns in reverse order
pub fn bfs_computed_end<F, S, T> (
    start_ent: Entity,
    end_determiner: F,
    query: &Query<(&GraphVertex, S), T>
) -> Result<Vec<Entity>, NoPathError> 
where
    F: for<'a> Fn(S::Item<'a>) -> bool,
    S: ReadOnlyWorldQuery,
    T: ReadOnlyWorldQuery
{
    //check if the start vertex is a valid end vertex
    let start_vert = query.get(start_ent)?;
    if end_determiner(start_vert.1) {return Ok(vec![start_ent])};
    //create the search queue. the next element to be checked is the one at the start of the list
    //we will add to the end of the list to obtain the breadth first behaviour
    let mut search_queue: VecDeque<(Entity, &GraphVertex)> = VecDeque::from([(start_ent, start_vert.0)]);

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
    Err(NoPathError)
}

/// Note:returns in reverse order
pub fn dfs<T:ReadOnlyWorldQuery>(
    start_ent: Entity,
    end_ent: Entity,
    query: &Query<&GraphVertex, T>
) -> Result<Vec<Entity>, NoPathError> {
    if start_ent == end_ent {return Ok(vec![start_ent])};

    //create the search queue. the next element to be checked is the one at the end of the list
    //we will add to the end of the list to obtain the depth first behaviour
    //the usize represents how many neighbours of this vertex we have visited
    let mut search_queue: Vec<(Entity, &GraphVertex, usize)> = vec![(start_ent, query.get(start_ent)?, 0)];

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
    Err(NoPathError)
}

/// Note:returns in reverse order
pub fn dfs_computed_end<F, S, T> (
    start_ent: Entity,
    end_determiner: F,
    query: &Query<(&GraphVertex, S), T>
) -> Result<Vec<Entity>, NoPathError> 
where
    F: for<'a> Fn(S::Item<'a>) -> bool,
    S: ReadOnlyWorldQuery,
    T: ReadOnlyWorldQuery
{
    let start_vert = query.get(start_ent)?;
    if end_determiner(start_vert.1) {return Ok(vec![start_ent])};

    //create the search queue. the next element to be checked is the one at the end of the list
    //we will add to the end of the list to obtain the depth first behaviour
    //the usize represents how many neighbours of this vertex we have visited
    let mut search_queue: Vec<(Entity, &GraphVertex, usize)> = vec![(start_ent, start_vert.0 , 0)];

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
    Err(NoPathError)
}

/// Note:returns in reverse order
pub fn dijkstra_search<T:ReadOnlyWorldQuery>(
    start_ent: Entity,
    end_ent: Entity,
    query: &Query<&GraphVertex, T>,
    none_response: NoneValueResponse
) -> Result<Vec<Entity>, NoPathError> {
    //hashmap that stores the previous vertex of the path for a given vertex
    let mut path_previous: HashMap<Entity, Option<Entity>> = HashMap::new();
    path_previous.insert(start_ent, None);

    //The list of visited entities. stores the cardinality (current minimum found distance to the vertex)
    let mut minimal_dist : HashMap<Entity, PathWeight> = HashMap::new();
    minimal_dist.insert(start_ent, PathWeight::new(0.0));

    //create the search queue
    let mut search_queue: PriorityQueue<Entity , Reverse<PathWeight>> = PriorityQueue::new();
    search_queue.push(start_ent, Reverse(PathWeight::new(0.0)));

    while let Some((sv_ent, Reverse(sv_dist))) = search_queue.pop() {
        //check if we are currently searching the end vertex, as this implies we have already found the minimum path
        if sv_ent == end_ent {
            return Ok(determine_path(path_previous, sv_ent).unwrap());
        }

        //get the GraphVertex info of the search vertex
        let Ok(sv_vert) = query.get(sv_ent) else {continue;};

        //loop over this vertex's neighbours
        for (neighbour_ent, edge_weight) in sv_vert.get_neighbours_with_weight(){

            //Determine the distance to this neighbour via the path to the searvh vertex
            let total_dist = match edge_weight {
                None => match none_response {
                    NoneValueResponse::Impassable => continue,
                    NoneValueResponse::Infinity => sv_dist.add_infinite(),
                    NoneValueResponse::Value(x) => sv_dist.add_weight(x),
                },
                Some(x) => sv_dist.add_weight(x),
            };

            //check if we have visited this vertex before
            //if so, compare the cardinalities to see if we should update
            if let Some(dist) = minimal_dist.get_mut(&neighbour_ent) {
                //check if the vertex was visited already at a closer distance
                //if so we ignore this vertex
                if total_dist > *dist {continue;}
                //otherwise update the vertex's distance, previous vertex and priority in the queue
                *path_previous.get_mut(&neighbour_ent).unwrap() = Some(sv_ent);
                *dist = total_dist.clone();
                search_queue.change_priority(&neighbour_ent, Reverse(total_dist));
            } else {
                //otherwise the vertex hasnt been visited before and so we add it to the queue, path_previous and the visited entities
                path_previous.insert(neighbour_ent, Some(sv_ent));
                minimal_dist.insert(neighbour_ent, total_dist.clone());
                search_queue.push(neighbour_ent, Reverse(total_dist));
            }
        }
    }

    //if we get to this point, then we must have found no path
    Err(NoPathError)
}

/// Note:returns in reverse order
pub fn dijkstra_search_computed_end<F, S, T>(
    start_ent: Entity,
    end_determiner: F,
    query: &Query<(&GraphVertex, S), T>,
    none_response: NoneValueResponse
) -> Result<Vec<Entity>, NoPathError> 
where
    F: for<'a> Fn(S::Item<'a>) -> bool,
    S: ReadOnlyWorldQuery,
    T: ReadOnlyWorldQuery
{
    //hashmap that stores the previous vertex of the path for a given vertex
    let mut path_previous: HashMap<Entity, Option<Entity>> = HashMap::new();
    path_previous.insert(start_ent, None);

    //The list of visited entities. stores the cardinality (current minimum found distance to the vertex)
    let mut minimal_dist : HashMap<Entity, PathWeight> = HashMap::new();
    minimal_dist.insert(start_ent, PathWeight::new(0.0));

    //create the search queue
    let mut search_queue: PriorityQueue<Entity , Reverse<PathWeight>> = PriorityQueue::new();
    search_queue.push(start_ent, Reverse(PathWeight::new(0.0)));

    while let Some((sv_ent, Reverse(sv_dist))) = search_queue.pop() {
        //get the info of the search vertex
        let Ok((sv_vert, sv_data)) = query.get(sv_ent) else {continue;};

        //check if we are currently searching a valid end vertex, as this implies we have already found a minimum path
        if end_determiner(sv_data) {return Ok(determine_path(path_previous, sv_ent).unwrap());}

        //loop over this vertex's neighbours
        for (neighbour_ent, edge_weight) in sv_vert.get_neighbours_with_weight(){

            //Determine the distance to this neighbour via the path to the searvh vertex
            let total_dist = match edge_weight {
                None => match none_response {
                    NoneValueResponse::Impassable => continue,
                    NoneValueResponse::Infinity => sv_dist.add_infinite(),
                    NoneValueResponse::Value(x) => sv_dist.add_weight(x),
                },
                Some(x) => sv_dist.add_weight(x),
            };

            //check if we have visited this vertex before
            //if so, compare the cardinalities to see if we should update
            if let Some(dist) = minimal_dist.get_mut(&neighbour_ent) {
                //check if the vertex was visited already at a closer distance
                //if so we ignore this vertex
                if total_dist > *dist {continue;}
                //otherwise update the vertex's distance, previous vertex and priority in the queue
                *path_previous.get_mut(&neighbour_ent).unwrap() = Some(sv_ent);
                *dist = total_dist.clone();
                search_queue.change_priority(&neighbour_ent, Reverse(total_dist));
            } else {
                //otherwise the vertex hasnt been visited before and so we add it to the queue, path_previous and the visited entities
                path_previous.insert(neighbour_ent, Some(sv_ent));
                minimal_dist.insert(neighbour_ent, total_dist.clone());
                search_queue.push(neighbour_ent, Reverse(total_dist));
            }
        }
    }

    //if we get to this point, then we must have found no path
    Err(NoPathError)
}

pub fn a_star_search<F, S, T>(
    start_ent: Entity,
    end_ent: Entity,
    query: &Query<(&GraphVertex, S), T>,
    none_response: NoneValueResponse,
    heuristic_determiner: F
) -> Result<Vec<Entity>, NoPathError> 
where
    F: for<'a> Fn(&S::Item<'a>, &S::Item<'a>) -> Heuristic,
    S: ReadOnlyWorldQuery,
    T: ReadOnlyWorldQuery
{
    //get the data of the end vertex to be used for the heuristic calcs
    //the start vertex data will noyl be used before starting the loop
    let (_, end_data)= query.get(end_ent)?;

    //hashmap that stores the previous vertex of the path for a given vertex
    let mut path_previous: HashMap<Entity, Option<Entity>> = HashMap::new();
    path_previous.insert(start_ent, None);

    //The list of visited entities. stores a cardinality for current minimum found distance to the vertex
    //and a cardinality to represent the value of the heuristic
    let mut minimal_dist : HashMap<Entity, (PathWeight, Heuristic)> = HashMap::new();
    minimal_dist.insert(start_ent, (PathWeight::new(0.0), Heuristic::Weight(0.0)));

    //create the search queue, here the pathweight is the sum of the heuristic and actual minimum found distacne
    let mut search_queue: PriorityQueue<Entity , Reverse<PathWeight>> = PriorityQueue::new();
    search_queue.push(start_ent, Reverse(PathWeight::new(0.0)));

    while let Some((sv_ent, _)) = search_queue.pop() {
        //check if we are currently searching the end vertex, as this implies we have already found the minimum path
        if sv_ent == end_ent {
            return Ok(determine_path(path_previous, sv_ent).unwrap());
        }

        //get the GraphVertex info of the search vertex
        let Ok((sv_vert, sv_data)) = query.get(sv_ent) else {continue;};

        //get the distance of the search vertex
        let sv_dist = minimal_dist.get(&sv_ent).unwrap().0.clone();

        //loop over this vertex's neighbours
        for (neighbour_ent, edge_weight) in sv_vert.get_neighbours_with_weight(){

            //Determine the distance to this neighbour via the path to the search vertex
            let total_dist = match edge_weight {
                None => match none_response {
                    NoneValueResponse::Impassable => continue,
                    NoneValueResponse::Infinity => sv_dist.add_infinite(),
                    NoneValueResponse::Value(x) => sv_dist.add_weight(x),
                },
                Some(x) => sv_dist.add_weight(x),
            };
            
            //check if we have visited this vertex before
            //if so, compare the cardinalities to see if we should update
            if let Some((neighbour_dist, neighbour_heuristic)) = minimal_dist.get_mut(&neighbour_ent) {
                //check if the vertex was visited already at a closer distance
                //if so we ignore this vertex
                if total_dist > *neighbour_dist {continue;}
                //otherwise update the vertex's distance, previous vertex and priority in the queue
                //we do not need to recalculate the heuristic in this case
                *path_previous.get_mut(&neighbour_ent).unwrap() = Some(sv_ent);
                search_queue.change_priority(&neighbour_ent, Reverse(total_dist.add_heuristic(neighbour_heuristic)));
                *neighbour_dist = total_dist;
            } else {
                //determine the heurstic of this new value
                let heuristic = heuristic_determiner(&sv_data, &end_data);
                //otherwise the vertex hasnt been visited before and so we add it to the queue, path_previous and the visited entities
                path_previous.insert(neighbour_ent, Some(sv_ent));
                search_queue.push(neighbour_ent, Reverse(total_dist.add_heuristic(&heuristic)));
                minimal_dist.insert(neighbour_ent, (total_dist, heuristic));
                
            }
        }
    }

    //if we get to this point, then we must have found no path
    Err(NoPathError)
}
