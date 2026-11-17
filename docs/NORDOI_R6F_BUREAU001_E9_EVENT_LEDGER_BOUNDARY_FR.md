# NORDOI G0.3 — BUREAU-001 / R6F-E9

## Événement déclaré ≠ ligne SQLite persistée : fausse absence de conflit local

**Prérequis exact E8 :** commit `bf4c0d66256ad3a871e44cae39808af5ea1418d4`, [CI spécialisée 4/4](https://github.com/eddarsglobal/nordoi/actions/runs/44036977230), [Rust général 5/5](https://github.com/eddarsglobal/nordoi/actions/runs/44036977539). La PR E8 #141 reste DRAFT et non fusionnée. C20 est satisfait pour *démarrer* E9, pas pour admettre une capacité native.

### Faiblesse nouvelle et falsifiable

E8 a réellement démontré qu'une consommation atomique de nonce dans deux fichiers SQLite distincts peut aboutir à deux historiques locaux. Sa fonction `reconcile(events)` détecte le conflit si les deux événements transmis reflètent les vrais enregistrements.

Toutefois, `e8.signed_event` nomme un **événement local avec SHA-256 non authentifié** : l'événement n'est PAS une signature SSHSIG du témoin. Rien dans `e8.reconcile` ne lit les véritables fichiers SQLite. Le même administrateur peut transmettre un événement B indiquant un autre nonce, recalculer son SHA-256, et obtenir `split_brain_detected=False` bien que **les deux vrais fichiers SQLite contiennent toujours le même nonce**. E8 conserve `result=BLOCK` dans ce cas : ce défaut est une **fausse conclusion du champ d'observation**, pas une ouverture de droit natif.

E9 utilise une vraie clé locale Ed25519, vérifie le vrai reçu E6 dans **deux processus OS distincts**, laisse chacun écrire une vraie ligne dans son fichier SQLite, puis prouve les étapes suivantes sans simulation :

1. Deux vrais enregistrements identiques dans les bases A/B ; réconciliation E8 honnête : conflit détecté et BLOCK.
2. Création d'un événement B contrefait localement, avec autre nonce et SHA-256 cohérent ; la réconciliation E8 ne signale plus le conflit (toujours BLOCK).
3. Un nouveau contrôle de recherche compare la déclaration de chaque événement à sa **ligne effectivement stockée** dans SQLite et au nonce du vrai reçu SSHSIG ; il rejette le B contrefait avec `E9_EVENT_PERSISTED_ROW_MISMATCH`.
4. Les vrais événements demeurent acceptables pour l'**analyse locale du conflit** ; ils n'autorisent jamais une capacité native. Le contrôle ne prétend ni prévenir la réécriture cohérente des deux bases, ni garantir l'existence ou la conservation de tous les forks par un témoin indépendant.

La correction est **additive et limitée au laboratoire E9** ; les fichiers historiques E2–E8, les sources Rust et la RoadMap ne sont pas modifiés.

### Bureau : affectation et limites

- **ATLAS :** structure réelle SQLite et unicité locale ; **FERRUM :** vrais processus OpenSSH/OS et contrôle XPC/CDHash Mac ; **BOUNDARY :** frontières entre déclarations IPC et lignes SQLite
- **CIPHER :** distinguer vraie signature Ed25519 du SHA événementiel non signé ; **LEDGER :** comparaison événement/registre et historique incomplet ; **CAPSTONE :** aucun effet natif à la lecture du statut
- **ADVERSARY :** substitution cohérente et fausse absence de conflit ; **VERITY :** HEAD exact, tests machines et CI ; **GUARDIAN :** C3/C8/C14/C20, refus et douze conditions inchangées

Ces neuf noms sont des **fonctions d'audit**, non neuf agents indépendants effectivement lancés, non neuf signatures externes.

### Qualification requise

24 tests E9 : 18 portables avec véritables bases SQLite, six POSIX utilisant de vrais processus et OpenSSH Ed25519 (Windows : six SKIP explicites). CI sur macOS Intel, Apple Silicon, Linux, Windows ; régressions E2 à E8 et RoadMapGuardian ; sur macOS R4M XPC/CDHash natif six en série, sans retry ni waiver. Le Rust général séparé doit passer.

Tous les CLI attendus sortent **code 2 et BLOCK**. Les erreurs d'expérience/sources sortent **code 3** et ne valent pas qualification. Avant achèvement de la CI E9 au commit exact, le jalon reste `IN_PROGRESS_UNQUALIFIED`.

```ini
E9_LOCAL_EVENT_TO_SQLITE_GUARD=RESEARCH_ONLY
E9_OLD_E8_FALSE_NEGATIVE=NOT_YET_CI_QUALIFIED
E9_EXTERNAL_FORK_COMPLETENESS=NOT_VERIFIED
HUMAN_TESTERS=0_PERMANENTLY_FORBIDDEN
INDEPENDENT_SPECIALIST_REVIEWS=0
EXTERNAL_CUSTODIAN=NOT_ENROLLED
OWNER_OOB_ROOT=NOT_AUTHENTICATED
GLOBAL_MONOTONIC_ANTIROLLBACK=NOT_VERIFIED
R6F=IN_PROGRESS_UNCERTIFIED
R6G=LOCKED_BY_SEQUENCE
NATIVE_ADMISSION_GATES=12_UNMET
PRODUCTION=NOT_AUTHORIZED
MERGE=NOT_AUTHORIZED
```
