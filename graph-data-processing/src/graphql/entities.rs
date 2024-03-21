use async_graphql::SimpleObject;
use models::data_collection_file_attachment;

#[derive(Clone, Debug, PartialEq, SimpleObject)]
#[graphql(name = "DataProcessing")]
pub struct DataProcessing {
    /// An opaque unique identifier for the collected file attachment 
    pub data_collection_file_attachment_id: u32, 
    /// Full path where the processed image is stored
    pub file_full_path: String, 
}

impl From<data_collection_file_attachment::Model> for DataProcessing {
    fn from(values: data_collection_file_attachment::Model) -> Self {
        Self {
            data_collection_file_attachment_id: values.data_collection_file_attachment_id,
            file_full_path: values.file_full_path,
        }
    }
}

#[derive(SimpleObject)]
#[graphql(name = "Datasets", complex)]
pub struct DataCollection {
    /// An opaque unique identifier for the data collection
    pub data_collection_id: u32,
}
