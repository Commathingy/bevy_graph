use bevy::{ecs::query::{QueryData, QueryFilter}, prelude::*, utils::{HashMap, HashSet}};
use priority_queue::PriorityQueue;
use std::{collections::VecDeque, cmp::Reverse};
use crate::types::*;
use crate::graph_vertex::GraphVertex;


pub(crate) mod helper;
pub mod bfs;
pub mod dfs;
pub mod astar;
pub mod dijkstra;

use bfs::*;
use dfs::*;
use dijkstra::*;
use astar::*;



//TODO:
//make Vec<Entity> a new type (eg Path)
//good docs! (those already existing need to be changed too)
//better tests
//a negative edge weight algo
//add a filter ability
//more options for how we use extra types (the &C's)
//-> would be nice if could use a (&C, &D) somehow


pub trait GraphFunctionExt{
    fn bfs<V: GraphVertex>(&mut self, start_ent: Entity, end_ent: Entity) -> Result<Vec<Entity>, GraphError>;

    fn bfs_computed_end<V, CE, FE>(&mut self, start_ent: Entity, end_determiner: FE) -> Result<Vec<Entity>, GraphError>
    where V: GraphVertex, CE: Component, FE: Fn(&CE) -> bool;

    fn bfs_multiple_end<V, CE, FE>(&mut self, start_ent: Entity, end_determiner: FE, limit: SearchLimits) -> Result<Vec<Vec<Entity>>, GraphError> //compute paths to every end point satisfying the end_determiner, up to a maximum amount provided (perhaps a limiter -> None, max_steps, max_number)
    where V: GraphVertex, CE: Component, FE: Fn(&CE) -> bool;

    fn dfs<V: GraphVertex>(&mut self, start_ent: Entity, end_ent: Entity) -> Result<Vec<Entity>, GraphError>;

    fn dfs_computed_end<V, CE, FE>(&mut self, start_ent: Entity, end_determiner: FE) -> Result<Vec<Entity>, GraphError>
    where V: GraphVertex, CE: Component, FE: Fn(&CE) -> bool;

    fn dfs_multiple_end<V, CE, FE>(&mut self, start_ent: Entity, end_determiner: FE, limit: SearchLimits) -> Result<Vec<Vec<Entity>>, GraphError> //compute paths to every end point satisfying the end_determiner, up to a maximum amount provided (perhaps a limiter -> None, max_steps, max_number)
    where V: GraphVertex, CE: Component, FE: Fn(&CE) -> bool;

    fn dijkstra_search<V: GraphVertex>(&mut self, start_ent: Entity, end_ent: Entity) -> Result<Vec<Entity>, GraphError>;

    fn dijkstra_computed_end<V, CE, FE>(&mut self, start_ent: Entity, end_determiner: FE) -> Result<Vec<Entity>, GraphError>
    where V: GraphVertex, CE: Component, FE: Fn(&CE) -> bool;

    fn dijkstra_multiple_end<V, CE, FE>(&mut self, start_ent: Entity, end_determiner: FE, limit: SearchLimits) -> Result<Vec<Vec<Entity>>, GraphError> //compute paths to every end point satisfying the end_determiner, up to a maximum amount provided (perhaps a limiter -> None, max_steps, max_number)
    where V: GraphVertex, CE: Component, FE: Fn(&CE) -> bool;

    fn a_star_search<V, CH, FH>(&mut self, start_ent: Entity, end_ent: Entity, heuristic_determiner: FH) -> Result<Vec<Entity>, GraphError> 
    where V: GraphVertex, CH: Component, FH: Fn(&CH, &CH) -> Heuristic;

    fn a_star_computed_end<V, CH, CE, FH, FE>(&mut self, start_ent: Entity, heuristic_determiner: FH, end_determiner: FE) -> Result<Vec<Vec<Entity>>, GraphError> //compute paths to every end point satisfying the end_determiner, up to a maximum amount provided (perhaps a limiter -> None, max_steps, max_number)
    where V: GraphVertex, CH: Component, CE: Component, FH: Fn(&CH) -> Heuristic, FE: Fn(&CE) -> bool; //heuristic depends on only &C rather than &C,&C

    fn a_star_multiple_ends<V, CH, CE, FH, FE>(&mut self, start_ent: Entity, heuristic_determiner: FH, end_determiner: FE, limit: SearchLimits) -> Result<Vec<Vec<Entity>>, GraphError> //compute paths to every end point satisfying the end_determiner, up to a maximum amount provided (perhaps a limiter -> None, max_steps, max_number)
    where V: GraphVertex, CH: Component, CE: Component, FH: Fn(&CH) -> Heuristic, FE: Fn(&CE) -> bool; //difference between this and computed end same as bfs, dfs

    fn within_steps<V:GraphVertex>(&mut self, start_ent: Entity, max_steps: usize) -> Result<Vec<(Entity, usize)>, GraphError>;

    fn within_distance<V:GraphVertex>(&mut self, start_ent: Entity, max_distance: f32) -> Result<Vec<(Entity, f32)>, GraphError>;

    fn at_step<V:GraphVertex>(&mut self, start_ent: Entity, at_step: usize) -> Result<Vec<Entity>, GraphError>;
}


impl<'world, 'state, S: QueryData, T: QueryFilter> GraphFunctionExt for Query<'world, 'state, S, T>{
    fn bfs<V: GraphVertex>(&mut self, start_ent: Entity, end_ent: Entity) -> Result<Vec<Entity>, GraphError> {
        let mut lensed = self.transmute_lens::<&V>();
        bfs(&lensed.query(), start_ent, end_ent)
    }
    
    fn bfs_computed_end<V, C, F> (&mut self, start_ent: Entity, end_determiner: F) -> Result<Vec<Entity>, GraphError>
    where V: GraphVertex, C: Component, F: for<'a> Fn(&'a C) -> bool {
        let mut lensed = self.transmute_lens::<(&V, &C)>();
        bfs_computed_end(&lensed.query(), start_ent, end_determiner)
    }
    
    fn dfs<V: GraphVertex>(&mut self, start_ent: Entity, end_ent: Entity) -> Result<Vec<Entity>, GraphError> {
        let mut lensed = self.transmute_lens::<&V>();
        dfs(&lensed.query(), start_ent, end_ent)
    }
    
    fn dfs_computed_end<V, C, F>(&mut self, start_ent: Entity, end_determiner: F) -> Result<Vec<Entity>, GraphError>
    where V: GraphVertex, C: Component, F: Fn(&C) -> bool {
        let mut lensed = self.transmute_lens::<(&V, &C)>();
        dfs_computed_end(&lensed.query(), start_ent, end_determiner)
    }
    
    fn dijkstra_search<V: GraphVertex>(&mut self, start_ent: Entity, end_ent: Entity) -> Result<Vec<Entity>, GraphError> {
        let mut lensed = self.transmute_lens::<&V>();
        dijkstra_search(&lensed.query(), start_ent, end_ent)
    }
    
    fn dijkstra_computed_end<V, C, F>(&mut self, start_ent: Entity, end_determiner: F) -> Result<Vec<Entity>, GraphError>
    where V: GraphVertex, C: Component, F: Fn(&C) -> bool {
        let mut lensed = self.transmute_lens::<(&V, &C)>();
        dijkstra_computed_end(&lensed.query(), start_ent, end_determiner)
    }
    
    fn a_star_search<V, C, F>(&mut self, start_ent: Entity, end_ent: Entity, heuristic_determiner: F) -> Result<Vec<Entity>, GraphError> 
    where V: GraphVertex, C: Component, F: Fn(&C, &C) -> Heuristic {
        let mut lensed = self.transmute_lens::<(&V, &C)>();
        a_star_search(&lensed.query(), start_ent, end_ent, heuristic_determiner)
    }
    
    fn within_steps<V:GraphVertex>(&mut self, start_ent: Entity, max_steps: usize) -> Result<Vec<(Entity, usize)>, GraphError> {
        let mut lensed = self.transmute_lens::<&V>();
        within_steps(&lensed.query(), start_ent, max_steps)
    }
    
    fn within_distance<V:GraphVertex>(&mut self, start_ent: Entity, max_distance: f32) -> Result<Vec<(Entity, f32)>, GraphError> {
        let mut lensed = self.transmute_lens::<&V>();
        within_distance(&lensed.query(), start_ent, max_distance)
    }
    
    fn at_step<V:GraphVertex>(&mut self, start_ent: Entity, step: usize) -> Result<Vec<Entity>, GraphError> {
        let mut lensed = self.transmute_lens::<&V>();
        at_step(&lensed.query(), start_ent, step)
        
    }
    
    fn bfs_multiple_end<V, CE, FE>(&mut self, start_ent: Entity, end_determiner: FE, limit: SearchLimits) -> Result<Vec<Vec<Entity>>, GraphError> where V: GraphVertex, CE: Component, FE: Fn(&CE) -> bool {
        todo!()
    }
    
    fn dfs_multiple_end<V, CE, FE>(&mut self, start_ent: Entity, end_determiner: FE, limit: SearchLimits) -> Result<Vec<Vec<Entity>>, GraphError> where V: GraphVertex, CE: Component, FE: Fn(&CE) -> bool {
        todo!()
    }
    
    fn dijkstra_multiple_end<V, CE, FE>(&mut self, start_ent: Entity, end_determiner: FE, limit: SearchLimits) -> Result<Vec<Vec<Entity>>, GraphError> where V: GraphVertex, CE: Component, FE: Fn(&CE) -> bool {
        todo!()
    }
    
    fn a_star_computed_end<V, CH, CE, FH, FE>(&mut self, start_ent: Entity, heuristic_determiner: FH, end_determiner: FE) -> Result<Vec<Vec<Entity>>, GraphError> where V: GraphVertex, CH: Component, CE: Component, FH: Fn(&CH) -> Heuristic, FE: Fn(&CE) -> bool {
        todo!()
    }
    
    fn a_star_multiple_ends<V, CH, CE, FH, FE>(&mut self, start_ent: Entity, heuristic_determiner: FH, end_determiner: FE, limit: SearchLimits) -> Result<Vec<Vec<Entity>>, GraphError> where V: GraphVertex, CH: Component, CE: Component, FH: Fn(&CH) -> Heuristic, FE: Fn(&CE) -> bool {
        todo!()
    }

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
pub fn within_steps<V: GraphVertex>(
    query: &Query<&V>,
    start_ent: Entity,
    max_steps: usize
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
pub fn within_distance<V: GraphVertex>(
    query: &Query<&V>,
    start_ent: Entity,
    max_distance: f32,
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
pub fn at_step<V:GraphVertex>(
    query: &Query<&V>,
    start_ent: Entity,
    at_step: usize,
) -> Result<Vec<Entity>, GraphError> {
    Ok(within_steps(query, start_ent, at_step)?.into_iter()
    .filter_map(|(ent, step)| if step == at_step {Some(ent)} else {None})
    .collect())
}