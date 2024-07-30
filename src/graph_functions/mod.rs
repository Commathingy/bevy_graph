use bevy::{ecs::query::{QueryData, QueryFilter}, prelude::{Component, Entity, Query}};
use crate::types::*;
use crate::graph_vertex::GraphVertex;


pub(crate) mod helper;
pub mod bfs;
pub mod dfs;
pub mod astar;
pub mod dijkstra;
pub mod neighbourhood;

use bfs::*;
use dfs::*;
use dijkstra::*;
use astar::*;
use neighbourhood::*;



//TODO:
//make Vec<Entity> a new type (eg Path)
//good docs! (those already existing need to be changed too)
//better tests
//a negative edge weight algo
//add a filter ability
//more options for how we use extra types (the &C's)
//-> would be nice if could use a (&C, &D) somehow


pub trait GraphFunctionExt{

    //====================================
    // Breadth First Search Based Algorithms
    //====================================
    fn bfs<V: GraphVertex>(&mut self, start_ent: Entity, end_ent: Entity) -> Result<GraphPath<()>, GraphError>;

    fn bfs_computed_end<V, CE, FE>(&mut self, start_ent: Entity, end_determiner: FE) -> Result<GraphPath<()>, GraphError>
    where V: GraphVertex, CE: Component, FE: Fn(&CE) -> bool;

    fn bfs_multiple_end<V, CE, FE>(&mut self, start_ent: Entity, end_determiner: FE, max_ends: Option<usize>, max_steps: Option<u64>) -> Result<Vec<GraphPath<()>>, GraphError> //compute paths to every end point satisfying the end_determiner, up to a maximum amount provided (perhaps a limiter -> None, max_steps, max_number)
    where V: GraphVertex, CE: Component, FE: Fn(&CE) -> bool;


    //====================================
    // Depth First Search Based Algorithms
    //====================================
    fn dfs<V: GraphVertex>(&mut self, start_ent: Entity, end_ent: Entity) -> Result<GraphPath<()>, GraphError>;

    fn dfs_computed_end<V, CE, FE>(&mut self, start_ent: Entity, end_determiner: FE) -> Result<GraphPath<()>, GraphError>
    where V: GraphVertex, CE: Component, FE: Fn(&CE) -> bool;

    fn dfs_multiple_end<V, CE, FE>(&mut self, start_ent: Entity, end_determiner: FE, max_ends: Option<usize>) -> Result<Vec<GraphPath<()>>, GraphError> //compute paths to every end point satisfying the end_determiner, up to a maximum amount provided (perhaps a limiter -> None, max_steps, max_number)
    where V: GraphVertex, CE: Component, FE: Fn(&CE) -> bool;


    //====================================
    // Dijsktra Search Based Algorithms
    //====================================
    fn dijkstra_search<V: GraphVertex>(&mut self, start_ent: Entity, end_ent: Entity) -> Result<GraphPath<f32>, GraphError>;

    fn dijkstra_computed_end<V, CE, FE>(&mut self, start_ent: Entity, end_determiner: FE) -> Result<GraphPath<f32>, GraphError>
    where V: GraphVertex, CE: Component, FE: Fn(&CE) -> bool;

    fn dijkstra_multiple_end<V, CE, FE>(&mut self, start_ent: Entity, end_determiner: FE, max_ends: Option<usize>, max_dist: Option<f32>) -> Result<Vec<GraphPath<f32>>, GraphError> //compute paths to every end point satisfying the end_determiner, up to a maximum amount provided (perhaps a limiter -> None, max_steps, max_number)
    where V: GraphVertex, CE: Component, FE: Fn(&CE) -> bool;


    //====================================
    // A Star Search Based Algorithms
    //====================================
    fn a_star_search<V, CH, FH>(&mut self, start_ent: Entity, end_ent: Entity, heuristic_determiner: FH) -> Result<GraphPath<f32>, GraphError> 
    where V: GraphVertex, CH: Component, FH: Fn(&CH, &CH) -> Heuristic;

    fn a_star_computed_end<V, CH, CE, FH, FE>(&mut self, start_ent: Entity, heuristic_determiner: FH, end_determiner: FE) -> Result<GraphPath<f32>, GraphError> //compute paths to every end point satisfying the end_determiner, up to a maximum amount provided (perhaps a limiter -> None, max_steps, max_number)
    where V: GraphVertex, CH: Component, CE: Component, FH: Fn(&CH) -> Heuristic, FE: Fn(&CE) -> bool; //heuristic depends on only &C rather than &C,&C

    fn a_star_multiple_ends<V, CH, CE, FH, FE>(&mut self, start_ent: Entity, heuristic_determiner: FH, end_determiner: FE, max_ends: Option<usize>, max_dist: Option<f32>) -> Result<Vec<GraphPath<f32>>, GraphError> //compute paths to every end point satisfying the end_determiner, up to a maximum amount provided (perhaps a limiter -> None, max_steps, max_number)
    where V: GraphVertex, CH: Component, CE: Component, FH: Fn(&CH) -> Heuristic, FE: Fn(&CE) -> bool; //difference between this and computed end same as bfs, dfs


    //====================================
    // Neighbourhood Algorithms
    //====================================
    fn within_steps<V:GraphVertex>(&mut self, start_ent: Entity, max_steps: usize) -> Result<Vec<(Entity, usize)>, GraphError>;

    fn within_distance<V:GraphVertex>(&mut self, start_ent: Entity, max_distance: f32) -> Result<Vec<(Entity, f32)>, GraphError>;

    fn at_step<V:GraphVertex>(&mut self, start_ent: Entity, at_step: usize) -> Result<Vec<Entity>, GraphError>;

}


impl<'world, 'state, S: QueryData, T: QueryFilter> GraphFunctionExt for Query<'world, 'state, S, T>{
    fn bfs<V: GraphVertex>(&mut self, start_ent: Entity, end_ent: Entity) -> Result<GraphPath<()>, GraphError> {
        let mut lensed = self.transmute_lens::<&V>();
        bfs(&lensed.query(), start_ent, end_ent)
    }
    
    fn bfs_computed_end<V, C, F> (&mut self, start_ent: Entity, end_determiner: F) -> Result<GraphPath<()>, GraphError>
    where V: GraphVertex, C: Component, F: for<'a> Fn(&'a C) -> bool {
        let mut lensed = self.transmute_lens::<(&V, &C)>();
        bfs_computed_end(&lensed.query(), start_ent, end_determiner)
    }
    
    fn dfs<V: GraphVertex>(&mut self, start_ent: Entity, end_ent: Entity) -> Result<GraphPath<()>, GraphError> {
        let mut lensed = self.transmute_lens::<&V>();
        dfs(&lensed.query(), start_ent, end_ent)
    }
    
    fn dfs_computed_end<V, C, F>(&mut self, start_ent: Entity, end_determiner: F) -> Result<GraphPath<()>, GraphError>
    where V: GraphVertex, C: Component, F: Fn(&C) -> bool {
        let mut lensed = self.transmute_lens::<(&V, &C)>();
        dfs_computed_end(&lensed.query(), start_ent, end_determiner)
    }
    
    fn dijkstra_search<V: GraphVertex>(&mut self, start_ent: Entity, end_ent: Entity) -> Result<GraphPath<f32>, GraphError> {
        let mut lensed = self.transmute_lens::<&V>();
        dijkstra_search(&lensed.query(), start_ent, end_ent)
    }
    
    fn dijkstra_computed_end<V, C, F>(&mut self, start_ent: Entity, end_determiner: F) -> Result<GraphPath<f32>, GraphError>
    where V: GraphVertex, C: Component, F: Fn(&C) -> bool {
        let mut lensed = self.transmute_lens::<(&V, &C)>();
        dijkstra_computed_end(&lensed.query(), start_ent, end_determiner)
    }
    
    fn a_star_search<V, C, F>(&mut self, start_ent: Entity, end_ent: Entity, heuristic_determiner: F) -> Result<GraphPath<f32>, GraphError> 
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

}







