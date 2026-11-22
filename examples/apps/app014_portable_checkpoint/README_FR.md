# NORDOI — APP-014 : checkpoint natif autonome, portable et vérifiable

**22 novembre 2026 · Recherche IA → IA · Sans testeur humain**

## Point de départ C20

Le commit parent exact est **APP-013** `af87e437968562412d6098c032b3f4445f6f383f`, qualifié **13/13 GitHub SUCCESS** (APP-013 4/4, Rust 5/5, E8 4/4). PR #164 DRAFT, ouverte, non fusionnée. APP-014 ne change aucun code privilégié, ne modifie pas `main` et reste entièrement dans le périmètre non productif.

## Évolution utile à l'agent IA

APP-013 produit un excellent **transfert vérifiable** d'un graphe, mais **ses fichiers P2.7 natifs doivent rester disponibles séparément** : le dossier scellé seul ne suffit pas. APP-014 ajoute un **paquet véritablement portable** : un agent peut déplacer/copier les octets natifs originaux, avec le plan et le journal, vers un autre répertoire ou système compatible, les revérifier en lecture seule, restaurer les publications historiques dans une racine vide puis continuer le DAG avec le **vrai moteur Rust**.

Le paquet complet se présente ainsi :

```text
checkpoint-portable/
  handoff/
    plan.json               # octets originaux APP-012
    native-prefix.jsonl     # événements natifs originaux APP-012
    manifest.json           # manifeste interne APP-013 exact
    SEAL.sha256             # marqueur non signé APP-013 exact
  native/
    job-0001/
      quality-decision/
        decision.txt        # vrais octets natifs P2.7
        report.txt
    job-0002/...
  portable.json             # inventaire déterministe contenu/adresses relatives
  PORTABLE.sha256           # marqueur final, NON signé
```

Chaque fichier est créé en exclusivité et jamais écrasé. Le moteur vérifie d'abord l'authenticité **de cohérence** de la source avec APP-013/010/006, copie exclusivement les fichiers P2.7 dont le schéma impose les noms `decision.txt` et `report.txt`, revérifie le contenu copié puis écrit le marqueur externe en dernier.

### Fonctionnement agent-à-agent

**1. PACK — empaqueter un vrai checkpoint APP-013** déjà qualifié par un sceau externe :

```bash
python3 -B examples/apps/app014_portable_checkpoint/portable_app014.py pack \
  --portable-dir /transfers/new-portable-checkpoint \
  --handoff-dir /original/app013-handoff \
  --native-root /original/native-p27-jobs \
  --expected-handoff-sha256 TRUSTED_APP013_MANIFEST_SHA256
```

Sortie : `PACKED` et **SHA-256 du manifeste portable**. Pour qu'une autre instance puisse établir l'origine du paquet, elle devra obtenir ce SHA-256 **par un canal de confiance séparé** ; le marqueur à l'intérieur du paquet ne remplace pas ce canal.

**2. VERIFY — auditer un paquet copié, sans moteur natif ni écriture :**

```bash
python3 -B examples/apps/app014_portable_checkpoint/portable_app014.py verify \
  --portable-dir /received/checkpoint-portable \
  --expected-portable-sha256 TRUSTED_EXTERNAL_PORTABLE_SHA256
```

L'auditeur contrôle l'empreinte externe, le marqueur, le manifeste canonique, l'intégrité du dossier APP-013 interne, le plan, le préfixe JSONL, tous les événements et dépendances, les reçus natifs et **chaque octet des vrais fichiers P2.7**. Il refuse tout contenu inattendu, symlink interdit, fichier absent ou octet divergent, et n'appelle jamais le moteur Rust.

**3. RESTORE — réhydrater les tâches historiques dans une racine *existante et vide* :**

```bash
python3 -B examples/apps/app014_portable_checkpoint/portable_app014.py restore \
  --portable-dir /received/checkpoint-portable \
  --expected-portable-sha256 TRUSTED_EXTERNAL_PORTABLE_SHA256 \
  --restore-root /new/existing/empty/native-root
```

Avant tout effet, le paquet est vérifié en totalité. La racine de destination doit être un vrai dossier vide, disjoint du paquet. Les répertoires `job-NNNN` sont créés exclusivement, sans copier de chemin arbitraire du manifeste. Chaque fichier est recopié sans écrasement, puis l'**audit indépendant APP-010** contrôle de nouveau le dossier réhydraté. Un échec en cours de restauration est `ERROR/UNKNOWN`, sans prétendre avoir annulé les fichiers précédemment copiés.

**4. RESUME — continuer le graphe sur les preuves restaurées :**

```bash
python3 -B examples/apps/app014_portable_checkpoint/portable_app014.py resume \
  --portable-dir /received/checkpoint-portable \
  --expected-portable-sha256 TRUSTED_EXTERNAL_PORTABLE_SHA256 \
  --restore-root /new/existing/restored/native-root \
  --engine /trusted/native/nordoi \
  --source-root /trusted/noi/src \
  --expected-engine-sha256 TRUSTED_ENGINE_SHA256 \
  --expected-source-sha256 TRUSTED_SOURCE_TREE_SHA256
```

Avant toute reprise : vérification complète du paquet et audit indépendant des dossiers restaurés. Le processus appelle ensuite **APP-011** avec le binaire Rust et les sources épinglés, puis émet **seulement le vrai suffixe JSONL natif**. La concaténation du préfixe du paquet et de ce suffixe peut être auditée avec APP-010 ; **le programme ne rejoue pas les publications historiques**. Une deuxième restauration ou reprise dans la même racine est refusée.

## Bureau — neuf axes documentés, non neuf auditeurs distincts

**ATLAS** : continuité APP-012 → APP-013 → APP-014 → APP-011 → APP-010 ; **FERRUM** : exécution native Rust uniquement pour les tâches nouvelles ; **BOUNDARY** : paquet et racine restaurée distincts, création exclusive et lecture sans effet lors de l'audit ; **CIPHER** : empreinte externe du paquet obligatoire, aucune signature inventée ; **LEDGER** : intégrité profonde des événements, reçus et fichiers ; **CAPSTONE** : aucun statut `PASS` artificiel d'un graphe incomplet, refus des tâches ambiguës ; **ADVERSARY** : fausses empreintes, fichiers modifiés, ajouts, double restauration et double reprise ; **VERITY** : vrais checkpoints Rust sur quatre systèmes et régressions ; **GUARDIAN** : parent exact C20, AI-only, aucune fusion ni privilège nouveau.

Les neuf axes sont une **grille d'examen**, pas neuf signataires externes indépendants.

## Limites scientifiques et de sécurité

**Une empreinte SHA-256 n'est pas une identité ni une signature.** La correspondance des octets ne prouve pas l'authenticité du canal ayant livré la valeur attendue. Le code vérifie les vrais fichiers contre les reçus P2.7 APP-006 et les événements APP-010 ; il ne reproduit pas indépendamment le calcul cryptographique interne P2.7. Les fichiers, reçus et empreintes tous compromis ensemble restent hors de la garantie. Le paquet n'inclut ni le binaire Rust exécutable ni les sources `.noi` : le destinataire doit sélectionner des versions compatibles **distinctement épinglées**.

Il n'existe aucune garantie de durabilité face à la panne de courant, ni de résistance à un hôte local malveillant concurrent, ni de transaction globale, ni de rollback après copie partielle, ni de restauration automatique d'une opération in-flight au résultat incertain. Un dossier restauré en partie est **rejeté**, pas complété aveuglément. Les permissions et marqueurs sont des contrôles de cohérence, pas une autorisation de production ou une attestation OS.

## Qualification CI demandée

**40 tests APP-014** ancrés sur les fichiers réellement publiés par NORDOI Rust au travers d'APP-012, puis scellés avec APP-013. **488 régressions APP-001…APP-013** conservées sur chaque système : **528 tests / système, 2 112 exécutions attendues** (Mac Intel, Apple Silicon, Linux, Windows). Rust Test Gate 5 jobs + E8 4 jobs ; **13 jobs** au SHA exact.

Démonstration obligatoire sur chaque OS : production réelle APP-012 de deux tâches → scellement APP-013 → paquet autonome APP-014 → copie du paquet vers un autre emplacement → vérification externe sans accès aux originaux → restauration dans une racine différente et vide → reprise réelle APP-011 → audit APP-010 du DAG complet → refus d'une seconde reprise et du paquet falsifié.

**Aucun verdict PASS sans 13/13 SUCCESS au même commit.**

```ini
R6F=IN_PROGRESS_UNCERTIFIED
R6G=LOCKED_BY_SEQUENCE
EXTERNAL_NATIVE_ADMISSION_GATES=12_UNMET
HUMAN_TESTERS=0
PR=DRAFT
MERGE=NONE
PRODUCTION=NOT_AUTHORIZED
```
