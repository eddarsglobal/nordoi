# NORDOI APP-012 — Véritable point d'arrêt propre des graphes d'agents

**22 novembre 2026 · Développement de recherche IA → IA · Zéro testeur humain**

## Parent C20 et distinction scientifique

APP-011, commit `57207b8ce9481040743d8ae9acf6f314ae993190`, est qualifié **13/13 jobs SUCCESS** (APP-011 4/4, Rust 5/5, E8 4/4). La PR #162 reste DRAFT, non fusionnée. APP-012 est une branche isolée basée sur ce parent exact, sans changement de `main`, admission native ou production.

**Besoin réel :** APP-011 savait reprendre un historique propre, mais son démonstrateur initial avait d'abord fait exécuter tout le graphe par APP-009, puis supprimé artificiellement les derniers dossiers. APP-012 produit le point d'arrêt **directement et honnêtement** : le moteur Rust NORDOI n'exécute jamais les tâches situées après la frontière demandée.

## Fonction APP-012

Un agent transmet un **plan DAG complet et épinglé** de 2 à 8 tâches, sa source Rust/NOI et un grant de sortie explicite vide. Il sélectionne `--stop-after N` tel que `1 <= N < nombre_de_noeuds`. L'ordonnanceur :
1. Valide **l'intégralité** du plan, les dépendances et les empreintes SHA-256 du plan, du véritable moteur Rust et de ses sources .noi avant tout effet.
2. Accepte uniquement une racine FileWrite réelle, vide, séparée du répertoire de sources. Aucune permission, aucun nom de fichier et aucune commande ne proviennent des données de l'agent.
3. Exécute **uniquement les N premiers nœuds** : chaque tâche déclenchée appelle le **vrai moteur Rust** `nordoi agent-request-json`, effectue une vraie publication P2.7 et obtient un reçu natif vérifié immédiatement via APP-006. Les nœuds `SKIPPED` n'ont aucun dossier.
4. Émet **exactement N événements JSONL** sur la sortie binaire standard en **LF sur tous les OS**, en conservant l'empreinte authentique du plan et les décisions natives. Aucune ligne fictive après le point d'arrêt, aucun dossier futur et aucun reçu inventé.

Le flux retourné est un **préfixe incomplet propre** pour APP-010 (`INCONCLUSIVE` de façon attendue), et peut être **repris par APP-011** une fois sauvegardé et épinglé séparément. L'audit APP-010 de la concaténation exacte `préfixe + suffixe` doit ensuite être `PASS`. Le code de sortie **0** d'APP-012 indique uniquement que le **préfixe demandé a été correctement généré**, et ne signifie jamais qu'un graphe incomplet est fini ni qu'une certification indépendante existe.

### Commande native, sans humain

```bash
python3 -B examples/apps/app012_clean_checkpoint/run_app012.py \
  --plan /trusted/agent-dag.json \
  --engine /trusted/native/nordoi \
  --source-root /trusted/noi/src \
  --grant-output-dir /existing/empty/native-job-root \
  --expected-plan-sha256 TRUSTED_DAG_PLAN_SHA256 \
  --expected-engine-sha256 TRUSTED_RUST_BINARY_SHA256 \
  --expected-source-sha256 TRUSTED_NOI_TREE_SHA256 \
  --stop-after 2 > /trusted/native-prefix.jsonl
```

La redirection du flux, son stockage durable et l'épinglage externe des empreintes **sont des responsabilités de l'appelant**. APP-012 n'atteste ni un `fsync` du journal de l'appelant, ni la résistance aux coupures d'alimentation. L'interruption du processus, un `stdout` tronqué ou une publication au statut inconnu ne forment **pas** un point d'arrêt propre et ne doivent pas être repris automatiquement. La fin volontaire `--stop-after` est la seule frontière de succès.

### Limites et refus

- `REJECTED` / code 2 / `publication=NOT_STARTED` uniquement avant tout effet : plan invalide, empreinte fausse, grant non vide, frontière `0` ou couvrant le graphe entier.
- `ERROR` / code 3 / `publication=UNKNOWN` après tentative de publication native échouée ; les dossiers antérieurs persistent et aucune restauration automatique de l'état incertain n'est prétendue.
- Aucun rollback de graphe, aucun processus en arrière-plan, aucune exécution concurrente, aucun modèle IA requis au runtime.
- Les SHA-256 prouvent la **concordance des octets avec les pins transmis** et non leur provenance externe. APP-006/010 vérifient la cohérence des fichiers avec les reçus natifs **sans recomputation cryptographique indépendante des témoins P2.7**. Une falsification coordonnée des reçus, fichiers et pins est hors modèle d'authenticité. Aucune isolation OS anti-hôte concurrent, aucune durabilité électrique ni signature indépendante.
- APP-012 ne modifie **aucun** jalon gouverné R6F/R6G, ne fusionne rien et n'autorise aucune activation privilégiée.

## Bureau : neuf axes, non neuf signataires autonomes indépendants

**ATLAS** : validité complète du graphe avant émission partielle ; **FERRUM** : véritable moteur Rust NORDOI à chaque tâche exécutée ; **BOUNDARY** : racine vide et noms de dossiers fixes avec création exclusive ; **CIPHER** : plan, moteur et sources épinglés sans signature inventée ; **LEDGER** : événements originaux enregistrés exactement par l'appelant, vrais reçus et fichiers ; **CAPSTONE** : frontière volontaire stricte et blocage des statuts incertains ; **ADVERSARY** : injection, reprise dupliquée, fichiers et reçus modifiés, branches fausses et échec après commit ; **VERITY** : quatre systèmes, vrais tests et dossier de démonstration ; **GUARDIAN** : séquence C20 et politique AI-only sans test humain.

Ces fonctions désignent **neuf axes de contrôle documentés**, pas neuf processus signataires indépendants.

## Qualification prévue

**40 nouveaux tests APP-012** utilisant le vrai moteur Rust, **408 tests hérités APP-001…APP-011** par système : **448 tests applicatifs par OS et 1 792 exécutions** sur Mac Intel, Apple Silicon, Linux et Windows. Les workflows natifs Rust Test Gate (5 jobs) et E8 (4 jobs) restent activés. Une démonstration CI crée un **vrai préfixe** de deux tâches sans supprimer de dossiers, l'audite `INCONCLUSIVE`, poursuit avec APP-011, puis audite le DAG complet `PASS`, en refusant la reprise dupliquée. Les quatre plateformes doivent réussir **13/13 jobs** au même HEAD avant qualification.

```ini
APP012_RESEARCH=UNQUALIFIED_UNTIL_CI
R6F=IN_PROGRESS_UNCERTIFIED
R6G=LOCKED_BY_SEQUENCE
EXTERNAL_NATIVE_ADMISSION_GATES=12_UNMET
HUMAN_TESTERS=0
PR=DRAFT
MERGE=NONE
PRODUCTION=NOT_AUTHORIZED
```
