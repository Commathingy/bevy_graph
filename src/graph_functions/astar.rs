use std::cmp::Reverse;

use bevy::{prelude::{Component, Entity, Query}, utils::HashMap};
use priority_queue::PriorityQueue;

use crate::graph_vertex::GraphVertex;

use super::{helper::determine_path, GraphError, Heuristic, PathWeight};


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
pub fn a_star_search<V, C, F>(
    query: &Query<(&V, &C)>,
    start_ent: Entity,
    end_ent: Entity,
    heuristic_determiner: F
) -> Result<Vec<Entity>, GraphError> 
where
    V: GraphVertex,
    C: Component,
    F: Fn(&C, &C) -> Heuristic
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
