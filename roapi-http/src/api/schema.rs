use std::sync::Arc;

use axum::body::Body;
use axum::extract;
use axum::http::Response;

use crate::api::{bytes_to_json_resp, HandlerContext};
use crate::error::ApiErrResp;

pub async fn schema(
    state: extract::Extension<Arc<HandlerContext>>,
) -> Result<Response<Body>, ApiErrResp> {
    let ctx = state.0;
    let payload =
        serde_json::to_vec(ctx.cq.schema_map()).map_err(ApiErrResp::json_serialization)?;
    Ok(bytes_to_json_resp(payload))
}

pub async fn get_by_table_name(
    state: extract::Extension<Arc<HandlerContext>>,
    extract::Path(table_name): extract::Path<String>,
) -> Result<Response<Body>, ApiErrResp> {
    let ctx = state.0;
    let payload = serde_json::to_vec(
        ctx.cq
            .schema_map()
            .get(&table_name)
            .ok_or_else(|| ApiErrResp::not_found("invalid table name"))?,
    )
    .map_err(ApiErrResp::json_serialization)?;
    Ok(bytes_to_json_resp(payload))
}
