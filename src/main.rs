use anyhow::{anyhow, Result};
use clap::Parser;
use std::fs::File;
use std::io::Write;
use zip::write::FileOptions;

/// MobaXterm许可证生成器
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// 用户名
    #[arg(short, long)]
    username: String,

    /// 版本号 (例如: 10.9)
    #[arg(short, long)]
    version: String,

    /// 许可证数量
    #[arg(short, long, default_value_t = 1)]
    count: u32,

    /// 输出文件名
    #[arg(short, long, default_value = "Custom.mxtpro")]
    output: String,
}

const VARIANT_BASE64_TABLE: &[u8] =
    b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/=";

fn variant_base64_encode(bytes: &[u8]) -> String {
    let blocks_count = bytes.len() / 3;
    let left_bytes = bytes.len() % 3;
    // 预计算结果字符串长度并预分配
    let result_capacity = 4 * blocks_count
        + match left_bytes {
            0 => 0,
            1 => 2,
            2 => 3,
            _ => unreachable!(),
        };
    let mut result = String::with_capacity(result_capacity);

    // 处理完整的3字节块
    for chunk in bytes.chunks(3).take(blocks_count) {
        let coding_int = u32::from_le_bytes([chunk[0], chunk[1], chunk[2], 0]);
        result.extend([
            VARIANT_BASE64_TABLE[(coding_int & 0x3f) as usize] as char,
            VARIANT_BASE64_TABLE[((coding_int >> 6) & 0x3f) as usize] as char,
            VARIANT_BASE64_TABLE[((coding_int >> 12) & 0x3f) as usize] as char,
            VARIANT_BASE64_TABLE[((coding_int >> 18) & 0x3f) as usize] as char,
        ]);
    }

    // 处理剩余字节
    if left_bytes > 0 {
        let remaining = &bytes[3 * blocks_count..];
        let coding_int = match left_bytes {
            1 => u32::from_le_bytes([remaining[0], 0, 0, 0]),
            2 => u32::from_le_bytes([remaining[0], remaining[1], 0, 0]),
            _ => unreachable!(),
        };

        result.push(VARIANT_BASE64_TABLE[(coding_int & 0x3f) as usize] as char);
        result.push(VARIANT_BASE64_TABLE[((coding_int >> 6) & 0x3f) as usize] as char);
        if left_bytes == 2 {
            result.push(VARIANT_BASE64_TABLE[((coding_int >> 12) & 0x3f) as usize] as char);
        }
    }

    result
}

fn encrypt_bytes(key: u32, bytes: &[u8]) -> Vec<u8> {
    let mut result = Vec::with_capacity(bytes.len());
    let mut current_key = key;

    result.extend(bytes.iter().map(|&byte| {
        let encrypted = byte ^ ((current_key >> 8) & 0xff) as u8;
        current_key = (encrypted as u32 & current_key) | 0x482D;
        encrypted
    }));

    result
}

fn generate_license(username: &str, version: &str, count: u32) -> Result<String> {
    let mut version_parts = version.split('.');
    let (major_version, minor_version) = match (version_parts.next(), version_parts.next()) {
        (Some(major), Some(minor)) if version_parts.next().is_none() => (major, minor),
        _ => return Err(anyhow!("版本号格式无效，应为 'x.y' 格式")),
    };

    let license_string = format!(
        "1#{username}|{major_version}{minor_version}#{count}#{major_version}3{minor_version}6{minor_version}#0#0#0#"
    );

    let encrypted_bytes = encrypt_bytes(0x787, license_string.as_bytes());
    Ok(variant_base64_encode(&encrypted_bytes))
}

fn create_zip_file(content: &str, output_path: &str) -> Result<()> {
    let file = File::create(output_path)?;
    let mut zip = zip::ZipWriter::new(file);

    let options: FileOptions<()> = FileOptions::default()
        .compression_method(zip::CompressionMethod::Stored)
        .unix_permissions(0o644);

    zip.start_file("Pro.key", options)?;
    zip.write_all(content.as_bytes())?;
    zip.finish()?;

    Ok(())
}

fn main() -> Result<()> {
    let args = Args::parse();

    let license_content = generate_license(&args.username, &args.version, args.count)?;
    create_zip_file(&license_content, &args.output)?;

    println!("许可证文件已生成: {}", args.output);
    println!("请将生成的文件移动或复制到MobaXterm的安装路径。");

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Read;

    #[test]
    fn test_generate_license() {
        // 测试基本许可证生成
        let license_content = generate_license("test", "10.9", 1).unwrap();
        assert!(!license_content.is_empty());

        // 测试ZIP文件生成
        let tmp_file = tempfile::NamedTempFile::new().unwrap();
        let tmp_file_path = tmp_file.path().to_str().unwrap();

        create_zip_file(&license_content, tmp_file_path).unwrap();

        // 验证文件存在和大小
        let metadata = std::fs::metadata(tmp_file_path).unwrap();
        assert!(metadata.len() > 0);

        // 验证ZIP文件结构
        let file = std::fs::File::open(tmp_file_path).unwrap();
        let mut archive = zip::ZipArchive::new(file).unwrap();

        // 确保只有一个文件
        assert_eq!(archive.len(), 1);

        // 验证文件名是否正确
        let mut zip_file = archive.by_index(0).unwrap();
        assert_eq!(zip_file.name(), "Pro.key");

        // 验证文件内容
        let mut contents = String::new();
        zip_file.read_to_string(&mut contents).unwrap();
        assert_eq!(contents, license_content);
    }

    #[test]
    fn test_generate_license_with_different_params() {
        // 测试不同用户名
        let license1 = generate_license("user1", "10.9", 1).unwrap();
        let license2 = generate_license("user2", "10.9", 1).unwrap();
        assert_ne!(license1, license2);

        // 测试不同版本号
        let license3 = generate_license("test", "10.9", 1).unwrap();
        let license4 = generate_license("test", "11.0", 1).unwrap();
        assert_ne!(license3, license4);

        // 测试不同数量
        let license5 = generate_license("test", "10.9", 1).unwrap();
        let license6 = generate_license("test", "10.9", 2).unwrap();
        assert_ne!(license5, license6);
    }

    #[test]
    fn test_invalid_version_format() {
        // 测试无效的版本号格式
        assert!(generate_license("test", "10", 1).is_err());
        assert!(generate_license("test", "10.9.1", 1).is_err());
        assert!(generate_license("test", "invalid", 1).is_err());
    }
}
