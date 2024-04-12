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
    DataProcessing, ProcessJob, ProcessingJob, ProcessingJobParameter,
};
use models::{
    auto_proc, auto_proc_integration, auto_proc_program, auto_proc_scaling,
    auto_proc_scaling_statistics, data_collection_file_attachment, processing_job,
    processing_job_parameter,
};
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};
use std::collections::HashMap;
use std::time::Duration;
use tracing::{instrument, Span};
use url::Url;

use self::entities::AutoProcProgram;

/// The GraphQL schema exposed by the service
pub type RootSchema = Schema<Query, EmptyMutation, EmptySubscription>;

pub trait AddDataLoadersExt {
    fn add_data_loaders(self, database: DatabaseConnection) -> Self;
}

impl AddDataLoadersExt for async_graphql::Request {
    fn add_data_loaders(self, database: DatabaseConnection) -> Self {
        self.data(DataLoader::new(
            ProcessedDataLoader::new(database.clone()),
            tokio::spawn,
        ))
        .data(DataLoader::new(
            ProcessingJobDataLoader::new(database.clone()),
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
        .data(DataLoader::new(
            AutoProcDataLoader::new(database.clone()),
            tokio::spawn,
        ))
        .data(DataLoader::new(
            AutoProcScalingDataLoader::new(database.clone()),
            tokio::spawn,
        ))
        .data(DataLoader::new(
            AutoProcScalingOverall::new(database.clone()),
            tokio::spawn,
        ))
        .data(DataLoader::new(
            AutoProcScalingInnerShell::new(database.clone()),
            tokio::spawn,
        ))
        .data(DataLoader::new(
            AutoProcScalingOuterShell::new(database.clone()),
            tokio::spawn,
        ))
        .data(database)
    }
}

/// A schema builder for the service
pub fn root_schema_builder() -> SchemaBuilder<Query, EmptyMutation, EmptySubscription> {
    Schema::build(Query, EmptyMutation, EmptySubscription).enable_federation()
}

/// The root query of the service
#[derive(Debug, Clone, Default)]
pub struct Query;

pub struct ProcessedDataLoader {
    database: DatabaseConnection,
    parent_span: Span,
}
pub struct ProcessingJobDataLoader {
    database: DatabaseConnection,
    parent_span: Span,
}
pub struct AutoProcIntegrationDataLoader {
    database: DatabaseConnection,
    parent_span: Span,
}
pub struct AutoProcProgramDataLoader {
    database: DatabaseConnection,
    parent_span: Span,
}
pub struct AutoProcDataLoader {
    database: DatabaseConnection,
    parent_span: Span,
}
pub struct AutoProcScalingDataLoader {
    database: DatabaseConnection,
    parent_span: Span,
}
pub struct AutoProcScalingOverall {
    database: DatabaseConnection,
    parent_span: Span,
}
pub struct AutoProcScalingInnerShell {
    database: DatabaseConnection,
    parent_span: Span,
}
pub struct AutoProcScalingOuterShell {
    database: DatabaseConnection,
    parent_span: Span,
}

impl ProcessingJobDataLoader {
    fn new(database: DatabaseConnection) -> Self {
        Self {
            database,
            parent_span: Span::current(),
        }
    }
}

impl ProcessedDataLoader {
    fn new(database: DatabaseConnection) -> Self {
        Self {
            database,
            parent_span: Span::current(),
        }
    }
}

impl AutoProcIntegrationDataLoader {
    fn new(database: DatabaseConnection) -> Self {
        Self {
            database,
            parent_span: Span::current(),
        }
    }
}

impl AutoProcProgramDataLoader {
    fn new(database: DatabaseConnection) -> Self {
        Self {
            database,
            parent_span: Span::current(),
        }
    }
}

impl AutoProcDataLoader {
    fn new(database: DatabaseConnection) -> Self {
        Self {
            database,
            parent_span: Span::current(),
        }
    }
}

impl AutoProcScalingDataLoader {
    fn new(database: DatabaseConnection) -> Self {
        Self {
            database,
            parent_span: Span::current(),
        }
    }
}

impl AutoProcScalingOverall {
    fn new(database: DatabaseConnection) -> Self {
        Self {
            database,
            parent_span: Span::current(),
        }
    }
}

impl AutoProcScalingInnerShell {
    fn new(database: DatabaseConnection) -> Self {
        Self {
            database,
            parent_span: Span::current(),
        }
    }
}

impl AutoProcScalingOuterShell {
    fn new(database: DatabaseConnection) -> Self {
        Self {
            database,
            parent_span: Span::current(),
        }
    }
}

impl Loader<u32> for ProcessedDataLoader {
    type Value = DataProcessing;
    type Error = async_graphql::Error;

    async fn load(&self, keys: &[u32]) -> Result<HashMap<u32, Self::Value>, Self::Error> {
        let span = tracing::info_span!(parent: &self.parent_span, "load_processed_data");
        let _span = span.enter();
        let mut results = HashMap::new();
        let keys_vec: Vec<u32> = keys.to_vec();
        let records = data_collection_file_attachment::Entity::find()
            .filter(data_collection_file_attachment::Column::DataCollectionId.is_in(keys_vec))
            .all(&self.database)
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
    type Value = Vec<ProcessJob>;
    type Error = async_graphql::Error;

    #[instrument(name = "load_processing_job", skip(self))]
    async fn load(&self, keys: &[u32]) -> Result<HashMap<u32, Self::Value>, Self::Error> {
        let span = tracing::info_span!(parent: &self.parent_span, "load_processed_data");
        let _span = span.enter();
        let mut results = HashMap::new();
        let keys_vec: Vec<u32> = keys.to_vec();
        let records = processing_job::Entity::find()
            .find_also_related(processing_job_parameter::Entity)
            .filter(processing_job::Column::DataCollectionId.is_in(keys_vec))
            .all(&self.database)
            .await?
            .into_iter()
            .map(|(job, parameter)| ProcessJob {
                processing_job: ProcessingJob::from(job),
                parameters: parameter.map(ProcessingJobParameter::from),
            })
            .collect::<Vec<_>>();

        for record in records {
            let data_collection_id = record.processing_job.data_collection_id.unwrap();
            results
                .entry(data_collection_id)
                .or_insert_with(Vec::new)
                .push(record)
        }
        Ok(results)
    }
}

impl Loader<u32> for AutoProcIntegrationDataLoader {
    type Value = Vec<AutoProcIntegration>;
    type Error = async_graphql::Error;

    #[instrument(name = "load_auto_proc_integration", skip(self))]
    async fn load(&self, keys: &[u32]) -> Result<HashMap<u32, Self::Value>, Self::Error> {
        let span = tracing::info_span!(parent: &self.parent_span, "load_processed_data");
        let _span = span.enter();
        let mut results = HashMap::new();
        let keys_vec: Vec<u32> = keys.to_vec();
        let records = auto_proc_integration::Entity::find()
            .filter(auto_proc_integration::Column::DataCollectionId.is_in(keys_vec))
            .all(&self.database)
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
        let span = tracing::info_span!(parent: &self.parent_span, "load_processed_data");
        let _span = span.enter();
        let mut results = HashMap::new();
        let keys_vec: Vec<u32> = keys.to_vec();
        let records = auto_proc_program::Entity::find()
            .filter(auto_proc_program::Column::AutoProcProgramId.is_in(keys_vec))
            .all(&self.database)
            .await?;

        for record in records {
            let program_id = record.auto_proc_program_id;
            let data = AutoProcProgram::from(record);
            results.insert(program_id, data);
        }

        Ok(results)
    }
}

impl Loader<u32> for AutoProcDataLoader {
    type Value = AutoProc;
    type Error = async_graphql::Error;

    #[instrument(name = "load_auto_proc", skip(self))]
    async fn load(&self, keys: &[u32]) -> Result<HashMap<u32, Self::Value>, Self::Error> {
        let span = tracing::info_span!(parent: &self.parent_span, "load_processed_data");
        let _span = span.enter();
        let mut results = HashMap::new();
        let keys_vec: Vec<u32> = keys.to_vec();
        let records = auto_proc::Entity::find()
            .filter(auto_proc::Column::AutoProcProgramId.is_in(keys_vec))
            .all(&self.database)
            .await?;

        for record in records {
            let program_id = record.auto_proc_program_id.unwrap();
            let data = AutoProc::from(record);
            results.insert(program_id, data);
        }

        Ok(results)
    }
}

impl Loader<u32> for AutoProcScalingDataLoader {
    type Value = AutoProcScaling;
    type Error = async_graphql::Error;

    #[instrument(name = "load_auto_proc_scaling", skip(self))]
    async fn load(&self, keys: &[u32]) -> Result<HashMap<u32, Self::Value>, Self::Error> {
        let span = tracing::info_span!(parent: &self.parent_span, "load_processed_data");
        let _span = span.enter();
        let mut results = HashMap::new();
        let keys_vec: Vec<u32> = keys.to_vec();
        let records = auto_proc_scaling::Entity::find()
            .filter(auto_proc_scaling::Column::AutoProcId.is_in(keys_vec))
            .all(&self.database)
            .await?;

        for record in records {
            let auto_proc_id = record.auto_proc_id.unwrap();
            let data = AutoProcScaling::from(record);
            results.insert(auto_proc_id, data);
        }

        Ok(results)
    }
}

impl Loader<u32> for AutoProcScalingOverall {
    type Value = AutoProcScalingStatics;
    type Error = async_graphql::Error;

    #[instrument(name = "load_auto_proc_scaling_statics", skip(self))]
    async fn load(&self, keys: &[u32]) -> Result<HashMap<u32, Self::Value>, Self::Error> {
        let span = tracing::info_span!(parent: &self.parent_span, "load_processed_data");
        let _span = span.enter();
        let mut results = HashMap::new();
        let keys_vec: Vec<u32> = keys.to_vec();
        let records = auto_proc_scaling_statistics::Entity::find()
            .filter(auto_proc_scaling_statistics::Column::AutoProcScalingId.is_in(keys_vec))
            .filter(auto_proc_scaling_statistics::Column::ScalingStatisticsType.eq("overall"))
            .all(&self.database)
            .await?;

        for record in records {
            let auto_proc_scaling_id = record.auto_proc_scaling_id.unwrap();
            let data = AutoProcScalingStatics::from(record);
            results.insert(auto_proc_scaling_id, data);
        }

        Ok(results)
    }
}

impl Loader<u32> for AutoProcScalingInnerShell {
    type Value = AutoProcScalingStatics;
    type Error = async_graphql::Error;

    #[instrument(name = "load_auto_proc_scaling_statics", skip(self))]
    async fn load(&self, keys: &[u32]) -> Result<HashMap<u32, Self::Value>, Self::Error> {
        let span = tracing::info_span!(parent: &self.parent_span, "load_processed_data");
        let _span = span.enter();
        let mut results = HashMap::new();
        let keys_vec: Vec<u32> = keys.to_vec();
        let records = auto_proc_scaling_statistics::Entity::find()
            .filter(auto_proc_scaling_statistics::Column::AutoProcScalingId.is_in(keys_vec))
            .filter(auto_proc_scaling_statistics::Column::ScalingStatisticsType.eq("innerShell"))
            .all(&self.database)
            .await?;

        for record in records {
            let auto_proc_scaling_id = record.auto_proc_scaling_id.unwrap();
            let data = AutoProcScalingStatics::from(record);
            results.insert(auto_proc_scaling_id, data);
        }

        Ok(results)
    }
}

impl Loader<u32> for AutoProcScalingOuterShell {
    type Value = AutoProcScalingStatics;
    type Error = async_graphql::Error;

    #[instrument(name = "load_auto_proc_scaling_statics", skip(self))]
    async fn load(&self, keys: &[u32]) -> Result<HashMap<u32, Self::Value>, Self::Error> {
        let span = tracing::info_span!(parent: &self.parent_span, "load_processed_data");
        let _span = span.enter();
        let mut results = HashMap::new();
        let keys_vec: Vec<u32> = keys.to_vec();
        let records = auto_proc_scaling_statistics::Entity::find()
            .filter(auto_proc_scaling_statistics::Column::AutoProcScalingId.is_in(keys_vec))
            .filter(auto_proc_scaling_statistics::Column::ScalingStatisticsType.eq("outerShell"))
            .all(&self.database)
            .await?;

        for record in records {
            let auto_proc_scaling_id = record.auto_proc_scaling_id.unwrap();
            let data = AutoProcScalingStatics::from(record);
            results.insert(auto_proc_scaling_id, data);
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
        loader.load_one(self.id).await
    }

    /// Fetched all the processing jobs
    async fn processing_jobs(
        &self,
        ctx: &Context<'_>,
    ) -> async_graphql::Result<Option<Vec<ProcessJob>>, async_graphql::Error> {
        let loader = ctx.data_unchecked::<DataLoader<ProcessingJobDataLoader>>();
        loader.load_one(self.id).await
    }

    /// Fetches all the automatic process
    async fn auto_proc_integration(
        &self,
        ctx: &Context<'_>,
    ) -> async_graphql::Result<Option<Vec<AutoProcIntegration>>, async_graphql::Error> {
        let loader = ctx.data_unchecked::<DataLoader<AutoProcIntegrationDataLoader>>();
        loader.load_one(self.id).await
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
impl AutoProcIntegration {
    /// Fetches the automatically processed programs
    async fn auto_proc_program(
        &self,
        ctx: &Context<'_>,
    ) -> async_graphql::Result<Option<AutoProcProgram>> {
        let loader = ctx.data_unchecked::<DataLoader<AutoProcProgramDataLoader>>();
        loader.load_one(self.auto_proc_program_id.unwrap()).await
    }
}

#[ComplexObject]
impl AutoProcProgram {
    /// Fetched the automatic process
    async fn auto_proc(&self, ctx: &Context<'_>) -> async_graphql::Result<Option<AutoProc>> {
        let loader = ctx.data_unchecked::<DataLoader<AutoProcDataLoader>>();
        loader.load_one(self.auto_proc_program_id).await
    }
}

#[ComplexObject]
impl AutoProc {
    /// Fetches the scaling for automatic process
    async fn scaling(&self, ctx: &Context<'_>) -> async_graphql::Result<Option<AutoProcScaling>> {
        let loader = ctx.data_unchecked::<DataLoader<AutoProcScalingDataLoader>>();
        loader.load_one(self.auto_proc_id).await
    }
}

#[ComplexObject]
impl AutoProcScaling {
    /// Fetches the scaling statistics
    async fn overall(
        &self,
        ctx: &Context<'_>,
    ) -> async_graphql::Result<Option<AutoProcScalingStatics>> {
        let loader = ctx.data_unchecked::<DataLoader<AutoProcScalingOverall>>();
        loader.load_one(self.auto_proc_scaling_id).await
    }

    async fn inner_shell(
        &self,
        ctx: &Context<'_>,
    ) -> async_graphql::Result<Option<AutoProcScalingStatics>> {
        let loader = ctx.data_unchecked::<DataLoader<AutoProcScalingInnerShell>>();
        loader.load_one(self.auto_proc_scaling_id).await
    }

    async fn outer_shell(
        &self,
        ctx: &Context<'_>,
    ) -> async_graphql::Result<Option<AutoProcScalingStatics>> {
        let loader = ctx.data_unchecked::<DataLoader<AutoProcScalingOuterShell>>();
        loader.load_one(self.auto_proc_scaling_id).await
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
