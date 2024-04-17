/// Collection of graphql entities
mod entities;
use crate::S3Bucket;
use async_graphql::{
    dataloader::{DataLoader, Loader},
    ComplexObject, Context, EmptyMutation, EmptySubscription, Object, Schema, SchemaBuilder,
};
use aws_sdk_s3::presigning::PresigningConfig;
use entities::{
    AutoProcScalingStatics, AutoProcessing, DataCollection, DataProcessing, ProcessingJob,
    StatisticsType,
};
use models::{
    auto_proc, auto_proc_integration, auto_proc_program, auto_proc_scaling,
    auto_proc_scaling_statistics, data_collection_file_attachment, processing_job,
    processing_job_parameter,
};
use sea_orm::{
    ColumnTrait, ConnectionTrait, DatabaseConnection, EntityTrait, QueryFilter, Statement,
};
use sea_query::{self, Asterisk, Expr};
use std::time::Duration;
use std::{collections::HashMap, ops::Deref};
use tracing::{instrument, Span};
use url::Url;

/// The GraphQL schema exposed by the service
pub type RootSchema = Schema<Query, EmptyMutation, EmptySubscription>;

/// router handler extension
pub trait AddDataLoadersExt {
    /// Adds dataloader to graphql request
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
            AutoProcessingDataLoader::new(database.clone()),
            tokio::spawn,
        ))
        .data(DataLoader::new(
            AutoProcScalingDataLoader::new(database.clone()),
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
/// DataLoader for Processed Data
#[allow(clippy::missing_docs_in_private_items)]
pub struct ProcessedDataLoader {
    database: DatabaseConnection,
    parent_span: Span,
}
/// DataLoader for Process Job
#[allow(clippy::missing_docs_in_private_items)]
pub struct ProcessingJobDataLoader {
    database: DatabaseConnection,
    parent_span: Span,
}
/// DataLoader for AutoProcessing
#[allow(clippy::missing_docs_in_private_items)]
pub struct AutoProcessingDataLoader {
    database: DatabaseConnection,
    parent_span: Span,
}
/// DataLoader for overall statistics type
#[allow(clippy::missing_docs_in_private_items)]
pub struct AutoProcScalingDataLoader {
    database: DatabaseConnection,
    parent_span: Span,
}

#[allow(clippy::missing_docs_in_private_items)]
impl ProcessingJobDataLoader {
    fn new(database: DatabaseConnection) -> Self {
        Self {
            database,
            parent_span: Span::current(),
        }
    }
}

#[allow(clippy::missing_docs_in_private_items)]
impl ProcessedDataLoader {
    fn new(database: DatabaseConnection) -> Self {
        Self {
            database,
            parent_span: Span::current(),
        }
    }
}

#[allow(clippy::missing_docs_in_private_items)]
impl AutoProcessingDataLoader {
    fn new(database: DatabaseConnection) -> Self {
        Self {
            database,
            parent_span: Span::current(),
        }
    }
}

#[allow(clippy::missing_docs_in_private_items)]
impl AutoProcScalingDataLoader {
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
    type Value = Vec<ProcessingJob>;
    type Error = async_graphql::Error;

    #[instrument(name = "load_process_job", skip(self))]
    async fn load(&self, keys: &[u32]) -> Result<HashMap<u32, Self::Value>, Self::Error> {
        let span = tracing::info_span!(parent: &self.parent_span, "load_process_job");
        let _span = span.enter();
        let mut results = HashMap::new();
        let keys_vec: Vec<u32> = keys.to_vec();

        let query = sea_query::Query::select()
            .column(Asterisk)
            .from(processing_job::Entity)
            .left_join(
                processing_job_parameter::Entity,
                Expr::col((
                    processing_job::Entity,
                    processing_job::Column::ProcessingJobId,
                ))
                .equals((
                    processing_job_parameter::Entity,
                    processing_job_parameter::Column::ProcessingJobId,
                )),
            )
            .and_where(Expr::col(processing_job::Column::DataCollectionId).is_in(keys_vec.clone()))
            .build_any(
                self.database
                    .get_database_backend()
                    .get_query_builder()
                    .deref(),
            );

        let records = self
            .database
            .query_all(Statement::from_sql_and_values(
                self.database.get_database_backend(),
                &query.0,
                query.1,
            ))
            .await?
            .into_iter()
            .map(ProcessingJob::from)
            .collect::<Vec<_>>();

        for record in records {
            let data_collection_id = record.data_collection_id.unwrap();
            results
                .entry(data_collection_id)
                .or_insert_with(Vec::new)
                .push(record)
        }
        Ok(results)
    }
}

impl Loader<u32> for AutoProcessingDataLoader {
    type Value = Vec<AutoProcessing>;
    type Error = async_graphql::Error;

    #[instrument(name = "load_auto_proc_integration", skip(self))]
    async fn load(&self, keys: &[u32]) -> Result<HashMap<u32, Self::Value>, Self::Error> {
        let span = tracing::info_span!(parent: &self.parent_span, "load_auto_proc_integration");
        let _span = span.enter();
        let mut results = HashMap::new();
        let keys_vec: Vec<u32> = keys.to_vec();

        let query = sea_query::Query::select()
            .column(Asterisk)
            .from(auto_proc_integration::Entity)
            .left_join(
                auto_proc_program::Entity,
                Expr::col((
                    auto_proc_integration::Entity,
                    auto_proc_integration::Column::AutoProcProgramId,
                ))
                .equals((
                    auto_proc_program::Entity,
                    auto_proc_program::Column::AutoProcProgramId,
                )),
            )
            .left_join(
                auto_proc::Entity,
                Expr::col((auto_proc::Entity, auto_proc::Column::AutoProcProgramId)).equals((
                    auto_proc_program::Entity,
                    auto_proc_program::Column::AutoProcProgramId,
                )),
            )
            .left_join(
                auto_proc_scaling::Entity,
                Expr::col((
                    auto_proc_scaling::Entity,
                    auto_proc_scaling::Column::AutoProcId,
                ))
                .equals((auto_proc::Entity, auto_proc::Column::AutoProcId)),
            )
            .and_where(
                Expr::col(auto_proc_integration::Column::DataCollectionId).is_in(keys_vec.clone()),
            )
            .build_any(
                self.database
                    .get_database_backend()
                    .get_query_builder()
                    .deref(),
            );

        let records = self
            .database
            .query_all(Statement::from_sql_and_values(
                self.database.get_database_backend(),
                &query.0,
                query.1,
            ))
            .await?
            .into_iter()
            .map(AutoProcessing::from)
            .collect::<Vec<_>>();

        for record in records {
            let data_collection_id = record.data_collection_id;
            results
                .entry(data_collection_id)
                .or_insert_with(Vec::new)
                .push(record)
        }

        Ok(results)
    }
}

impl Loader<(u32, StatisticsType)> for AutoProcScalingDataLoader {
    type Value = AutoProcScalingStatics;
    type Error = async_graphql::Error;

    #[instrument(name = "load_auto_proc_scaling", skip(self))]
    #[allow(clippy::type_complexity)]
    async fn load(
        &self,
        keys: &[(u32, StatisticsType)],
    ) -> Result<HashMap<(u32, StatisticsType), Self::Value>, Self::Error> {
        let span = tracing::info_span!(parent: &self.parent_span, "load_auto_proc_scaling");
        let _span = span.enter();
        let mut results = HashMap::new();
        let query = sea_query::Query::select()
            .column(Asterisk)
            .from(auto_proc_scaling_statistics::Entity)
            .and_where(
                Expr::tuple([
                    Expr::col(auto_proc_scaling_statistics::Column::AutoProcScalingId).into(),
                    Expr::col(auto_proc_scaling_statistics::Column::ScalingStatisticsType).into(),
                ])
                .in_tuples(
                    keys.iter()
                        .map(|(id, stat_type)| (*id, stat_type.to_string())),
                ),
            )
            .build_any(
                self.database
                    .get_database_backend()
                    .get_query_builder()
                    .deref(),
            );

        let records = auto_proc_scaling_statistics::Entity::find()
            .from_raw_sql(Statement::from_sql_and_values(
                self.database.get_database_backend(),
                query.0,
                query.1,
            ))
            .all(&self.database)
            .await?;

        for record in records {
            let keys: (u32, StatisticsType) = (
                record.auto_proc_scaling_id.unwrap(),
                record.scaling_statistics_type.into(),
            );
            let data = AutoProcScalingStatics::from(record);
            results.insert(keys, data);
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
    ) -> async_graphql::Result<Option<Vec<ProcessingJob>>, async_graphql::Error> {
        let loader = ctx.data_unchecked::<DataLoader<ProcessingJobDataLoader>>();
        loader.load_one(self.id).await
    }

    /// Fetches all the automatic process
    async fn auto_processing(
        &self,
        ctx: &Context<'_>,
    ) -> async_graphql::Result<Option<Vec<AutoProcessing>>, async_graphql::Error> {
        let loader = ctx.data_unchecked::<DataLoader<AutoProcessingDataLoader>>();
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
impl AutoProcessing {
    /// Fetches the overall scaling statistics type
    async fn overall(
        &self,
        ctx: &Context<'_>,
    ) -> async_graphql::Result<Option<AutoProcScalingStatics>> {
        let loader = ctx.data_unchecked::<DataLoader<AutoProcScalingDataLoader>>();
        match self.auto_proc_id {
            Some(id) => loader
                .load_one((id,  StatisticsType::Overall))
                .await,
            None => Ok(None)
        }
    }

    /// Fetches the innershell scaling statistics type
    async fn inner_shell(
        &self,
        ctx: &Context<'_>,
    ) -> async_graphql::Result<Option<AutoProcScalingStatics>> {
        let loader = ctx.data_unchecked::<DataLoader<AutoProcScalingDataLoader>>();
        match self.auto_proc_id {
            Some(id) => loader
                .load_one((id,  StatisticsType::InnerShell))
                .await,
            None => Ok(None)
        }

    }

    /// Fetches the outershell scaling statistics type
    async fn outer_shell(
        &self,
        ctx: &Context<'_>,
    ) -> async_graphql::Result<Option<AutoProcScalingStatics>> {
        let loader = ctx.data_unchecked::<DataLoader<AutoProcScalingDataLoader>>();
        match self.auto_proc_id {
            Some(id) => loader
                .load_one((id, StatisticsType::OuterShell))
                .await,
            None => Ok(None)
        }
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
