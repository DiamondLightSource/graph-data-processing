use async_graphql::{Enum, SimpleObject};
use models::{
    auto_proc_scaling, auto_proc_scaling_statistics, data_collection_file_attachment,
    sea_orm_active_enums::ScalingStatisticsType,
};
use sea_orm::QueryResult;

/// Combines autoproc integration, autoproc program, autoproc and autoproc scaling
#[derive(Debug, Clone, SimpleObject)]
#[graphql(
    name = "AutoProcessing",
    unresolvable = "autoProcIntegrationId",
    complex
)]
pub struct AutoProcessing {
    /// An opaque unique identifier for the auto processing integration
    pub auto_proc_integration_id: u32,
    /// An opaque unique identifier for the data collection
    pub data_collection_id: u32,
    /// An opaque unique identifier for the auto processing program
    pub auto_proc_program_id: Option<u32>,
    /// Refined X position of the beam
    pub refined_x_beam: Option<f32>,
    /// Refined Y position of the beam
    pub refined_y_beam: Option<f32>,
    /// Name of the processing programs
    pub processing_programs: Option<String>,
    /// Processing program status
    pub processing_status: Option<i8>,
    /// Processing program message
    pub processing_message: Option<String>,
    /// An opaque unique identifier for the  processing processing job
    pub processing_job_id: Option<u32>,
    /// An opaque unique identifier for the auto processing
    pub auto_proc_id: Option<u32>,
    /// Space group of the processing job
    pub space_group: Option<String>,
    /// Refined cell a in the auto processing job
    pub refined_cell_a: Option<f32>,
    /// Refined cell b in the auto processing job
    pub refined_cell_b: Option<f32>,
    /// Refined cell c in the auto processing job
    pub refined_cell_c: Option<f32>,
    /// Refined cell alpha in the auto processing job
    pub refined_cell_alpha: Option<f32>,
    /// Refined cell beta in the auto processing job
    pub refined_cell_beta: Option<f32>,
    /// Refined cell gamma in the auto processing job
    pub refined_cell_gamma: Option<f32>,
    /// An opaque unique identifier for the auto processing scaling
    pub auto_proc_scaling_id: Option<u32>,
}

impl From<QueryResult> for AutoProcessing {
    fn from(value: QueryResult) -> Self {
        Self {
            auto_proc_integration_id: value.try_get("", "autoProcIntegrationId").unwrap(),
            data_collection_id: value.try_get("", "dataCollectionId").unwrap(),
            auto_proc_program_id: value.try_get("", "autoProcProgramId").unwrap_or(None),
            refined_x_beam: value.try_get("", "refinedXBeam").unwrap_or(None),
            refined_y_beam: value.try_get("", "refinedYBeam").unwrap_or(None),
            processing_programs: value.try_get("", "processingPrograms").unwrap_or(None),
            processing_status: value.try_get("", "processingStatus").unwrap_or(None),
            processing_message: value.try_get("", "processingMessage").unwrap_or(None),
            processing_job_id: value.try_get("", "processingJobId").unwrap_or(None),
            auto_proc_id: value.try_get("", "autoProcId").unwrap_or(None),
            space_group: value.try_get("", "spaceGroup").unwrap_or(None),
            refined_cell_a: value.try_get("", "refinedCell_a").unwrap_or(None),
            refined_cell_b: value.try_get("", "refinedCell_b").unwrap_or(None),
            refined_cell_c: value.try_get("", "refinedCell_c").unwrap_or(None),
            refined_cell_alpha: value.try_get("", "refinedCell_alpha").unwrap_or(None),
            refined_cell_beta: value.try_get("", "refinedCell_beta").unwrap_or(None),
            refined_cell_gamma: value.try_get("", "refinedCell_gamma").unwrap_or(None),
            auto_proc_scaling_id: value.try_get("", "autoProcScalingId").unwrap_or(None),
        }
    }
}

/// Represents processed image file stored in s3 bucket
#[derive(Clone, Debug, PartialEq, SimpleObject)]
#[graphql(name = "DataProcessing", unresolvable, complex)]
pub struct DataProcessing {
    /// An opaque unique identifier for the collected file attachment
    pub id: u32,
    /// Full path where the processed image is stored
    #[graphql(skip)]
    pub file_full_path: String,
}

impl From<data_collection_file_attachment::Model> for DataProcessing {
    fn from(value: data_collection_file_attachment::Model) -> Self {
        Self {
            id: value.data_collection_file_attachment_id,
            file_full_path: value.file_full_path,
        }
    }
}

/// Represents a processing job
#[derive(Clone, Debug, PartialEq, SimpleObject)]
#[graphql(name = "ProcessingJobs", unresolvable)]
pub struct ProcessingJob {
    /// An opaque unique identifier for the processing job
    pub processing_job_id: Option<u32>,
    /// An opaque unique identifier for the data collection
    pub data_collection_id: Option<u32>,
    /// Processing job display name
    pub display_name: Option<String>,
    /// Represents if the job is automatic or downstream
    pub automatic: Option<i8>,
    /// An opaque unique identifier for the processing job parameter
    pub processing_job_parameter_id: Option<u32>,
    /// Parameter key
    pub parameter_key: Option<String>,
    /// Parameter values
    pub parameter_value: Option<String>,
}

impl From<QueryResult> for ProcessingJob {
    fn from(value: QueryResult) -> Self {
        Self {
            processing_job_id: value.try_get("", "processingJobId").unwrap_or(None),
            data_collection_id: value.try_get("", "dataCollectionId").unwrap_or(None),
            display_name: value.try_get("", "displayName").unwrap_or(None),
            automatic: value.try_get("", "automatic").unwrap_or(None),
            processing_job_parameter_id: value
                .try_get("", "processingJobParameterId")
                .unwrap_or(None),
            parameter_key: value.try_get("", "parameterKey").unwrap_or(None),
            parameter_value: value.try_get("", "parameterValue").unwrap_or(None),
        }
    }
}

/// Represents and auto processing scaling
#[derive(Clone, Debug, PartialEq, SimpleObject)]
#[graphql(name = "AutoProcScaling", unresolvable)]
pub struct AutoProcScaling {
    /// An opaque unique identifier for the auto processing scaling
    pub auto_proc_scaling_id: u32,
    /// An opaque unique identifier for the auto processing
    pub auto_proc_id: Option<u32>,
}

impl From<auto_proc_scaling::Model> for AutoProcScaling {
    fn from(value: auto_proc_scaling::Model) -> Self {
        Self {
            auto_proc_scaling_id: value.auto_proc_scaling_id,
            auto_proc_id: value.auto_proc_id,
        }
    }
}

#[derive(Enum, Copy, Clone, Debug, PartialEq, Eq, Hash)]
#[allow(clippy::missing_docs_in_private_items)]
pub enum StatisticsType {
    Overall,
    InnerShell,
    OuterShell,
}

impl ToString for StatisticsType {
    fn to_string(&self) -> String {
        match self {
            StatisticsType::Overall => "overall".to_string(),
            StatisticsType::InnerShell => "innershell".to_string(),
            StatisticsType::OuterShell => "outershell".to_string(),
        }
    }
}

/// Represents auto processing scaling statics
#[derive(Clone, Debug, PartialEq, SimpleObject)]
#[graphql(name = "AutoProcScalingStatics", unresolvable)]
#[allow(clippy::missing_docs_in_private_items)]
pub struct AutoProcScalingStatics {
    pub auto_proc_scaling_statistics_id: u32,
    pub auto_proc_scaling_id: Option<u32>,
    pub scaling_statistics_type: StatisticsType,
    pub resolution_limit_low: Option<f32>,
    pub resolution_limit_high: Option<f32>,
    pub r_merge: Option<f32>,
    pub r_meas_all_i_plus_i_minus: Option<f32>,
    pub n_total_observations: Option<i32>,
    pub n_total_unique_observations: Option<i32>,
    pub mean_i_over_sig_i: Option<f32>,
    pub completeness: Option<f32>,
    pub multiplicity: Option<f32>,
    pub anomalous_completeness: Option<f32>,
    pub anomalous_multiplicity: Option<f32>,
    pub cc_half: Option<f32>,
    pub cc_anomalous: Option<f32>,
}

impl From<ScalingStatisticsType> for StatisticsType {
    fn from(value: ScalingStatisticsType) -> Self {
        match value {
            ScalingStatisticsType::Overall => StatisticsType::Overall,
            ScalingStatisticsType::InnerShell => StatisticsType::InnerShell,
            ScalingStatisticsType::OuterShell => StatisticsType::OuterShell,
        }
    }
}

impl From<auto_proc_scaling_statistics::Model> for AutoProcScalingStatics {
    fn from(value: auto_proc_scaling_statistics::Model) -> Self {
        Self {
            auto_proc_scaling_id: value.auto_proc_scaling_id,
            auto_proc_scaling_statistics_id: value.auto_proc_scaling_statistics_id,
            resolution_limit_low: value.resolution_limit_low,
            resolution_limit_high: value.resolution_limit_high,
            r_merge: value.r_merge,
            r_meas_all_i_plus_i_minus: value.r_meas_all_i_plus_i_minus,
            n_total_observations: value.n_total_observations,
            n_total_unique_observations: value.n_total_unique_observations,
            mean_i_over_sig_i: value.mean_i_over_sig_i,
            completeness: value.completeness,
            multiplicity: value.multiplicity,
            anomalous_completeness: value.anomalous_completeness,
            anomalous_multiplicity: value.anomalous_multiplicity,
            cc_half: value.cc_half,
            cc_anomalous: value.cc_anomalous,
            scaling_statistics_type: StatisticsType::from(value.scaling_statistics_type),
        }
    }
}

impl DataProcessing {
    /// S3 bucket object key
    pub fn object_key(&self) -> String {
        self.file_full_path.to_string()
    }
}

/// Datasets subgraph extension
#[derive(SimpleObject)]
#[graphql(name = "Datasets", complex)]
pub struct DataCollection {
    /// An opaque unique identifier for the data collection
    pub id: u32,
}
