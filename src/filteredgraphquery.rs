use std::marker::PhantomData;

use bevy::ecs::{query::{WorldQuery, ReadOnlyWorldQuery, QueryComponentError}, component::Component, entity::Entity, system::Query};

use super::GraphVertex;


//todo: add a base level GraphQuery, which contains an actual query and no filter (maybe a heuristic?)
//impl filter graph for it


pub enum FilterGraphError{
    ComponentNotFound,
    ComponentNotAccessible,
    InvalidEntity
}

impl From<QueryComponentError> for FilterGraphError{
    fn from(value: QueryComponentError) -> Self {
        match value{
            QueryComponentError::MissingReadAccess => Self::ComponentNotAccessible,
            QueryComponentError::MissingWriteAccess => Self::ComponentNotAccessible,
            QueryComponentError::MissingComponent => Self::ComponentNotFound,
            QueryComponentError::NoSuchEntity => Self::InvalidEntity,
        }
    }
}


// ===============
// Turn a Fn(&F) -> bool or Fn(Option<&F>) -> bool into something we can store
// ===============


pub trait FilterComponent<'a, F: Component> : Sized{
    fn from_option(f: Option<&'a F>) -> Result<Self, FilterGraphError>;
}

impl<'a:'b, 'b, F: Component> FilterComponent<'a, F> for &'b F 
{
    fn from_option(f: Option<&'a F>) -> Result<Self, FilterGraphError> {
        f.ok_or(FilterGraphError::ComponentNotFound)
    }
}
impl<'a:'b, 'b, F: Component> FilterComponent<'a, F> for Option<&'b F> {
    fn from_option(f: Option<&'a F>) -> Result<Self, FilterGraphError> {
        Ok(f)
    }
}


pub trait IntoGraphFilter<S:WorldQuery, T: ReadOnlyWorldQuery, F: Component, G: for<'a> FilterComponent<'a, F>>{
    type Output: GraphFilter<S, T> + 'static;
    fn into_filter(self) -> Self::Output;
}

pub trait GraphFilter<S:WorldQuery, T: ReadOnlyWorldQuery>{
    fn apply(&self, query: &Query<S,T>, ent: Entity) -> Result<bool, FilterGraphError>;
}

pub struct FunctionFilter<S, T, F, G, Func>
where 
    S: WorldQuery, 
    T: ReadOnlyWorldQuery,
    F: Component,
    G: for<'a> FilterComponent<'a, F>,
    Func: Fn(G) -> bool
{
    function: Func,
    _phantom: PhantomData<(S, T, F, G)>,
}


impl<S, T, F, G, Func> GraphFilter<S, T> for FunctionFilter<S, T, F, G, Func>
where 
    S: WorldQuery, 
    T: ReadOnlyWorldQuery,
    F: Component,
    G: for<'a> FilterComponent<'a, F>,
    Func: Fn(G) -> bool
{
    fn apply(&self, query: &Query<S,T>, ent: Entity) -> Result<bool, FilterGraphError> {
        //get the inner F component
        //since missing components could be allowed depending on G, we keep that as a None
        let data = match query.get_component::<F>(ent){
            Ok(val) => Some(val),
            Err(QueryComponentError::MissingComponent) => None,
            Err(err) => Err(err)?
        };
        //convert our value from an Option<&F> to a G then run our function on it
        Ok((self.function)(G::from_option(data)?))
    }
}


impl<S, T, F, G, Func> IntoGraphFilter<S, T, F, G> for Func
where 
    S: WorldQuery + 'static, 
    T: ReadOnlyWorldQuery + 'static,
    F: Component + 'static,
    G: for<'a> FilterComponent<'a, F> + 'static,
    Func: Fn(G) -> bool + 'static,
{
    type Output = FunctionFilter<S, T, F, G, Func>;

    fn into_filter(self) -> Self::Output {
        FunctionFilter{
            function: self,
            _phantom: PhantomData
        }
    }
}



// ===============
// The main filtering + graph function mechanism
// ===============

pub struct FilteredGraphQuery<'q, 'w, 's, S, T, V>
where
    S: WorldQuery,
    T: ReadOnlyWorldQuery,
    V: GraphVertex
{
    query: &'q Query<'w, 's, S, T>,
    filters: Vec<Box<dyn GraphFilter<S, T>>>,
    _phantom: PhantomData<V>
}


impl<'q, 'w, 's, S, T, V> FilteredGraphQuery<'q, 'w, 's, S, T, V>
where
    S: WorldQuery,
    T: ReadOnlyWorldQuery,
    V: GraphVertex
{
    ///Create a new Filter graph from the provided query.
    /// 
    /// A FilteredGraphQuery can be used to add dynamic filters* to a query, which can then be used as the set of vertices for
    /// various graph search algorithms
    /// 
    /// *Filters currently only support filtering on only one component each
    /// 
    /// TODO
    pub fn new(query: &'q Query<'w, 's, S, T>) -> Self {
        Self {query, filters: Vec::new(), _phantom: PhantomData}
    }

    pub fn add_filter<F: Component, G: for<'a> FilterComponent<'a, F>>(&mut self, filter: impl IntoGraphFilter<S, T, F, G> + 'static) -> &mut Self 
    {
        self.filters.push(Box::new(filter.into_filter()));
        self
    }

    pub fn get_vertex(&self, ent: Entity) -> Result<&V, FilterGraphError> {
        Ok(self.query.get_component::<V>(ent)?)
    }

    pub fn get_vertex_filtered(&self, ent: Entity) -> Result<Option<&V>, FilterGraphError> {
        for filter in self.filters.iter() {
            if !filter.apply(self.query, ent)? {
                return Ok(None);
            }
        }
        Ok(Some(self.get_vertex(ent)?))
    }




}