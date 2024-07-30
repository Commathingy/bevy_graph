use std::cmp::Reverse;

use bevy::{prelude::{Component, Entity, Query}, utils::HashMap};
use priority_queue::PriorityQueue;

use crate::graph_vertex::GraphVertex;

use super::{GraphError, GraphPath, Heuristic, PathWeight, VisitedNodes};


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
) -> Result<GraphPath<f32>, GraphError> 
where
    V: GraphVertex,
    C: Component,
    F: Fn(&C, &C) -> Heuristic
{
    let (_, end_data)= query.get(end_ent)?;

    let mut visited = VisitedNodes::new_from_start(start_ent);

    let mut minimal_dist : HashMap<Entity, (PathWeight, Heuristic)> = HashMap::new();
    minimal_dist.insert(start_ent, (PathWeight{weight: 0.0}, Heuristic{value: 0.0}));

    let mut search_queue: PriorityQueue<Entity , Reverse<PathWeight>> = PriorityQueue::new();
    search_queue.push(start_ent, Reverse(PathWeight{weight: 0.0}));

    while let Some((sv_ent, _)) = search_queue.pop() {
        //check if we are currently searching the end vertex, as this implies we have already found the minimum path
        if sv_ent == end_ent {return Ok(visited.determine_path_weighted(sv_ent).expect("The created path should be valid"));}

        let Ok((sv_vert, sv_data)) = query.get(sv_ent) else {continue;};

        let sv_dist = minimal_dist.get(&sv_ent).unwrap().0; //true minimum distance to this vertex

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
                visited.set_previous(neighbour_ent, sv_ent, total_dist.weight); 
                todo!("need to also change the value inside visited nodes at this point");
                search_queue.change_priority(&neighbour_ent, Reverse(total_dist + *neighbour_heuristic));
                *neighbour_dist = total_dist;
            } else {
                //determine the heuristic of this new value
                let heuristic = heuristic_determiner(&sv_data, &end_data);
                //otherwise the vertex hasnt been visited before and so we add it to the queue, visited and min distances
                visited.insert(neighbour_ent, sv_ent, 0, total_dist.weight);
                search_queue.push(neighbour_ent, Reverse(total_dist + heuristic));
                minimal_dist.insert(neighbour_ent, (total_dist, heuristic));
                
            }
        }
    }

    //if we get to this point, then we must have found no path
    Err(GraphError::NoPath)
}


