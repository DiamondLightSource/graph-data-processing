use sea_orm_codegen::{
    DateTimeCrate, EntityTransformer, EntityWriterContext, OutputFile, WithSerde,
};
use sea_schema::mysql::discovery::SchemaDiscovery;
use sqlx::{MySql, Pool};
use std::path::Path;
use tokio::{
    fs::{create_dir_all, File},
    io::AsyncWriteExt,
};
use url::Url;

struct Table<'a> {
    name: &'a str,
    columns: &'a [&'a str],
}

const TABLES_SPECS: &[&Table] = &[
    &Table {
        name: "DataCollectionFileAttachment",
        columns: &[
            "dataCollectionFileAttachmentId",
            "dataCollectionId",
            "fileFullPath",
        ],
    },
    &Table {
        name: "ProcessingJob",
        columns: &[
            "processingJobId",
            "dataCollectionId",
            "displayName",
            "automatic",
        ],
    },
    &Table {
        name: "AutoProc",
        columns: &[
            "autoProcId",
            "autoProcProgramId",
            "spaceGroup",
            "refinedCell_a",
            "refinedCell_b",
            "refinedCell_c",
            "refinedCell_alpha",
            "refinedCell_beta",
            "refinedCell_gamma",
        ],
    },
    &Table {
        name: "AutoProcProgram",
        columns: &[
            "autoProcProgramId",
            "processingPrograms",
            "processingStatus",
            "processingMessage",
            "processingJobId",
        ],
    },
    &Table {
        name: "AutoProcIntegration",
        columns: &[
            "autoProcIntegrationId",
            "dataCollectionId",
            "autoProcProgramId",
            "refinedXBeam",
            "refinedYBeam",
        ],
    },
    &Table {
        name: "AutoProcScaling",
        columns: &["autoProcScalingId", "autoProcId"],
    },
    &Table {
        name: "AutoProcScalingStatistics",
        columns: &[
            "autoProcScalingStatisticsId",
            "autoProcScalingId",
            "ccHalf",
            "ccAnomalous",
            "nTotalObservations",
            "nTotalUniqueObservations",
            "resolutionLimitLow",
            "resolutionLimitHigh",
            "scalingStatisticsType",
            "rMeasAllIPlusIMinus",
            "anomalousCompleteness",
            "anomalousMultiplicity",
            "rMerge",
            "completeness",
            "multiplicity",
            "meanIOverSigI",
            "resioversigi2",
        ],
    },
];

fn main() {
    tokio::runtime::Builder::new_current_thread()
        .enable_time()
        .enable_io()
        .build()
        .unwrap()
        .block_on(async {
            let database_url = std::env::var("DATABASE_URL")
                .unwrap()
                .parse::<Url>()
                .unwrap();
            let database_name = database_url.path_segments().unwrap().next().unwrap();
            let connection = Pool::<MySql>::connect(database_url.as_str()).await.unwrap();
            let schema_discovery = SchemaDiscovery::new(connection, database_name);
            let schema = schema_discovery.discover().await.unwrap();
            let table_statements = schema
                .tables
                .into_iter()
                .filter_map(|mut def| {
                    if let Some(spec) = TABLES_SPECS.iter().find(|spec| spec.name == def.info.name)
                    {
                        def.foreign_keys.retain(|fk| {
                            TABLES_SPECS
                                .iter()
                                .any(|spec| spec.name == fk.referenced_table)
                        });
                        def.columns
                            .retain(|column| spec.columns.contains(&column.name.as_str()));
                        Some(def.write())
                    } else {
                        None
                    }
                })
                .collect();

            let writer_context = EntityWriterContext::new(
                false,
                WithSerde::None,
                true,
                DateTimeCrate::Chrono,
                None,
                true,
                false,
                false,
                vec![],
                vec![],
                vec![],
                vec![],
                false,
            );

            let output = EntityTransformer::transform(table_statements)
                .unwrap()
                .generate(&writer_context);

            let dir = Path::new("src/");
            create_dir_all(dir).await.unwrap();

            for OutputFile { name, content } in output.files {
                println!("Writing: {name}");
                let mut file = File::create(dir.join(name)).await.unwrap();
                file.write_all(content.as_bytes()).await.unwrap();
            }
        })
}
