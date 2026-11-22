# NORDOI — APP-009 : graphe de dépendances pour agents IA

**22 novembre 2026 — Recherche IA → IA, zéro testeur humain.**

## Parent exact et contrôle C20

APP-008, `e15eb88d6e67652f3206d423d4a4b7d83ae8cd3e` : **13/13 jobs GitHub SUCCESS** (APP-008 4/4, Rust 5/5, E8 4/4). L'ancienne PR #159 reste DRAFT. APP-009 est un candidat distinct basé sur ce SHA ; ni fusion, ni modification des frontières natives certifiées.

## Nouvelle fonctionnalité utilisateur

Un agent transmet un petit **graphe orienté acyclique de 1 à 8 tâches**, rédigé en JSON. Chaque tâche a uniquement un `key_code`, un tableau `depends_on` de références numériques et un `gate`. Les références sont **strictement croissantes, uniques et dirigées uniquement vers des tâches précédentes** ; cette construction interdit les cycles avant toute exécution.

Exemple réel :

```json
{
  "schema": "NORDOI_AGENT_DAG_V1",
  "nodes": [
    {"key_code": 73, "depends_on": [], "gate": "always"},
    {"key_code": 59, "depends_on": [], "gate": "always"},
    {"key_code": 60, "depends_on": [1, 2], "gate": "all_committed"}
  ]
}
```

`all_committed` déclenche la tâche seulement si **tous les parents ont réellement publié**. Les cinq conditions disponibles en plus du démarrage sans parent `always` sont : `all_committed`, `all_approved`, `all_rejected`, `any_approved` et `any_rejected`. Une tâche `SKIPPED` n'est jamais considérée comme `COMMITTED`. Le calcul d'un `approved/rejected` vient **du véritable moteur Rust V1.3**, jamais d'une copie en Python. Les conditions de jonction se basent exclusivement sur les vrais reçus précédents.

**Sémantique délibérée :** l'exécution est **topologiquement ordonnée et séquentielle**. Ce DAG n'est pas un système distribué, une file durable, un parallélisme ou une transaction de groupe. Chaque tâche exécutée appelle la vraie commande Rust `agent-request-json`, publie son véritable bundle P2.7 dans son dossier `job-NNNN`, puis utilise APP-006 pour vérifier les fichiers et le reçu natif. Une tâche non déclenchée ne crée aucun dossier.

## Utilisation machine

```bash
python3 -B examples/apps/app009_agent_dag/run_app009.py \
  --plan /path/to/dag.json \
  --engine /path/to/pinned/nordoi \
  --source-root /path/to/pinned/noi/src \
  --grant-output-dir /path/to/EXISTING_EMPTY_DIR \
  --expected-engine-sha256 PINNED_NATIVE_BINARY_SHA256 \
  --expected-source-sha256 PINNED_SOURCE_TREE_SHA256
```

L'empreinte source est calculée avec le contrat APP-007 `verify_app007.source_tree_sha256`. **Les pins doivent avoir une origine fiable indépendante de l'entrée hostile** : le programme vérifie leur concordance mais ne certifie pas cette origine. Aucun chemin ni grant FileWrite ne peut être fourni dans le JSON agent. Le répertoire accordé doit exister, être réel, être vide et ne pas correspondre à l'arbre source.

Chaque tâche donne une ligne JSON `NORDOI_AGENT_DAG_EVENT_V1` avec son numéro fixe, le statut `COMMITTED | SKIPPED | ERROR`, les références du graphe et le SHA-256 observé du plan. `REJECTED` avant démarrage est code de sortie 2, sans effet ; `ERROR` natif est code 3 et `publication=UNKNOWN`. Après un échec, les publications antérieures persistent : aucun rollback fictif. Une exécution terminée sans erreur renvoie code 0 même si des tâches ont été sautées.

## Sécurité, limites scientifiques et Bureau

**ATLAS** valide le DAG non cyclique et la séparation entre planificateur et véritable calcul métier Rust ; **FERRUM** vérifie l'absence de privilège OS supplémentaire ; **BOUNDARY** vérifie les paths internes et le grant CLI séparé ; **CIPHER** vérifie l'usage honnête des SHA-256 sans affirmer une signature tierce ; **LEDGER** vérifie les preuves authentiques P2.7 pour chaque tâche et l'empreinte du plan ; **CAPSTONE** refuse permission/chemin injecté dans le JSON ; **ADVERSARY** teste références circulaires/ultérieures, faux états de dépendance et échecs partiels ; **VERITY** contrôle les véritables tests macOS Intel/Apple Silicon, Linux et Windows ; **GUARDIAN** conserve la séquence C20 et la politique permanente zéro humain.

**Ces neuf axes de revue ne sont pas neuf agents ni neuf signataires indépendants réellement exécutés.** L'interface Python ne rajoute aucune garantie de sandbox OS, aucune résistance à la modification locale concurrente, aucun mécanisme de crash durability ni aucune authentification de l'origine des pins. Le runtime normal ne nécessite aucun modèle IA ni accès réseau.

## Qualification attendue

- **40 nouveaux tests réels APP-009** (branches multiples, cas limites, mutations adversariales, fichiers P2.7 réellement publiés).
- **288 régressions APP-001…APP-008** déjà qualifiées sur chacun des quatre systèmes.
- **328 tests applicatifs par OS**, soit **1 312 exécutions** attendues sur les quatre systèmes, plus Rust Test Gate 5 jobs et E8 4 jobs.
- Démonstration native d'un DAG avec trois tâches réelles / six fichiers ; archives de leurs reçus exacts et rapport de cohérence.
- **Aucun PASS final avant tous les jobs GitHub SUCCESS au même commit.**

```ini
R6F=IN_PROGRESS_UNCERTIFIED
R6G=LOCKED_BY_SEQUENCE
EXTERNAL_NATIVE_ADMISSION_GATES=12_UNMET
HUMAN_TESTERS=0
PR=DRAFT
MERGE=NONE
PRODUCTION=NOT_AUTHORIZED
```
