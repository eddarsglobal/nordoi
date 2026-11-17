# NORDOI — G0.3 R6F / BUREAU-001 E7

## Race interprocessus réelle sur un nonce signé ; correction locale atomique et limite R5Q

**Directive propriétaire impérative : NORDOI ne sera jamais testé sur des humains et n'utilisera jamais de testeurs humains.** Ce travail suit `RoadMap.md` et `Bureau.md` sans les modifier, ni les 99 blobs de sources gelés R1. Le Bureau se compose de neuf **fonctions d'expertise**, et non de neuf indépendants certifiés.

### Prérequis réellement qualifié

E6, PR #139 source exacte `ab3858d2c909b91c997782993b5c1739f49286b8`, a passé **9/9 jobs GitHub** sur deux workflows : [4/4 spécialisés](https://github.com/eddarsglobal/nordoi/actions/runs/44030442674) et [5/5 Test Gate Rust](https://github.com/eddarsglobal/nordoi/actions/runs/44030442683). Sur POSIX, de vraies signatures locales Ed25519 SSHSIG lient source/manifeste/défi/namespace, mais un autre processus sans état de nonces consommés peut accepter à nouveau les octets signés. Une clé locale ne prouve aucune indépendance administrative.

### Nouvelle question falsifiable, différente de E6

E6 a montré le rejeu d'un reçu signé par des processus qui **n'ont pas de mémoire commune fiable**. E7 examine le cas **où deux processus partagent au contraire une vraie base locale**, mais où chacun effectue d'abord un `SELECT` puis un traitement séparé :

*Les deux processus peuvent-ils voir « nonce non consommé » avant de se synchroniser ? Une transaction avec verrou d'écriture et clé unique rend-elle cette consommation locale atomique malgré deux processus qui se présentent simultanément ? Et la restauration de l'ancienne base après fermeture des processus efface-t-elle toujours cet état ?*

Le prototype de recherche réalise ces trois situations sur une **vraie SQLite temporaire** et avec de **vrais processus OS**. Ce n'est ni une mesure simulée ni une admission native.

### Expérience concrète E7

1. Le laboratoire crée un nouveau répertoire privé temporaire et une vraie clé locale Ed25519. Il crée un véritable reçu E6 et le signe dans le namespace SSHSIG de E6. Aucun témoin extérieur ne signe quoi que ce soit.
2. Une vraie base SQLite est initialisée avec une contrainte `PRIMARY KEY(namespace,challenge_nonce_sha256)`. Le protocole est `DELETE` journal, `FULL` synchronous, et les écritures sont encadrées par `BEGIN IMMEDIATE`.
3. **Contre-exemple TOCTOU** : deux vrais processus vérifient chacun la signature, lisent indépendamment la base encore vide, puis sont synchronisés par deux marqueurs de fichiers réellement écrits. Ils constatent tous deux `0` avant la barrière. **Aucune capacité native n'est accordée**, il s'agit de montrer ce que verrait une logique naïve `check-then-act`.
4. **Correction atomique** : deux vrais autres processus vérifient chacun la même signature, puis passent la même barrière et entrent dans des transactions SQLite concurrentes. Un seul peut insérer le couple namespace + nonce ; le deuxième voit le nonce déjà présent et doit le refuser. On exige un vrai résultat **1 insertion locale, 1 doublon bloqué**, pas une moyenne statistique.
5. Un troisième processus redémarré observe le nonce réellement conservé et refuse sa nouvelle consommation.
6. Le laboratoire restaure ensuite **une copie locale de la base SQLite antérieure, hors utilisation par les processus**, au moyen d'un vrai remplacement de fichier. Le nonce réapparaît comme libre, et un nouveau processus le consomme à nouveau. Cela prouve que l'atomicité locale n'est pas un témoin antirollback global.
7. Le laboratoire vérifie également la vraie restauration historique **R5Q de trois fichiers**, puis conserve le commit Git d'origine sans modification.

Les données et processus sont **réels**, mais tous les chemins de résultats portent `BLOCK`, **y compris le libellé « local spent record committed »**. Cette expression signifie uniquement qu'une preuve de consommation locale a été écrite : aucune permission, privilège, installation, admission de NORDOI ou action de production n'a été réalisée.

### Vingt-quatre tests E7 et qualification

`scripts/test_bureau_001_e7_atomic_local_nonce_race.py` est composé de **18 tests portables** (source, politique et vraies transactions SQLite temporaires) et **six tests réellement POSIX** (clé et signature Ed25519 réelles ; deux vrais processus concurrents ; persistance ; restauration et E7 complet). Windows ignore explicitement les six tests propres à POSIX. Les régressions E2–E6 demeurent distinctes.

Le nouveau workflow GitHub vérifie le **commit exact de la PR**, les 24 cas E7 et les contrôles E2–E6 sur Mac Intel, Apple Silicon, Ubuntu et Windows. Sur les systèmes POSIX, il exécute **en plus** l'expérience E7 réelle complète et vérifie les valeurs observées, le code **2 = refus attendu** et l'absence du code **3 = défaut scientifique ou source**. Sur macOS, le workflow conserve la vérification native signée XPC/CDHash R4M **six tests en série sans waiver ni retry**. La CI Rust générale doit rester verte.

### Contrôle de confiance : ce qui est corrigé et ce qui ne l'est pas

| Propriété | Conclusion possible après véritable CI |
|---|---|
| Deux processus vérifient simultanément la même signature et le même nonce local | Démonstration reproductible sur OS réel |
| Contrôle naïf `SELECT` avant action | **Deux observations périmées** possibles |
| `BEGIN IMMEDIATE` et unicité namespace + nonce | **Exactement un enregistrement local** possible |
| Processus redémarré sans restauration | Doublon refusé à partir de l'état SQLite conservé |
| Restauration par le même administrateur | **Ancien nonce réutilisable**, limite persistante |
| Historique de nonces mondial indépendant | **Non démontré** |
| Identité de gardien OOB, contrôle externe et quorum | **Non démontrés** |
| R6F certifié / R6G ou production | **Interdit** |

**Le Bureau examine** ATLAS (contraintes SQLite), BOUNDARY (barrière vraie entre PID), CIPHER (SSHSIG réel), LEDGER (nonce/persistance), ADVERSARY (TOCTOU et restauration), CAPSTONE (absence de droits), FERRUM (OS/XPC), VERITY (CI/HEAD exact) et GUARDIAN (séquence). Ces intitulés sont des responsabilités de recherche automatisée, **pas neuf agents réellement lancés, ni neuf signatures d'autorité**.

### Verdict de sécurité permanent

```ini
E7_SCIENTIFIC_TEST=REAL_TWO_PROCESS_NONCE_RACE
E7_LOCAL_ATOMIC_SQLITE_PROOF=RESEARCH_ONLY
E7_TRUE_EXTERNAL_MONOTONIC_PROOF=NOT_VERIFIED
E7_EXTERNAL_WITNESS_RECEIPTS=0
HUMAN_TESTERS=0_FORBIDDEN
INDEPENDENT_SPECIALIST_REVIEWS=0
OWNER_OOB_ROOT=NOT_AUTHENTICATED
EXTERNAL_CUSTODIAN=NOT_ENROLLED
NATIVE_ADMISSION_GATES=12_UNMET
R6F=IN_PROGRESS_UNCERTIFIED
R6G=LOCKED_BY_SEQUENCE
PRODUCTION=NOT_AUTHORIZED
```

Aucune mutation du logiciel de production, aucune utilisation de personnes, aucun service externe contacté, aucune clé exportée, aucune fusion de PR. **E7 ajoute un correctif de laboratoire et une preuve de son périmètre limité**, et non une prétendue solution indépendante à E4–E6.
