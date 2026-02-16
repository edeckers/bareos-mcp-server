# Bareos MCP Server

[![Release](https://github.com/edeckers/bareos-mcp-server/actions/workflows/release.yml/badge.svg)](https://github.com/edeckers/bareos-mcp-server/actions/workflows/release.yml)
[![License](https://img.shields.io/badge/License-MPL--2.0-blue.svg)](https://opensource.org/licenses/MPL-2.0)

A Model Context Protocol (MCP) server for [Bareos backup system](https://github.com/bareos/bareos), providing read-only operations for monitoring and querying backup infrastructure.

## Quick Start

Once configured, ask your AI assistant naturally about your backups:

```
"Show me the last 10 backup jobs"
"Are there any failed backups today?"
"What's the status of job 12345?"
"How much storage is left in the Full pool?"
"List all volumes ready for pruning"
```

## Features

### Read-Only Operations

- **Jobs**: List recent jobs, get detailed status, view job logs
- **Clients**: List all Bareos file daemon clients
- **Filesets**: List configured backup filesets
- **Storage**: List pools and volumes with capacity info

All operations are read-only by design for safety in production environments.

## Prerequisites

- **Rust 1.70+** - For building the server
- **Bareos Director** - With bconsole access
- **bconsole** - Command-line interface to Bareos Director

## Installation

### 1. Clone and Build

```bash
git clone https://github.com/edeckers/bareos-mcp-server.git
cd bareos-mcp-server
cargo build --release
```

The binary will be at: `target/release/bareos-mcp-server`

### 2. Configure bconsole Access

The MCP server calls `bconsole` from your PATH. You have several options:

#### Option A: Local bconsole (Direct Access)

If bconsole is installed locally and you have access:

```bash
# Test that bconsole works
bconsole -c /etc/bareos/bconsole.conf
```

No additional setup needed - the server will use bconsole directly.

#### Option B: Remote Bareos via SSH

If your Bareos Director is on a remote host, create a wrapper script:

```bash
# Copy the example
cp bconsole.example.sh bconsole
chmod +x bconsole

# Edit bconsole and set your hostname
vim bconsole
```

Example wrapper content:
```bash
#!/usr/bin/env bash
ssh your-bareos-host "sudo bconsole $*"
```

Add the wrapper directory to your PATH or reference it in the MCP config.

#### Option C: Docker/Container Setup

Run bconsole in a container that has network access to your Bareos Director.

### 3. Configure MCP Client

#### For Claude Code (CLI)

Create `.mcp.json` in your project directory:

```bash
cp .mcp.json.example .mcp.json
# Edit with your paths
vim .mcp.json
```

Example config:
```json
{
  "bareos": {
    "command": "/absolute/path/to/bareos-mcp-server/target/release/bareos-mcp-server",
    "args": [],
    "env": {
      "PATH": "/absolute/path/to/bareos-mcp-server:${PATH}"
    }
  }
}
```

#### For Claude Desktop

Add to your Claude Desktop config file:

**macOS:** `~/Library/Application Support/Claude/claude_desktop_config.json`
**Windows:** `%APPDATA%\Claude\claude_desktop_config.json`

```json
{
  "mcpServers": {
    "bareos": {
      "command": "/absolute/path/to/bareos-mcp-server/target/release/bareos-mcp-server",
      "args": []
    }
  }
}
```

## Usage

### With Claude Code

```bash
cd your-project
claude
```

Example queries:
- "Show me the last 10 backup jobs"
- "What's the status of job 12345?"
- "List all Bareos clients"
- "How much storage is in the Full pool?"
- "Show me volumes ready for pruning"

### Available Tools

| Tool | Description | Parameters |
|------|-------------|------------|
| `list_jobs` | List backup jobs with filters | `job`, `client`, `jobstatus`, `jobtype`, `joblevel`, `volume`, `pool` (all optional filters); `days`, `hours` (time filters, hours wins); `last`, `count` (output modes, count wins) |
| `get_job_status` | Get detailed status of a job | `job_id` (required) |
| `get_job_log` | View complete job log | `job_id` (required) |
| `list_files` | List files backed up in a job | `job_id` (required) |
| `list_clients` | List all file daemon clients | None |
| `list_filesets` | List backup filesets | None |
| `list_pools` | List storage pools | None |
| `list_volumes` | List volumes/media | `pool` (optional filter) |

#### `list_jobs` Parameters Detail

The `list_jobs` tool supports multiple filters and options that can be combined:

**Filters** (all optional, can be combined):
- `job` - Filter by job name
- `client` - Filter by client name
- `jobstatus` - Filter by status (e.g., `T`=terminated, `f`=failed, `R`=running)
- `jobtype` - Filter by type (e.g., `B`=backup, `R`=restore, `V`=verify, `D`=admin, `C`=copy, `M`=migration)
- `joblevel` - Filter by level (e.g., `F`=full, `I`=incremental, `D`=differential)
- `volume` - Filter by volume name
- `pool` - Filter by pool name

**Understanding Job Types**:
- When users ask about "backups" or "backup performance", they typically mean **backup jobs only** (`jobtype: "B"`). Do not include verification, restore, or other job types unless explicitly requested.
- Verification jobs (`jobtype: "V"`) validate backup integrity but are not backups themselves
- Use `jobtype` filter to focus on specific operations when ambiguity exists

**Time Filters** (mutually exclusive):
- `days` - Show jobs from last N days
- `hours` - Show jobs from last N hours
- **Precedence**: If both provided, `hours` takes precedence (matches Bareos behavior)

**Output Modes** (mutually exclusive):
- `last` - Show only the most recent run of each job (WARNING: if jobs ran multiple times in the time range, only the LAST run will be returned)
- `count` - Show count of matching jobs instead of details
- **Precedence**: If both provided, `count` takes precedence (matches Bareos behavior)

**Examples**:
- `{"client": "web-server", "days": 7}` - All jobs for web-server in last 7 days
- `{"jobstatus": "f", "hours": 24, "count": true}` - Count of failed jobs in last 24 hours
- `{"pool": "Full", "last": true}` - Most recent run of each job type in the Full pool (only one run per job)

### Direct Testing

Test the server without an MCP client:

```bash
# Initialize
echo '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}' | ./target/release/bareos-mcp-server

# List tools
echo '{"jsonrpc":"2.0","id":1,"method":"tools/list","params":{}}' | ./target/release/bareos-mcp-server

# Call a tool - list jobs from last 7 days
echo '{"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"list_jobs","arguments":{"days":7}}}' | ./target/release/bareos-mcp-server

# List the most recent jobs for a specific client
echo '{"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"list_jobs","arguments":{"client":"backup-client-1","last":true}}}' | ./target/release/bareos-mcp-server

# Count failed jobs in the last 24 hours
echo '{"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"list_jobs","arguments":{"hours":24,"jobstatus":"f","count":true}}}' | ./target/release/bareos-mcp-server
```

## Troubleshooting

### bconsole not found
```bash
# Check if bconsole is in PATH
which bconsole

# Or create wrapper script (see Setup section)
```

### Permission denied
```bash
# Check bconsole config permissions
ls -l /etc/bareos/bconsole.conf

# May need to add user to bareos group
sudo usermod -a -G bareos $USER
```

### Connection refused
```bash
# Test bconsole directly
echo "version" | bconsole

# Check Director is running
systemctl status bareos-dir
```

### SSH issues (remote setup)
```bash
# Test SSH connection
ssh your-host "bconsole" << EOF
version
quit
EOF

# Check SSH key authentication
ssh -v your-host
```

## License

[MPL-2.0](LICENSE)

