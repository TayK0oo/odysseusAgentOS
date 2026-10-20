# ADR-0012 — Reproductibilité : le lockfile est commité, et le timeout tue toute la descendance

**Date** : 2026-10-20 · **Statut** : Accepté · **Décidé par** : chef de projet
**Travail** : P0-1 (G0.a) · **Constaté par** : la porte G0.a en CI

## Constat (observed)

La porte G0.a (`verify` exit 0 sur clone vierge) est **verte en local** et
**rouge en CI**, sur le même commit. Deux causes, trouvées en jouant la
porte :

1. **`omega/.gitignore` ignorait `Cargo.lock`.** La CI résolvait donc des
   versions fraîches à chaque run (tokio 1.54.0 au lieu de 1.53.3), et des
   versions de dépendances non vérifiées. C'est exactement ce que le plan
   refuse : « installations non reproductibles : versions épinglées,
   lockfiles » (cas limites, §6).

2. **`omega-sandbox` : un petit-fils survivait au timeout.** Le test
   `timeout_kills_grandchildren_not_just_sh` a échoué en CI avec
   `un petit-fils 'sleep 57' a survécu au timeout ! (pids ["6263"])`. Le
   mécanisme du backend tue le **groupe de processus** du leader
   (`kill -9 -<pgid>`), ce qui suppose que `setpgid` a été appliqué à
   l'enfant. Quand il ne l'est pas, l'enfant reste dans le groupe du parent :
   le kill de groupe ne touche que le leader, et le petit-fils survit. Le
   comportement du sandbox dépendait donc d'un détail de la toolchain —
   invisible en local, visible en CI.

## Décision

1. **`omega/Cargo.lock` est commité** (`omega/.gitignore` ne l'ignore plus).
   La CI construit exactement ce qui a été vérifié en local.
2. **Le timeout tue toute la descendance, pas seulement le groupe.** Le
   backend capture la descendance (lecture de `/proc`,|ppid) **avant** les
   kills, puis tue : le groupe du leader, le leader, et chaque descendant
   capturé. Le kill de groupe reste (c'est le chemin rapide et le plus
   propre) ; la capture est le filet de sécurité, et elle ne dépend d'aucun
   détail de toolchain.
3. **La toolchain n'est pas épinglée.** Elle reste celle du runner
   (`rustc 1.99.1` en CI, `1.98.1` en local). Le fix ci-dessus est
   indépendant de la version : la CI sert alors à **prouver** que le sandbox
   tient, pas à le contourner. La version utilisée est imprimée dans le
   journal du job.

## Conséquences

- G0.a est **reproductible** : même lockfile, mêmes versions, même résultat.
- La propriété de sécurité « au timeout, aucun petit-fils ne survit » ne
  dépend plus d'une supposition sur `setpgid`.
- Le test `timeout_kills_grandchildren_not_just_sh` ne compte plus les
  **zombies** comme survivants : un zombie est un processus mort (il attend
  son reap). La CI l'a vu rouge pour cette raison ; en cas d'échec, le test
  dit les pids.

## Alternatives rejetées

- **Épingler la toolchain (`rust-toolchain.toml`)** — rejeté pour l'instant :
  cela coûte un téléchargement de toolchain à chaque run CI, et ça
  **masquerait** le bug au lieu de le corriger. Si le fix se révèle dépendant
  d'une version, on l'épinglera et on l'écrira dans un ADR.
- **Déclarer la suite Rust non bloquante en CI** — interdit : ce serait
  déclarer verts des tests rouges.
- **Supprimer le test instable** — interdit : c'est la propriété de sécurité
  elle-même qui est testée.
