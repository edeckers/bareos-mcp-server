use anyhow::Result;
use serde_json::{json, Value};
use std::io::{self, BufRead, Write};
use tracing::{error, info};

mod bareos;

use bareos::{BareosClient, JobListParams};

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let stdin = io::stdin();
    let mut stdout = io::stdout();
    let client = BareosClient::new();

    info!("Bareos MCP Server starting...");

    // Send server info on startup
    let server_info = json!({
        "jsonrpc": "2.0",
        "id": null,
        "result": {
            "protocolVersion": "2024-11-05",
            "capabilities": {
                "tools": {}
            },
            "serverInfo": {
                "name": "bareos-mcp-server",
                "version": "0.1.0"
            }
        }
    });
    writeln!(stdout, "{}", server_info)?;
    stdout.flush()?;

    for line in stdin.lock().lines() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }

        let request: Value = match serde_json::from_str(&line) {
            Ok(req) => req,
            Err(e) => {
                error!("Failed to parse request: {}", e);
                continue;
            }
        };

        let response = handle_request(&client, request).await;
        writeln!(stdout, "{}", serde_json::to_string(&response)?)?;
        stdout.flush()?;
    }

    Ok(())
}

async fn handle_request(client: &BareosClient, request: Value) -> Value {
    let method = request["method"].as_str().unwrap_or("");
    let id = request["id"].clone();

    match method {
        "initialize" => json!({
            "jsonrpc": "2.0",
            "id": id,
            "result": {
                "protocolVersion": "2024-11-05",
                "capabilities": {
                    "tools": {}
                },
                "serverInfo": {
                    "name": "bareos-mcp-server",
                    "version": "0.1.0"
                }
            }
        }),
        "tools/list" => {
            let tools = vec![
                json!({
                    "name": "list_jobs",
                    "description": "List Bareos jobs with optional filters. Returns ALL job types by default (backup, restore, verify, admin, copy, migration). IMPORTANT: When users ask about 'backups' or 'backup performance', filter by jobtype='B' to show only backup jobs, excluding verification and other operations. COMBINATION RULES: All filter parameters (job, client, jobstatus, jobtype, joblevel, volume, pool) can be freely combined with each other and with time/output parameters. Time parameters (days, hours) are mutually exclusive - if both provided, hours wins. Output parameters (last, count) are mutually exclusive - if both provided, count wins. Examples: {jobtype:'B',hours:24}, {client:'web',days:7}, {jobstatus:'f',hours:24,count:true}",
                    "inputSchema": {
                        "type": "object",
                        "properties": {
                            "job": {
                                "type": "string",
                                "description": "Filter by job name. Can combine with any other parameters."
                            },
                            "client": {
                                "type": "string",
                                "description": "Filter by client name. Can combine with any other parameters."
                            },
                            "jobstatus": {
                                "type": "string",
                                "description": "Filter by job status: T (terminated/success), f (failed), R (running), C (created), A (canceled), E (error). Can combine with any other parameters."
                            },
                            "jobtype": {
                                "type": "string",
                                "description": "Filter by job type: B (backup), R (restore), V (verify), D (admin), C (copy), M (migration). Can combine with any other parameters."
                            },
                            "joblevel": {
                                "type": "string",
                                "description": "Filter by job level: F (full), I (incremental), D (differential). Can combine with any other parameters."
                            },
                            "volume": {
                                "type": "string",
                                "description": "Filter by volume name. Can combine with any other parameters."
                            },
                            "pool": {
                                "type": "string",
                                "description": "Filter by pool name. Can combine with any other parameters."
                            },
                            "days": {
                                "type": "number",
                                "description": "Show jobs from last N days. Mutually exclusive with hours (hours wins if both given). Can combine with all filter and output parameters."
                            },
                            "hours": {
                                "type": "number",
                                "description": "Show jobs from last N hours. Mutually exclusive with days (this takes precedence if both given). Can combine with all filter and output parameters."
                            },
                            "last": {
                                "type": "boolean",
                                "description": "Show only the most recent run of each job (within the filter criteria). WARNING: If jobs ran multiple times in the time range, only the LAST run will be returned. Mutually exclusive with count (count wins if both given). Can combine with all filter and time parameters."
                            },
                            "count": {
                                "type": "boolean",
                                "description": "Show count of matching jobs instead of job details. Mutually exclusive with last (this takes precedence if both given). Can combine with all filter and time parameters."
                            }
                        }
                    }
                }),
                json!({
                    "name": "get_job_status",
                    "description": "Get detailed status of a specific job by ID",
                    "inputSchema": {
                        "type": "object",
                        "properties": {
                            "job_id": {
                                "type": "string",
                                "description": "The job ID to query"
                            }
                        },
                        "required": ["job_id"]
                    }
                }),
                json!({
                    "name": "get_job_log",
                    "description": "Get the log output for a specific job",
                    "inputSchema": {
                        "type": "object",
                        "properties": {
                            "job_id": {
                                "type": "string",
                                "description": "The job ID to get logs for"
                            }
                        },
                        "required": ["job_id"]
                    }
                }),
                json!({
                    "name": "list_clients",
                    "description": "List all Bareos clients (file daemons)",
                    "inputSchema": {
                        "type": "object",
                        "properties": {}
                    }
                }),
                json!({
                    "name": "list_filesets",
                    "description": "List all configured filesets",
                    "inputSchema": {
                        "type": "object",
                        "properties": {}
                    }
                }),
                json!({
                    "name": "list_pools",
                    "description": "List all storage pools",
                    "inputSchema": {
                        "type": "object",
                        "properties": {}
                    }
                }),
                json!({
                    "name": "list_volumes",
                    "description": "List all volumes/media in storage",
                    "inputSchema": {
                        "type": "object",
                        "properties": {
                            "pool": {
                                "type": "string",
                                "description": "Filter by specific pool name (optional)"
                            }
                        }
                    }
                }),
                json!({
                    "name": "list_files",
                    "description": "List all files backed up in a specific job",
                    "inputSchema": {
                        "type": "object",
                        "properties": {
                            "job_id": {
                                "type": "string",
                                "description": "The job ID to list files for"
                            }
                        },
                        "required": ["job_id"]
                    }
                }),
            ];

            json!({
                "jsonrpc": "2.0",
                "id": id,
                "result": {
                    "tools": tools
                }
            })
        }
        "tools/call" => {
            let tool_name = request["params"]["name"].as_str().unwrap_or("");
            let arguments = &request["params"]["arguments"];

            let result = match tool_name {
                "list_jobs" => {
                    // Pass all parameters through - bconsole handles precedence
                    let params = JobListParams {
                        job: arguments["job"].as_str().map(|s| s.to_string()),
                        client: arguments["client"].as_str().map(|s| s.to_string()),
                        jobstatus: arguments["jobstatus"].as_str().map(|s| s.to_string()),
                        jobtype: arguments["jobtype"].as_str().map(|s| s.to_string()),
                        joblevel: arguments["joblevel"].as_str().map(|s| s.to_string()),
                        volume: arguments["volume"].as_str().map(|s| s.to_string()),
                        pool: arguments["pool"].as_str().map(|s| s.to_string()),
                        days: arguments["days"].as_u64().map(|n| n as u32),
                        hours: arguments["hours"].as_u64().map(|n| n as u32),
                        last: arguments["last"].as_bool().unwrap_or(false),
                        count: arguments["count"].as_bool().unwrap_or(false),
                    };

                    client.list_jobs(params).await
                }
                "get_job_status" => {
                    let job_id = arguments["job_id"].as_str().unwrap_or("");
                    client.get_job_status(job_id).await
                }
                "get_job_log" => {
                    let job_id = arguments["job_id"].as_str().unwrap_or("");
                    client.get_job_log(job_id).await
                }
                "list_clients" => client.list_clients().await,
                "list_filesets" => client.list_filesets().await,
                "list_pools" => client.list_pools().await,
                "list_volumes" => {
                    let pool = arguments["pool"].as_str();
                    client.list_volumes(pool).await
                }
                "list_files" => {
                    let job_id = arguments["job_id"].as_str().unwrap_or("");
                    client.list_files(job_id).await
                }
                _ => Err(anyhow::anyhow!("Unknown tool: {}", tool_name)),
            };

            match result {
                Ok(content) => json!({
                    "jsonrpc": "2.0",
                    "id": id,
                    "result": {
                        "content": [
                            {
                                "type": "text",
                                "text": content
                            }
                        ]
                    }
                }),
                Err(e) => json!({
                    "jsonrpc": "2.0",
                    "id": id,
                    "error": {
                        "code": -32000,
                        "message": format!("Tool execution failed: {}", e)
                    }
                }),
            }
        }
        _ => json!({
            "jsonrpc": "2.0",
            "id": id,
            "error": {
                "code": -32601,
                "message": format!("Method not found: {}", method)
            }
        }),
    }
}
