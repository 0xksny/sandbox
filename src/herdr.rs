use std::{
    io::{BufRead, BufReader, Write},
    os::unix::net::UnixStream,
    path::PathBuf,
};

use anyhow::{Context, Result, anyhow};
use serde::Deserialize;
use serde_json::{Value, json};

pub struct Herdr {}

#[derive(Debug, Deserialize)]
struct Workspace {
    label: String,
}

#[derive(Debug, Deserialize)]
struct WorkspaceListResult {
    #[serde(rename = "type")]
    result_type: String,
    workspaces: Vec<Workspace>,
}

#[derive(Debug, Deserialize)]
struct Response {
    id: String,
    result: Option<WorkspaceListResult>,
    error: Option<ErrorResponse>,
}

#[derive(Debug, Deserialize)]
struct ErrorResponse {
    code: String,
    message: String,
}

impl Herdr {
    pub fn new() -> Self {
        Herdr {}
    }

    fn send(self, request: Value) -> Result<Response> {
        let socket_path = socket_path()?;
        let mut stream = UnixStream::connect(&socket_path).context(format!(
            "error connecting to herdr socket {}",
            socket_path.display()
        ))?;

        writeln!(stream, "{}", request).context("error writing herdr request")?;
        stream.flush().context("error flushing herdr request")?;

        let mut response_line = String::new();
        BufReader::new(stream)
            .read_line(&mut response_line)
            .context("error reading herdr response")?;

        if response_line.is_empty() {
            return Err(anyhow!("herdr returned an empty response"));
        }

        serde_json::from_str(&response_line).context("error parsing herdr response")
    }

    pub fn list_workspaces(self) -> Result<Vec<String>> {
        let request = json!({
            "id": "sandbox-list-workspaces",
            "method": "workspace.list",
            "params": {}
        });

        let response = self.send(request).context("error sending request")?;

        if response.id != "sandbox-list-workspaces" {
            return Err(anyhow!("herdr response has unexpected request id"));
        }

        if let Some(error) = response.error {
            return Err(anyhow!(
                "herdr API error ({}): {}",
                error.code,
                error.message
            ));
        }

        let result = response
            .result
            .context("herdr response did not contain a result")?;

        if result.result_type != "workspace_list" {
            return Err(anyhow!(
                "herdr response has unexpected result type: {}",
                result.result_type
            ));
        }

        Ok(result
            .workspaces
            .into_iter()
            .map(|workspace| workspace.label)
            .collect())
    }
}

fn socket_path() -> Result<PathBuf> {
    let home_dir = dirs::home_dir().context("error getting home directory")?;

    Ok(home_dir.join(".config").join("herdr").join("herdr.sock"))
}
