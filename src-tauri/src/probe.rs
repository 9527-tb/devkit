//! HTTP 探活。
//!
//! 对应 docs/DEV_PRODUCTIVITY_DESIGN.md ⑦

use crate::models::HttpProbeResult;
use std::time::{Duration, Instant};

/// 对 `url` 发 GET，超时 `timeout_ms`（默认 5000）。
pub fn probe_http(url: String, timeout_ms: Option<u64>) -> HttpProbeResult {
    let timeout = Duration::from_millis(timeout_ms.unwrap_or(5000).clamp(200, 60_000));
    let url = url.trim().to_string();
    if url.is_empty() {
        return HttpProbeResult {
            ok: false,
            status: None,
            ms: 0,
            error: Some("URL 为空".into()),
        };
    }

    let client = match reqwest::blocking::Client::builder()
        .timeout(timeout)
        .redirect(reqwest::redirect::Policy::limited(5))
        .build()
    {
        Ok(c) => c,
        Err(e) => {
            return HttpProbeResult {
                ok: false,
                status: None,
                ms: 0,
                error: Some(e.to_string()),
            };
        }
    };

    let started = Instant::now();
    match client.get(&url).send() {
        Ok(resp) => {
            let ms = started.elapsed().as_millis() as u64;
            let status = resp.status().as_u16();
            let ok = resp.status().is_success();
            HttpProbeResult {
                ok,
                status: Some(status),
                ms,
                error: if ok {
                    None
                } else {
                    Some(format!("HTTP {status}"))
                },
            }
        }
        Err(e) => HttpProbeResult {
            ok: false,
            status: None,
            ms: started.elapsed().as_millis() as u64,
            error: Some(e.to_string()),
        },
    }
}
