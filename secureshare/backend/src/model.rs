use serde::{Deserialize, Serialize};

// 1. FIXED SYNTAX ERRORS: Changed `#[derive{...}]` to `#[derive(...)]`
// 2. FIXED TYPES: Changed `Datetime<Utc>` to `chrono::DateTime<chrono::Utc>`
// 3. FIXED COLON: Changed `sqlx:FromRow` to `sqlx::FromRow`

#[derive(Debug, Clone, Deserialize, Serialize, sqlx::FromRow)]
pub struct User {
    pub id: uuid::Uuid, // Added missing colon
    pub name: String,
    pub email: String,
    pub password: String,
    
    // 💡 WHY Option IS USED HERE:
    // This field can be empty (NULL) in the database. 
    // Rust uses `Option<T>` to safely handle values that might be missing,
    // forcing you to handle the `Some` or `None` cases explicitly.
    pub public_key: Option<String>,
    
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
    pub updated_at: Option<chrono::DateTime<chrono::Utc>>,
}

// 💡 WHY `sqlx::Type` WAS REMOVED:
// `sqlx::Type` is used for custom SQL enums or single-value types, NOT for tables.
// Applying it to an entire struct representing a database row will cause a compilation error.
#[derive(Debug, Clone, Deserialize, Serialize, sqlx::FromRow)]
pub struct File {
    pub id: uuid::Uuid,
    pub user_id: Option<uuid::Uuid>,
    pub file_name: String,
    pub file_size: i64,
    pub encrypted_aes_key: Vec<u8>,
    pub encrypted_file: Vec<u8>,
    pub iv: Vec<u8>,
    pub created_at: Option<chrono::DateTime<chrono::Utc>>
}

#[derive(Debug, Clone, Deserialize, Serialize, sqlx::FromRow)]
pub struct ShareLink {
    pub id: uuid::Uuid,
    pub file_id: Option<uuid::Uuid>,
    pub recipient_user_id: Option<uuid::Uuid>,
    pub password: String,
    pub expiration_date: Option<chrono::DateTime<chrono::Utc>>,
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
}

// 💡 WHY `sqlx::FromRow` IS USED HERE:
// This struct maps custom query results (like a database JOIN operation).
// If a SQL query selects `file_id`, `file_name`, and `recipient_email`,
// SQLx will automatically bind those columns into this struct.
#[derive(sqlx::FromRow)]
pub struct SendFileDetails {
    pub file_id: uuid::Uuid,
    pub file_name: String,
    pub recipient_email: String,
    pub expiration_date: Option<chrono::DateTime<chrono::Utc>>,
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(sqlx::FromRow)]
pub struct ReceiveFileDetails {
    pub file_id: uuid::Uuid,
    pub file_name: String,
    pub sender_email: String,
    pub expiration_date: Option<chrono::DateTime<chrono::Utc>>,
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
}
