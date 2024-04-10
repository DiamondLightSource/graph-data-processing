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
            ProcessedDataLoader::new(database.clone()),
            tokio::spawn,
        ))
        .data(DataLoader::new(
            ProcessingJobDataLoader::new(database.clone()),
            tokio::spawn,
        ))
        .data(DataLoader::new(
            ProcessingJobParameterDataLoader::new(database.clone()),
            tokio::spawn,
        ))
        .data(DataLoader::new(
            AutoProcIntegrationDataLoader::new(database.clone()),
            tokio::spawn,
        ))
        .data(DataLoader::new(
            AutoProcProgramDataLoader::new(database.clone()),
            tokio::spawn,
        ))
        .data(database)
        .enable_federation()
}

/// The root query of the service
#[derive(Debug, Clone, Default)]
pub struct Query;

pub struct ProcessedDataLoader(DatabaseConnection);
pub struct ProcessingJobDataLoader(DatabaseConnection);
pub struct ProcessingJobParameterDataLoader(DatabaseConnection);
pub struct AutoProcIntegrationDataLoader(DatabaseConnection);
pub struct AutoProcProgramDataLoader(DatabaseConnection);

impl ProcessingJobDataLoader {
    fn new(database: DatabaseConnection) -> Self {
        Self(database)
    }
}

impl ProcessedDataLoader {
    fn new(database: DatabaseConnection) -> Self {
        Self(database)
    }
}

impl ProcessingJobParameterDataLoader {
    fn new(database: DatabaseConnection) -> Self {
        Self(database)
    }
}

impl AutoProcIntegrationDataLoader {
    fn new(database: DatabaseConnection) -> Self {
        Self(database)
    }
}

impl AutoProcProgramDataLoader {
    fn new(database: DatabaseConnection) -> Self {
        Self(database)
    }
}

impl Loader<u32> for ProcessedDataLoader {
    type Value = DataProcessing;
    type Error = async_graphql::Error;

    #[instrument(name = "load_processed_data", skip(self))]
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

impl Loader<u32> for ProcessingJobDataLoader {
    type Value = Vec<ProcessingJob>;
    type Error = async_graphql::Error;

    #[instrument(name = "load_processing_job", skip(self))]
    async fn load(&self, keys: &[u32]) -> Result<HashMap<u32, Self::Value>, Self::Error> {
        let mut results = HashMap::new();
        let keys_vec: Vec<u32> = keys.iter().cloned().collect();
        let records = processing_job::Entity::find()
            .filter(processing_job::Column::DataCollectionId.is_in(keys_vec))
            .all(&self.0)
            .await?;

        for record in records {
            let data_collection_id = record.data_collection_id.unwrap();
            let data = ProcessingJob::from(record);

            results
                .entry(data_collection_id)
                .or_insert_with(Vec::new)
                .push(data)
        }
        Ok(results)
    }
}

impl Loader<u32> for ProcessingJobParameterDataLoader {
    type Value = Vec<ProcessingJobParameter>;
    type Error = async_graphql::Error;

    #[instrument(name = "load_processing_job_parameter", skip(self))]
    async fn load(&self, keys: &[u32]) -> Result<HashMap<u32, Self::Value>, Self::Error> {
        let mut results = HashMap::new();
        let keys_vec: Vec<u32> = keys.iter().cloned().collect();
        let records = processing_job_parameter::Entity::find()
            .filter(processing_job_parameter::Column::ProcessingJobId.is_in(keys_vec))
            .all(&self.0)
            .await?;

        for record in records {
            let processing_job_id = record.processing_job_id.unwrap();
            let data = ProcessingJobParameter::from(record);
            results
                .entry(processing_job_id)
                .or_insert_with(Vec::new)
                .push(data)
        }

        Ok(results)
    }
}

impl Loader<u32> for AutoProcIntegrationDataLoader {
    type Value = Vec<AutoProcIntegration>;
    type Error = async_graphql::Error;

    #[instrument(name = "load_auto_proc_integration", skip(self))]
    async fn load(&self, keys: &[u32]) -> Result<HashMap<u32, Self::Value>, Self::Error> {
        let mut results = HashMap::new();
        let keys_vec: Vec<u32> = keys.iter().cloned().collect();
        let records = auto_proc_integration::Entity::find()
            .filter(auto_proc_integration::Column::DataCollectionId.is_in(keys_vec))
            .all(&self.0)
            .await?;

        for record in records {
            let data_collection_id = record.data_collection_id;
            let data = AutoProcIntegration::from(record);
            results
                .entry(data_collection_id)
                .or_insert_with(Vec::new)
                .push(data)
        }

        Ok(results)
    }
}

impl Loader<u32> for AutoProcProgramDataLoader {
    type Value = AutoProcProgram;
    type Error = async_graphql::Error;

    #[instrument(name = "load_auto_proc_program", skip(self))]
    async fn load(&self, keys: &[u32]) -> Result<HashMap<u32, Self::Value>, Self::Error> {
        let mut results = HashMap::new();
        let keys_vec: Vec<u32> = keys.iter().cloned().collect();
        let records = auto_proc_program::Entity::find()
            .filter(auto_proc_program::Column::AutoProcProgramId.is_in(keys_vec))
            .all(&self.0)
            .await?;

        for record in records {
            let program_id = record.auto_proc_program_id;
            let data = AutoProcProgram::from(record);
            results.insert(program_id, data);
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
        let loader = ctx.data_unchecked::<DataLoader<ProcessedDataLoader>>();
        Ok(loader.load_one(self.id).await?)
    }

    /// Fetched all the processing jobs
    async fn processing_jobs(
        &self,
        ctx: &Context<'_>,
    ) -> async_graphql::Result<Option<Vec<ProcessingJob>>, async_graphql::Error> {
        let loader = ctx.data_unchecked::<DataLoader<ProcessingJobDataLoader>>();
        Ok(loader.load_one(self.id).await?)
    }

    /// Fetches all the automatic process
    async fn auto_proc_integration(
        &self,
        ctx: &Context<'_>,
    ) -> async_graphql::Result<Option<Vec<AutoProcIntegration>>, async_graphql::Error> {
        let loader = ctx.data_unchecked::<DataLoader<AutoProcIntegrationDataLoader>>();
        Ok(loader.load_one(self.id).await?)
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
    ) -> async_graphql::Result<Option<Vec<ProcessingJobParameter>>> {
        let loader = ctx.data_unchecked::<DataLoader<ProcessingJobParameterDataLoader>>();
        Ok(loader.load_one(self.processing_job_id).await?)
    }
}

#[ComplexObject]
impl AutoProcIntegration {
    /// Fetches the automatically processed programs
    async fn auto_proc_program(
        &self,
        ctx: &Context<'_>,
    ) -> async_graphql::Result<Option<AutoProcProgram>> {
        let loader = ctx.data_unchecked::<DataLoader<AutoProcProgramDataLoader>>();
        Ok(loader.load_one(self.auto_proc_program_id.unwrap()).await?)
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
