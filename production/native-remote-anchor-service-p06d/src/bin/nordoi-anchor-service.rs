#![forbid(unsafe_code)]

use nordoi_remote_anchor_service_p06d::RemoteAnchorService;
use std::env;
use std::fs;
use std::net::SocketAddr;
use std::path::Path;
use std::time::Duration;

fn read_exact_key(path: &Path) -> Result<[u8; 32], String> {
    let bytes = fs::read(path).map_err(|_| format!("cannot read key file: {}", path.display()))?;
    bytes.try_into().map_err(|_| {
        format!(
            "key file must contain exactly 32 raw bytes: {}",
            path.display()
        )
    })
}

fn usage() -> &'static str {
    "usage: nordoi-anchor-service <bind_addr> <store_path> <client_public_key_file> <server_secret_key_file> <timeout_ms>"
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 6 {
        eprintln!("{}", usage());
        std::process::exit(2);
    }

    let bind: SocketAddr = match args[1].parse() {
        Ok(value) => value,
        Err(_) => {
            eprintln!("invalid bind address");
            std::process::exit(2);
        }
    };

    let timeout_ms: u64 = match args[5].parse() {
        Ok(value) if value > 0 => value,
        _ => {
            eprintln!("timeout_ms must be a positive integer");
            std::process::exit(2);
        }
    };

    let client_public = match read_exact_key(Path::new(&args[3])) {
        Ok(value) => value,
        Err(message) => {
            eprintln!("{message}");
            std::process::exit(2);
        }
    };

    let server_secret = match read_exact_key(Path::new(&args[4])) {
        Ok(value) => value,
        Err(message) => {
            eprintln!("{message}");
            std::process::exit(2);
        }
    };

    let service = match RemoteAnchorService::open(
        Path::new(&args[2]),
        client_public,
        server_secret,
        Duration::from_millis(timeout_ms),
    ) {
        Ok(service) => service,
        Err(error) => {
            eprintln!("service initialization failed: {error:?}");
            std::process::exit(1);
        }
    };

    if let Err(error) = service.run(bind) {
        eprintln!("service terminated: {error:?}");
        std::process::exit(1);
    }
}
