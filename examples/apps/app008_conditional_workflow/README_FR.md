# NORDOI APP-008 — Workflow conditionnel réel pour agents IA

**22 novembre 2026 — Candidat de recherche, conçu par agents IA pour agents IA.**

## Jalon parent, C20 et périmètre

Parent exact : **APP-007** commit `3d1da54a1539d34f03de4be94daf3b00f78ec9f1`, avec **13/13 jobs GitHub SUCCESS**, dont APP-007 4/4, Rust 5/5, E8 4/4. PR #158 toujours DRAFT, non fusionnée. APP-008 ne fusionne rien et ne touche pas aux versions certifiées, au noyau Rust ni à la production.

## Valeur fonctionnelle : décisions NORDOI conditionnant les tâches suivantes

APP-005 lançait jusqu'à huit requêtes indépendantes ; APP-006 vérifiait les fichiers ; APP-007 rejouait les sorties avec un moteur épinglé. APP-008 ajoute un **workflow agentique conditionnel** de **1 à 8 tâches** utilisant les **véritables résultats V1.3/P2.7 du moteur Rust NORDOI**.

Un agent soumet ce plan JSON, sans aucun nom de fichier ou permission :

```json
{"schema":"NORDOI_AGENT_WORKFLOW_V1","steps":[
  {"key_code":73,"when":"always"},
  {"key_code":59,"when":"if_previous_approved"},
  {"key_code":60,"when":"if_previous_rejected"}
]}
```

Le workflow valide **l'intégralité du plan avant toute création de dossier**, épingle SHA-256 du **vrai binaire Rust** et des **sources .noi**, refuse une racine de sortie non vide, puis invoque le **protocole natif réel** `agent-request-json` pour chaque tâche à exécuter. Un résultat `decision=1` autorise `if_previous_approved`, un résultat `decision=0` autorise `if_previous_rejected`. **Ces décisions sont produites par NORDOI et jamais recalculées par l'adaptateur.** Si une étape est ignorée, sa suivante conditionnée par la précédente est également ignorée ; un nouvel `always` redémarre l'enchaînement.

L'adaptateur est délibérément une interface Python *au-dessus du véritable binaire Rust* : il ne prétend pas introduire un nouvel interpréteur Rust ou implémenter les décisions scientifiques en Python. Chaque étape validée possède son dossier `job-NNNN/`, un vrai bundle P2.7 avec deux fichiers, et un reçu JSON natif vérifié en lecture seule par APP-006. Chaque étape produit une ligne machine `COMMITTED`, `SKIPPED` ou `ERROR`, jamais de chemin absolu ni de preuve inventée.

## Interface machine, sans humain

```bash
python3 -B examples/apps/app008_conditional_workflow/run_app008.py \
  --plan /path/workflow.json \
  --engine /path/to/verified/nordoi \
  --source-root /path/to/verified/noi/src \
  --grant-output-dir /path/to/existing/empty/output \
  --expected-engine-sha256 <trusted-pinned-binary-sha256> \
  --expected-source-sha256 <trusted-pinned-source-tree-sha256>
```

Le format d'empreinte d'arbre est documenté par APP-007, `verify_app007.source_tree_sha256()`. **Les empreintes doivent provenir d'une source de confiance distincte** : un adversaire contrôlant simultanément le moteur, les sources et les empreintes peut tromper ce mécanisme. Aucun certificat indépendant n'est créé.

## Cas d'échec et bornes

- Plan JSON invalidé **avant tout effet** : code 2, `REJECTED`, aucune sortie créée.
- Workflow correct, tâche conditionnelle non activée : `SKIPPED`, aucun dossier.
- Tâche exécutée : vraie publication P2.7, vérification des octets avec APP-006, `COMMITTED`.
- Échec natif ou vérification contradictoire : code 3 et `ERROR`, `publication=UNKNOWN` ; les tâches déjà publiées **ne sont pas annulées**. Jamais de fausse transaction multi-jobs.
- Aucune permission dans le plan ; le droit FileWrite est exclusivement `--grant-output-dir` fourni par la commande de confiance, limité à un sous-dossier neuf par tâche.
- Sans réseau, sans modèle IA runtime, sans opérateur humain, sans service résident, sans sandbox OS revendiquée. La protection contre modifications concurrentes malveillantes de l'hôte et la durabilité électrique demeurent **non démontrées**.

## Bureau, neuf fonctions (axes de contrôle, pas neuf signataires indépendants)

ATLAS : réutilisation du moteur natif existant, sans autre logique métier ; FERRUM : aucune permission privilégiée ajoutée ; BOUNDARY : dossiers individuels fixes et grant explicite ; CIPHER : SHA-256 épinglés mais pas d'authentification externe inventée ; LEDGER : reçu natif complet par tâche ; CAPSTONE : autorité refusée par défaut ; ADVERSARY : injection de permission, états sautés, échec après premier commit ; VERITY : tests de vrais binaires 4 OS et Rust/E8 ; GUARDIAN : parent C20 et politique agents uniquement.

**Plan de qualification : 33 nouveaux tests, 255 régressions APP-001…007 = 288 tests applicatifs par OS, 1 152 exécutions sur quatre OS**, plus Rust Test Gate, sécurité E8 et vrais artefacts. Aucun verdict définitif avant résultats exact-HEAD.

`R6F=IN_PROGRESS_UNCERTIFIED`; `R6G=LOCKED_BY_SEQUENCE`; `NATIVE_ADMISSION_GATES=12_UNMET`; `MERGE=NONE`; `PRODUCTION=NOT_AUTHORIZED`.
