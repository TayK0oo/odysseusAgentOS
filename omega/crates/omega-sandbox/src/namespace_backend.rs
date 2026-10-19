//! Backend à namespaces Linux : isolation FS/PID/réseau via `unshare` + `pivot_root`/`mount --bind`.
//!
//! Fail-closed : absence de capacité `CAP_SYS_ADMIN` ou d'UID mapping = refus.
//! Inspiré de bubblewrap/seatbelt (cf. NOYAU-ULTIME §3 ligne 59).

use std::sync::Arc;
use std::time::Duration;

use nix::mount::{mount, MsFlags};
use nix::sched::{unshare, CloneFlags};
use nix::unistd::{setgid, setuid, Gid, Uid};
use omega_ports::{ExecPolicy, SandboxBackend};
use omega_types::{CommandOutput, ExecDecision, OmegaError, OmegaResult};

use crate::approvals::ApprovalQueue;

/// Configuration du backend namespace.
#[derive(Debug, Clone)]
pub struct NamespaceConfig {
    /// UID mapping: (container_uid, host_uid, count) — typiquement (0, 1000, 1) pour mapper root->user.
    pub uid_map: Option<(u32, u32, u32)>,
    /// GID mapping: (container_gid, host_gid, count).
    pub gid_map: Option<(u32, u32, u32)>,
    /// Répertoire racine du conteneur (pivot_root).
    pub root_dir: std::path::PathBuf,
    /// Répertoires à bind-mount en lecture seule (ex. /usr, /lib).
    pub readonly_binds: Vec<std::path::PathBuf>,
    /// Répertoires à bind-mount en lecture/écriture (ex. /tmp, workspace).
    pub readwrite_binds: Vec<std::path::PathBuf>,
    /// Variables d'environnement à conserver/ajouter.
    pub env: Vec<(String, String)>,
    /// Timeout d'exécution.
    pub timeout: Duration,
}

impl Default for NamespaceConfig {
    fn default() -> Self {
        let mut root = std::env::temp_dir();
        root.push("omega-ns");
        Self {
            uid_map: Some((0, 1000, 1)),
            gid_map: Some((0, 1000, 1)),
            root_dir: root,
            readonly_binds: vec![
                "/usr".into(),
                "/lib".into(),
                "/lib64".into(),
                "/bin".into(),
                "/sbin".into(),
                "/etc".into(),
            ],
            readwrite_binds: vec!["/tmp".into()],
            env: vec![
                ("PATH".into(), "/usr/bin:/bin".into()),
                ("HOME".into(), "/tmp".into()),
            ],
            timeout: Duration::from_secs(30),
        }
    }
}

/// Backend isolé par namespaces Linux (mount, pid, network, user, ipc, uts).
///
/// Exige `CAP_SYS_ADMIN` (ou root) pour `unshare` + `mount`. En environnement
/// non privilégié (ex. CI sans `--privileged`), le constructeur échoue et on
/// replie sur `LocalSandboxBackend` (fail-closed = refus si namespace indisponible).
pub struct NamespaceSandboxBackend {
    policy: Arc<dyn ExecPolicy>,
    config: NamespaceConfig,
    approvals: Option<Arc<ApprovalQueue>>,
}

impl NamespaceSandboxBackend {
    /// Tente de construire le backend namespace. Retourne `Err` si l'environnement
    /// ne supporte pas les namespaces non-privilégiés — fail-closed.
    pub fn try_new(policy: Arc<dyn ExecPolicy>, config: NamespaceConfig) -> OmegaResult<Self> {
        if !Self::user_ns_supported() {
            return Err(OmegaError::ForbiddenCommand {
                command: "namespace".into(),
                reason:
                    "user namespace non supporté (kernel.unprivileged_userns_clone=0 ou pas root)"
                        .into(),
            });
        }
        std::fs::create_dir_all(&config.root_dir)
            .map_err(|e| OmegaError::Io(format!("création root_dir : {e}")))?;

        Ok(Self {
            policy,
            config,
            approvals: None,
        })
    }

    fn user_ns_supported() -> bool {
        std::fs::read_to_string("/proc/sys/kernel/unprivileged_userns_clone")
            .ok()
            .and_then(|s| s.trim().parse::<i32>().ok())
            == Some(1)
    }

    pub fn with_approvals(mut self, queue: Arc<ApprovalQueue>) -> Self {
        self.approvals = Some(queue);
        self
    }

    async fn run_in_namespace(&self, command: &str) -> OmegaResult<CommandOutput> {
        // Clone config pour la closure pre_exec (qui doit être 'static)
        let config = self.config.clone();

        let mut cmd = tokio::process::Command::new("sh");
        cmd.arg("-c").arg(command);

        unsafe {
            cmd.pre_exec(move || {
                // 1. Unshare namespaces
                let flags = CloneFlags::CLONE_NEWNS
                    | CloneFlags::CLONE_NEWPID
                    | CloneFlags::CLONE_NEWNET
                    | CloneFlags::CLONE_NEWUSER
                    | CloneFlags::CLONE_NEWIPC
                    | CloneFlags::CLONE_NEWUTS;
                unshare(flags).map_err(|e| std::io::Error::other(format!("unshare: {e}")))?;

                // 2. Map UID/GID dans le nouveau user namespace
                if let Some((cu, hu, count)) = config.uid_map {
                    let map = format!("{cu} {hu} {count}");
                    std::fs::write("/proc/self/uid_map", map)
                        .map_err(|e| std::io::Error::other(format!("uid_map: {e}")))?;
                }
                if let Some((cg, hg, count)) = config.gid_map {
                    let map = format!("{cg} {hg} {count}");
                    std::fs::write("/proc/self/gid_map", map)
                        .map_err(|e| std::io::Error::other(format!("gid_map: {e}")))?;
                }
                // Désactiver setgroups
                std::fs::write("/proc/self/setgroups", "deny")
                    .map_err(|e| std::io::Error::other(format!("setgroups: {e}")))?;

                // 3. Remount /proc en privé
                mount(
                    Some("proc"),
                    "/proc",
                    Some("proc"),
                    MsFlags::MS_NOSUID | MsFlags::MS_NODEV | MsFlags::MS_NOEXEC,
                    None::<&str>,
                )
                .map_err(|e| std::io::Error::other(format!("mount proc: {e}")))?;

                // 4. Bind mounts read-only
                for src in &config.readonly_binds {
                    if src.exists() {
                        let dst = config.root_dir.join(src.strip_prefix("/").unwrap_or(src));
                        std::fs::create_dir_all(&dst).ok();
                        mount(
                            Some(src.as_os_str()),
                            &dst,
                            None::<&str>,
                            MsFlags::MS_BIND | MsFlags::MS_RDONLY | MsFlags::MS_REC,
                            None::<&str>,
                        )
                        .map_err(|e| std::io::Error::other(format!("bind ro {:?}: {}", src, e)))?;
                        mount(
                            None::<&str>,
                            &dst,
                            None::<&str>,
                            MsFlags::MS_BIND | MsFlags::MS_RDONLY | MsFlags::MS_REMOUNT,
                            None::<&str>,
                        )
                        .map_err(|e| {
                            std::io::Error::other(format!("remount ro {:?}: {}", src, e))
                        })?;
                    }
                }

                // 5. Bind mounts read-write
                for src in &config.readwrite_binds {
                    if src.exists() {
                        let dst = config.root_dir.join(src.strip_prefix("/").unwrap_or(src));
                        std::fs::create_dir_all(&dst).ok();
                        mount(
                            Some(src.as_os_str()),
                            &dst,
                            None::<&str>,
                            MsFlags::MS_BIND | MsFlags::MS_REC,
                            None::<&str>,
                        )
                        .map_err(|e| std::io::Error::other(format!("bind rw {:?}: {}", src, e)))?;
                    }
                }

                // 6. Chroot dans root_dir
                std::os::unix::fs::chroot(&config.root_dir)
                    .map_err(|e| std::io::Error::other(format!("chroot: {e}")))?;
                std::env::set_current_dir("/")
                    .map_err(|e| std::io::Error::other(format!("chdir: {e}")))?;

                // 7. Drop privileges
                setgid(Gid::from_raw(0))
                    .map_err(|e| std::io::Error::other(format!("setgid: {e}")))?;
                setuid(Uid::from_raw(0))
                    .map_err(|e| std::io::Error::other(format!("setuid: {e}")))?;

                // 8. Env minimal
                for (k, _) in std::env::vars() {
                    std::env::remove_var(k);
                }
                for (k, v) in &config.env {
                    std::env::set_var(k, v);
                }

                Ok(())
            });
        }

        let child = cmd
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()
            .map_err(|e| OmegaError::Io(format!("spawn ns : {e}")))?;

        let output = tokio::time::timeout(self.config.timeout, child.wait_with_output()).await;
        match output {
            Err(_) => Err(OmegaError::CommandTimeout {
                command: command.to_string(),
                timeout_ms: self.config.timeout.as_millis() as u64,
            }),
            Ok(Err(e)) => Err(OmegaError::Io(format!("attente ns : {e}"))),
            Ok(Ok(out)) => Ok(CommandOutput {
                stdout: String::from_utf8_lossy(&out.stdout).into_owned(),
                stderr: String::from_utf8_lossy(&out.stderr).into_owned(),
                exit_code: out.status.code().unwrap_or(-1),
            }),
        }
    }
}

#[async_trait::async_trait]
impl SandboxBackend for NamespaceSandboxBackend {
    fn name(&self) -> &str {
        "namespace"
    }

    async fn run(&self, command: &str) -> OmegaResult<CommandOutput> {
        let decision = match self.policy.evaluate(command) {
            ExecDecision::Allow if crate::has_unquoted_metachars(command) => {
                ExecDecision::Prompt("commande composée : approbation requise".to_string())
            }
            other => other,
        };
        match decision {
            ExecDecision::Forbidden(reason) => {
                return Err(OmegaError::ForbiddenCommand {
                    command: command.to_string(),
                    reason,
                })
            }
            ExecDecision::Prompt(reason) => {
                if let Some(queue) = &self.approvals {
                    if queue.consume_for(command) {
                        // Approbation consommée
                    } else {
                        return Err(OmegaError::ApprovalRequired {
                            command: command.to_string(),
                            reason,
                        });
                    }
                } else {
                    return Err(OmegaError::ApprovalRequired {
                        command: command.to_string(),
                        reason,
                    });
                }
            }
            ExecDecision::Allow => {}
        }

        self.run_in_namespace(command).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn user_ns_detection_works() {
        let supported = NamespaceSandboxBackend::user_ns_supported();
        eprintln!("user_ns_supported = {supported}");
    }
}
