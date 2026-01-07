use std::path::Path;

use anyhow::Result;

pub async fn cd(path: &Path) -> Result<()> {
    match tokio::process::Command::new("cd").arg(path).output().await {
        Ok(output) => {
            if output.status.success() {
                tracing::info!("successfully changed directory to {:?}", path);
                Ok(())
            } else {
                tracing::error!(
                    "failed to change directory to {:?}: {}",
                    path,
                    String::from_utf8_lossy(&output.stderr)
                );
                Err(anyhow::anyhow!("failed to change directory to {:?}", path))
            }
        }
        Err(e) => {
            tracing::error!("command failed to execute: cd {:?}: {e:?}", path);
            Err(anyhow::anyhow!("command failed to execute: cd {:?}", path))
        }
    }
}
