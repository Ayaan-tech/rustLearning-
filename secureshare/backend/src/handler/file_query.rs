use std::sync::Arc;

use axum::{
    extract::Query,
    response::IntoResponse,
    routing::get,
    Extension, Json, Router,
};
use validator::Validate;

use crate::{
    db::UserExt,
    dto::{
        RequestQueryDto, UserReceiveFileDto, UserReceiveFileListResponseDto, UserSendFileDto,
        UserSendFileListResponseDto,
    },
    error::{ErrorResponse, HttpError},
    middleware::JWTAuthMiddleware,
    AppState,
};

pub fn get_file_list_handler() -> Router {
    Router::new()
        .route("/sent", get(get_user_shared_files))
        .route("/received", get(get_receive_shared_files))
}

#[utoipa::path(
    get,
    path = "/api/list/sent",
    tag = "Lists",
    security(("bearer_auth" = [])),
    params(RequestQueryDto),
    responses(
        (status = 200, description = "Files sent by the authenticated user", body = UserSendFileListResponseDto),
        (status = 400, description = "Invalid pagination parameters", body = ErrorResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse),
    )
)]
pub async fn get_user_shared_files(
    Query(query_params): Query<RequestQueryDto>,
    Extension(app_state): Extension<Arc<AppState>>,
    Extension(user): Extension<JWTAuthMiddleware>,
) -> Result<impl IntoResponse, HttpError> {
    query_params
        .validate()
        .map_err(|e| HttpError::bad_request(e.to_string()))?;

    let page = query_params.page.unwrap_or(1) as u32;
    let limit = query_params.limit.unwrap_or(10);
    let user_id = user.user.id;

    let (shared_files, total_count) = app_state
        .db_client
        .get_sent_files(user_id, page, limit)
        .await
        .map_err(|e| HttpError::server_error(e.to_string()))?;

    let filter_send_files = UserSendFileDto::filter_send_user_files(&shared_files);
    let response = UserSendFileListResponseDto {
        status: "success".to_string(),
        files: filter_send_files,
        results: total_count,
    };
    Ok(Json(response))
}

#[utoipa::path(
    get,
    path = "/api/list/received",
    tag = "Lists",
    security(("bearer_auth" = [])),
    params(RequestQueryDto),
    responses(
        (status = 200, description = "Files received by the authenticated user", body = UserReceiveFileListResponseDto),
        (status = 400, description = "Invalid pagination parameters", body = ErrorResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse),
    )
)]
pub async fn get_receive_shared_files(
    Query(query_params): Query<RequestQueryDto>,
    Extension(app_state): Extension<Arc<AppState>>,
    Extension(user): Extension<JWTAuthMiddleware>,
) -> Result<impl IntoResponse, HttpError> {
    query_params
        .validate()
        .map_err(|e| HttpError::bad_request(e.to_string()))?;

    let page = query_params.page.unwrap_or(1) as u32;
    let limit = query_params.limit.unwrap_or(10);
    let user_id = user.user.id;

    let (receive_files, total_count) = app_state
        .db_client
        .get_receive_files(user_id, page, limit)
        .await
        .map_err(|e| HttpError::server_error(e.to_string()))?;

    let filter_receive_files = UserReceiveFileDto::filter_receive_user_files(&receive_files);
    let response = UserReceiveFileListResponseDto {
        status: "success".to_string(),
        files: filter_receive_files,
        results: total_count,
    };
    Ok(Json(response))
}
