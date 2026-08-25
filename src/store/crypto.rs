use aes_gcm::aead::{Aead, KeyInit, Payload};
use aes_gcm::{Aes256Gcm, Nonce};
use rand::RngCore;
use sha2::{Digest, Sha256};

pub const FILE_MAGIC: &[u8; 4] = b"JSR1";
pub const JSB_MAGIC: &[u8; 4] = b"JSB1";

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

/// 从密码派生密钥（Scrypt），用于 .jsb 加密备份
pub fn derive_key_from_password(password: &str, salt: &[u8]) -> Result<[u8; 32], String> {
    let params = scrypt::Params::new(15, 8, 1, 32).map_err(|e| e.to_string())?;
    let mut key = [0u8; 32];
    scrypt::scrypt(password.as_bytes(), salt, &params, &mut key).map_err(|e| e.to_string())?;
    Ok(key)
}

/// .jsb 备份格式：magic(4) + ver(1) + salt(16) + nonce(12) + AES-256-GCM 密文(zlib 压缩)
pub fn encrypt_jsb(password: &str, plain: &[u8]) -> Result<Vec<u8>, String> {
    let mut salt = [0u8; 16];
    rand::thread_rng().fill_bytes(&mut salt);
    let key = derive_key_from_password(password, &salt)?;
    let mut enc = flate2::write::ZlibEncoder::new(Vec::new(), flate2::Compression::default());
    std::io::Write::write_all(&mut enc, plain).map_err(|e| e.to_string())?;
    let compressed = enc.finish().map_err(|e| e.to_string())?;
    let cipher = Aes256Gcm::new_from_slice(&key).map_err(|e| e.to_string())?;
    let mut nonce_bytes = [0u8; 12];
    rand::thread_rng().fill_bytes(&mut nonce_bytes);
    let ct = cipher
        .encrypt(Nonce::from_slice(&nonce_bytes), compressed.as_slice())
        .map_err(|e| format!("加密失败: {}", e))?;
    let mut out = Vec::with_capacity(33 + ct.len());
    out.extend_from_slice(JSB_MAGIC);
    out.push(1u8);
    out.extend_from_slice(&salt);
    out.extend_from_slice(&nonce_bytes);
    out.extend_from_slice(&ct);
    Ok(out)
}

pub fn decrypt_jsb(password: &str, data: &[u8]) -> Result<Vec<u8>, String> {
    if data.len() < 33 || &data[0..4] != JSB_MAGIC {
        return Err("不是有效的 .jsb 加密备份文件".into());
    }
    let salt = &data[5..21];
    let nonce = Nonce::from_slice(&data[21..33]);
    let key = derive_key_from_password(password, salt)?;
    let cipher = Aes256Gcm::new_from_slice(&key).map_err(|e| e.to_string())?;
    let compressed = cipher
        .decrypt(nonce, &data[33..])
        .map_err(|_| "密码错误或文件已损坏".to_string())?;
    let mut dec = flate2::read::ZlibDecoder::new(compressed.as_slice());
    let mut out = Vec::new();
    std::io::Read::read_to_end(&mut dec, &mut out).map_err(|e| e.to_string())?;
    Ok(out)
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

    #[test]
    fn jsb_password_roundtrip() {
        let key = generate_key();
        let plain = "{\"novel\":{\"title\":\"剑出昆仑\"}}".as_bytes();
        let blob = encrypt_jsb("我的密码123", plain).unwrap();
        assert_eq!(decrypt_jsb("我的密码123", &blob).unwrap(), plain);
        assert!(decrypt_jsb("错误密码", &blob).is_err());
        let _ = key;
    }
}
