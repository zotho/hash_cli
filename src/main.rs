use std::{
    fs::File,
    io::Read,
    path::{Path, PathBuf},
    time::{Duration, Instant},
};

use clap::Parser;
use md5::Digest;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(long, default_value_t=1024*4)]
    buffer_size: usize,
    files: Vec<PathBuf>,
}

fn main() {
    let Args { buffer_size, files } = Args::parse();

    let results: Vec<_> = std::thread::scope(|scope| {
        let mut handles = Vec::with_capacity(files.len());
        for file in files {
            handles.push(scope.spawn(move || (file.clone(), md5_file(buffer_size, &file))));
        }
        handles.into_iter().map(|t| t.join().unwrap()).collect()
    });
    for (path, hash) in results {
        println!(
            "{hash:?} {name}",
            name = path.file_name().unwrap().to_str().unwrap()
        );
    }
}

fn md5_file(buffer_size: usize, path: &Path) -> Digest {
    let mut context = md5::Context::new();
    let mut file = File::open(path).unwrap();
    let meta = file.metadata().unwrap();
    let total_size = meta.len();
    let mut buffer = vec![0_u8; buffer_size];
    let mut processed_size = 0;
    let start = Instant::now();
    let mut printed = false;
    loop {
        let size = file.read(&mut buffer).unwrap();
        if size == 0 {
            break;
        }
        context.consume(&buffer[..size]);
        processed_size += size;
        if !printed {
            let elapsed = start.elapsed();
            if elapsed > Duration::from_secs(2) {
                let bytes_per_second = processed_size as f64 / elapsed.as_secs_f64();
                let bytes_left = total_size.checked_sub(processed_size as u64).unwrap();
                let time_left = Duration::from_secs_f64(bytes_left as f64 / bytes_per_second);
                eprintln!(
                    "{name}: ETA: {time_left:?}",
                    name = path.file_name().unwrap().to_str().unwrap()
                );
                printed = true;
            }
        }
    }
    context.compute()
}
