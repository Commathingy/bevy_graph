mod systems;
pub mod types;
pub use types::*;

// Overall Structure:
//  Graph Resource
//  Vertex Components, which contain a list of seen neighbours and seen by neighbours (outgoing/incoming edges)

//  A Graph Component stores:
//      A list of vertex tuples:
//          *Index (0, 1, 2, 3,..., n) 
//          *Component number (0, ..., m) (vertices connected by a path iff they have same number)
//          *A GraphVertexEntity struct contaning the entity id for which that vertex is attached
//          *It's heuristic
//      A list of edge tuples (these are directed), in a tuple with their weights (not allowing -ve probably)
//          *The vertices in the edge (this is an ordered pair, the edge goes from the first to the second vertex)
//              -Note that this will be by vertex index
//          *The edge weight, an optional f32/64, disallowing -ves
//      The number of


// System layout:
//      1) At startup, create empty graph resource
//      2) allow for a graph to be read from a file or similar (see hurdles below)
//      3) user cannot mutate the graph -> this should be done inside a system in this crate
//          3.5) i.e a system that listens for changed<GraphVertex> and updates the graph based on that, then updates the GraphVertex's themselves
//      4) Give some standard things to the graph resource impl:
//          -Add/remove/alter edge
//          -Add a vertex (note the hurdle on indices and vertices not assigned to components) - this should probably be the only way of creating a vertex component
//          -NOT remove a vertex - unless this forcibly removes the component from the entity
//          -Various searches (depth first, breadth first - with an option for an unweighted version and what to do in weighted case if encountering Nones)
//              -some way to add a heuristic for something like A*? (hurdle)
//          -is connected (literally jsut checks if two vertices have same commponent number)
//          -other various stuff as i think of it i guess




//hurdles:
//  should a vertex component contain its neighbours (and should it contain its seen by neighbours)
//      -> yes
//
//  what to do with a given index when a vertex is removed
// perhaps renumber all the vertices
//
//  when removing vertex, need to check and update the vertex components
//      -start a BFS from one of the two vertices in the edge, if reach other vertex, all good,
//          if dont reach, then every vertex found in the bfs is part of a new component
//  
//  what to do when a graph is constructed from given data 
//      -ie we give data but perhaps do not assign some vertices to components
//      -also we might have negative edge weights -> give some default value for this case (perhaps even allowing for user to be notified via error?)
//
//  when updating the graph and vertex components due to changes, 
//  watch out for the same edges being removed and added at the same time or being added/removed/both multiple times
//
//  two options for adding/removing vertices: directly with the graph resource or indirectly by deleting the component
//      -checking for deletion can be difficult in indirect method
//      -using direct method, need some way to not also run in the indirect checking system
//      -(note that indirect is slower ())
//
//  allow for some way to create a heuristic for A* or similar algorithms