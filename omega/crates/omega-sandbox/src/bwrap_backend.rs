//! Backend **bubblewrap (`bwrap`)** — sandbox de *processus* : FS par liste
//! blanche (binds lecture seule + répertoire de travail jetable en écriture)
//! + réseau fermé, **fail-closed**.
//!
//! ## Décision de primitive (bwrap vs Landlock) — court ADR en tête de module
//!
//! **bwrap retenu** comme primitive unique de la phase 1 `[observed]` : un
//! seul exécutable couvre FS *et* réseau — racine vide (sans bind, rien n'est
//! visible), whitelistes en `--ro-bind-try` (un chemin absent *rétrécit* la
//! vue, ne l'élargit jamais), `--unshare-all` (pas de `lo` : toute connexion
//! sortante échoue). Sémantique vérifiée sur bwrap 0.12.0 : `cat /etc/passwd`
//! → « No such file or directory », `/dev/tcp/127.0.0.1/9` → connection
//! refused, `touch /usr/x` → « Read-only file system ».
//!
//! **Landlock écarté** pour cette phase : ses restrictions in-process (FS)
//! héritées par les descendants ne ferment ni le réseau ni les namespaces, et
//! la crate n'est pas disponible hors-ligne — elle ne couvre pas la moitié
//! réseau de P1-6. Un backend Landlock reste possible plus tard comme *plugin
//! séparé* (everything is a plugin) — pas une décision opposée ici.
//!
//! ## Fail-closed
//!
//! - bwrap absent → `try_new` : `Err(ForbiddenCommand)`, et côté scénario
//!   `sandbox_mode()` : `SKIPPED(bwrap_absent)` (jamais PASS, règle L4).
//! - bwrap disparu entre construction et `run` → erreur explicite, jamais
//!   d'exécution à découvert.
//! - userns indisponible (machine verrouillée) → `bwrap` lui-même échoue au
//!   spawn → erreur ; jamais « si l'outil absent, on laisse passer ».
//!
//! ## Frontière A2 / omega-tools
//!
//! Ici : le **processus** (FS global visible, réseau, namespaces). A2 : le
//! **paramètre chemin** d'un outil (canonicalisation, symlink, `..`, octets
//! nuls). Les deux portées se composent, ne se doublonent pas.
//!
//! ## Canon unique de refus
//!
//! Décision (policy + anti-composition + approbations) : `crate::policy::gate`.
//! Timeout + kill de descendance : `crate::timeout` (les correctifs récents de
//! concurrence — zombie-survivants, kill au-delà du leader — vivent en un seul
//! exemplaire pour tous les backends).

use std::path::Path;
use std::sync::Arc;
use std::time::Duration;

use omega_ports::{ExecPolicy, SandboxBackend};
use omega_types::{CommandOutput, OmegaError, OmegaResult};

use crate::approvals::ApprovalQueue;

/// Raison canonique quand le binaire bwrap n'est pas disponible.
pub const BWRAP_ABSENT: &str = "bwrap_absent";

/// Mode d'isolation du run : `Active` (sandbox levée) ou `Skipped(raison)`.
///
/// Un scénario qui obtient `Skipped` ne peut jamais le présenter comme `PASS`
/// (plan §6 : « `SKIPPED(raison)`, jamais `PASS` ») : l'affichage verrouille
/// la distinction — `SKIPPED(bwrap_absent)`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SandboxMode {
    /// La sandbox est levée sur cette exécution.
    Active,
    /// La primitive n'a pas pu être levée, avec sa raison canonique.
    Skipped(&'static str),
}

impl std::fmt::Display for SandboxMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Active => f.write_str("SandboxActive"),
            Self::Skipped(r) => write!(f, "SKIPPED({r})"),
        }
    }
}

/// Un chemin est un bwrap exécutable : direct si le chemin contient un
/// séparateur (`/bin/bwrap`, `./bwrap`), sinon recherche dans `PATH`.
/// Épreuve : fichier régulier + bit exécutable (`std::os::unix::fs::`), zéro fork.
pub fn bwrap_available(bwrap_path: &Path) -> bool {
    fn executable(p: &Path) -> bool {
        use std::os::unix::fs::PermissionsExt;
        std::fs::metadata(p)
            .map(|m| m.is_file() && (m.permissions().mode() & 0o111) != 0)
            .unwrap_or(false)
    }
    if bwrap_path.to_str().is_some_and(|s| s.contains('/')) {
        return executable(bwrap_path);
    }
    // Nom nu : chaque répertoire de `PATH` (parcouru jusqu'au premier hit).
    let Some(path) = std::env::var_os("PATH") else {
        return false;
    };
    std::env::split_paths(&path)
        .map(|dir| dir.join(bwrap_path))
        .any(|p| executable(&p))
}

/// Mode de la sandbox pour le chemin bwrap donné : `Active` si le binaire est
/// présent, `Skipped(bwrap_absent)` sinon — jamais de faux vert (L4/L5).
pub fn sandbox_mode(bwrap_path: &Path) -> SandboxMode {
    if bwrap_available(bwrap_path) {
        SandboxMode::Active
    } else {
        SandboxMode::Skipped(BWRAP_ABSENT)
    }
}

/// Configuration du backend bwrap (tout fail-closed par défaut).
#[derive(Debug, Clone)]
pub struct BwrapConfig {
    /// Chemin (ou nom) du binaire bwrap. Nom sans `/` → recherche `PATH`.
    pub bwrap_path: std::path::PathBuf,
    /// Interpréteur exécuté *dans* le jail : `<sh_path> -c <commande>` ; le
    /// shell n'existe dans le jail QUE si `/bin` (ou `/usr`) est dans
    /// `ro_binds` (racine vide sinon).
    pub sh_path: std::path::PathBuf,
    /// Dirs en **lecture seule** (liste blanche ; absent → vue rétrécie).
    pub ro_binds: Vec<std::path::PathBuf>,
    /// Dirs en **lecture/écriture** (le répertoire de travail jetable, etc.).
    pub rw_binds: Vec<std::path::PathBuf>,
    /// Variables d'environnement injectées (`--clearenv` d'abord).
    pub env: Vec<(String, String)>,
    /// Timeout d'exécution.
    pub timeout: Duration,
}

impl Default for BwrapConfig {
    fn default() -> Self {
        Self {
            bwrap_path: "bwrap".into(),
            sh_path: "/bin/sh".into(),
            ro_binds: vec!["/usr".into(), "/bin".into(), "/lib".into(), "/lib64".into()],
            rw_binds: vec![],
            env: vec![
                ("PATH".into(), "/usr/bin:/bin".into()),
                ("HOME".into(), "/tmp".into()),
            ],
            timeout: Duration::from_secs(30),
        }
    }
}

/// Backend `bwrap` : policy + whitelistes + réseau fermé, fail-closed.
pub struct BwrapSandboxBackend {
    policy: Arc<dyn ExecPolicy>,
    config: BwrapConfig,
    approvals: Option<Arc<ApprovalQueue>>,
}

impl BwrapSandboxBackend {
    /// Construit le backend. **Fail-closed** : bwrap absent → `Err`, jamais
    /// un backend qui exécuterait à découvert. Le refus précède tout fork.
    pub fn try_new(policy: Arc<dyn ExecPolicy>, config: BwrapConfig) -> OmegaResult<Self> {
        if !bwrap_available(&config.bwrap_path) {
            return Err(OmegaError::ForbiddenCommand {
                command: config.bwrap_path.display().to_string(),
                reason: BWRAP_ABSENT.to_string(),
            });
        }
        if !config.sh_path.is_absolute() {
            return Err(OmegaError::ForbiddenCommand {
                command: config.bwrap_path.display().to_string(),
                reason: "sh_path_absolu_requis".to_string(),
            });
        }
        Ok(Self {
            policy,
            config,
            approvals: None,
        })
    }

    /// Attache une file d'approbations (même canon que les autres backends).
    pub fn with_approvals(mut self, queue: Arc<ApprovalQueue>) -> Self {
        self.approvals = Some(queue);
        self
    }
}

#[async_trait::async_trait]
impl SandboxBackend for BwrapSandboxBackend {
    fn name(&self) -> &str {
        "bwrap"
    }

    async fn run(&self, command: &str) -> OmegaResult<CommandOutput> {
        // Fail-closed re-vérifié au run : bwrap disparu entre construction et
        // exécution → refus explicite, jamais de fork à découvert.
        if !bwrap_available(&self.config.bwrap_path) {
            return Err(OmegaError::ForbiddenCommand {
                command: self.config.bwrap_path.display().to_string(),
                reason: BWRAP_ABSENT.to_string(),
            });
        }
        // Canon de refus : évaluation + garde anti-composition + octets nuls +
        // approbations — partagé par tous les backends sandbox.
        crate::policy::gate(self.policy.as_ref(), command, self.approvals.as_ref())?;

        // Toutes les options AVANT la racine positionnelle : après le premier
        // non-option (la nouvelle racine), bwrap ne lit plus d'options.
        let mut cmd = tokio::process::Command::new(&self.config.bwrap_path);
        cmd.arg("--unshare-all") // pas de `lo` : toute connexion sortante échoue
            .arg("--clearenv")
            .arg("--die-with-parent");
        // Racine vide + whitelistes : tout ce qui n'est pas bindé n'existe
        // pas ; `*-try` rétrécit la vue (chemin absent), ne l'élargit jamais.
        for dir in &self.config.ro_binds {
            let l = dir.to_string_lossy();
            cmd.args(["--ro-bind-try", l.as_ref(), &l]);
        }
        for dir in &self.config.rw_binds {
            let l = dir.to_string_lossy();
            cmd.args(["--bind-try", l.as_ref(), &l]);
        }
        for (k, v) in &self.config.env {
            cmd.args(["--setenv", k, v]);
        }
        // Première position = nouvelle racine (vide sans bind).
        // Le shell n'est exécuté QUE dans le jail (jamais côté hôte).
        cmd.arg(&self.config.sh_path).arg("-c").arg(command);

        // Groupe de processus dédié + kill descendance au timeout : canon
        // dans `crate::timeout` (correctifs CONCURRENCE partagés).
        crate::timeout::spawn_and_wait(
            cmd,
            self.config.timeout,
            command,
            "spawn bwrap",
            "attente bwrap",
        )
        .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::policy::RuleBasedExecPolicy;
    use omega_types::{CommandOutput, ExecDecision, OmegaError};

    /// Politique de test : `cat`/`bash`/`ls`/`touch`/`echo` autorisés,
    /// deny-by-default sinon — c'est le jail bwrap qui borne le reste.
    fn politique() -> Arc<dyn ExecPolicy> {
        Arc::new(
            RuleBasedExecPolicy::new("test-jail")
                .rule("cat", ExecDecision::Allow)
                .rule("bash", ExecDecision::Allow)
                .rule("ls", ExecDecision::Allow)
                .rule("touch", ExecDecision::Allow)
                .rule("echo", ExecDecision::Allow),
        )
    }

    fn backend() -> BwrapSandboxBackend {
        BwrapSandboxBackend {
            policy: politique(),
            config: BwrapConfig::default(),
            approvals: None,
        }
    }

    fn bwrap_present() -> bool {
        bwrap_available(Path::new("bwrap"))
    }

    /// Utilitaire des tests `#[test]` purs : bloque sur le futur du `run`.
    fn block_run(fut: impl std::future::Future<Output = OmegaResult<CommandOutput>>) -> OmegaResult<CommandOutput> {
        tokio::runtime::Runtime::new().unwrap().block_on(fut)
    }

    #[test]
    fn mode_skipped_display_is_never_pass() {
        // L4 : `SKIPPED(raison)` ≠ `PASS` — l'affichage verrouille la borne.
        assert_eq!(
            SandboxMode::Skipped(BWRAP_ABSENT).to_string(),
            "SKIPPED(bwrap_absent)"
        );
        assert_eq!(SandboxMode::Active.to_string(), "SandboxActive");
    }

    #[test]
    fn bwrap_absent_is_detected_pure_rust() {
        // Test pur Rust, sans binaire : un chemin inexistant n'est PAS bwrap.
        assert!(!bwrap_available(Path::new("/nonexistent/bwrap-omega-test")));
        assert_eq!(
            sandbox_mode(Path::new("/nonexistent/bwrap-omega-test")),
            SandboxMode::Skipped(BWRAP_ABSENT)
        );
        // Le chemin canonique détecte le bwrap présent de la machine.
        if !bwrap_present() {
            eprintln!("SKIP: bwrap absent");
            return;
        }
        assert_eq!(sandbox_mode(Path::new("bwrap")), SandboxMode::Active);
    }

    #[test]
    fn try_new_without_binary_fails_closed() {
        // Fail-closed, sans exécution : bwrap absent → Err explicite, jamais un
        // backend ouvert (le refus précède tout fork).
        let err = BwrapSandboxBackend::try_new(
            politique(),
            BwrapConfig {
                bwrap_path: "/nonexistent/bwrap-omega-test".into(),
                ..BwrapConfig::default()
            },
        )
        .map(|_: BwrapSandboxBackend| ())
        .unwrap_err();
        assert!(matches!(err, OmegaError::ForbiddenCommand { .. }), "obtenu : {err:?}");
    }

    #[test]
    fn nul_byte_command_is_canonical_refusal() {
        // Octets nuls : refus canonique identique aux autres évasions — jamais
        // laissé au hasard d'un `execve` raté.
        let err = block_run(backend().run("cat /etc/\0passwd"));
        assert!(
            matches!(err, Err(OmegaError::ForbiddenCommand { .. })),
            "obtenu : {err:?}"
        );
    }

    #[test]
    fn fs_escape_read_denied() {
        // Évasion FS : `/etc/passwd` est *hors* du jail (aucun bind-ro /etc).
        // La commande DOIT échouer, sans jamais livrer le contenu.
        if !bwrap_present() {
            eprintln!("SKIP: bwrap absent");
            return;
        }
        let out = block_run(backend().run("cat /etc/passwd")).unwrap();
        assert_ne!(out.exit_code, 0, "cat /etc/passwd a réussi dans le jail : {out:?}");
        assert!(!out.stdout.contains("root:"), "évasion FS : {out:?}");
    }

    #[test]
    fn network_escape_connect_refused() {
        // Évasion réseau : `--unshare-all` → pas de `lo` dans le jail, toute
        // connexion sortante échoue. `CONNECTED` ne peut apparaître que si le
        // namespace réseau n'a PAS été créé (= évasion à attraper).
        if !bwrap_present() {
            eprintln!("SKIP: bwrap absent");
            return;
        }
        if !Path::new("/usr/bin/bash").exists() {
            eprintln!("SKIP: /usr/bin/bash absent");
            return;
        }
        // Métacaractères entre simples quotes → pas d'escalade anti-composition.
        let cmd = "bash -c 'exec 3<>/dev/tcp/127.0.0.1/9 && echo CONNECTED || echo BLOCKED'";
        assert!(!crate::has_unquoted_metachars(cmd));
        let out = block_run(backend().run(cmd)).unwrap();
        assert_eq!(out.stdout.trim(), "BLOCKED", "évasion réseau : {out:?}");
        assert_eq!(out.exit_code, 0);
    }

    #[test]
    fn whitelist_readonly_enforced() {
        // La liste blanche `/usr` est en LECTURE SEULE : écrire échoue, et le
        // fichier n'apparaît pas côté hôte.
        if !bwrap_present() {
            eprintln!("SKIP: bwrap absent");
            return;
        }
        let out = block_run(backend().run("touch /usr/omega-bwrap-probe-ro")).unwrap();
        assert_ne!(out.exit_code, 0, "écriture dans /usr a réussi : {out:?}");
        assert!(!Path::new("/usr/omega-bwrap-probe-ro").exists());
    }

    #[test]
    fn unicode_paths_survive_the_jail() {
        // Cas limite plan : chemins Unicode. Un répertoire de travail à nom
        // composé (aucun métacaractère shell) reste lisible dans le jail.
        if !bwrap_present() {
            eprintln!("SKIP: bwrap absent");
            return;
        }
        let dir = std::env::temp_dir().join(format!("omega-bwrap-éàü-{}", std::process::id()));
        let _ = std::fs::create_dir_all(&dir);
        let fichier = dir.join("f-éàü.txt");
        std::fs::write(&fichier, "temoin").unwrap();
        let b = BwrapSandboxBackend {
            policy: politique(),
            config: BwrapConfig {
                rw_binds: vec![dir.clone()],
                ..BwrapConfig::default()
            },
            approvals: None,
        };
        let out = block_run(b.run(&format!("ls {}", dir.display()))).unwrap();
        let _ = std::fs::remove_dir_all(&dir);
        assert_eq!(out.exit_code, 0, "ls unicode a échoué : {out:?}");
        assert!(out.stdout.contains("f-éàü.txt"), "fichier absent : {out:?}");
    }

    #[tokio::test]
    async fn two_concurrent_runs_are_independent() {
        // Cas limite plan : deux appels concurrents — bwrap est un processus
        // jetable par appel, aucun état partagé n'échappe au canon.
        if !bwrap_present() {
            eprintln!("SKIP: bwrap absent");
            return;
        }
        let b = backend();
        let (a, c) = tokio::join!(b.run("echo un"), b.run("echo deux"));
        let (a, c) = (a.unwrap(), c.unwrap());
        assert_eq!(a.stdout.trim(), "un");
        assert_eq!(c.stdout.trim(), "deux");
    }
}
