use aes_gcm::aead::{Aead, KeyInit, Payload};
use aes_gcm::{Aes256Gcm, Nonce};
use rand::RngCore;
use sha2::{Digest, Sha256};

pub const FILE_MAGIC: &[u8; 4] = b"JSR1";

/// 本地文件格式：magic(4) + ver(1) + nonce(12) + AES-256-GCM 密文
pub fn encrypt_bytes(key: &[u8; 32], aad: &[u8], plain: &[u8]) -> Result<Vec<u8>, String> {
    let cipher = Aes256Gcm::new_from_slice(key).map_err(|e| e.to_string())?;
    let mut nonce_bytes = [0u8; 12];
    rand::thread_rng().fill_bytes(&mut nonce_bytes);
    let ct = cipher
        .encrypt(
            Nonce::from_slice(&nonce_bytes),
            Payload { msg: plain, aad },
        )
        .map_err(|e| format!("加密失败: {}", e))?;
    let mut out = Vec::with_capacity(17 + ct.len());
    out.extend_from_slice(FILE_MAGIC);
    out.push(1u8);
    out.extend_from_slice(&nonce_bytes);
    out.extend_from_slice(&ct);
    Ok(out)
}

pub fn decrypt_bytes(key: &[u8; 32], aad: &[u8], data: &[u8]) -> Result<Vec<u8>, String> {
    if data.len() < 17 || &data[0..4] != FILE_MAGIC {
        return Err("文件格式损坏或不是锦书加密文件".into());
    }
    let cipher = Aes256Gcm::new_from_slice(key).map_err(|e| e.to_string())?;
    let nonce = Nonce::from_slice(&data[5..17]);
    let ct = &data[17..];
    cipher
        .decrypt(nonce, Payload { msg: ct, aad })
        .map_err(|_| "解密失败：密钥不匹配或文件已损坏".to_string())
}

pub fn encrypt_file(path: &std::path::Path, key: &[u8; 32], plain: &[u8]) -> Result<(), String> {
    let aad = path.file_name().and_then(|n| n.to_str()).unwrap_or("data");
    let blob = encrypt_bytes(key, aad.as_bytes(), plain)?;
    // 先写临时文件再替换，避免写一半崩溃损坏原文件
    let tmp = path.with_extension("tmp");
    std::fs::write(&tmp, &blob).map_err(|e| format!("写入文件失败: {}", e))?;
    std::fs::rename(&tmp, path).map_err(|e| format!("替换文件失败: {}", e))?;
    Ok(())
}

pub fn decrypt_file(path: &std::path::Path, key: &[u8; 32]) -> Result<Vec<u8>, String> {
    let data = std::fs::read(path).map_err(|e| format!("读取文件失败: {}", e))?;
    let aad = path.file_name().and_then(|n| n.to_str()).unwrap_or("data");
    decrypt_bytes(key, aad.as_bytes(), &data)
}

pub fn generate_key() -> [u8; 32] {
    let mut k = [0u8; 32];
    rand::thread_rng().fill_bytes(&mut k);
    k
}

pub fn fingerprint(key: &[u8; 32]) -> String {
    let mut h = Sha256::new();
    h.update(key);
    let d = h.finalize();
    d.iter().take(6).map(|b| format!("{:02X}", b)).collect::<Vec<_>>().join("")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn crypto_roundtrip() {
        let key = generate_key();
        let plain = "第一章 少年出山
他握紧了剑柄。".as_bytes();
        let blob = encrypt_bytes(&key, b"novel.jsr", plain).unwrap();
        // 密文中不应包含明文
        assert!(!blob.windows(plain.len()).any(|w| w == plain));
        let dec = decrypt_bytes(&key, b"novel.jsr", &blob).unwrap();
        assert_eq!(dec, plain);
        // 错误密钥必须失败
        let key2 = generate_key();
        assert!(decrypt_bytes(&key2, b"novel.jsr", &blob).is_err());
        // 错误 AAD 必须失败
        assert!(decrypt_bytes(&key, b"other", &blob).is_err());
    }
}
