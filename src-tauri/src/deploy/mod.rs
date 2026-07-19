//! 部署 / 交付子系统。
//!
//! MVP：SSH 上传（rsync/scp）+ 可选远程命令。见 `targets::ssh`。
//! 后续可扩展 Jenkins / Docker / K8s Adapter（DESIGN.md §9）。

pub mod targets;
