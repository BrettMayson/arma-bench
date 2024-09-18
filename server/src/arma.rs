use std::path::PathBuf;

use arma_bench::ServerConfig;
use tokio::process::{Child, Command};
use tracing::debug;
use uuid::Uuid;

use crate::build::BuiltRequest;

pub async fn install(config: &ServerConfig) -> Result<PathBuf, String> {
    // check if there is a server at the path defined
    let fs_branch = config.branch.to_lowercase();
    let path = PathBuf::from("/opt/servers").join(&fs_branch);
    // if the path exists and is less than 12 hours old, return it
    if path.exists() {
        if let Ok(metadata) = path.metadata() {
            if let Ok(modified) = metadata.modified() {
                if let Ok(elapsed) = modified.elapsed() {
                    if elapsed.as_secs() < 43200 {
                        debug!("Using existing server {} at {:?}", fs_branch, path);
                        return Ok(path);
                    }
                }
            }
        }
    }
    let steam_user = std::env::var("STEAM_USER").map_err(|_| "STEAM_USER not set")?;
    let steam_pass = std::env::var("STEAM_PASS").map_err(|_| "STEAM_PASS not set")?;
    // otherwise, download the server and return the path
    debug!("Downloading {} server to {:?}", fs_branch, path);
    let mut command = Command::new("/steamcmd/steamcmd.sh");
    command.arg(format!("+login {steam_user} {steam_pass}"))
        .arg("+force_install_dir")
        .arg(&path)
        .arg("+app_update 233780");

    if config.branch != "public" {
        command.arg("-beta").arg(&config.branch);
    }
    if !config.branch_password.is_empty() {
        command.arg("-betapassword").arg(&config.branch_password);
    }
    let command = command
        .arg("validate")
        .arg("+quit")
        .output()
        .await
        .map_err(|e| e.to_string())?;
    if !command.status.success() {
        return Err(format!("Failed to install server: {command:?}"));
    }
    Ok(path)
}

pub async fn start(config: &ServerConfig, built: &BuiltRequest) -> Result<(Uuid, Child), String> {
    let path = install(config).await?;
    let name = Uuid::new_v4();
    let command = Command::new(path.join(&config.binary))
        .current_dir(&path)
        .arg(format!("-name={name}"))
        .arg("-world=empty")
        .arg("-limitFPS=1000")
        .arg("-profiles=\"/tmp/arma_profiles\"")
        .arg("-mod=\"../../@tab\"")
        .arg(format!("\"-mod=../../..{}\"", built.path.to_string_lossy()))
        .spawn()
        .map_err(|e| e.to_string())?;
    Ok((name, command))
}
