use async_graphql::{
    EmptyMutation, EmptySubscription, Object, Schema, SchemaBuilder,
};

/// The GraphQL schema exposed by the service
pub type RootSchema = Schema<Query, EmptyMutation, EmptySubscription>;

/// A schema builder for the service
pub fn root_schema_builder() -> SchemaBuilder<Query, EmptyMutation, EmptySubscription> {
    Schema::build(Query, EmptyMutation, EmptySubscription).enable_federation()
}

/// The root query of the service
#[derive(Debug, Clone, Default)]
pub struct Query;

#[Object]
impl Query {
    async fn query(&self) -> &str {
        ""
    }
}
