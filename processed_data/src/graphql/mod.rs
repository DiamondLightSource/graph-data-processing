/// Collection of graphql entities
mod entities;
use crate::S3Bucket;
use async_graphql::{
    dataloader::{DataLoader, Loader},
    ComplexObject, Context, EmptyMutation, EmptySubscription, Object, Schema, SchemaBuilder,
};
use aws_sdk_s3::presigning::PresigningConfig;
use entities::{
    AutoProc, AutoProcIntegration, AutoProcScaling, AutoProcScalingStatics, DataCollection,
    DataProcessing, ProcessingJob, ProcessingJobParameter,
};
use models::{
    auto_proc, auto_proc_integration, auto_proc_program, auto_proc_scaling,
    auto_proc_scaling_statistics, data_collection_file_attachment, processing_job,
    processing_job_parameter,
};
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};
use std::collections::HashMap;
use std::time::Duration;
use tracing::instrument;
use url::Url;

use self::entities::AutoProcProgram;

/// The GraphQL schema exposed by the service
pub type RootSchema = Schema<Query, EmptyMutation, EmptySubscription>;

/// A schema builder for the service
pub fn root_schema_builder(
    database: DatabaseConnection,
) -> SchemaBuilder<Query, EmptyMutation, EmptySubscription> {
    Schema::build(Query, EmptyMutation, EmptySubscription)
        .data(DataLoader::new(
            DataCollectionLoader::new(database.clone()),
            tokio::spawn,
        ))
        .data(database)
        .enable_federation()
}

/// The root query of the service
#[derive(Debug, Clone, Default)]
pub struct Query;

pub struct DataCollectionLoader(DatabaseConnection);

impl DataCollectionLoader {
    fn new(database: DatabaseConnection) -> Self {
        Self(database)
    }
}

impl Loader<u32> for DataCollectionLoader {
    type Value = DataProcessing;
    type Error = async_graphql::Error;

    async fn load(&self, keys: &[u32]) -> Result<HashMap<u32, Self::Value>, Self::Error> {
        let mut results = HashMap::new();
        let keys_vec: Vec<u32> = keys.iter().cloned().collect();
        let records = data_collection_file_attachment::Entity::find()
            .filter(data_collection_file_attachment::Column::DataCollectionId.is_in(keys_vec))
            .all(&self.0)
            .await?;

        for record in records {
            let data_collection_id = record.data_collection_id;
            let data = DataProcessing::from(record);

            results.insert(data_collection_id, data);
        }

        Ok(results)
    }
}

#[ComplexObject]
impl DataCollection {
    /// Fetched all the processed data from data collection during a session
    async fn processed_data(
        &self,
        ctx: &Context<'_>,
    ) -> Result<Option<DataProcessing>, async_graphql::Error> {
        let loader = ctx.data_unchecked::<DataLoader<DataCollectionLoader>>();
        Ok(loader.load_one(self.id).await?)
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

    /// Fetches all the automatic process
    async fn auto_proc_integration(
        &self,
        ctx: &Context<'_>,
    ) -> async_graphql::Result<Vec<AutoProcIntegration>, async_graphql::Error> {
        let database = ctx.data::<DatabaseConnection>()?;
        Ok(auto_proc_integration::Entity::find()
            .filter(auto_proc_integration::Column::DataCollectionId.eq(self.id))
            .all(database)
            .await?
            .into_iter()
            .map(AutoProcIntegration::from)
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
    /// Fetches the processing job parameters
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
    /// Fetches the automatically processed programs
    async fn auto_proc_program(
        &self,
        ctx: &Context<'_>,
    ) -> async_graphql::Result<Option<AutoProcProgram>> {
        let database = ctx.data::<DatabaseConnection>()?;
        Ok(auto_proc_program::Entity::find()
            .filter(auto_proc_program::Column::AutoProcProgramId.eq(self.auto_proc_program_id))
            .one(database)
            .await?
            .map(AutoProcProgram::from))
    }
}

#[ComplexObject]
impl AutoProcProgram {
    /// Fetched the automatic process
    async fn auto_proc(&self, ctx: &Context<'_>) -> async_graphql::Result<Option<AutoProc>> {
        let database = ctx.data::<DatabaseConnection>()?;
        Ok(auto_proc::Entity::find()
            .filter(auto_proc::Column::AutoProcProgramId.eq(self.auto_proc_program_id))
            .one(database)
            .await?
            .map(AutoProc::from))
    }
}

#[ComplexObject]
impl AutoProc {
    /// Fetches the scaling for automatic process
    async fn scaling(&self, ctx: &Context<'_>) -> async_graphql::Result<Option<AutoProcScaling>> {
        let database = ctx.data::<DatabaseConnection>()?;
        Ok(auto_proc_scaling::Entity::find()
            .filter(auto_proc_scaling::Column::AutoProcId.eq(self.auto_proc_id))
            .one(database)
            .await?
            .map(AutoProcScaling::from))
    }
}

#[ComplexObject]
impl AutoProcScaling {
    /// Fetches the scaling statistics
    async fn statistics(
        &self,
        ctx: &Context<'_>,
    ) -> async_graphql::Result<Option<AutoProcScalingStatics>> {
        let database = ctx.data::<DatabaseConnection>()?;
        Ok(auto_proc_scaling_statistics::Entity::find()
            .filter(
                auto_proc_scaling_statistics::Column::AutoProcScalingId
                    .eq(self.auto_proc_scaling_id),
            )
            .one(database)
            .await?
            .map(AutoProcScalingStatics::from))
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
