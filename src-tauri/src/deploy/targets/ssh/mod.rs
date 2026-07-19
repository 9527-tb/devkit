//! SSH 上传部署（MVP：rsync / scp + 可选远程命令）。

use std::io::{BufRead, BufReader};
use std::path::Path;
use std::process::{Command, Stdio};

#[derive(serde::Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SshUploadRequest {
    pub local_path: String,
    pub host: String,
    pub user: String,
    pub remote_path: String,
    #[serde(default)]
    pub remote_cmd: Option<String>,
    #[serde(default)]
    pub identity_file: Option<String>,
}

#[derive(serde::Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SshUploadResult {
    pub ok: bool,
    pub logs: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

pub fn ssh_upload(req: SshUploadRequest) -> SshUploadResult {
    let mut logs = Vec::new();
    match run_upload(&req, &mut logs) {
        Ok(()) => SshUploadResult {
            ok: true,
            logs,
            error: None,
        },
        Err(e) => SshUploadResult {
            ok: false,
            logs,
            error: Some(e),
        },
    }
}

fn run_upload(req: &SshUploadRequest, logs: &mut Vec<String>) -> Result<(), String> {
    let local_path = req.local_path.trim();
    let local = Path::new(local_path);
    if !local.exists() {
        return Err(format!("本地路径不存在: {}", req.local_path));
    }
    let host = req.host.trim();
    let user = req.user.trim();
    let remote = req.remote_path.trim();
    if host.is_empty() || user.is_empty() || remote.is_empty() {
        return Err("host / user / remotePath 不能为空".into());
    }

    let target = format!("{user}@{host}:{remote}");
    let local_s = local.to_string_lossy();
    let trailing = if local.is_dir() { "/" } else { "" };
    let source = format!("{local_s}{trailing}");

    if crate::platform::find_executable("rsync").is_some() {
        logs.push(format!("[DevKit] rsync → {target}"));
        let mut cmd = Command::new("rsync");
        cmd.args(["-avz"]);
        if let Some(id) = identity_arg(&req.identity_file) {
            cmd.args(["-e", &format!("ssh -i {id} -o StrictHostKeyChecking=accept-new")]);
        }
        cmd.arg(&source);
        cmd.arg(&target);
        run_cmd(&mut cmd, logs)?;
    } else if let Some(scp) = crate::platform::find_executable("scp") {
        logs.push(format!("[DevKit] scp → {target}"));
        let mut cmd = Command::new(&scp);
        cmd.args(["-r"]);
        if let Some(id) = identity_arg(&req.identity_file) {
            cmd.args(["-i", &id, "-o", "StrictHostKeyChecking=accept-new"]);
        }
        cmd.arg(&req.local_path);
        cmd.arg(&target);
        run_cmd(&mut cmd, logs)?;
    } else {
        return Err("未找到 rsync 或 scp，请安装 OpenSSH 客户端".into());
    }

    if let Some(rc) = req.remote_cmd.as_deref().map(str::trim).filter(|s| !s.is_empty()) {
        logs.push(format!("[DevKit] ssh remote: {rc}"));
        let ssh_bin = crate::platform::find_executable("ssh").ok_or("未找到 ssh 命令")?;
        let mut cmd = Command::new(&ssh_bin);
        if let Some(id) = identity_arg(&req.identity_file) {
            cmd.args(["-i", &id, "-o", "StrictHostKeyChecking=accept-new"]);
        }
        cmd.arg(format!("{user}@{host}"));
        cmd.arg(rc);
        run_cmd(&mut cmd, logs)?;
    }

    logs.push("[DevKit] 部署完成".into());
    Ok(())
}

fn identity_arg(identity_file: &Option<String>) -> Option<String> {
    identity_file
        .as_ref()
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string())
}

fn run_cmd(cmd: &mut Command, logs: &mut Vec<String>) -> Result<(), String> {
    cmd.stdout(Stdio::piped()).stderr(Stdio::piped());
    let mut child = cmd.spawn().map_err(|e| format!("启动命令失败: {e}"))?;
    if let Some(out) = child.stdout.take() {
        for line in BufReader::new(out).lines().map_while(Result::ok) {
            logs.push(line);
        }
    }
    if let Some(err) = child.stderr.take() {
        for line in BufReader::new(err).lines().map_while(Result::ok) {
            logs.push(line);
        }
    }
    let status = child.wait().map_err(|e| format!("等待命令结束失败: {e}"))?;
    if status.success() {
        Ok(())
    } else {
        Err(format!(
            "命令失败 (exit {})",
            status.code().map(|c| c.to_string()).unwrap_or_else(|| "?".into())
        ))
    }
}
