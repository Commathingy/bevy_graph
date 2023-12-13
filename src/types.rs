use std::{error::Error, fmt::Display};

use bevy::{prelude::*, ecs::query::QueryEntityError};


#[derive(Component)]
pub struct GraphLabel {
    pub value: usize
}

#[derive(Component)]
pub struct GraphVertex {
    neighbours: Vec<(Entity, Option<f32>)>
}

impl GraphVertex {
    pub fn new() -> Self{
        Self{neighbours: Vec::new()}
    }
    pub fn new_with_edges(edges: Vec<(Entity, Option<f32>)>) -> Self{
        Self{neighbours: edges}
    }

    pub fn get_neighbours(&self) -> Vec<Entity>{
        self.neighbours.iter().map(|(ent, _)| *ent).collect()
    }
    pub fn get_neighbours_with_weight(&self) -> Vec<(Entity, Option<f32>)> {
        self.neighbours.clone()
    }
}

/// The response to take when encountering a None value for an edge weight inside a graph algorithm such as Dijkstra Search or A* Search
/// 
/// Impassable means this edge shall be ignored
/// 
/// Infinity means the edge shall not be ignored, but it will consider the weight of using this edge to be arbitraily large and as such a path shall only
/// use this edge if it is the only one available. If two routes to a vertex both cross None edges, the one that crosses the least will be the one that is used. 
/// If both cross the same number, the one that has the smallest total weight ignoring infinite edges will be used
/// 
/// Value means the edge will be given the deafult value provided inside the variant
pub enum NoneValueResponse{
    Impassable,
    Infinity,
    Value(f32)
}

/// Error encountered when trying to determine_path on a set of (Entity, Option<Entity>) pairs where there is either a loop or a missing entity
#[derive(Debug)]
pub struct InvalidPathError;
impl Display for InvalidPathError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "The provided set of (vertex, previous vertex) pairs does not result in a valid path")
    }
}
impl Error for InvalidPathError {}

#[derive(Debug)]
pub struct NoPathError;

impl From<QueryEntityError> for NoPathError {
    fn from(_: QueryEntityError) -> Self {
        Self
    }
}
impl Display for NoPathError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "No path could be found between the provided vertices.")
    }
}
impl Error for NoPathError {}

#[derive(Clone)]
pub struct PathWeight{
    weight: Vec<f32>,
}
impl PathWeight{
    pub fn new(starting_weight: f32) -> Self{
        Self{weight: vec![starting_weight]}
    }
    pub fn add_infinite(&self) -> Self{
        let mut new = self.weight.clone();
        new.push(0.0);
        Self{weight: new}
    }
    pub fn add_weight(&self, weight: f32) -> Self{
        let mut new = self.weight.clone();
        *new.last_mut().unwrap() += weight;
        Self{weight: new}
    }
    pub fn add_heuristic(&self, weight: &Heuristic) -> Self{
        match weight{
            Heuristic::Weight(val) => self.add_weight(*val),
            Heuristic::Infinite => self.add_infinite(),
        }
    }
    pub fn num_infinite(&self) -> usize {
        self.weight.len() - 1
    }
    pub fn total_weight_no_infinite(&self) -> f32{
        self.weight.iter().sum()
    }
}
impl PartialEq for PathWeight{
    fn eq(&self, other: &Self) -> bool {
        self.weight.len() == other.weight.len() &&
        self.weight.iter().zip(other.weight.iter()).all(|(a,b)| *a == *b)
    }
}
impl Eq for PathWeight{}
impl PartialOrd for PathWeight{
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        if self.weight.len() < other.weight.len() {Some(std::cmp::Ordering::Less)}
        else if self.weight.len() > other.weight.len() {Some(std::cmp::Ordering::Greater)}
        else if self.weight.iter().copied().sum::<f32>() < other.weight.iter().copied().sum() {Some(std::cmp::Ordering::Less)}
        else {Some(std::cmp::Ordering::Greater)}
    }
}
impl Ord for PathWeight{
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        if self.weight.len() < other.weight.len() {std::cmp::Ordering::Less}
        else if self.weight.len() > other.weight.len() {std::cmp::Ordering::Greater}
        else if self.weight.iter().copied().sum::<f32>() < other.weight.iter().copied().sum() {std::cmp::Ordering::Less}
        else {std::cmp::Ordering::Greater}
    }
}

pub enum Heuristic{
    Weight(f32),
    Infinite
}


