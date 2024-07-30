use std::{fs::File, io::{BufRead, BufReader}};

use bevy::{prelude::{Entity, World}, utils::HashMap};

use crate::graph_vertex::StandardGraphVertex;

use super::{GraphLabel, InvalidPathError};




/// Takes the last vertex in the provided vector and follows it's [optional](Option) value to the previous entity in the path.
/// Terminates once it hits a [None]. Returns an [`InvalidPathError`] if there is a loop or a previous [entity](Entity) is not in the vector.
/// 
/// Returns the path as a vector in reverse order
pub(crate) fn determine_path(visited: HashMap<Entity, Option<Entity>>, final_vert: Entity) -> Result<Vec<Entity>, InvalidPathError>{
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
/// Used for testing the graph algorithms, reads from a graph file and adds a [StandardGraphVertex] and [GraphLabel] for each vertex in the file.
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
        let graph_vert = StandardGraphVertex::new_with_edges(edges);
        current_ent.insert((graph_vert,GraphLabel{value: pos}));
    });
    entity_vec
}

