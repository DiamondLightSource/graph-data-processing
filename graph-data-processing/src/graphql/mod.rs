/// Collection of graphql entities
mod entities;
use crate::S3Bucket;
use async_graphql::{
    ComplexObject, Context, EmptyMutation, EmptySubscription, Object, Schema, SchemaBuilder,
};
use aws_sdk_s3::presigning::PresigningConfig;
use entities::{DataCollection, DataProcessing};
use models::data_collection_file_attachment;
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};
use std::time::Duration;
use url::Url;

/// The GraphQL schema exposed by the service
pub type RootSchema = Schema<Query, EmptyMutation, EmptySubscription>;

/// A schema builder for the service
pub fn root_schema_builder() -> SchemaBuilder<Query, EmptyMutation, EmptySubscription> {
    Schema::build(Query, EmptyMutation, EmptySubscription).enable_federation()
}

/// The root query of the service
#[derive(Debug, Clone, Default)]
pub struct Query;

#[ComplexObject]
impl DataCollection {
    /// Fetched all the processed data from data collection during a session
    async fn processed_data(
        &self,
        ctx: &Context<'_>,
    ) -> Result<Vec<DataProcessing>, async_graphql::Error> {
        let database = ctx.data::<DatabaseConnection>()?;
        Ok(data_collection_file_attachment::Entity::find()
            .filter(data_collection_file_attachment::Column::DataCollectionId.eq(self.id))
            .all(database)
            .await?
            .into_iter()
            .map(DataProcessing::from)
            .collect())
    }
}

#[ComplexObject]
impl DataProcessing {
    async fn download_url(&self, ctx: &Context<'_>) -> async_graphql::Result<String> {
        let s3_client = ctx.data::<aws_sdk_s3::Client>()?;
        let bucket = ctx.data::<S3Bucket>()?;
        let object_uri = s3_client
            .get_object()
            .bucket(bucket.clone())
            .key(self.object_key())
            .presigned(PresigningConfig::expires_in(Duration::from_secs(10 * 60))?)
            .await?
            .uri()
            .clone();
        let object_url = Url::parse(&object_uri.to_string())?;
        Ok(object_url.to_string())
    }
}

#[Object]
impl Query {
    /// Reference datasets resolver for the router
    #[graphql(entity)]
    async fn router_data_collection(&self, id: u32) -> DataCollection {
        DataCollection { id }
    }
}
