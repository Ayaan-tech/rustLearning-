use std::sync::Arc;
use axum::{extract::Query, response::IntoResponse, routing::{get,put},Extension,Json,Router};
use validator::Validate;

use crate::{db::UserExt, dto::{EmailListResponseDto, FilterEmailDto, FilterUserDto, NameUpdateDto, Response, SearchQueryByEmailDTO, UserData, UserPasswordUpdateDto, UserResponseDto}, error::{ErrorMessage, ErrorResponse, HttpError}, middleware::JWTAuthMiddleware, utils::password, AppState};
pub fn users_handler() -> Router {
    Router::new()
        .route(
            "/me",
            get(get_me)
        )
        .route("/name",put(update_user_name))
        .route("/password",put(update_user_password))
        .route("/search-emails",get(search_by_email))
}
#[utoipa::path(
    get,
    path = "/api/users/me",
    tag = "Users",
    security(("bearer_auth" = [])),
    responses(
        (status = 200, description = "Current authenticated user profile", body = UserResponseDto),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
    )
)]
pub async fn get_me(
    Extension(_app_state): Extension<Arc<AppState>>,
    Extension(user): Extension<JWTAuthMiddleware>,
) -> Result<impl IntoResponse, HttpError> {
    let filtered_user = FilterUserDto::filter_user(&user.user);
    let response_dto = UserResponseDto {
        status: "success".to_string(),
        data: UserData {user: filtered_user},
    };
    Ok(Json(response_dto))
}

#[utoipa::path(
    put,
    path = "/api/users/name",
    tag = "Users",
    security(("bearer_auth" = [])),
    request_body = NameUpdateDto,
    responses(
        (status = 200, description = "Name updated successfully", body = UserResponseDto),
        (status = 400, description = "Validation error", body = ErrorResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse),
    )
)]
pub async fn update_user_name(
    Extension(app_state):Extension<Arc<AppState>>,
    Extension(user): Extension<JWTAuthMiddleware>,
    Json(body): Json<NameUpdateDto>
)-> Result<impl IntoResponse, HttpError> {
    body.validate()
        .map_err(|e| HttpError::bad_request(e.to_string()))?;
    let user = &user.user;
    let user_id = uuid::Uuid::parse_str(&user.id.to_string()).unwrap();
    let result = app_state.db_client
        .update_user_name(user_id.clone(), &body.name)
        .await
        .map_err(|e| HttpError::server_error(e.to_string()))?;
    let filtered_user = FilterUserDto::filter_user(&result);
    let response = UserResponseDto{
        status:"success".to_string(),
        data: UserData {user: filtered_user}
    };
    Ok(Json(response))
}

#[utoipa::path(
    put,
    path = "/api/users/password",
    tag = "Users",
    security(("bearer_auth" = [])),
    request_body = UserPasswordUpdateDto,
    responses(
        (status = 200, description = "Password updated successfully", body = Response),
        (status = 400, description = "Validation error or incorrect old password", body = ErrorResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse),
    )
)]
pub async fn update_user_password(
    Extension(app_state): Extension<Arc<AppState>>,
    Extension(user): Extension<JWTAuthMiddleware>,
    Json(body): Json<UserPasswordUpdateDto>
) -> Result<impl IntoResponse, HttpError> {
    body.validate().map_err(|e| HttpError::bad_request(e.to_string()))?;
    let user = &user.user;
    let user_id = uuid::Uuid::parse_str(&user.id.to_string()).unwrap();
    let result = app_state.db_client.get_user(Some(user_id.clone()), None, None)
                    .await.map_err(|e| HttpError::server_error(e.to_string()))?;
    let user = result.ok_or(HttpError::unauthorized(ErrorMessage::InvalidToken.to_string()))?;
    let password_match = password::compare_password(
        &body.old_password,
        &user.password
    ).map_err(|e| HttpError::server_error(e.to_string()))?;
    if !password_match {
        return Err(HttpError::bad_request("Old password is incorrect".to_string()));
    }
    let hashed_password = password::hash(&body.new_password).map_err(|e| HttpError::server_error(e.to_string()))?;
    app_state.db_client
    .update_user_password(user_id.clone(), hashed_password).await.map_err(|e| HttpError::server_error(e.to_string()))?;
    let response = Response {
        message:"Password updated successfully".to_string(),
        status:"success".to_string()
    };
    Ok(Json(response))
}

#[utoipa::path(
    get,
    path = "/api/users/search-emails",
    tag = "Users",
    security(("bearer_auth" = [])),
    params(SearchQueryByEmailDTO),
    responses(
        (status = 200, description = "Matching user emails", body = EmailListResponseDto),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse),
    )
)]
pub async fn search_by_email(
    Query(params): Query<SearchQueryByEmailDTO>,
    Extension(app_state): Extension<Arc<AppState>>,
    Extension(user): Extension<JWTAuthMiddleware>,
) -> Result<impl IntoResponse, HttpError> {
    params.validate().map_err(|e| HttpError::server_error(e.to_string()))?;
    let query_pattern = format!("%{}%",params.query);
    let user_id = uuid::Uuid::parse_str(&user.user.id.to_string()).unwrap();
    let users =  app_state.db_client
                .search_by_email(user_id.clone(),query_pattern.clone())
                .await
                .map_err(|e| HttpError::server_error(e.to_string()))?;
    let filtered_email = FilterEmailDto::filter_emails(&users);
    let response_data = EmailListResponseDto {
        status: "success".to_string(),
        emails:filtered_email
    };
    Ok(Json(response_data))
}