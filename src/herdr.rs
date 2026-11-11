use std::{
    io::{BufRead, BufReader, Write},
    os::unix::net::UnixStream,
    path::PathBuf,
};

use anyhow::{Context, Result, anyhow};
use serde::Deserialize;
use serde_json::{Value, json};

pub struct Herdr {}

#[derive(Clone, Debug, Deserialize)]
pub struct Workspace {
    pub workspace_id: String,
    pub label: String,
    pub focused: bool,
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
    result: Option<Value>,
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

    fn send(&self, request: Value) -> Result<Response> {
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

    fn check_response(response: Response, request_id: &str) -> Result<Value> {
        if response.id != request_id {
            return Err(anyhow!("herdr response has unexpected request id"));
        }
        if let Some(error) = response.error {
            return Err(anyhow!(
                "herdr API error ({}): {}",
                error.code,
                error.message
            ));
        }
        response
            .result
            .context("herdr response did not contain a result")
    }

    pub fn list_workspaces(&self) -> Result<Vec<Workspace>> {
        let request = json!({
            "id": "sandbox-list-workspaces",
            "method": "workspace.list",
            "params": {}
        });

        let response = self.send(request).context("error sending request")?;
        let result: WorkspaceListResult =
            serde_json::from_value(Self::check_response(response, "sandbox-list-workspaces")?)
                .context("error parsing workspace list result")?;

        if result.result_type != "workspace_list" {
            return Err(anyhow!(
                "herdr response has unexpected result type: {}",
                result.result_type
            ));
        }

        Ok(result.workspaces)
    }

    pub fn focus_workspace(&self, workspace_id: &str) -> Result<()> {
        let request_id = "sandbox-focus-workspace";
        let response = self
            .send(json!({
                "id": request_id,
                "method": "workspace.focus",
                "params": {
                    "workspace_id": workspace_id
                }
            }))
            .context("error sending workspace focus request")?;
        Self::check_response(response, request_id)?;
        Ok(())
    }
}

fn socket_path() -> Result<PathBuf> {
    let home_dir = dirs::home_dir().context("error getting home directory")?;

    Ok(home_dir.join(".config").join("herdr").join("herdr.sock"))
}
