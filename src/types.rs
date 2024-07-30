use std::{error::Error, fmt::Display, ops::Add};

use bevy::{ecs::query::QueryEntityError, prelude::*, utils::HashMap};


#[derive(Component)]
pub struct GraphLabel {
    pub value: usize
}


/// Error encountered when trying to determine_path on a set of (Entity, Option<Entity>) pairs where there is either a loop or a missing entity
#[derive(Debug)]
pub(crate) struct InvalidPathError;
impl Display for InvalidPathError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "the provided set of (vertex, previous vertex) pairs does not result in a valid path")
    }
}
impl Error for InvalidPathError {}

#[derive(Debug)]
pub enum GraphError{
    NoPath,
    InvalidEntity,
    NegativeWeight
}

impl From<QueryEntityError> for GraphError {
    fn from(_: QueryEntityError) -> Self {
        Self::InvalidEntity
    }
}
impl From<InvalidPathError> for GraphError {
    fn from(_: InvalidPathError) -> Self {
        Self::NoPath
    }
}
impl Display for GraphError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self{
            GraphError::NoPath => write!(f, "no path could be found between the provided vertices"),
            GraphError::InvalidEntity => write!(f, "the provided entity is not a valid GraphVertex"),
            GraphError::NegativeWeight => write!(f, "a provided edge weight was negative"),
        }
    }
}
impl Error for GraphError {}

#[derive(Clone, Copy)]
pub struct Heuristic{
    pub value: f32
}


#[derive(Clone, Copy)]
pub(crate) struct PathWeight{
    pub weight: f32
}

impl Add<PathWeight> for PathWeight{
    type Output = Self;

    fn add(self, rhs: PathWeight) -> Self::Output {
        Self{weight: self.weight + rhs.weight}
    }
}

impl Add<f32> for PathWeight{
    type Output = Self;

    fn add(self, rhs: f32) -> Self::Output {
        Self{weight: self.weight + rhs}
    }
}

impl Add<Heuristic> for PathWeight{
    type Output = Self;

    fn add(self, rhs: Heuristic) -> Self::Output {
        Self{weight: self.weight + rhs.value}
    }
}

impl PartialEq for PathWeight{
    fn eq(&self, other: &Self) -> bool {
        self.weight == other.weight
    }
}
impl Eq for PathWeight{}
impl PartialOrd for PathWeight{
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        if self.weight < other.weight {Some(std::cmp::Ordering::Less)}
        else {Some(std::cmp::Ordering::Greater)}
    }
}
impl Ord for PathWeight{
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        if self.weight < other.weight {std::cmp::Ordering::Less}
        else {std::cmp::Ordering::Greater}
    }
}


pub struct GraphPath<D>{
    path: Vec<(Entity, D)>
}

impl<D> GraphPath<D>{
    pub fn new(path: Vec<(Entity,D)>) -> Self {
        Self{path}
    }

    pub fn single(start_ent: Entity, val: D) -> Self {
        Self { path: vec![(start_ent, val)] }
    }
}

pub struct VisitedNodes{
    nodes: HashMap<Entity, (Option<Entity>, u64, f32)>
}

impl VisitedNodes{
    pub fn new_from_start(start_ent: Entity) -> Self{
        let mut nodes = HashMap::new();
        nodes.insert(start_ent, (None, 0, 0.0));
        Self{nodes}
    }

    pub fn is_visited(&self, ent: &Entity) -> bool {
        self.nodes.contains_key(ent)
    }

    pub fn insert(&mut self, ent: Entity, previous: Entity, step: u64, dist: f32) {
        self.nodes.insert(ent, (Some(previous), step, dist));
    }

    pub fn determine_path(&self, final_vert: Entity) -> Result<GraphPath<()>, InvalidPathError> {
        let Some(&(mut to_follow, _, _)) = self.nodes.get(&final_vert) else {return Err(InvalidPathError)};
        let mut path = vec![(final_vert, ())];
        let max_length = self.nodes.len();
        while to_follow.is_some(){
            path.push((to_follow.unwrap(), ()));
            to_follow = match self.nodes.get(&to_follow.unwrap()){
                Some(&(val, _, _)) => val,
                None => return Err(InvalidPathError)
            };
            //check for a loop
            if path.len() > max_length {return Err(InvalidPathError)}
        }
        Ok(GraphPath::new(path))
    }

    pub fn determine_path_weighted(&self, final_vert: Entity) -> Result<GraphPath<f32>, InvalidPathError> {
        let Some(&(mut to_follow, _, mut dist)) = self.nodes.get(&final_vert) else {return Err(InvalidPathError)};
        let mut path = vec![(final_vert, dist)];
        let max_length = self.nodes.len();
        while to_follow.is_some(){
            let prev = to_follow.unwrap();
            (to_follow, dist) = match self.nodes.get(&to_follow.unwrap()){
                Some(&(val, _, dist)) => (val, dist),
                None => return Err(InvalidPathError)
            };
            path.push((prev, dist));
            //check for a loop
            if path.len() > max_length {return Err(InvalidPathError)}
        }
        Ok(GraphPath::new(path))
    }
}