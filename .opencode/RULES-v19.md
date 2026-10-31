# RÈGLES v19 (non négociables)
R1  Zéro modif de production. Écriture permise : omega/crates/omega-tools/tests/** et docs/omega/reports/** (append-only).
R2  Zéro commit/push/prune. Untracked et proptests.proptest-regressions intacts. Seuls les worktrees `v19-*` sont supprimables.
R3  Aucun chiffre sans preuve : toute mesure passe par `scripts/v19/evidence.sh` (produit un [E-nnn]). Aucun chiffre repris d'un ancien rapport.
R4  Statuts autorisés : TERMINÉ VÉRIFIÉ · ÉQUI-PROUVÉ · UNVIABLE-PROUVÉ · INATTEIGNABLE-API-PUBLIQUE · PROTÉGÉ · OUVERT(cause+action).
    Mots interdits : probablement, semble, partiel, environnement-dépendant, devrait, à peu près.
R5  Un worktree par agent qui lance cargo (`scripts/v19/wt.sh add <id>`), son CARGO_TARGET_DIR, son --output. Jamais deux cargo-mutants sur un même arbre.
R6  Aucun agent ne valide sa propre conclusion : passage obligatoire par `verifier`.
R7  ÉQUI-PROUVÉ = raisonnement écrit + recherche exhaustive citée (grep/AST) + échec documenté de l'adversaire.
R8  Un test compte seulement s'il passe sur code sain ET échoue sous le mutant visé (commande rejouable).
R9  Déterminisme : test rejoué 20 fois, en --test-threads=1 et en parallèle. 1 échec = rejeté.
R10 Max 6 agents cargo simultanés, timeout explicite sur toute commande longue.
R11 Sortie de périmètre : annuler les fichiers de l'agent, consigner l'incident (Livrable D).
R12 Invoquer les scripts v19 DIRECTEMENT (`scripts/v19/x.sh args`), sans cd, &&, pipe ni redirection : les permissions matchent la chaîne de commande, toute forme composée retombe en « ask ».
PROTÉGÉ : RECALL_LIMIT = 20 (ADR-0022). Un mutant sur cette constante n'est ni à tuer ni à déclarer équivalent.
Format de retour d'un subagent (JSON) :
{"id","perimetre","commandes":[{"cmd","exit","evidence_id"}],"chiffres":{},"conclusions":[{"enonce","statut","preuve_ids":[]}],"ouverts":[],"fichiers_modifies":[]}
