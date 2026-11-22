# NORDOI — APP-011 : reprise vérifiée d'un graphe natif pour agents IA

**22 novembre 2026. Recherche autonome IA → IA, pas de testeur humain.**

## Constitution / parent C20

Le parent exact est **APP-010** au commit `d3f038fbed15d5080281375acb00586c92a2ab36`, avec **13/13 jobs GitHub SUCCESS** (APP-010 4/4, Rust 5/5, E8 4/4). La PR #161 reste DRAFT et non fusionnée. APP-011 n'autorise aucune promotion d'une branche protégée et n'élargit aucune admission native.

## Ce que fait réellement APP-011

APP-009 savait exécuter un graphe complet ; APP-010 savait auditer en lecture seule un historique. **APP-011 peut continuer un graphe à partir d'un préfixe propre, déjà attesté par les vrais fichiers natifs**, sans réexécuter les tâches historiques et sans réécrire le journal original.

Exemple : un graphe de quatre tâches a terminé les tâches 1 et 2, avec leurs reçus natifs APP-009 et fichiers P2.7 présents. L'agent fournit :
- le **plan DAG JSON d'origine** et son SHA-256 attendu ;
- le **préfixe exact des événements natifs JSONL**, arrêté à une frontière de ligne complète, et son SHA-256 attendu ;
- le **vrai moteur Rust NORDOI**, les **sources .noi** et leurs deux pins SHA-256 ;
- le répertoire explicitement accordé qui contient déjà **uniquement** les dossiers historiques attendus.

L'application vérifie tous les pins, le graphe, les événements, les décisions déjà prises et **les vrais octets publiés** grâce à APP-010 + APP-006 **avant de créer quoi que ce soit**. Elle refuse un préfixe `ERROR` ou un dossier de tâche en cours d'état inconnu. Seul un préfixe **incomplet, propre, composé exclusivement d'événements `COMMITTED` et `SKIPPED` et dont le dossier de sortie est cohérent** peut être repris.

Le programme reprend les dépendances avec les **décisions natives enregistrées** dans les reçus vérifiés, puis invoque le vrai binaire Rust `nordoi agent-request-json` pour **les tâches restantes uniquement**. Aucun grant ne provient du plan JSON. Chaque nouvelle publication est dans un dossier déterministe `job-NNNN` créé exclusivement, donc **sans écraser les dossiers antérieurs**. Les étapes ignorées ne créent aucun dossier. Le moteur demeure le calculateur métier, et l'adaptateur demeure un ordonnanceur Python borné.

## Commande pour agent

```bash
python3 -B examples/apps/app011_verified_resume/run_app011.py \
  --plan original-dag.json \
  --prefix-events native-committed-prefix.jsonl \
  --grant-output-dir EXISTING_NATIVE_JOB_ROOT \
  --engine PINNED_REAL_NORDOI_BINARY \
  --source-root PINNED_NOI_SOURCE_ROOT \
  --expected-plan-sha256 TRUSTED_PLAN_SHA256 \
  --expected-prefix-sha256 TRUSTED_PREFIX_SHA256 \
  --expected-engine-sha256 TRUSTED_ENGINE_SHA256 \
  --expected-source-sha256 TRUSTED_SOURCE_TREE_SHA256
```

**Les quatre pins doivent avoir une provenance de confiance distincte du contenu à auditer**. Les recopier d'un attaquant n'authentifie rien.

### Retour machine et reconstruction du journal

- Code **0** : tous les nœuds restants ont été traités ; stdout contient **uniquement les événements des tâches nouvelles** (un suffixe JSONL). Les lignes historiques ne sont jamais réimprimées ni réécrites.
- Code **2**, `REJECTED`, `publication=NOT_STARTED` : plan, pins, préfixe, preuve native, dossier ou état historique **refusés avant tout nouveau travail**. Aucune reprise sur une tâche déjà existante ou incertaine.
- Code **3**, `ERROR`, `publication=UNKNOWN` : une tentative native de reprise a échoué. Les tâches précédemment publiées restent intactes et les tâches de reprise déjà publiées ne sont **pas annulées**. Une nouvelle reprise ne doit pas deviner l'état de la dernière tâche : il faut une preuve propre et complète.
- Les événements nouveaux respectent le format de fin de ligne **LF ou CRLF du préfixe**. La concaténation **exacte** `préfixe original + suffixe stdout` peut alors être vérifiée avec APP-010 pour obtenir le compte rendu complet. L'auditeur conserve les octets d'origine dans le SHA-256.

**Attention :** une reprise sûre exige un **point d'arrêt propre et vérifiable**. APP-011 ne constitue PAS une reprise automatique à n'importe quel moment après panne, ne résout PAS les publications potentiellement effectuées avant la perte d'un reçu, et ne promet ni atomicité globale, ni durabilité électrique, ni rollback multi-tâches, ni isolation contre des écritures concurrentes malveillantes.

APP-010 contrôle la **cohérence** d'un reçu avec un fichier ; il ne garantit **pas la signature cryptographique externe** d'un reçu P2.7. Un attaquant contrôlant à la fois les pins, le reçu et les fichiers demeure hors du modèle d'authenticité. APP-007 est la voie distincte de rejeu natif épinglé.

## Bureau — neuf fonctions de revue, non neuf signatures indépendantes

**ATLAS** : reprise topologique des nœuds restants ; **FERRUM** : compilation du vrai moteur Rust et maintien des limites OS ; **BOUNDARY** : aucun écrasement des publications antérieures et grant hors données ; **CIPHER** : pins externes obligatoires, pas de fausse signature ; **LEDGER** : historique exact et suffixe de nouveaux événements reliés au plan ; **CAPSTONE** : refus systématique de l'in-flight inconnu ; **ADVERSARY** : mutation des reçus, des fichiers, des dépendances et des fins de lignes ; **VERITY** : vraies exécutions natives et régressions sur quatre systèmes ; **GUARDIAN** : C20, RoadMapGuardian et autorisation de recherche AI-only.

## Qualification GitHub

**40 nouveaux tests APP-011**, tous ancrés sur des historiques authentiques produits par le vrai NORDOI Rust, plus **368 tests APP-001…APP-010**, soit **408 tests applicatifs / système et 1 632 exécutions** sur Mac Intel, Apple Silicon, Linux et Windows ; Rust Test Gate 5 jobs et E8 4 jobs. Des démonstrations produisent un préfixe réel, reprennent une tâche, vérifient le DAG recomposé avec APP-010 et archivent reçus/fichiers originaux et finaux. Les cas négatifs vérifient qu'un préfixe douteux ou un dossier de tâche existant est refusé avant tout effet.

**Aucun PASS final avant 13/13 jobs au SHA exact.**

```ini
R6F=IN_PROGRESS_UNCERTIFIED
R6G=LOCKED_BY_SEQUENCE
EXTERNAL_NATIVE_ADMISSION_GATES=12_UNMET
HUMAN_TESTERS=0
MERGE=NONE
PRODUCTION=NOT_AUTHORIZED
```
