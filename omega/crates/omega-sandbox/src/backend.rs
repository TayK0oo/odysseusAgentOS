//! Backend local : exécute via `sh -c` **après** décision de l'`ExecPolicy`.
//!
//! - `Allow` → exécute avec timeout (tue le processus au dépassement).
//! - `Prompt` → refusé sans approbation (`ApprovalRequired`) : l'approbation humaine
//!   arrive en incrément 2 (file d'approbations + `ApprovalReviewContributor`).
//! - `Forbidden` → refusé **sans exécuter** (jamais de fork).

use std::sync::Arc;
use std::time::Duration;

use omega_ports::{ExecPolicy, SandboxBackend};
use omega_types::{CommandOutput, ExecDecision, OmegaError, OmegaResult};

/// Backend local borné (timeout + politique).
pub struct LocalSandboxBackend {
    policy: Arc<dyn ExecPolicy>,
    timeout: Duration,
}

impl LocalSandboxBackend {
    pub fn new(policy: Arc<dyn ExecPolicy>, timeout: Duration) -> Self {
        Self { policy, timeout }
    }
}

#[async_trait::async_trait]
impl SandboxBackend for LocalSandboxBackend {
    fn name(&self) -> &str {
        "local"
    }

    async fn run(&self, command: &str) -> OmegaResult<CommandOutput> {
        match self.policy.evaluate(command) {
            ExecDecision::Forbidden(reason) => {
                return Err(OmegaError::ForbiddenCommand {
                    command: command.to_string(),
                    reason,
                })
            }
            ExecDecision::Prompt(reason) => {
                return Err(OmegaError::ApprovalRequired {
                    command: command.to_string(),
                    reason,
                })
            }
            ExecDecision::Allow => {}
        }

        let child = tokio::process::Command::new("sh")
            .arg("-c")
            .arg(command)
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()
            .map_err(|e| OmegaError::Io(format!("spawn local : {e}")))?;

        let pid = child.id();
        let output = tokio::time::timeout(self.timeout, child.wait_with_output()).await;
        match output {
            Err(_) => {
                // `wait_with_output` a consommé `child` : on tue par PID (Linux).
                // Le processus peut déjà avoir terminé entre-temps : on ignore l'échec du kill.
                if let Some(pid) = pid {
                    let _ = std::process::Command::new("kill")
                        .arg("-9")
                        .arg(pid.to_string())
                        .output();
                }
                Err(OmegaError::CommandTimeout {
                    command: command.to_string(),
                    timeout_ms: self.timeout.as_millis() as u64,
                })
            }
            Ok(Err(e)) => Err(OmegaError::Io(format!("attente locale : {e}"))),
            Ok(Ok(out)) => Ok(CommandOutput {
                stdout: String::from_utf8_lossy(&out.stdout).into_owned(),
                stderr: String::from_utf8_lossy(&out.stderr).into_owned(),
                exit_code: out.status.code().unwrap_or(-1),
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::policy::RuleBasedExecPolicy;

    fn backend() -> LocalSandboxBackend {
        let policy: Arc<dyn ExecPolicy> = Arc::new(
            RuleBasedExecPolicy::new("test")
                .rule("echo", ExecDecision::Allow)
                .rule("rm", ExecDecision::Forbidden("non".to_string()))
                .rule("sudo", ExecDecision::Prompt("elev".to_string())),
        );
        LocalSandboxBackend::new(policy, Duration::from_secs(5))
    }

    #[tokio::test]
    async fn allowed_command_runs_and_captures() {
        let out = backend().run("echo hello").await.unwrap();
        assert_eq!(out.stdout.trim(), "hello");
        assert_eq!(out.exit_code, 0);
    }

    #[tokio::test]
    async fn forbidden_command_never_executes() {
        // `rm /tmp/omega-ne-doit-pas-exister-<pid>` : si exécuté, le fichier
        // disparaîtrait. On vérifie l'erreur typée (le refus précède le fork).
        let path = std::env::temp_dir().join(format!("omega-sandbox-{}", std::process::id()));
        std::fs::write(&path, "temoin").unwrap();
        let cmd = format!("rm {}", path.display());
        let err = backend().run(&cmd).await.unwrap_err();
        assert!(matches!(err, OmegaError::ForbiddenCommand { .. }));
        assert!(path.exists(), "la commande interdite a ete executee !");
        let _ = std::fs::remove_file(&path);
    }

    #[tokio::test]
    async fn prompt_command_requires_approval() {
        let err = backend().run("sudo ls").await.unwrap_err();
        assert!(matches!(err, OmegaError::ApprovalRequired { .. }));
    }

    #[tokio::test]
    async fn timeout_kills_long_command() {
        let policy: Arc<dyn ExecPolicy> =
            Arc::new(RuleBasedExecPolicy::new("t").rule("sleep", ExecDecision::Allow));
        let b = LocalSandboxBackend::new(policy, Duration::from_millis(200));
        let err = b.run("sleep 30").await.unwrap_err();
        assert!(matches!(err, OmegaError::CommandTimeout { .. }));
    }
}
