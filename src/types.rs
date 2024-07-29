use std::{error::Error, fmt::Display, ops::Add};

use bevy::{prelude::*, ecs::query::QueryEntityError};


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


