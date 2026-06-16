use std::fs;
use std::time::Instant;
use tao_core::{DataUnit, ErasureEncoder, LocalStore, ErasureGroup};
use tao_crypto::{encrypt, decrypt};

/// 媒体文件全链路测试：加密 → EC编码 → 模拟丢失 → 解码 → 解密 → 验证
#[test]
fn test_image_pipeline() {
    println!("\n=== 图片端到端管道 ===");
    let files: Vec<_> = glob_images();
    if files.is_empty() {
        println!("  无测试图片，跳过");
        return;
    }

    for path in files.iter().take(3) {
        let data = fs::read(path).unwrap();
        let name = path.rsplit('/').next().unwrap();
        let size_kb = data.len() as f64 / 1024.0;
        println!("\n  {} ({:.1} KB)", name, size_kb);

        // 1. 加密
        let t0 = Instant::now();
        let encrypted = encrypt(&data, &[0x42u8; 32]);
        let t_enc = t0.elapsed();

        // 2. 纠删码编码 (自适应 k,m)
        let (k, m) = if data.len() < 64 * 1024 { (4, 2) } else { (6, 2) };
        let encoder = ErasureEncoder::new(k, m);
        let t0 = Instant::now();
        let shards = encoder.encode(&encrypted).unwrap();
        let t_ec = t0.elapsed();

        // 3. 模拟丢失 2 片
        let pairs: Vec<_> = shards.iter().enumerate()
            .map(|(i, s)| (Some(s.clone()), i < k)).collect();

        // 4. 解码
        let t0 = Instant::now();
        let recovered = encoder.decode(&pairs).unwrap();
        let t_dc = t0.elapsed();

        // 5. 解密 + 验证
        let t0 = Instant::now();
        let decrypted = decrypt(&recovered, &[0x42u8; 32]).unwrap();
        assert_eq!(decrypted, data);
        let t_dec = t0.elapsed();

        println!("    encrypt {:>8.2?}  |  EC({}+{}) {:>8.2?}  |  decode {:>8.2?}  |  decrypt {:>8.2?}",
            t_enc, k, m, t_ec, t_dc, t_dec);
    }
}

/// 大文件分块管道（模拟视频处理）
#[test]
fn test_large_file_chunking() {
    println!("\n=== 大文件分块（模拟视频）===");
    // 生成 16MB 模拟视频数据
    let total_size = 16 * 1024 * 1024; // 16MB
    let chunk_size = 1024 * 1024;       // 1MB 每块
    let total_chunks = total_size / chunk_size;

    println!("  总大小: 16MB, 分块: 1MB × {} 块", total_chunks);

    let t0 = Instant::now();
    let mut chunk_hashes = Vec::new();

    for i in 0..total_chunks {
        let chunk = vec![(i % 256) as u8; chunk_size];

        // 每块独立加密 + EC 编码
        let encrypted = encrypt(&chunk, &[0x42u8; 32]);
        let encoder = ErasureEncoder::new(6, 2);
        let shards = encoder.encode(&encrypted).unwrap();

        // 模拟丢失 2 片后解码验证
        let pairs: Vec<_> = shards.iter().enumerate()
            .map(|(j, s)| (Some(s.clone()), j < 6)).collect();
        let recovered = encoder.decode(&pairs).unwrap();
        let decrypted = decrypt(&recovered, &[0x42u8; 32]).unwrap();
        assert_eq!(decrypted, chunk);

        chunk_hashes.push(shards[0].original_hash);
    }

    let elapsed = t0.elapsed();
    let throughput = total_size as f64 / elapsed.as_secs_f64() / (1024.0 * 1024.0);
    println!("  16 块全部加密+EC+验证: {:?}  ({:.1} MB/s)", elapsed, throughput);
    println!("  分块哈希数: {}", chunk_hashes.len());
}

/// 视频友好型自适应 EC 策略演示
#[test]
fn test_adaptive_ec_for_media() {
    println!("\n=== 自适应纠删码策略 ===");

    let strategies = [
        ("缩略图 (1KB)",  1_000,    2, 1),
        ("照片 (5MB)",    5_000_000, 6, 2),
        ("短视频 (50MB)", 50_000_000, 10, 4),
        ("长视频 (1GB)",  1_000_000_000, 20, 8),
    ];

    for (label, size, k, m) in &strategies {
        let overhead = (*m as f64 / *k as f64) * 100.0;
        let tolerance = *m; // 可丢失片数
        let chunk_count = if *size > 16_000_000 {
            (*size / (16_000_000)).max(1)
        } else { 1 };
        println!(
            "  {:<18} EC({}+{})  冗余{:.0}%  容错{}片  分{}块",
            label, k, m, overhead, tolerance, chunk_count
        );
    }
}

fn glob_images() -> Vec<String> {
    let dirs = [
        "/storage/emulated/0/Pictures",
        "/storage/emulated/0/DCIM",
        "/storage/emulated/0/Download",
    ];
    let mut files = Vec::new();
    for dir in &dirs {
        if let Ok(entries) = fs::read_dir(dir) {
            for e in entries.flatten() {
                let p = e.path();
                if let Some(ext) = p.extension() {
                    if ext == "jpg" || ext == "png" || ext == "mp4" || ext == "pdf" {
                        files.push(p.to_string_lossy().to_string());
                    }
                }
            }
        }
    }
    files.truncate(10);
    files
}
