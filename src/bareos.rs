use anyhow::{Context, Result};
use tokio::io::AsyncWriteExt;
use tokio::process::Command;

pub struct BareosClient {
    bconsole_path: String,
}

impl Default for BareosClient {
    fn default() -> Self {
        Self {
            bconsole_path: "bconsole".to_string(),
        }
    }
}

impl BareosClient {
    pub fn new() -> Self {
        Self::default()
    }

    async fn execute_command(&self, command: &str) -> Result<String> {
        let mut child = Command::new(&self.bconsole_path)
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()
            .context("Failed to spawn bconsole")?;

        let mut stdin = child.stdin.take().context("Failed to open stdin")?;

        // Write commands to bconsole
        stdin.write_all(format!("{}\n", command).as_bytes()).await?;
        stdin.write_all(b"quit\n").await?;
        drop(stdin);

        let output = child
            .wait_with_output()
            .await
            .context("Failed to read bconsole output")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("bconsole command failed: {}", stderr);
        }

        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }

    pub async fn list_jobs(&self, limit: usize) -> Result<String> {
        self.execute_command(&format!("list jobs last={}", limit))
            .await
    }

    pub async fn get_job_status(&self, job_id: &str) -> Result<String> {
        self.execute_command(&format!("list jobid={}", job_id))
            .await
    }

    pub async fn get_job_log(&self, job_id: &str) -> Result<String> {
        self.execute_command(&format!("list joblog jobid={}", job_id))
            .await
    }

    pub async fn list_clients(&self) -> Result<String> {
        self.execute_command("list clients").await
    }

    pub async fn list_filesets(&self) -> Result<String> {
        self.execute_command("list filesets").await
    }

    pub async fn list_pools(&self) -> Result<String> {
        self.execute_command("list pools").await
    }

    pub async fn list_volumes(&self, pool: Option<&str>) -> Result<String> {
        let cmd = if let Some(pool_name) = pool {
            format!("list volumes pool={}", pool_name)
        } else {
            "list volumes".to_owned()
        };
        self.execute_command(&cmd).await
    }
}
