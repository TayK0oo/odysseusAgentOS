//! Exécution bornée d'un enfant : spawn avec groupe dédié, attente avec
//! timeout, puis kill de **toute la descendance**. Canon partagé par tous les
//! backends sandbox (`local`, `namespace`, `bwrap`) : les correctifs récents
//! de concurrence — zombie-survivants (les morts `Z` ne comptent pas) et kill
//! au-delà du leader — vivent en un seul exemplaire, ici.
//!
//! Kill cascade, dans l'ordre : (1) le groupe du leader (`kill -9 -pgid`) ne
//! touche que les membres d'un groupe correctement formé, (2) le leader
//! lui-même, (3) la descendance capturée AVANT les kills — le `setpgid` n'est
//! pas garant (`process_group(0)` tokio peut être sans effet selon la
//! toolchain) ; le child reste dans le groupe du parent, le kill de groupe ne
//! touche que le leader et le petit-fils survit (REVIEW-W8-W9 B2 ; constaté en
//! CI le 2026-10-20 : `sleep 57` vivait encore, dans le groupe du parent).
//! On ÉVITE de descender PUIS de tuer du zombie : un mort `Z` ne fait plus
//! rien, on ne le compte pas.

use std::time::Duration;

use omega_types::{CommandOutput, OmegaError, OmegaResult};

/// Spawn, applique `process_group(0)` + stdio piped, attend avec timeout ;
/// au dépassement, tue groupe + leader + descendance puis renvoi
/// `CommandTimeout`.
pub(crate) async fn spawn_and_wait(
    mut cmd: tokio::process::Command,
    timeout: Duration,
    command: &str,
    spawn_prefix: &str,
    wait_prefix: &str,
) -> OmegaResult<CommandOutput> {
    cmd.process_group(0);
    cmd.stdout(std::process::Stdio::piped());
    cmd.stderr(std::process::Stdio::piped());
    let child = cmd
        .spawn()
        .map_err(|e| OmegaError::Io(format!("{spawn_prefix} : {e}")))?;

    let pid = child.id();
    let output = tokio::time::timeout(timeout, child.wait_with_output()).await;
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
                timeout_ms: timeout.as_millis() as u64,
            })
        }
        Ok(Err(e)) => Err(OmegaError::Io(format!("{wait_prefix} : {e}"))),
        Ok(Ok(out)) => Ok(CommandOutput {
            stdout: String::from_utf8_lossy(&out.stdout).into_owned(),
            stderr: String::from_utf8_lossy(&out.stderr).into_owned(),
            exit_code: out.status.code().unwrap_or(-1),
        }),
    }
}

/// Descendants (enfants, petits-enfants, …) d'un PID, lus dans `/proc`.
///
/// Le kill de groupe ne suffit pas : si `setpgid` n'a pas été appliqué, les
/// petits-enfants restent dans le groupe du parent et survivent au timeout.
/// On les capture donc avant de tuer quoi que ce soit.
pub(crate) fn descendants(pid: u32) -> Vec<u32> {
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
