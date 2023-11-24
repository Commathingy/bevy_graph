use std::{collections::HashMap, error::Error, fmt::Display};

use bevy::prelude::*;

pub struct BevyGraphPlugin;

#[derive(Clone, Copy)]
pub struct GraphVertexEntity(Entity);

impl GraphVertexEntity {
    fn get(&self) -> Entity {
        self.0
    }
}

enum VertexIndex {
    Unassigned,
    Assigned(usize)
}
impl VertexIndex{
    fn is_assigned(&self) -> bool {
        match *self {
            VertexIndex::Unassigned => false,
            VertexIndex::Assigned(_) => true,
        }
    }
}

#[derive(Component)]
pub struct GraphVertex {
    self_id: GraphVertexEntity,
    index: VertexIndex,
}

impl GraphVertex{
    /// Creates a new GraphVertex with the Entity id of the entity it will be attached to
    /// 
    /// It is important that the provided ID is correct, else it will cause unexpected problems
    pub fn new_with_id(id: Entity) -> Self {
        GraphVertex {self_id: GraphVertexEntity(id), index: VertexIndex::Unassigned}
    }

    fn assign_index(&mut self, index: usize){
        self.index = VertexIndex::Assigned(index);
    }

    fn unassign(&mut self){
        self.index = VertexIndex::Unassigned;
    }

    pub fn is_assigned(&self) -> bool {
        self.index.is_assigned()
    }
}

struct InnerGraphEdge {
    vertices: (usize, usize),
    weight: Option<f32>
}

struct InnerGraphVertex {
    entity: Option<GraphVertexEntity>,
    heuristic: Option<Vec3>
}

#[derive(Resource)]
pub struct Graph {
    vertices: HashMap<usize, InnerGraphVertex>,
    edges: Vec<InnerGraphEdge>,
    missing_indices: Vec<usize>,
    free_indices: Vec<usize>
}

#[derive(Debug)]
pub enum GraphError{
    VertexAlreadyAssigned,
    NoSuchVertex,
    InvalidWeight,
    NoEntity,
    VertexNotFree,
    EdgeAlreadyExists,
    NoSuchEdge
}

impl Display for GraphError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}

impl Error for GraphError{}


impl Graph {
    /// Create a new empty graph
    pub(crate) fn new() -> Self {
        Graph{vertices: HashMap::new(), edges: Vec::new(), missing_indices: Vec::new(), free_indices: Vec::new()}
    }

    /// Adds the given GraphVertex to the Graph with a provided heuristic, setting the index field of the provided vertex to the index used by the graph.
    /// Returns the index of the added vertex
    /// 
    /// The heuristic is a value attached to the vertex that is used in the A* calculation. It may represent a position in world space, or just a single f32 value.
    /// How this heuristic is used is determined by the closure passed to the a_star_search method
    /// 
    /// Note that this will result in an VertexAlreadyAssigned error if the vertex has already been assigned an index
    pub fn add_vertex(&mut self, vertex: &mut GraphVertex, heuristic: Option<Vec3>) -> Result<usize, GraphError> {
        //check if the vertex is already assigned
        if vertex.index.is_assigned() {
            return Err(GraphError::VertexAlreadyAssigned);
        }

        //determine the index we should use for this vertex
        let index = self.missing_indices.pop().unwrap_or_else(|| self.vertices.len());

        //set the index of the GraphVertex
        vertex.assign_index(index);

        //add the vertex to the graph
        self.vertices.insert(
            index,
            InnerGraphVertex{  
                entity: Some(vertex.self_id), 
                heuristic 
            }
        );

        return Ok(index);
    }

    /// Does the same as add_vertex, but does not require a GraphVertex. This will instead create a vertex that exists inside the graph, but has 
    /// no assosciated GraphVertex component. 
    pub fn add_unattached_vertex(&mut self, heuristic: Option<Vec3>) -> usize {
        //determine the index we should use for this vertex
        let index = self.missing_indices.pop().unwrap_or_else(|| self.vertices.len());
        self.free_indices.push(index);
        self.vertices.insert(
            index,
            InnerGraphVertex{  
                entity: None, 
                heuristic 
            }
        );
        index
    }

    pub fn attach_vertex(&mut self, vertex: &mut GraphVertex, index: usize) -> Result<(), GraphError> {
        //check if the vertex is already assigned
        if vertex.index.is_assigned() {
            return Err(GraphError::VertexAlreadyAssigned);
        }
        //check if the given index is actually a free index
        match self.free_indices.iter().position(|x| *x == index){
            Some(pos) => {self.free_indices.swap_remove(pos);},
            None => {return Err(GraphError::VertexNotFree);},
        }

        //asign the index
        vertex.assign_index(index);

        //add the entity to the vertex inside the graph
        self.vertices.get_mut(&index).unwrap().entity = Some(vertex.self_id);

        Ok(())
    }

    /// Removes the given vertex from the graph, returning a NoSuchVertex error if the vertex has not been added to the graph previously
    pub fn remove_vertex(&mut self, vertex: &mut GraphVertex) -> Result<(), GraphError> {
        //get the index of the vertex or return error
        let VertexIndex::Assigned(index) = vertex.index else {return Err(GraphError::NoSuchVertex);};

        //TODO: determine what to do with the removed index
        //remove all edges containing this vertex
        todo!();

        //remove the vertex from the map
        self.vertices.remove(&index);
        Ok(())
    }

    /// Removes the given unattached vertex from the graph, returning a NoSuchVertex error if it cannot be found 
    /// or a AttachedVertex error if it has an associated entity attached to it, in which remove_vertex should be used instead.
    pub fn remove_unattached_vertex(&mut self, index: usize) -> Result<(), GraphError> {
        todo!()
    }

    pub fn alter_vertex_heuristic(&mut self, index: usize, new_heuristic: Option<Vec3>) -> Result<(), GraphError> {
        todo!()
    }

    /// Adds a new edge to the graph, from the first vertex to the second. Returns a NoSuchVertex error if one of the indices does not correspond to a vertex, 
    /// an EdgeAlreadyExists error if the edge is already in the graph or an InvalidWeight error if the weight is negative
    pub fn add_edge(&mut self, index1: usize, index2: usize, weight: Option<f32>) -> Result<(), GraphError> {
        todo!()
    }

    /// Removes the given edge from the graph, possibly returning a NoSuchVertex error or a NoSuchEdge error.
    pub fn remove_edge(&mut self, index1: usize, index2: usize) -> Result<(), GraphError> {
        todo!()
    }

    /// Sets the weight of the edge from the first to the second vertex to the provided new_weight
    /// 
    /// Returns a NoSuchVertex error if one of the indices does not correspond to a vertex, a NoSuchEdge if this does not correspond to
    /// an edge inside the graph and an InvalidWeight error if the provided weight is negative
    pub fn alter_edge_weight(&self, index1: usize, index2: usize, new_weight: Option<f32>) -> Result<GraphVertexEntity, GraphError> {
        todo!()
    }

    /// Does the same as add_edge, unless an EdgeAlreadyExists error is returned, in which case it does the same as alter_edge_weight
    pub fn add_or_alter_edge(&mut self, index1: usize, index2: usize, weight: Option<f32>) -> Result<GraphVertexEntity, GraphError>  {
        todo!()
    }

    /// Returns the GraphVertexEntity of the given index, returning a NoSuchVertex error if it does not exist or a NoEntity if 
    pub fn at_index(&self, index: usize) -> Result<GraphVertexEntity, GraphError> {
        todo!()
    }

    /// Returns a hashmap that map from a vertex to its set of neighbours
    fn construct_adjacency(&self) -> HashMap<usize, Vec<usize>>{
        todo!()
    }

    /// Perform Dijkstra's algorithm starting at the first vertex, until the second vertex is found. When encountering a None edge weight, uses the 
    /// output of the none_response closure to decide what weight should be used. This closure is provided with the indices of the two vertices of the edge, in order. 
    /// If a none is returned, the edge is ignored, otherwise the provided value inside the some will be used as the weight.
    /// 
    /// Returns a vector with the indices of the found path, alongside the edge weight of the edge after that one, with the last edge weight being 0
    pub fn dijkstra_search(&self, index1: usize, index2: usize, none_edge: impl Fn(usize, usize) -> f32) -> Result<Vec<(usize, f32)>, GraphError> {
        todo!()
    }

    /// Perform a depth first search starting at the first vertex, until the second vertex is found, treating all edge weights as one
    /// 
    /// Returns a vector with the indices of the found path
    pub fn depth_first_search(&self, index1: usize, index2: usize) -> Result<Vec<usize>, GraphError> {
        todo!()
    }

    /// Perform a breadth first search starting at the first vertex, until the second vertex is found, treating all edge weights as one
    /// 
    /// Returns a vector with the indices of the found path
    pub fn breadth_first_search(&self, index1: usize, index2: usize) -> Result<Vec<usize>, GraphError> {
        todo!()
    }

    /// Perform the A* search algorithm, using the provided closure to determine a heuristic value for each vertex. The closure takes in the heuristic of 
    /// any vertex, alongside that of the final vertex and outputs a non-negative heuristic value
    pub fn a_star_search(
        &self, index1: usize, 
        index2: usize, 
        none_edge: impl Fn(usize, usize) -> f32,
        determine_heuristic: impl Fn(Option<Vec3>, Option<Vec3>) -> f32
    ) -> Result<Vec<(usize, f32)>, GraphError> {
        todo!()
    }
}
