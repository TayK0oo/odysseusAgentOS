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
        // Garde anti-composition (REVIEW-W8-W9 B1) : une règle `Allow` par préfixe
        // (`echo`) ne doit pas autoriser silencieusement une commande composée
        // (`echo hi; rm -rf /`). Les métacaractères hors quotes escaladent en `Prompt`.
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
                return Err(OmegaError::ApprovalRequired {
                    command: command.to_string(),
                    reason,
                })
            }
            ExecDecision::Allow => {}
        }

        let mut cmd = tokio::process::Command::new("sh");
        cmd.arg("-c").arg(command);
        // Groupe de processus dédié (leader = `sh`) : au timeout, on tue le groupe
        // entier (`kill -9 -pgid`), pas le seul `sh` — sinon les petits-enfants
        // (`sleep` en arrière-plan) survivent (REVIEW-W8-W9 B2). Linux-only, Phase 1.
        cmd.process_group(0);
        let child = cmd
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()
            .map_err(|e| OmegaError::Io(format!("spawn local : {e}")))?;

        let pid = child.id();
        let output = tokio::time::timeout(self.timeout, child.wait_with_output()).await;
        match output {
            Err(_) => {
                // Tue le groupe entier via le PGID (= PID du leader).
                if let Some(pid) = pid {
                    let _ = std::process::Command::new("kill")
                        .arg("-9")
                        .arg(format!("-{pid}"))
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

    #[tokio::test]
    async fn compound_command_escalates_to_prompt_and_never_executes() {
        // (B1) `echo hi; touch <temoin>` matche `echo` mais doit escalader en `Prompt`
        // au lieu d'autoriser : le fichier témoin ne doit jamais être créé.
        let dir =
            std::env::temp_dir().join(format!("omega-sandbox-compound-{}", std::process::id()));
        let _ = std::fs::create_dir_all(&dir);
        let witness = dir.join("ne-doit-pas-exister");
        let _ = std::fs::remove_file(&witness);
        let cmd = format!("echo hi; touch {}", witness.display());
        let err = backend().run(&cmd).await.unwrap_err();
        assert!(
            matches!(err, OmegaError::ApprovalRequired { .. }),
            "attendu ApprovalRequired, obtenu : {err:?}"
        );
        assert!(!witness.exists(), "la commande composée a été exécutée !");
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[tokio::test]
    async fn timeout_kills_grandchildren_not_just_sh() {
        // (B2) petit-fils en arrière-plan : la commande externe `sh <script>` ne
        // contient aucun métacaractère (→ `Allow`), mais le script lance `sleep 57`
        // en tâche de fond. Après timeout, aucun `sleep 57` ne doit survivre
        // (kill du groupe, pas du seul PID).
        let dir =
            std::env::temp_dir().join(format!("omega-sandbox-grandchild-{}", std::process::id()));
        let _ = std::fs::create_dir_all(&dir);
        let script = dir.join("bg.sh");
        std::fs::write(&script, "sleep 57 & wait\n").unwrap();
        let policy: Arc<dyn ExecPolicy> =
            Arc::new(RuleBasedExecPolicy::new("t").rule("sh", ExecDecision::Allow));
        let b = LocalSandboxBackend::new(policy, Duration::from_millis(300));
        let err = b
            .run(&format!("sh {}", script.display()))
            .await
            .unwrap_err();
        assert!(matches!(err, OmegaError::CommandTimeout { .. }));
        // Boucle courte : le SIGKILL de groupe est quasi-synchrone, on tolère 2 s.
        let mut survived = true;
        for _ in 0..20 {
            let out = std::process::Command::new("pgrep")
                .arg("-f")
                .arg("sleep 57")
                .output()
                .expect("pgrep indisponible");
            if out.stdout.is_empty() {
                survived = false;
                break;
            }
            std::thread::sleep(Duration::from_millis(100));
        }
        let _ = std::fs::remove_dir_all(&dir);
        assert!(!survived, "un petit-fils `sleep 57` a survécu au timeout !");
    }
}
