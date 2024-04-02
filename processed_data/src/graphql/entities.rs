use async_graphql::SimpleObject;
use models::data_collection_file_attachment;

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
    fn from(values: data_collection_file_attachment::Model) -> Self {
        Self {
            id: values.data_collection_file_attachment_id,
            file_full_path: values.file_full_path,
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
