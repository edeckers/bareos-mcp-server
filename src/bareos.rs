use anyhow::{Context, Result};
use tokio::io::AsyncWriteExt;
use tokio::process::Command;

pub struct JobListParams {
    pub job: Option<String>,
    pub client: Option<String>,
    pub jobstatus: Option<String>,
    pub jobtype: Option<String>,
    pub joblevel: Option<String>,
    pub volume: Option<String>,
    pub pool: Option<String>,
    pub days: Option<u32>,
    pub hours: Option<u32>,
    pub last: bool,
    pub count: bool,
}

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
        let bconsole_path =
            std::env::var("BAREOS_BCONSOLE_PATH").unwrap_or_else(|_| "bconsole".to_string());
        Self { bconsole_path }
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

        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();

        // bconsole writes connection info to stderr and may return non-zero exit codes
        // Only fail if we have no stdout and stderr contains actual error messages
        if stdout.is_empty() && !output.status.success() {
            anyhow::bail!("bconsole command failed: {}", stderr);
        }

        Ok(stdout)
    }

    pub async fn list_jobs(&self, params: JobListParams) -> Result<String> {
        let mut cmd = "list jobs".to_string();

        // Pass all parameters to bconsole - it handles precedence and filtering
        if let Some(job) = params.job {
            cmd.push_str(&format!(" job={}", job));
        }
        if let Some(client) = params.client {
            cmd.push_str(&format!(" client={}", client));
        }
        if let Some(jobstatus) = params.jobstatus {
            cmd.push_str(&format!(" jobstatus={}", jobstatus));
        }
        if let Some(jobtype) = params.jobtype {
            cmd.push_str(&format!(" jobtype={}", jobtype));
        }
        if let Some(joblevel) = params.joblevel {
            cmd.push_str(&format!(" joblevel={}", joblevel));
        }
        if let Some(volume) = params.volume {
            cmd.push_str(&format!(" volume={}", volume));
        }
        if let Some(pool) = params.pool {
            cmd.push_str(&format!(" pool={}", pool));
        }
        if let Some(days) = params.days {
            cmd.push_str(&format!(" days={}", days));
        }
        if let Some(hours) = params.hours {
            cmd.push_str(&format!(" hours={}", hours));
        }
        if params.last {
            cmd.push_str(" last");
        }
        if params.count {
            cmd.push_str(" count");
        }

        self.execute_command(&cmd).await
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

    pub async fn list_files(&self, job_id: &str) -> Result<String> {
        self.execute_command(&format!("list files jobid={}", job_id))
            .await
    }

    pub async fn show_job(&self, job_name: &str) -> Result<String> {
        self.execute_command(&format!("show job={}", job_name))
            .await
    }

    pub async fn show_jobdefs(&self, jobdefs_name: &str) -> Result<String> {
        self.execute_command(&format!("show jobdefs={}", jobdefs_name))
            .await
    }

    pub async fn show_schedule(&self, schedule_name: &str) -> Result<String> {
        self.execute_command(&format!("show schedule={}", schedule_name))
            .await
    }
}
