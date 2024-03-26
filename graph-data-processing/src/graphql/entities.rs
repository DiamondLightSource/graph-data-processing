use async_graphql::SimpleObject;
use models::data_collection_file_attachment;

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
    pub fn object_key(&self) -> String {
        format!("{}", self.file_full_path)
    }
}

#[derive(SimpleObject)]
#[graphql(name = "Datasets", complex)]
pub struct DataCollection {
    /// An opaque unique identifier for the data collection
    pub id: u32,
}
