use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Nonce,
};
use rsa::{Pkcs1v15Encrypt, RsaPrivateKey};
use crate::error::HttpError;

pub async fn decrypt_file(
    encrypted_aes_key:Vec<u8>,
    encrypted_file_data:Vec<u8>,
    nonce: Vec<u8>,
    user_private_key:&RsaPrivateKey,
)->Result<Vec<u8>, HttpError> {
    let aes_key = user_private_key
        .decrypt(Pkcs1v15Encrypt, &encrypted_aes_key,)
        .map_err(|e| HttpError::server_error(e.to_string()))?;
    if aes_key.len() != 32 {
        return Err(HttpError::server_error(
            "Invalid AES-256 length".to_string(),
        ));
    }
    if nonce.len() != 12 {
        return Err(HttpError::server_error(
            "Invalid AES-GCM nonce length".to_string(),
        ));
    }
    let mut nonce_bytes = [0u8; 12];
    nonce_bytes.copy_from_slice(&nonce);
    let nonce = Nonce::from(nonce_bytes);
    let cipher = Aes256Gcm::new_from_slice(&aes_key)
        .map_err(|e| HttpError::server_error(e.to_string()))?;
    let decrypted_data = cipher
        .decrypt(&nonce, encrypted_file_data.as_ref())
        .map_err(|e| HttpError::server_error(e.to_string()))?;
    Ok(decrypted_data)
}