/// Collection of graphql entities
mod entities;
use crate::S3Bucket;
use async_graphql::{
    dataloader::{DataLoader, Loader},
    ComplexObject, Context, EmptyMutation, EmptySubscription, Object, Schema, SchemaBuilder,
};
use aws_sdk_s3::presigning::PresigningConfig;
use entities::{
    AutoProc, AutoProcScaling, AutoProcScalingStatics, AutoProcess, AutoProcessing, DataCollection,
    DataProcessing, ProcessJob, ProcessingJob, ProcessingJobParameter, StatisticsType, AP,
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
            ProcessJobDataLoader::new(database.clone()),
            tokio::spawn,
        ))
        .data(DataLoader::new(
            AutoProcIntegrationDataLoader::new(database.clone()),
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
pub struct ProcessJobDataLoader {
    database: DatabaseConnection,
    parent_span: Span,
}
/// DataLoader for AutoProcIntegration
#[allow(clippy::missing_docs_in_private_items)]
pub struct AutoProcIntegrationDataLoader {
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
impl ProcessJobDataLoader {
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
impl AutoProcIntegrationDataLoader {
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

impl Loader<u32> for ProcessJobDataLoader {
    type Value = Vec<ProcessJob>;
    type Error = async_graphql::Error;

    #[instrument(name = "load_process_job", skip(self))]
    async fn load(&self, keys: &[u32]) -> Result<HashMap<u32, Self::Value>, Self::Error> {
        let span = tracing::info_span!(parent: &self.parent_span, "load_process_job");
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
    type Value = Vec<AP>;
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
            .map(|record| AP {
                auto_proc_integration_id: record
                    .try_get("AutoProcIntegration", "auto_proc_integration_id")
                    .unwrap(),
                data_collection_id: record
                    .try_get("AutoProcIntegration", "data_collection_id")
                    .unwrap(),
                auto_proc_program_id: record
                    .try_get("AutoProcIntegration", "auto_proc_program_id")
                    .unwrap(),
                refined_x_beam: record
                    .try_get("AutoProcIntegration", "refined_x_beam")
                    .unwrap(),
                refined_y_beam: record
                    .try_get("AutoProcIntegration", "refined_y_beam")
                    .unwrap(),
                processing_programs: record
                    .try_get("AutoProcProgram", "processing_programs")
                    .unwrap(),
                processing_status: record
                    .try_get("AutoProcProgram", "processing_status")
                    .unwrap(),
                processing_message: record
                    .try_get("AutoProcProgram", "processing_message")
                    .unwrap(),
                processing_job_id: record
                    .try_get("AutoProcProgram", "processing_job_id")
                    .unwrap(),
                auto_proc_id: record.try_get("AutoProc", "auto_proc_id").unwrap(),
                space_group: record.try_get("AutoProc", "space_group").unwrap(),
                refined_cell_a: record.try_get("AutoProc", "refined_cell_a").unwrap(),
                refined_cell_b: record.try_get("AutoProc", "refined_cell_b").unwrap(),
                refined_cell_c: record.try_get("AutoProc", "refined_cell_c").unwrap(),
                refined_cell_alpha: record.try_get("AutoProc", "refined_cell_alpha").unwrap(),
                refined_cell_beta: record.try_get("AutoProc", "refined_cell_beta").unwrap(),
                refined_cell_gamma: record.try_get("AutoProc", "refined_cell_gamma").unwrap(),
                auto_proc_scaling_id: record
                    .try_get("AutoProcScaling", "auto_proc_scaling_id")
                    .unwrap(),
            })
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

impl Loader<u32> for AutoProcessingDataLoader {
    type Value = AutoProcess;
    type Error = async_graphql::Error;

    #[instrument(name = "load_process", skip(self))]
    async fn load(&self, keys: &[u32]) -> Result<HashMap<u32, Self::Value>, Self::Error> {
        let span = tracing::info_span!(parent: &self.parent_span, "load_process");
        let _span = span.enter();
        let mut results = HashMap::new();
        let keys_vec: Vec<u32> = keys.to_vec();
        let records = auto_proc::Entity::find()
            .filter(auto_proc::Column::AutoProcProgramId.is_in(keys_vec))
            .find_also_related(auto_proc_scaling::Entity)
            .all(&self.database)
            .await?
            .into_iter()
            .map(|(auto_proc, scaling)| AutoProcess {
                auto_proc: AutoProc::from(auto_proc),
                auto_proc_scaling: scaling.map(AutoProcScaling::from),
            })
            .collect::<Vec<_>>();

        for record in records {
            let program_id = record.auto_proc.auto_proc_program_id.unwrap();
            results.insert(program_id, record);
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
    ) -> async_graphql::Result<Option<Vec<ProcessJob>>, async_graphql::Error> {
        let loader = ctx.data_unchecked::<DataLoader<ProcessJobDataLoader>>();
        loader.load_one(self.id).await
    }

    /// Fetches all the automatic process
    async fn auto_proc_integration(
        &self,
        ctx: &Context<'_>,
    ) -> async_graphql::Result<Option<Vec<AP>>, async_graphql::Error> {
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
impl AutoProcessing {
    /// Fetched the automatic process
    async fn auto_proc(&self, ctx: &Context<'_>) -> async_graphql::Result<Option<AutoProcess>> {
        let loader = ctx.data_unchecked::<DataLoader<AutoProcessingDataLoader>>();
        let id = self.auto_proc_integration.auto_proc_program_id;
        loader.load_one(id.unwrap()).await
    }
}

#[ComplexObject]
impl AutoProcess {
    /// Fetches the overall scaling statistics type
    async fn overall(
        &self,
        ctx: &Context<'_>,
    ) -> async_graphql::Result<Option<AutoProcScalingStatics>> {
        let loader = ctx.data_unchecked::<DataLoader<AutoProcScalingDataLoader>>();
        let id = <Option<entities::AutoProcScaling> as Clone>::clone(&self.auto_proc_scaling)
            .unwrap()
            .auto_proc_id;
        loader
            .load_one((id.unwrap(), StatisticsType::Overall))
            .await
    }

    /// Fetches the innershell scaling statistics type
    async fn inner_shell(
        &self,
        ctx: &Context<'_>,
    ) -> async_graphql::Result<Option<AutoProcScalingStatics>> {
        let loader = ctx.data_unchecked::<DataLoader<AutoProcScalingDataLoader>>();
        let id = <Option<entities::AutoProcScaling> as Clone>::clone(&self.auto_proc_scaling)
            .unwrap()
            .auto_proc_id;
        loader
            .load_one((id.unwrap(), StatisticsType::InnerShell))
            .await
    }

    /// Fetches the outershell scaling statistics type
    async fn outer_shell(
        &self,
        ctx: &Context<'_>,
    ) -> async_graphql::Result<Option<AutoProcScalingStatics>> {
        let loader = ctx.data_unchecked::<DataLoader<AutoProcScalingDataLoader>>();
        let id = <Option<entities::AutoProcScaling> as Clone>::clone(&self.auto_proc_scaling)
            .unwrap()
            .auto_proc_id;
        loader
            .load_one((id.unwrap(), StatisticsType::OuterShell))
            .await
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
