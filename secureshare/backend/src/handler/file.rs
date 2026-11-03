use std::{fs, path::PathBuf, sync::Arc};

use axum::{
    Extension, Json, Router,
    body::Body,
    extract::Multipart,
    http::{Response, StatusCode},
    response::IntoResponse,
    routing::post,
};
use base64::{engine::general_purpose::STANDARD, Engine};
use chrono::{DateTime, Utc};
use rsa::{
    pkcs1::{DecodeRsaPrivateKey, DecodeRsaPublicKey},
    RsaPrivateKey, RsaPublicKey,
};
use validator::Validate;

use crate::{
    AppState,
    db::UserExt,
    dto::{FileUploadDtos, FileUploadForm, Response as ApiResponse, RetrieveFileDto},
    error::{ErrorResponse, HttpError},
    middleware::JWTAuthMiddleware,
    utils::{decrypt::decrypt_file, encrypt::encrypt_file, password},
};

pub fn file_handle() -> Router {
    Router::new()
        .route("/upload", post(upload_file))
        .route("/retrieve", post(retrieve_file))
}

#[utoipa::path(
    post,
    path = "/api/file/upload",
    tag = "Files",
    security(("bearer_auth" = [])),
    request_body(content = FileUploadForm, content_type = "multipart/form-data", description = "Multipart form with file and share metadata"),
    responses(
        (status = 200, description = "File uploaded and encrypted successfully", body = crate::dto::Response),
        (status = 400, description = "Validation error or recipient not found", body = ErrorResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse),
    )
)]
pub async fn upload_file(
    Extension(app_state): Extension<Arc<AppState>>,
    Extension(user): Extension<JWTAuthMiddleware>,
    mut multipart: Multipart,
) -> Result<impl IntoResponse, HttpError> {
    let mut file_data = Vec::new();
    let mut file_name = String::new();
    let mut file_size: i64 = 0;
    let mut form_data = FileUploadDtos {
        recipient_email: String::new(),
        password: String::new(),
        expiration_date: String::new(),
    };

    while let Some(field) = multipart.next_field().await.map_err(|e| {
        HttpError::bad_request(e.to_string())
    })? {
        let name = field
            .name()
            .ok_or_else(|| HttpError::bad_request("Missing field name".to_string()))?
            .to_string();

        match name.as_str() {
            "file_upload" => {
                file_name = field
                    .file_name()
                    .unwrap_or("unknown_file")
                    .to_string();
                file_data = field
                    .bytes()
                    .await
                    .map_err(|e| HttpError::bad_request(e.to_string()))?
                    .to_vec();
                file_size = file_data.len() as i64;
            }
            "recipient_email" => {
                form_data.recipient_email = field
                    .text()
                    .await
                    .map_err(|e| HttpError::bad_request(e.to_string()))?;
            }
            "password" => {
                form_data.password = field
                    .text()
                    .await
                    .map_err(|e| HttpError::bad_request(e.to_string()))?;
            }
            "expiration_date" => {
                form_data.expiration_date = field
                    .text()
                    .await
                    .map_err(|e| HttpError::bad_request(e.to_string()))?;
            }
            _ => {}
        }
    }

    form_data
        .validate()
        .map_err(|e| HttpError::bad_request(e.to_string()))?;

    let recipient_result = app_state
        .db_client
        .get_user(None, None, Some(&form_data.recipient_email))
        .await
        .map_err(|e| HttpError::server_error(e.to_string()))?;

    let recipient_user = recipient_result.ok_or_else(|| {
        HttpError::bad_request("Recipient user not found".to_string())
    })?;

    let public_key_str = match &recipient_user.public_key {
        Some(key) => key,
        None => return Err(HttpError::bad_request("Recipient user has no public_key".to_string())),
    };

    let public_key_bytes = STANDARD
        .decode(public_key_str)
        .map_err(|e| HttpError::server_error(e.to_string()))?;
    let public_key = String::from_utf8(public_key_bytes)
        .map_err(|e| HttpError::server_error(e.to_string()))?;
    let public_key_pem = RsaPublicKey::from_pkcs1_pem(&public_key)
        .map_err(|e| HttpError::server_error(e.to_string()))?;

    let (encrypted_aes_key, encrypted_data, iv) =
        encrypt_file(file_data, &public_key_pem).await?;

    let user_id = user.user.id;
    let hash_password = password::hash(&form_data.password)
        .map_err(|e| HttpError::server_error(e.to_string()))?;
    let expiration_date = DateTime::parse_from_rfc3339(&form_data.expiration_date)
        .map_err(|e| HttpError::bad_request(e.to_string()))?
        .with_timezone(&Utc);
    let recipient_user_id = recipient_user.id;

    app_state
        .db_client
        .save_encrypted_file(
            user_id,
            file_name,
            file_size,
            recipient_user_id,
            hash_password,
            expiration_date,
            encrypted_aes_key,
            encrypted_data,
            iv,
        )
        .await
        .map_err(|e| HttpError::server_error(e.to_string()))?;

    let response = ApiResponse {
        message: "File uploaded and encrypted successfully".to_string(),
        status: "success".to_string(),
    };
    Ok(Json(response))
}

#[utoipa::path(
    post,
    path = "/api/file/retrieve",
    tag = "Files",
    security(("bearer_auth" = [])),
    request_body = RetrieveFileDto,
    responses(
        (status = 200, description = "Decrypted file download", content_type = "application/octet-stream"),
        (status = 400, description = "Invalid shared link or password", body = ErrorResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse),
    )
)]
pub async fn retrieve_file(
    Extension(app_state): Extension<Arc<AppState>>,
    Extension(user): Extension<JWTAuthMiddleware>,
    Json(body): Json<RetrieveFileDto>,
) -> Result<impl IntoResponse, HttpError> {
    body.validate()
        .map_err(|e| HttpError::bad_request(e.to_string()))?;

    let user_id = user.user.id;
    let shared_id = uuid::Uuid::parse_str(&body.shared_id)
        .map_err(|e| HttpError::bad_request(e.to_string()))?;

    let shared_result = app_state
        .db_client
        .get_shared(shared_id, user_id)
        .await
        .map_err(|e| HttpError::server_error(e.to_string()))?;

    let shared_data = shared_result.ok_or_else(|| {
        HttpError::bad_request(
            "The requested shared link either does not exist or has expired.".to_string(),
        )
    })?;

    let match_password = password::compare_password(&body.password, &shared_data.password)
        .map_err(|e| HttpError::server_error(e.to_string()))?;

    if !match_password {
        return Err(HttpError::bad_request(
            "The provided password is incorrect.".to_string(),
        ));
    }

    let file_id = match shared_data.file_id {
        Some(id) => id,
        None => {
            return Err(HttpError::bad_request("File ID is missing".to_string()));
        }
    };

    let file_data = app_state
        .db_client
        .get_file(file_id)
        .await
        .map_err(|e| HttpError::server_error(e.to_string()))?
        .ok_or_else(|| {
            HttpError::bad_request(
                "The requested file either does not exist or has expired.".to_string(),
            )
        })?;

    let mut path = PathBuf::from("assets/private_keys");
    path.push(format!("{}.pem", user_id));

    let private_key = fs::read_to_string(&path)
        .map_err(|e| HttpError::server_error(e.to_string()))?;
    let private_key_pem = RsaPrivateKey::from_pkcs1_pem(&private_key)
        .map_err(|e| HttpError::server_error(e.to_string()))?;

    let decrypted_file = decrypt_file(
        file_data.encrypted_aes_key,
        file_data.encrypted_file,
        file_data.iv,
        &private_key_pem,
    )
    .await?;

    let response = Response::builder()
        .status(StatusCode::OK)
        .header(
            "Content-Disposition",
            format!("attachment; filename=\"{}\"", file_data.file_name),
        )
        .header("Content-Type", "application/octet-stream")
        .body(Body::from(decrypted_file))
        .map_err(|e| HttpError::server_error(e.to_string()))?;

    Ok(response)
}
