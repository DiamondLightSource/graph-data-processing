/// Collection of graphql entities
mod entities;
use crate::S3Bucket;
use async_graphql::{
    ComplexObject, Context, EmptyMutation, EmptySubscription, Object, Schema, SchemaBuilder,
};
use aws_sdk_s3::presigning::PresigningConfig;
use entities::{
    AutoProcIntegration, DataCollection, DataProcessing, ProcessingJob, ProcessingJobParameter,
    AutoProc, AutoProcScaling,
};
use models::{
    auto_proc_integration, auto_proc_program, data_collection_file_attachment, processing_job,
    processing_job_parameter, auto_proc, auto_proc_scaling,
};
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};
use std::time::Duration;
use url::Url;

use self::entities::AutoProcProgram;

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

    /// Fetched all the processing jobs
    async fn processing_jobs(
        &self,
        ctx: &Context<'_>,
    ) -> async_graphql::Result<Vec<ProcessingJob>, async_graphql::Error> {
        let database = ctx.data::<DatabaseConnection>()?;
        Ok(processing_job::Entity::find()
            .filter(processing_job::Column::DataCollectionId.eq(self.id))
            .all(database)
            .await?
            .into_iter()
            .map(ProcessingJob::from)
            .collect())
    }
}

#[ComplexObject]
impl DataProcessing {
    /// Gives downloadable link for the processed image in the s3 bucket
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

#[ComplexObject]
impl ProcessingJob {
    async fn parameters(
        &self,
        ctx: &Context<'_>,
    ) -> async_graphql::Result<Vec<ProcessingJobParameter>> {
        let database = ctx.data::<DatabaseConnection>()?;
        Ok(processing_job_parameter::Entity::find()
            .filter(processing_job_parameter::Column::ProcessingJobId.eq(self.processing_job_id))
            .all(database)
            .await?
            .into_iter()
            .map(ProcessingJobParameter::from)
            .collect())
    }
}

#[ComplexObject]
impl AutoProcIntegration {
    async fn auto_proc_program(
        &self,
        ctx: &Context<'_>,
    ) -> async_graphql::Result<Vec<AutoProcProgram>> {
        let database = ctx.data::<DatabaseConnection>()?;
        Ok(auto_proc_program::Entity::find()
            .filter(auto_proc_program::Column::AutoProcProgramId.eq(self.auto_proc_program_id))
            .all(database)
            .await?
            .into_iter()
            .map(AutoProcProgram::from)
            .collect())
    }
}

#[ComplexObject]
impl AutoProcProgram {
    async fn auto_proc(&self, ctx: &Context<'_>,) -> async_graphql::Result<Option<AutoProc>> {
        let database = ctx.data::<DatabaseConnection>()?;
        Ok(auto_proc::Entity::find()
            .filter(auto_proc::Column::AutoProcProgramId.eq(self.auto_proc_program_id))
            .one(database)
            .await?
            .map(AutoProc::from)
            )
    }
}

#[Object]
impl Query {
    /// Reference datasets resolver for the router
    #[graphql(entity)]
    async fn router_data_collection(&self, id: u32) -> DataCollection {
        DataCollection { id }
    }

    async fn auto_proc_integration(
        &self,
        ctx: &Context<'_>,
        data_collection_id: u32,
    ) -> async_graphql::Result<Vec<AutoProcIntegration>, async_graphql::Error> {
        let database = ctx.data::<DatabaseConnection>()?;
        Ok(auto_proc_integration::Entity::find()
            .filter(auto_proc_integration::Column::DataCollectionId.eq(data_collection_id))
            .all(database)
            .await?
            .into_iter()
            .map(AutoProcIntegration::from)
            .collect())
    }
}
