use clap::Parser;
use regex::Regex;
use std::error::Error;
use std::fs;
use std::io::Write;
use std::path::PathBuf;
use std::process::Stdio;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tempfile::tempdir;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::{Child, Command};
use tokio::sync::Mutex;
use tokio::time::timeout;

mod logging;

use logging::LogColorizer;

#[derive(Parser, Debug)]
#[command(author, version, about = "TES3MP Runner", long_about = None)]
struct Args {
    #[arg(long, default_value_t = 25565, env = "SERVER_PORT")]
    server_port: u16,

    #[arg(long, default_value_t = false, env = "LOG_AUTOPRUNE")]
    log_autoprune: bool,

    #[arg(long, default_value_t = 30, env = "SHUTDOWN_TIMEOUT")]
    shutdown_timeout: u64,

    #[arg(long, default_value = ".", env = "TES3MP_PATH")]
    tes3mp_path: PathBuf,

    #[arg(long, default_value_t = false, env = "CHECK_PUBLIC_IP")]
    check_public_ip: bool,

    #[arg(long, default_value = "info", env = "LOG_LEVEL")]
    log_level: String,

    #[arg(long, default_value_t = false, env = "USE_DREAMWEAVE_LUAJIT")]
    use_dreamweave_luajit: bool,
}

static SHUTTING_DOWN: AtomicBool = AtomicBool::new(false);

fn parse_config(path: &PathBuf) -> Option<(u16, String)> {
    let config_path = path.join("tes3mp-server-default.cfg");
    if !config_path.exists() {
        return None;
    }

    let content = std::fs::read_to_string(&config_path).ok()?;

    let port_regex = Regex::new(r"(?m)^port\s*=\s*(\d+)").ok()?;
    let host_regex = Regex::new(r"(?m)^hostname\s*=\s*(.+)").ok()?;

    let port = port_regex.captures(&content)
        .and_then(|c| c.get(1))
        .and_then(|m| m.as_str().parse().ok())?;

    let hostname = host_regex.captures(&content)
        .and_then(|c| c.get(1))
        .map(|m| m.as_str().trim().to_string())
        .unwrap_or_else(|| "Unknown".to_string());

    Some((port, hostname))
}

fn fetch_public_ip() -> Option<String> {
    if let Ok(ip) = fetch_public_ip_curl() {
        return Some(ip);
    }

    let client = reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(5))
        .build()
        .ok()?;

    let endpoints = [
        "https://api.ipify.org",
        "https://ifconfig.me/ip",
        "https://icanhazip.com",
        "https://checkip.amazonaws.com",
    ];

    for endpoint in endpoints {
        tracing::debug!("Trying IP endpoint: {}", endpoint);
        match client.get(endpoint).send() {
            Ok(response) => {
                tracing::debug!("Response status: {}", response.status());
                match response.text() {
                    Ok(ip) => {
                        let ip = ip.trim().to_string();
                        tracing::debug!("Got IP: {}", ip);
                        if !ip.is_empty() && ip.parse::<std::net::IpAddr>().is_ok() {
                            return Some(ip);
                        }
                    }
                    Err(e) => tracing::debug!("Failed to read response: {}", e),
                }
            }
            Err(e) => {
                let error_str = if let Some(source) = e.source() {
                    format!("{}: {}", e, source)
                } else {
                    e.to_string()
                };
                tracing::warn!("Request to {} failed: {}", endpoint, error_str);
            }
        }
    }

    None
}

fn fetch_public_ip_curl() -> Result<String, Box<dyn std::error::Error>> {
    let output = std::process::Command::new("curl")
        .args(["-s", "https://api.ipify.org"])
        .output()?;

    let ip = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if ip.parse::<std::net::IpAddr>().is_ok() {
        tracing::info!("Fetched public IP via curl: {}", ip);
        return Ok(ip);
    }

    Err("Invalid IP".into())
}

fn download_luajit() -> Option<PathBuf> {
    const LUAJIT_URL: &str = "https://github.com/DreamWeave-MP/luajit2/releases/download/Stable-CI/LuaJIT-Linux.7z";
    const LUAJIT_LIB_PATH: &str = "bin/libluajit.so";

    tracing::info!("Dreamweave LuaJIT enabled, downloading...");

    // Create a temporary directory
    let temp_dir = tempdir().ok()?;
    let temp_path = temp_dir.path();

    // Download the 7z file
    let archive_path = temp_path.join("LuaJIT-Linux.7z");
    let client = reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(30))
        .build()
        .ok()?;

    tracing::info!("Downloading LuaJIT from {}", LUAJIT_URL);
    let mut response = client.get(LUAJIT_URL).send().ok()?;
    if !response.status().is_success() {
        tracing::error!("Failed to download LuaJIT: HTTP {}", response.status());
        return None;
    }

    let mut file = fs::File::create(&archive_path).ok()?;
    std::io::copy(&mut response, &mut file).ok()?;

    // Extract the 7z file
    tracing::info!("Extracting LuaJIT archive...");
    match sevenz_rust::decompress_file(&archive_path, temp_path) {
        Ok(_) => {
            let lib_path = temp_path.join(LUAJIT_LIB_PATH);
            if lib_path.exists() {
                // Set execute permissions
                #[cfg(unix)]
                {
                    use std::os::unix::fs::PermissionsExt;
                    let mut perms = fs::metadata(&lib_path).ok()?.permissions();
                    perms.set_mode(0o755);
                    fs::set_permissions(&lib_path, perms).ok()?;
                }
                tracing::info!("Using Dreamweave LuaJIT loaded from: {}", lib_path.display());
                // Leak the temp dir to keep it alive
                Some(temp_dir.keep())
            } else {
                tracing::error!("libluajit.so not found in extracted archive");
                None
            }
        }
        Err(e) => {
            tracing::error!("Failed to extract LuaJIT archive: {}", e);
            None
        }
    }
}

fn check_nat_issue(local_address: &str, public_ip: &str, port: u16) {
    if local_address == "0.0.0.0" || local_address == "127.0.0.1" {
        tracing::info!(
            "Server bound to {} - master server will see {}:{}",
            local_address, public_ip, port
        );
    }
}

fn find_our_server(
    body: &str,
    public_ip: &str,
    port: u16,
) -> Option<(String, String)> {
    static SERVER_RE: once_cell::sync::Lazy<Regex> =
        once_cell::sync::Lazy::new(|| {
            Regex::new(
                r#""([^"]+:\d+)"\s*:\s*\{([\s\S]*?)\}"#
            ).unwrap()
        });

    static HOSTNAME_RE: once_cell::sync::Lazy<Regex> =
        once_cell::sync::Lazy::new(|| {
            Regex::new(
                r#""hostname"\s*:\s*"([^"]+)""#
            ).unwrap()
        });

    let target = format!("{}:{}", public_ip, port);

    for cap in SERVER_RE.captures_iter(body) {
        let addr = cap.get(1)?.as_str();
        let server_body = cap.get(2)?.as_str();

        tracing::debug!("Found server entry: {}", addr);

        if addr != target {
            continue;
        }

        let hostname = HOSTNAME_RE
            .captures(server_body)
            .and_then(|c| c.get(1))
            .map(|m| m.as_str().to_string())?;

        tracing::info!("Matched server: {} -> {}", hostname, addr);

        return Some((addr.to_string(), hostname));
    }

    None
}

fn wait_for_master_clear(public_ip: &str, port: u16) {
    let client = match reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(5))
        .build()
    {
        Ok(c) => c,
        Err(e) => {
            tracing::warn!("Failed to create HTTP client: {}", e);
            return;
        }
    };

    let poll_interval = Duration::from_secs(2);

    tracing::info!("Waiting for master to clear old registration...");

loop {
        let response = match client.get("http://master.tes3mp.com:8081/api/servers").send() {
            Ok(r) => r,
            Err(e) => {
                tracing::debug!("Failed to query master: {}", e);
                std::thread::sleep(poll_interval);
                continue;
            }
        };

        if response.status() != 200 {
            tracing::debug!("Master returned status: {}", response.status());
            std::thread::sleep(poll_interval);
            continue;
        }

        let body = match response.text() {
            Ok(b) => b,
            Err(e) => {
                tracing::debug!("Failed to read master response: {}", e);
                std::thread::sleep(poll_interval);
                continue;
            }
        };

        if let Some((addr, found_hostname)) = find_our_server(&body, public_ip, port) {
            let has_wrong_port = !addr.contains(&format!(":{}", port));
            if has_wrong_port {
                tracing::warn!("NAT issue: Master sees {} at {} (expected {}:{})",
                    found_hostname, addr, public_ip, port);
                std::thread::sleep(poll_interval);
                continue;
            } else {
                tracing::info!("Registration OK (correct port)");
                return;
            }
        } else {
            tracing::info!("No existing registration found - ready to start");
            return;
        }
    }
}

fn setup_logging(log_level: &str) {
    use tracing_subscriber::prelude::*;
    use tracing_subscriber::{fmt, EnvFilter, Registry};

    let filter = if std::env::var("RUST_LOG").is_ok() {
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(log_level))
    } else {
        EnvFilter::new(log_level)
    };

    let subscriber = Registry::default()
        .with(filter)
        .with(fmt::layer().with_writer(std::io::stdout));

    tracing::subscriber::set_global_default(subscriber).ok();
}

fn autoprune_logs(path: &PathBuf) {
    use std::fs;
    let openmw_dir = path.join(".config/openmw");
    if !openmw_dir.exists() {
        return;
    }
    let logs_dir = openmw_dir.join("logs");
    if !logs_dir.exists() {
        return;
    }
    let now = std::time::SystemTime::now();
    let cutoff = Duration::from_secs(7 * 24 * 60 * 60);

    if let Ok(entries) = fs::read_dir(logs_dir) {
        for entry in entries.filter_map(|e| e.ok()) {
            if let Ok(metadata) = entry.metadata() {
                if let Ok(modified) = metadata.modified() {
                    if let Ok(elapsed) = now.duration_since(modified) {
                        if elapsed > cutoff {
                            let _ = fs::remove_file(entry.path());
                            tracing::info!("Pruned old log: {}", entry.file_name().display());
                        }
                    }
                }
            }
        }
    }
}

fn check_master_timeout(line: &str) -> bool {
    static MASTER_REGEX: once_cell::sync::Lazy<Regex> =
        once_cell::sync::Lazy::new(|| {
            Regex::new(r"(?i)(cannot connect to master|master.*timed?|master.*timeout|stale.*port|incorrect.*ip|nat.*issue)")
                .unwrap()
        });

    MASTER_REGEX.is_match(line)
}

fn get_tes3mp_executable(path: &PathBuf) -> PathBuf {
    #[cfg(windows)]
    {
        let exe = path.join("tes3mp-server.exe");
        if exe.exists() {
            return exe;
        }
        let bat = path.join("tes3mp-server.bat");
        if bat.exists() {
            return bat;
        }
        return exe;
    }
    #[cfg(not(windows))]
    {
        path.join("tes3mp-server")
    }
}

async fn spawn_tes3mp(tes3mp_path: &PathBuf, ld_preload: Option<&PathBuf>) -> std::io::Result<Child> {
    let exe = get_tes3mp_executable(tes3mp_path);

    tracing::info!("Starting TES3MP from: {}", exe.display());

    #[cfg(not(windows))]
    {
        let lib_path = tes3mp_path.join("lib");
        let mut child_builder = Command::new(&exe);
        child_builder
            .env("LD_LIBRARY_PATH", lib_path)
            .env_remove("RUST_BACKTRACE")
            .current_dir(tes3mp_path)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        if let Some(preload_path) = ld_preload {
            let luajit_lib = preload_path.join("bin/libluajit.so");
            tracing::info!("Setting LD_PRELOAD to: {}", luajit_lib.display());
            child_builder.env("LD_PRELOAD", luajit_lib);
        }

        let child = child_builder.spawn()?;

        Ok(child)
    }

    #[cfg(windows)]
    {
        let mut child_builder = Command::new(&exe);
        child_builder
            .env_remove("RUST_BACKTRACE")
            .current_dir(tes3mp_path)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        let _ = ld_preload;

        let child = child_builder.spawn()?;

        Ok(child)
    }
}

async fn run_process_loop(_tes3mp_path: &PathBuf, child: Arc<Mutex<Option<Child>>>, _shutdown_timeout: u64) -> std::io::Result<()> {
    let pid = {
        let mut guard = child.lock().await;
        if let Some(ref mut c) = *guard {
            c.id()
        } else {
            None
        }
    };

    tracing::info!("TES3MP started with PID: {:?}", pid);

    let stdout = {
        let mut guard = child.lock().await;
        if let Some(ref mut c) = *guard {
            c.stdout.take()
        } else {
            None
        }
    }.expect("Failed to capture stdout");

    let stderr = {
        let mut guard = child.lock().await;
        if let Some(ref mut c) = *guard {
            c.stderr.take()
        } else {
            None
        }
    }.expect("Failed to capture stderr");

    let colorizer = LogColorizer::new();
    let is_tty = is_atty();

    let stdout_handle = tokio::spawn({
        let colorizer = colorizer.clone();
        async move {
            let reader = BufReader::new(stdout);
            let mut lines = reader.lines();

            while let Ok(Some(line)) = lines.next_line().await {
                if check_master_timeout(&line) {
                    tracing::warn!(
                        "Master server timeout/ip issue detected in output: {}",
                        line
                    );
                }

                if is_tty {
                    let colored = colorizer.colorize(&line);
                    let _ = std::io::stdout().write_all(colored.as_bytes());
                    let _ = std::io::stdout().write_all(b"\n");
                } else {
                    let _ = std::io::stdout().write_all(line.as_bytes());
                    let _ = std::io::stdout().write_all(b"\n");
                }
            }
        }
    });

    let stderr_handle = tokio::spawn({
        let colorizer = colorizer;
        async move {
            let reader = BufReader::new(stderr);
            let mut lines = reader.lines();

            while let Ok(Some(line)) = lines.next_line().await {
                if check_master_timeout(&line) {
                    tracing::warn!(
                        "Master server timeout/ip issue detected in stderr: {}",
                        line
                    );
                }

                if is_tty {
                    let colored = colorizer.colorize(&line);
                    let _ = std::io::stderr().write_all(colored.as_bytes());
                    let _ = std::io::stderr().write_all(b"\n");
                } else {
                    let _ = std::io::stderr().write_all(line.as_bytes());
                    let _ = std::io::stderr().write_all(b"\n");
                }
            }
        }
    });

    let status = {
        let mut guard = child.lock().await;
        if let Some(ref mut c) = *guard {
            c.wait().await
        } else {
            #[cfg(unix)]
            {
                use std::os::unix::process::ExitStatusExt;
                Ok(std::process::ExitStatus::from_raw(0))
            }
            #[cfg(windows)]
            {
                use std::os::windows::process::ExitStatusExt;
                Ok(std::process::ExitStatus::from_raw(0))
            }
        }
    }?;

    let _ = stdout_handle.abort();
    let _ = stderr_handle.abort();

    if SHUTTING_DOWN.load(Ordering::SeqCst) {
        tracing::info!("TES3MP exited gracefully: {}", status);
    } else {
        tracing::warn!("TES3MP exited unexpectedly: {}", status);
    }

    Ok(())
}

async fn graceful_shutdown(child: Arc<Mutex<Option<Child>>>, timeout_secs: u64) {
    tracing::info!("Initiating graceful shutdown...");

    let mut child_guard = child.lock().await;
    if let Some(ref mut c) = *child_guard {
#[cfg(unix)]
        {
            use tokio::process::Command;

            let pid = c.id().unwrap();
            let mut kill_cmd = Command::new("kill");
            kill_cmd.arg("-TERM").arg(pid.to_string());
            
            if let Err(e) = kill_cmd.spawn() {
                tracing::warn!("Failed to send SIGTERM: {}", e);
            } else {
                let shutdown_future = c.wait();
                match timeout(Duration::from_secs(timeout_secs), shutdown_future).await {
                    Ok(Ok(status)) => {
                        tracing::info!("TES3MP shut down cleanly: {}", status);
                    }
                    Ok(Err(e)) => {
                        tracing::error!("Error waiting for TES3MP: {}", e);
                    }
                    Err(_) => {
                        tracing::warn!("TES3MP did not shut down gracefully, force killing...");
                        if let Err(e) = c.kill().await {
                            tracing::error!("Failed to kill TES3MP: {}", e);
                        }
                    }
                }
            }
        }

        #[cfg(not(unix))]
        {
            tracing::info!("Sending termination signal to TES3MP...");

            if let Err(e) = c.kill().await {
                tracing::warn!("Failed to kill: {}", e);
            }

            let shutdown_future = c.wait();
            match timeout(Duration::from_secs(timeout_secs), shutdown_future).await {
                Ok(Ok(status)) => {
                    tracing::info!("TES3MP shut down: exit code {:?}", status.code());
                }
                Ok(Err(e)) => {
                    tracing::error!("Error waiting for TES3MP: {}", e);
                }
                Err(_) => {
                    tracing::warn!("Shutdown timeout exceeded");
                    let _ = c.kill().await;
                }
            }
        }
    }
    *child_guard = None;
}

fn is_atty() -> bool {
    is_terminal::IsTerminal::is_terminal(&std::io::stdout())
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    setup_logging(&args.log_level);

    tracing::info!("TES3MP Runner v{}", env!("CARGO_PKG_VERSION"));
    tracing::info!("Server port: {}", args.server_port);
    tracing::info!("Shutdown timeout: {}s", args.shutdown_timeout);
    tracing::info!("TES3MP path: {}", args.tes3mp_path.display());

    if let Some((config_port, hostname)) = parse_config(&args.tes3mp_path) {
        tracing::info!("Config: hostname={}, port={}", hostname, config_port);
        if config_port != args.server_port {
            tracing::warn!(
                "Port mismatch: Config has {} but SERVER_PORT env is {}. Using SERVER_PORT.",
                config_port, args.server_port
            );
        }
    }

    if args.check_public_ip {
        tracing::info!("Checking public IP...");
        if let Some(public_ip) = fetch_public_ip() {
            tracing::info!("Public IP: {}", public_ip);
            check_nat_issue("0.0.0.0", &public_ip, args.server_port);

            let _hostname = parse_config(&args.tes3mp_path)
                .map(|(_, h)| h)
                .unwrap_or_else(|| "Unknown".to_string());

            wait_for_master_clear(&public_ip, args.server_port);
        } else {
            tracing::warn!("Failed to fetch public IP");
        }
    }

    if args.log_autoprune {
        tracing::info!("Log autoprune enabled");
        autoprune_logs(&args.tes3mp_path);
    }

    // Handle Dreamweave LuaJIT if enabled
    let ld_preload_path: Option<PathBuf> = if args.use_dreamweave_luajit {
        download_luajit()
    } else {
        None
    };

    SHUTTING_DOWN.store(false, Ordering::SeqCst);

    let child = Arc::new(Mutex::new(Some(spawn_tes3mp(&args.tes3mp_path, ld_preload_path.as_ref()).await?)));

    #[cfg(unix)]
    {
        use tokio::signal::unix::{signal, SignalKind};

        let mut term_signal = signal(SignalKind::terminate())?;
        let mut int_signal = signal(SignalKind::interrupt())?;
        let child_clone = child.clone();
        let timeout = args.shutdown_timeout;

        tokio::spawn(async move {
            tokio::select! {
                _ = term_signal.recv() => {
                    tracing::info!("Received SIGTERM");
                    SHUTTING_DOWN.store(true, Ordering::SeqCst);
                    graceful_shutdown(child_clone, timeout).await;
                }
                _ = int_signal.recv() => {
                    tracing::info!("Received SIGINT");
                    SHUTTING_DOWN.store(true, Ordering::SeqCst);
                    graceful_shutdown(child_clone, timeout).await;
                }
            }
        });
    }

    #[cfg(windows)]
    {
        use tokio::signal::windows::ctrl_c;

        if let Some(mut c_signal) = ctrl_c().ok() {
            let child_clone = child.clone();
            let timeout = args.shutdown_timeout;

            tokio::spawn(async move {
                while let Some(_) = c_signal.recv().await {
                    tracing::info!("Received CtrlC");
                    SHUTTING_DOWN.store(true, Ordering::SeqCst);
                    graceful_shutdown(child_clone.clone(), timeout).await;
                    break;
                }
            });
        }
    }

    run_process_loop(&args.tes3mp_path, child, args.shutdown_timeout).await?;

    Ok(())
}

