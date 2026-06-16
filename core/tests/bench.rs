use std::time::Instant;
use tao_core::{DataUnit, ErasureEncoder, LocalStore, decide};
use tao_crypto::{encrypt, decrypt};

fn time<F>(label: &str, iterations: u32, mut f: F)
where F: FnMut() {
    let start = Instant::now();
    for _ in 0..iterations { f(); }
    let elapsed = start.elapsed();
    let per_op = elapsed / iterations;
    println!("  {:<35} {:>6} ops | {:>10.2?}/op | {:>8.0} ops/s",
        label, iterations, per_op,
        iterations as f64 / elapsed.as_secs_f64());
}

#[test]
fn bench_erasure_encode() {
    println!("\n=== 纠删码编码 ===");
    for (label, k, m, kb) in [
        ("RS(2+1)   1KB", 2, 1, 1),
        ("RS(6+2)   1KB", 6, 2, 1),
        ("RS(6+2)   1MB", 6, 2, 1024),
        ("RS(10+4)  1MB", 10, 4, 1024),
        ("RS(30+10) 1MB", 30, 10, 1024),
    ] {
        let data = vec![0x42u8; kb * 1024];
        let encoder = ErasureEncoder::new(k, m);
        let n = if kb > 100 { 5 } else { 500 };
        time(label, n, || { encoder.encode(&data).unwrap(); });
    }
}

#[test]
fn bench_erasure_decode() {
    println!("\n=== 纠删码解码 ===");
    for (label, k, m, kb) in [
        ("RS(6+2)   1KB", 6, 2, 1),
        ("RS(6+2)   1MB", 6, 2, 1024),
        ("RS(10+4)  1MB", 10, 4, 1024),
    ] {
        let data = vec![0xabu8; kb * 1024];
        let encoder = ErasureEncoder::new(k, m);
        let shards = encoder.encode(&data).unwrap();
        let pairs: Vec<_> = shards.iter().enumerate()
            .map(|(i, s)| (Some(s.clone()), i < k)).collect();
        let n = if kb > 100 { 5 } else { 500 };
        time(label, n, || { encoder.decode(&pairs).unwrap(); });
    }
}

#[test]
fn bench_encryption() {
    println!("\n=== ChaCha20-Poly1305 加密 ===");
    let key = [0x42u8; 32];
    for &kb in &[1, 1024] {
        let plain = vec![0x42u8; kb * 1024];
        let enc = encrypt(&plain, &key);
        let n = if kb > 100 { 50 } else { 5000 };
        time(&format!("encrypt {}KB", kb), n, || { encrypt(&plain, &key); });
        time(&format!("decrypt {}KB", kb), n, || { decrypt(&enc, &key); });
    }
}

#[test]
fn bench_storage() {
    println!("\n=== sled 本地存储 ===");
    let path = format!("./target/bench_{}", std::process::id());
    let store = LocalStore::open(&path).unwrap();
    let unit = DataUnit::new(vec![0x42u8; 4096], "b".into(), [0u8; 32]);
    let id = unit.yin.content_hash;

    time("write 4KB", 500, || { store.store(&unit).unwrap(); });
    time("read  4KB", 500, || { store.get(&id).unwrap(); });
    let _ = std::fs::remove_dir_all(&path);
}

#[test]
fn bench_qi() {
    println!("\n=== 气机决策 ===");
    let u = DataUnit::new(vec![0x42u8; 64], "q".into(), [0u8; 32]);
    time("decide()", 500_000, || { decide(&u.qi, 1.0); });
}
