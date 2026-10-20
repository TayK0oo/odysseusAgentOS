//! Backend local : exécute via `sh -c` **après** décision de l'`ExecPolicy`.
//!
//! - `Allow` → exécute avec timeout (tue le processus au dépassement).
//! - `Prompt` → sans approbation consommée dans la file (`ApprovalQueue`) :
//!   `ApprovalRequired` **sans exécuter**. Avec une approbation `Approved` pour
//!   la commande exacte, elle est **consommée (usage unique)** puis la commande
//!   s'exécute normalement.
//! - `Forbidden` → refusé **sans exécuter** (jamais de fork, jamais de file).

use std::sync::Arc;
use std::time::Duration;

use omega_ports::{ExecPolicy, SandboxBackend};
use omega_types::{CommandOutput, ExecDecision, OmegaError, OmegaResult};

use crate::approvals::ApprovalQueue;

/// Backend local borné (timeout + politique + file d'approbations optionnelle).
pub struct LocalSandboxBackend {
    policy: Arc<dyn ExecPolicy>,
    timeout: Duration,
    approvals: Option<Arc<ApprovalQueue>>,
}

impl LocalSandboxBackend {
    pub fn new(policy: Arc<dyn ExecPolicy>, timeout: Duration) -> Self {
        Self {
            policy,
            timeout,
            approvals: None,
        }
    }

    /// Attache une file d'approbations (incrément 2 : lève le `Prompt`).
    pub fn with_approvals(mut self, queue: Arc<ApprovalQueue>) -> Self {
        self.approvals = Some(queue);
        self
    }

    /// File d'approbations attachée, s'il y en a une.
    pub fn approval_queue(&self) -> Option<&Arc<ApprovalQueue>> {
        self.approvals.as_ref()
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
                // File d'approbations (inc2) : une approbation `Approved` pour la
                // commande exacte est consommée (usage unique) et l'exécution
                // continue ; sinon refus sans fork.
                if let Some(queue) = &self.approvals {
                    if queue.consume_for(command) {
                        // Approbation consommée : on continue vers l'exécution.
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
                // La descendance est capturée AVANT les kills : le kill de
                // groupe suppose que `setpgid` a été appliqué (au child). Sur
                // certaines toolchains il ne l'est pas — le child reste alors
                // dans le groupe du parent, le kill de groupe ne touche que le
                // leader, et le petit-fils survit (constaté en CI le
                // 2026-10-20 : `sleep 57` vivait encore). On capture donc la
                // descendance et on la tue explicitement.
                let descendants = pid.map(descendants).unwrap_or_default();
                if let Some(pid) = pid {
                    // 1. le groupe du leader (sans effet si le groupe n'existe pas)
                    let _ = std::process::Command::new("kill")
                        .arg("-9")
                        .arg(format!("-{pid}"))
                        .output();
                    // 2. le leader lui-même
                    let _ = std::process::Command::new("kill")
                        .arg("-9")
                        .arg(pid.to_string())
                        .output();
                }
                // 3. les petits-enfants capturés
                for d in descendants {
                    let _ = std::process::Command::new("kill")
                        .arg("-9")
                        .arg(d.to_string())
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

/// Descendants (enfants, petits-enfants, …) d'un PID, lus dans `/proc`.
///
/// Le kill de groupe ne suffit pas : si `setpgid` n'a pas été appliqué, les
/// petits-enfants restent dans le groupe du parent et survivent au timeout.
/// On les capture donc avant de tuer quoi que ce soit.
fn descendants(pid: u32) -> Vec<u32> {
    let mut trouves = Vec::new();
    let mut frontier = vec![pid];
    while let Some(parent) = frontier.pop() {
        for entry in std::fs::read_dir("/proc").into_iter().flatten().flatten() {
            let Ok(nom) = entry.file_name().into_string() else {
                continue;
            };
            let Ok(child) = nom.parse::<u32>() else {
                continue;
            };
            let Ok(stat) = std::fs::read_to_string(format!("/proc/{child}/stat")) else {
                continue;
            };
            // Après la dernière ')' : etat, ppid, pgrp, …
            let Some((_, reste)) = stat.rsplit_once(')') else {
                continue;
            };
            let champs: Vec<&str> = reste.split_whitespace().collect();
            let Some(ppid) = champs.get(1).and_then(|v| v.parse::<u32>().ok()) else {
                continue;
            };
            if ppid == parent {
                trouves.push(child);
                frontier.push(child);
            }
        }
    }
    trouves
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

    /// Backend dont `echo` demande une approbation (commande inoffensive).
    fn prompt_echo_backend(queue: Arc<ApprovalQueue>) -> LocalSandboxBackend {
        let policy: Arc<dyn ExecPolicy> = Arc::new(
            RuleBasedExecPolicy::new("t").rule("echo", ExecDecision::Prompt("test".to_string())),
        );
        LocalSandboxBackend::new(policy, Duration::from_secs(5)).with_approvals(queue)
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
    async fn approval_queue_runs_once_then_asks_again() {
        // (inc2) request → approve → exécute une fois → second appel re-demande.
        let queue = Arc::new(ApprovalQueue::new());
        let b = prompt_echo_backend(queue.clone());
        // Sans approbation : refus sans exécution.
        assert!(matches!(
            b.run("echo hello").await.unwrap_err(),
            OmegaError::ApprovalRequired { .. }
        ));
        // Approbation : exécute.
        let id = queue.request("echo hello", "test");
        assert!(queue.approve(id));
        let out = b.run("echo hello").await.unwrap();
        assert_eq!(out.stdout.trim(), "hello");
        // Approbation consommée (usage unique) : second appel re-demande.
        assert!(matches!(
            b.run("echo hello").await.unwrap_err(),
            OmegaError::ApprovalRequired { .. }
        ));
    }

    #[tokio::test]
    async fn denied_approval_never_executes() {
        let queue = Arc::new(ApprovalQueue::new());
        let b = prompt_echo_backend(queue.clone());
        let id = queue.request("echo hello", "test");
        assert!(queue.deny(id));
        assert!(matches!(
            b.run("echo hello").await.unwrap_err(),
            OmegaError::ApprovalRequired { .. }
        ));
        // Toujours pas d'approbation : second appel refusé aussi.
        assert!(matches!(
            b.run("echo hello").await.unwrap_err(),
            OmegaError::ApprovalRequired { .. }
        ));
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
    async fn timeout_kills_long_command() {
        let policy: Arc<dyn ExecPolicy> =
            Arc::new(RuleBasedExecPolicy::new("t").rule("sleep", ExecDecision::Allow));
        let b = LocalSandboxBackend::new(policy, Duration::from_millis(200));
        let err = b.run("sleep 30").await.unwrap_err();
        assert!(matches!(err, OmegaError::CommandTimeout { .. }));
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
        // Un **zombie** est un processus mort (il ne fait plus rien, il attend
        // seulement d'être reapé) : il ne compte pas comme survivant. Sinon la
        // porte est flaky — la CI l'a vue rouge le 2026-10-20 alors que le
        // petit-fils était tué.
        let mut survived: Vec<String> = Vec::new();
        for _ in 0..20 {
            let out = std::process::Command::new("pgrep")
                .arg("-f")
                .arg("sleep 57")
                .output()
                .expect("pgrep indisponible");
            let vivants: Vec<String> = String::from_utf8_lossy(&out.stdout)
                .lines()
                .map(str::trim)
                .filter(|pid| !pid.is_empty() && !est_zombie(pid))
                .map(str::to_string)
                .collect();
            if vivants.is_empty() {
                break;
            }
            survived = vivants;
            std::thread::sleep(Duration::from_millis(100));
        }
        let _ = std::fs::remove_dir_all(&dir);
        assert!(
            survived.is_empty(),
            "un petit-fils `sleep 57` a survécu au timeout ! (pids {survived:?})"
        );
    }

    /// `/proc/<pid>/stat` : l'état du processus. `Z` = zombie (mort, non reapé).
    fn est_zombie(pid: &str) -> bool {
        let Ok(stat) = std::fs::read_to_string(format!("/proc/{pid}/stat")) else {
            return false;
        };
        // Le champ `comm` peut contenir des espaces et des parenthèses : on
        // reprend après la dernière ')' — l'état est le champ suivant.
        stat.rsplit_once(')')
            .and_then(|(_, reste)| reste.split_whitespace().next())
            == Some("Z")
    }
}
