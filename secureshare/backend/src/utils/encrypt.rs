use aes_gcm::{
    aead::{Aead,KeyInit},
    Aes256Gcm, Nonce,
};
use rand::{RngCore, rngs::OsRng};
use rsa::{Pkcs1v15Encrypt, RsaPublicKey};

use crate::error::HttpError;

pub async fn encrypt_file(
    file_data: Vec<u8>,
    user_public_key: &RsaPublicKey
) -> Result<(Vec<u8>,Vec<u8>,Vec<u8>), HttpError> {
    let mut aes_key = [0u8;32];
    let mut nonce_bytes = [0u8;12];
    OsRng.fill_bytes(&mut aes_key);
    OsRng.fill_bytes(&mut nonce_bytes);
    let cipher = Aes256Gcm::new_from_slice(&aes_key)
        .map_err(|e| HttpError::server_error(e.to_string()))?;
    let nonce = Nonce::from(nonce_bytes);
    let encrypted_data = cipher
        .encrypt(&nonce, file_data.as_ref())
        .map_err(|e| HttpError::server_error(e.to_string()))?;
    let encrypted_aes_key = user_public_key
        .encrypt(&mut OsRng,
             Pkcs1v15Encrypt, &aes_key,
            )
            .map_err(|e| HttpError::server_error(e.to_string()))?;
    Ok((
        encrypted_aes_key,
        encrypted_data,
        nonce_bytes.to_vec(),
    ))

}