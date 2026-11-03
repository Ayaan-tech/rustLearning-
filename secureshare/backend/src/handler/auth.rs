use std::sync::Arc;

use axum::{
    Extension, Json, Router,
    http::{HeaderMap, StatusCode, header},
    response::IntoResponse,
    routing::post,
};
use axum_extra::extract::cookie::Cookie;
use validator::Validate;

use crate::{
    AppState,
    db::UserExt,
    dto::{LoginUserDto, RegisterUserDto, Response, UserLoginResponseDto},
    error::{ErrorMessage, ErrorResponse, HttpError},
    utils::{keys::generate_key, password, token},
};

pub fn auth_handler() -> Router {
    Router::new()
        .route("/register", post(register))
        .route("/login", post(login))
}

#[utoipa::path(
    post,
    path = "/api/auth/register",
    tag = "Auth",
    request_body = RegisterUserDto,
    responses(
        (status = 201, description = "User registered successfully", body = Response),
        (status = 400, description = "Validation error", body = ErrorResponse),
        (status = 409, description = "Email already exists", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse),
    )
)]
pub async fn register(
    Extension(app_state): Extension<Arc<AppState>>,
    Json(body): Json<RegisterUserDto>,
) -> Result<impl IntoResponse, HttpError> {
    body.validate()
        .map_err(|e| HttpError::bad_request(e.to_string()))?;

    let hash_password = password::hash(&body.password)
        .map_err(|e| HttpError::server_error(e.to_string()))?;

    let user = match app_state
        .db_client
        .save_user(body.name.clone(), body.email.clone(), hash_password)
        .await
    {
        Ok(user) => user,
        Err(sqlx::Error::Database(db_err)) if db_err.is_unique_violation() => {
            return Err(HttpError::unique_constraint_violation(
                ErrorMessage::EmailExist.to_string(),
            ));
        }
        Err(e) => return Err(HttpError::server_error(e.to_string())),
    };

    generate_key(app_state.clone(), user).await?;

    Ok((
        StatusCode::CREATED,
        Json(Response {
            message: "Registration successful!".to_string(),
            status: "success".to_string(),
        }),
    ))
}

#[utoipa::path(
    post,
    path = "/api/auth/login",
    tag = "Auth",
    request_body = LoginUserDto,
    responses(
        (status = 200, description = "Login successful; JWT returned in body and Set-Cookie header", body = UserLoginResponseDto),
        (status = 400, description = "Invalid credentials", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse),
    )
)]
pub async fn login(
    Extension(app_state): Extension<Arc<AppState>>,
    Json(body): Json<LoginUserDto>,
) -> Result<impl IntoResponse, HttpError> {
    body.validate()
        .map_err(|e| HttpError::bad_request(e.to_string()))?;

    let result = app_state
        .db_client
        .get_user(None, None, Some(&body.email))
        .await
        .map_err(|e| HttpError::server_error(e.to_string()))?;

    let user = result.ok_or_else(|| {
        HttpError::bad_request(ErrorMessage::WrongCredentials.to_string())
    })?;

    let password_matched = password::compare_password(&body.password, &user.password)
        .map_err(|e| HttpError::server_error(e.to_string()))?;

    if !password_matched {
        return Err(HttpError::bad_request(
            ErrorMessage::WrongCredentials.to_string(),
        ));
    }

    let token = token::create_token(
        &user.id.to_string(),
        app_state.env.jwt_secret.as_bytes(),
        app_state.env.jwt_max_age,
    )
    .map_err(|e| HttpError::server_error(e.to_string()))?;

    let cookie_duration = time::Duration::seconds(app_state.env.jwt_max_age);
    let cookie = Cookie::build(("token", token.clone()))
        .path("/")
        .max_age(cookie_duration)
        .http_only(true)
        .build();

    let response = Json(UserLoginResponseDto {
        status: "success".to_string(),
        token,
    });

    let mut headers = HeaderMap::new();
    headers.append(header::SET_COOKIE, cookie.to_string().parse().unwrap());

    let mut response = response.into_response();
    response.headers_mut().extend(headers);
    Ok(response)
}
