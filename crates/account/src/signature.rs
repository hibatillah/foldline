use windows::Win32::Security::Cryptography::{
    BCRYPT_ECCPUBLIC_BLOB, BCRYPT_ECDSA_P256_ALG_HANDLE, BCRYPT_ECDSA_PUBLIC_P256_MAGIC, BCRYPT_FLAGS,
    BCRYPT_KEY_HANDLE, BCRYPT_SHA256_ALG_HANDLE, BCryptDestroyKey, BCryptHash, BCryptImportKeyPair,
    BCryptVerifySignature,
};

/// Verifies an ECDSA P-256 / SHA-256 signature (raw r||s) over `message`; any failure is `false`.
pub fn verify(public_key: &[u8; 64], message: &[u8], signature: &[u8; 64]) -> bool {
    verify_checked(public_key, message, signature).unwrap_or(false)
}

fn verify_checked(public_key: &[u8; 64], message: &[u8], signature: &[u8; 64]) -> windows::core::Result<bool> {
    let mut digest = [0u8; 32];
    // SAFETY: both buffers are live slices for the duration of the call.
    unsafe { BCryptHash(BCRYPT_SHA256_ALG_HANDLE, None, message, &mut digest).ok()? };

    let mut blob = Vec::with_capacity(8 + 64);
    blob.extend_from_slice(&BCRYPT_ECDSA_PUBLIC_P256_MAGIC.to_le_bytes());
    blob.extend_from_slice(&32u32.to_le_bytes());
    blob.extend_from_slice(public_key);

    let mut key = BCRYPT_KEY_HANDLE::default();
    // SAFETY: `blob` is a well-formed ECC public blob and `key` receives the new handle.
    unsafe { BCryptImportKeyPair(BCRYPT_ECDSA_P256_ALG_HANDLE, None, BCRYPT_ECCPUBLIC_BLOB, &mut key, &blob, 0).ok()? };
    // SAFETY: `key` is the valid handle imported above; a mismatching signature returns an error status.
    let verified = unsafe { BCryptVerifySignature(key, None, &digest, signature, BCRYPT_FLAGS(0)) }.is_ok();
    // SAFETY: `key` is destroyed exactly once.
    unsafe { BCryptDestroyKey(key).ok()? };
    Ok(verified)
}
