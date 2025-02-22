use std::convert::TryFrom;

use axum::body::Body;
use axum::http;
use axum::http::header;
use axum::http::Response;
use columnq::datafusion::arrow;
use columnq::encoding;
use columnq::ColumnQ;
use log::info;

use crate::config::Config;
use crate::error::ApiErrResp;

pub struct HandlerContext {
    pub cq: ColumnQ,
    // TODO: store pre serialized schema in handler context
}

impl HandlerContext {
    pub async fn new(config: &Config) -> anyhow::Result<Self> {
        let mut cq = ColumnQ::new();

        if config.tables.is_empty() {
            anyhow::bail!("No table found in tables config");
        }

        for t in config.tables.iter() {
            info!("loading `{}` as table `{}`", t.io_source, t.name);
            cq.load_table(t).await?;
            info!("registered `{}` as table `{}`", t.io_source, t.name);
        }

        Ok(Self { cq })
    }
}

#[inline]
pub fn bytes_to_resp(bytes: Vec<u8>, content_type: &'static str) -> Response<Body> {
    let mut res = Response::new(Body::from(bytes));
    res.headers_mut().insert(
        header::CONTENT_TYPE,
        header::HeaderValue::from_static(content_type),
    );
    res
}

#[inline]
pub fn bytes_to_json_resp(bytes: Vec<u8>) -> Response<Body> {
    bytes_to_resp(bytes, "application/json")
}

pub fn encode_type_from_hdr(
    headers: header::HeaderMap,
) -> Result<encoding::ContentType, ApiErrResp> {
    match headers.get(header::ACCEPT) {
        None => Ok(encoding::ContentType::Json),
        Some(hdr_value) => {
            encoding::ContentType::try_from(hdr_value.as_bytes()).map_err(|_| ApiErrResp {
                code: http::StatusCode::BAD_REQUEST,
                error: "unsupported_content_type".to_string(),
                message: format!("{:?} is not a supported response content type", hdr_value),
            })
        }
    }
}

pub fn encode_record_batches(
    content_type: encoding::ContentType,
    batches: &[arrow::record_batch::RecordBatch],
) -> Result<Response<Body>, ApiErrResp> {
    let payload = match content_type {
        encoding::ContentType::Json => encoding::json::record_batches_to_bytes(batches)
            .map_err(ApiErrResp::json_serialization)?,
        encoding::ContentType::Csv => encoding::csv::record_batches_to_bytes(batches)
            .map_err(ApiErrResp::csv_serialization)?,
        encoding::ContentType::ArrowFile => encoding::arrow::record_batches_to_file_bytes(batches)
            .map_err(ApiErrResp::arrow_file_serialization)?,
        encoding::ContentType::ArrowStream => {
            encoding::arrow::record_batches_to_stream_bytes(batches)
                .map_err(ApiErrResp::arrow_stream_serialization)?
        }
        encoding::ContentType::Parquet => encoding::parquet::record_batches_to_bytes(batches)
            .map_err(ApiErrResp::parquet_serialization)?,
    };

    Ok(bytes_to_resp(payload, content_type.to_str()))
}

pub mod graphql;
pub mod rest;
pub mod routes;
pub mod schema;
pub mod sql;

pub use routes::register_app_routes;
