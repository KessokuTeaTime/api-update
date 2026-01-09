use anyhow::Result;
use tokio::{io, task};

use crate::env::{DOCKER_PASSWORD, DOCKER_USERNAME};

pub async fn login() -> Result<()> {
    let mut password = tokio::process::Command::new("echo")
        .arg(&*DOCKER_PASSWORD)
        .stdout(std::process::Stdio::piped())
        .spawn()?;

    let mut login = tokio::process::Command::new("docker")
        .arg("login")
        .arg("-u")
        .arg(&*DOCKER_USERNAME)
        .arg("--password-stdin")
        .stdin(std::process::Stdio::piped())
        .spawn()?;

    let mut password_out = password.stdout.take().unwrap();
    let mut login_in = login.stdin.take().unwrap();

    let pipe = task::spawn(async move { io::copy(&mut password_out, &mut login_in).await });

    pipe.await??;

    password.wait().await?;
    login.wait().await?;

    tracing::info!(
        "successfully logged in to docker with username {}",
        &*DOCKER_USERNAME,
    );

    Ok(())
}

pub async fn logout() -> Result<()> {
    match tokio::process::Command::new("docker")
        .arg("logout")
        .output()
        .await
    {
        Ok(output) => {
            if output.status.success() {
                tracing::info!("successfully logged out from docker");
                Ok(())
            } else {
                tracing::error!(
                    "failed to logout from docker: {}",
                    String::from_utf8_lossy(&output.stderr)
                );
                Err(anyhow::anyhow!("failed to logout from docker"))
            }
        }
        Err(e) => {
            tracing::error!("command failed to execute: docker logout: {e:?}");
            Err(anyhow::anyhow!("command failed to execute: docker logout"))
        }
    }
}

pub async fn pull_image(image: &str) -> Result<()> {
    match tokio::process::Command::new("docker")
        .arg("pull")
        .arg(image)
        .output()
        .await
    {
        Ok(output) => {
            if output.status.success() {
                tracing::info!("successfully pulled image {image}");
                Ok(())
            } else {
                tracing::error!(
                    "failed to pull image {image}: {}",
                    String::from_utf8_lossy(&output.stderr)
                );
                Err(anyhow::anyhow!("failed to pull image {image}"))
            }
        }
        Err(e) => {
            tracing::error!("command failed to execute: pulling image {image}: {e:?}");
            Err(anyhow::anyhow!(
                "command failed to execute: pulling image {image}"
            ))
        }
    }
}

pub async fn compose_up(container_name: &str) -> Result<()> {
    match tokio::process::Command::new("docker")
        .arg("compose")
        .arg("up")
        .arg("-d")
        .arg(container_name)
        .output()
        .await
    {
        Ok(output) => {
            if output.status.success() {
                tracing::info!("successfully upped container {container_name}");
                Ok(())
            } else {
                tracing::error!(
                    "failed to up container {container_name}: {}",
                    String::from_utf8_lossy(&output.stderr)
                );
                Err(anyhow::anyhow!("failed to up container {container_name}"))
            }
        }
        Err(e) => {
            tracing::error!("command failed to execute: upping container {container_name}: {e:?}");
            Err(anyhow::anyhow!(
                "command failed to execute: upping container {container_name}"
            ))
        }
    }
}
