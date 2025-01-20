use base64::{engine::general_purpose::STANDARD as BASE64, Engine as _};
use rand::Rng;
use sha2::{Digest, Sha256};

fn generate_hash() -> String {
    let random_bytes = rand::thread_rng().gen::<[u8; 32]>();
    let mut hasher = Sha256::new();
    hasher.update(random_bytes);
    hex::encode(hasher.finalize())
}

fn obfuscate_bytes(bytes: &mut [u8]) {
    let mut prev: u8 = 165;
    for (idx, byte) in bytes.iter_mut().enumerate() {
        let old_value = *byte;
        *byte = (old_value ^ prev).wrapping_add((idx % 256) as u8);
        prev = *byte;
    }
}

fn deobfuscate_bytes(bytes: &mut [u8]) {
    let mut prev: u8 = 165;
    for (idx, byte) in bytes.iter_mut().enumerate() {
        let temp = *byte;
        *byte = (*byte).wrapping_sub((idx % 256) as u8) ^ prev;
        prev = temp;
    }
}

fn generate_checksum(device_id: &str, mac_addr: Option<&str>) -> String {
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs()
        / 1_000;

    let mut timestamp_bytes = vec![
        ((timestamp >> 8) & 0xFF) as u8,
        (0xFF & timestamp) as u8,
        ((timestamp >> 24) & 0xFF) as u8,
        ((timestamp >> 16) & 0xFF) as u8,
        ((timestamp >> 8) & 0xFF) as u8,
        (0xFF & timestamp) as u8,
    ];

    obfuscate_bytes(&mut timestamp_bytes);
    let encoded = BASE64.encode(&timestamp_bytes);

    match mac_addr {
        Some(mac) => format!("{}{}/{}", encoded, device_id, mac),
        None => format!("{}{}", encoded, device_id),
    }
}

pub fn generate_checksum_with_default() -> String {
    generate_checksum(&generate_hash(), Some(&generate_hash()))
}

pub fn generate_checksum_with_repair(bad_checksum: &str) -> String {
    // 如果是空字符串，直接返回默认值
    if bad_checksum.is_empty() {
        return generate_checksum_with_default();
    }

    // 尝试修复时间戳头的函数
    fn try_fix_timestamp(timestamp_base64: &str) -> Option<String> {
        if let Ok(timestamp_bytes) = BASE64.decode(timestamp_base64) {
            if timestamp_bytes.len() == 6 {
                let mut fixed_bytes = timestamp_bytes.clone();
                deobfuscate_bytes(&mut fixed_bytes);

                // 检查前3位是否为0
                if fixed_bytes[0..3].iter().all(|&x| x == 0) {
                    // 从后四位构建时间戳
                    let timestamp = ((fixed_bytes[2] as u64) << 24)
                        | ((fixed_bytes[3] as u64) << 16)
                        | ((fixed_bytes[4] as u64) << 8)
                        | (fixed_bytes[5] as u64);

                    let current_timestamp = std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap()
                        .as_secs()
                        / 1_000;

                    if timestamp <= current_timestamp {
                        // 修复时间戳字节
                        fixed_bytes[0] = fixed_bytes[4];
                        fixed_bytes[1] = fixed_bytes[5];

                        obfuscate_bytes(&mut fixed_bytes);
                        return Some(BASE64.encode(&fixed_bytes));
                    }
                }
            }
        }
        None
    }

    if bad_checksum.len() == 8 {
        // 尝试修复时间戳头
        if let Some(fixed_timestamp) = try_fix_timestamp(bad_checksum) {
            return format!("{}{}/{}", fixed_timestamp, generate_hash(), generate_hash());
        }

        // 验证原始时间戳
        if let Some(timestamp) = extract_time_ks(bad_checksum) {
            let current_timestamp = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs()
                / 1_000;

            if timestamp <= current_timestamp {
                return format!("{}{}/{}", bad_checksum, generate_hash(), generate_hash());
            }
        }
    } else if bad_checksum.len() > 8 {
        // 处理可能包含hash的情况
        let parts: Vec<&str> = bad_checksum.split('/').collect();
        match parts.len() {
            1 => {
                let timestamp_base64 = &bad_checksum[..8];
                let device_id = &bad_checksum[8..];

                if is_valid_hash(device_id) {
                    // 先尝试修复时间戳
                    if let Some(fixed_timestamp) = try_fix_timestamp(timestamp_base64) {
                        return format!("{}{}/{}", fixed_timestamp, device_id, generate_hash());
                    }

                    // 验证原始时间戳
                    if let Some(timestamp) = extract_time_ks(timestamp_base64) {
                        let current_timestamp = std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH)
                            .unwrap()
                            .as_secs()
                            / 1_000;

                        if timestamp <= current_timestamp {
                            return format!(
                                "{}{}/{}",
                                timestamp_base64,
                                device_id,
                                generate_hash()
                            );
                        }
                    }
                }
            }
            2 => {
                let first_part = parts[0];
                let mac_hash = parts[1];

                if is_valid_hash(mac_hash) && first_part.len() == mac_hash.len() + 8 {
                    let timestamp_base64 = &first_part[..8];
                    let device_id = &first_part[8..];

                    if is_valid_hash(device_id) {
                        // 先尝试修复时间戳
                        if let Some(fixed_timestamp) = try_fix_timestamp(timestamp_base64) {
                            return format!("{}{}/{}", fixed_timestamp, device_id, mac_hash);
                        }

                        // 验证原始时间戳
                        if let Some(timestamp) = extract_time_ks(timestamp_base64) {
                            let current_timestamp = std::time::SystemTime::now()
                                .duration_since(std::time::UNIX_EPOCH)
                                .unwrap()
                                .as_secs()
                                / 1_000;

                            if timestamp <= current_timestamp {
                                return bad_checksum.to_string();
                            }
                        }
                    }
                }
            }
            _ => {}
        }
    }

    // 如果所有修复尝试都失败，返回默认值
    generate_checksum_with_default()
}

pub fn extract_time_ks(timestamp_base64: &str) -> Option<u64> {
    let mut timestamp_bytes = BASE64.decode(timestamp_base64).ok()?;

    if timestamp_bytes.len() != 6 {
        return None;
    }

    deobfuscate_bytes(&mut timestamp_bytes);

    if timestamp_bytes[0] != timestamp_bytes[4] || timestamp_bytes[1] != timestamp_bytes[5] {
        return None;
    }

    // 使用后四位还原 timestamp
    Some(
        ((timestamp_bytes[2] as u64) << 24)
            | ((timestamp_bytes[3] as u64) << 16)
            | ((timestamp_bytes[4] as u64) << 8)
            | (timestamp_bytes[5] as u64),
    )
}

pub fn validate_checksum(checksum: &str) -> bool {
    if checksum.is_empty() {
        return false;
    }
    // 首先检查是否包含基本的 base64 编码部分和 hash 格式的 device_id
    let parts: Vec<&str> = checksum.split('/').collect();

    match parts.len() {
        // 没有 MAC 地址的情况
        1 => {
            if checksum.len() < 72 {
                // 8 + 64 = 72
                return false;
            }

            // 解码前8个字符的base64时间戳
            let timestamp_base64 = &checksum[..8];
            let timestamp = match extract_time_ks(timestamp_base64) {
                Some(ts) => ts,
                None => return false,
            };

            let current_timestamp = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs()
                / 1_000;

            if current_timestamp < timestamp {
                return false;
            }

            // 验证 device_id hash 部分
            is_valid_hash(&checksum[8..])
        }
        // 包含 MAC hash 的情况
        2 => {
            let first_part = parts[0];
            let mac_hash = parts[1];

            // MAC hash 必须是64字符的十六进制
            if !is_valid_hash(mac_hash) {
                return false;
            }

            // 检查第一部分比MAC hash多8个字符
            if first_part.len() != mac_hash.len() + 8 {
                return false;
            }

            // 递归验证第一部分
            validate_checksum(first_part)
        }
        _ => false,
    }
}

fn is_valid_hash(hash: &str) -> bool {
    if hash.len() < 64 {
        return false;
    }

    // 检查是否都是有效的十六进制字符
    hash.chars().all(|c| c.is_ascii_hexdigit())
}
