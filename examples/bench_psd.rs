use psd_great::{read_psd, write_psd, ReadOptions, WriteOptions};
use std::env;
use std::fs;
use std::hint::black_box;
use std::io::Cursor;
use std::time::Instant;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = env::args().skip(1);
    let mode = args.next().unwrap_or_else(|| "parse".to_string());
    let path = args
        .next()
        .ok_or("usage: bench_psd <parse|write> <file> [iterations]")?;
    let iterations = args
        .next()
        .map(|value| value.parse())
        .transpose()?
        .unwrap_or(3);
    let input = fs::read(&path)?;

    let start = Instant::now();
    match mode.as_str() {
        "parse" => {
            for _ in 0..iterations {
                let psd = read_psd(Cursor::new(&input), ReadOptions::default())?;
                black_box(psd);
            }
        }
        "write" => {
            let psd = read_psd(Cursor::new(&input), ReadOptions::default())?;
            for _ in 0..iterations {
                let bytes = write_psd(&psd, &WriteOptions::default())?;
                black_box(bytes);
            }
        }
        _ => return Err("mode must be parse or write".into()),
    }

    let elapsed = start.elapsed();
    println!(
        "mode={mode} file={path} iterations={iterations} total_ms={:.3} per_iter_ms={:.3}",
        elapsed.as_secs_f64() * 1_000.0,
        elapsed.as_secs_f64() * 1_000.0 / iterations as f64
    );
    Ok(())
}
