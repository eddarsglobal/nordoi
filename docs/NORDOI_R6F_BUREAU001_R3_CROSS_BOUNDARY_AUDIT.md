# NORDOI G0.3 R6F — BUREAU-001 R3 : audit inter-domaines Rust, OS natif, SSHSIG, preuves

**Candidat de recherche R3 / PR DRAFT / aucune admission native.** Parent exact : `80010bdddd5e4743f023c0eed888a1c3360339da` (R6F R2). R2 : CI GitHub 9/9 PASS et rejeu propriétaire Mac Intel **1 111 tests réguliers PASS, 188/188 empreintes historiques, 99/99 empreintes R1 et 9/9 perspectives sources**, `SCRIPT_EXIT=0`, selon le journal transmis. Ces résultats ne prouvent ni neuf agents indépendants ni un gardien externe.

## Bureau, méthode et objet de l'audit

Les neuf fonctions **ATLAS (Rust), FERRUM (OS natif), BOUNDARY (processus), CIPHER (cryptographie), LEDGER (preuves), CAPSTONE (capacités), ADVERSARY (contre-exemples), VERITY (tests) et GUARDIAN (gouvernance)** sont mobilisées comme **perspectives d'examen de sources**. Les observations et leurs sources sont tracées dans `governance/NORDOI_R6F_BUREAU001_R3_CROSS_AUDIT.json`. Il s'agit d'un audit croisé de coordination exécuté avec vrais tests ; ce document ne prétend pas que neuf agents séparément exécutés ou neuf vérificateurs administrativement indépendants ont signé les conclusions. Les neuf rapports spécialisés restent `NOT_SUBMITTED` jusqu'à preuves d'exécution authentifiées.

## R6F-R3-001 — Défaut d'attribution cryptographique découvert

Dans le fichier **original inchangé** `research/g03_r6e_oob_origin_intake.py`, la méthode `assess` :

- vérifie que l'empreinte SHA-256 de `local_public` correspond au `candidate_key_sha256` déclaré (lignes 228–231) ;
- mais vérifie la sixième signature via `ssh-keygen -Y verify -f local_allowed` (lignes 244–249) ;
- **ne compare pas** le contenu du fichier `local_allowed` avec la clé publique exactement épinglée dans `local_public`.

Un `allowed_signers` local changé pour contenir une **troisième clé Ed25519 authentique**, avec un `SSHSIG` réel de cette clé sur le contenu R6E original, peut donc entraîner le libellé historique `SIX_VALID_LOCAL_SIGNATURES_NOT_OWNER_OOB_ENROLLMENT` : la signature cryptographique est valide pour une **autre clé** que celle déclarée. Cela constitue une faiblesse précise de **l'attribution de l'identité du signataire dans le laboratoire**, pas une faille de permission native : **tous les résultats historiques restent `BLOCK / NOT_AUTHORIZED`**.

### Correctif additif sans réécriture des commits gelés

`research/g03_r6f_r3_signer_binding.py` introduit un chemin de validation **strict et explicitement non autorisant** :
- lit et vérifie la clé publique `ssh-ed25519` selon son encodage binaire SSH canonique, avec taille et limites précises ;
- impose un seul signataire `same-admin-research-fixture` dans chaque fichier `allowed_signers`, correspondant **exactement** à la clé `.pub` propriétaire, candidate et sixième signataire, sans autre clé ni options ;
- compare ces trois fichiers à leurs identifiants SHA-256 du dossier canonique ;
- n'accepte le libellé `R3_SIX_LOCAL_SSHSIGS_BOUND_TO_EXACT_DECLARED_KEYS_NO_EXTERNAL_CUSTODY` qu'après revalidation de la véritable chaîne R6E OpenSSH signée ;
- refuse la substitution sous `R3_SIGNATURE_ATTRIBUTION_QUARANTINED`, sans jamais promouvoir l'autorité.

Le fichier R6E, ses 24 tests et le noyau Rust restent strictement inchangés pour conserver l'histoire. **R3 devient la version de recherche recommandée pour l'attribution exacte de la sixième signature**, tandis que le résultat historique R6E doit être interprété de façon limitée. Cette protection de fichiers locaux **ne promet pas une résistance complète aux courses de remplacement dans un environnement dont l'administrateur contrôle tous les fichiers** ; aucun gardien externe réel n'est enrolé.

### Contre-exemple réel, sans signature inventée

Le laboratoire crée un troisième couple Ed25519 via le véritable `/usr/bin/ssh-keygen` dans un répertoire temporaire privé, signe **les octets R6E originaux** sous le namespace réel R6E, puis exécute l'ancien contrôle et le nouveau contrôle sur **la même signature et la même liste autorisée** :

| Contrôle | Attendu |
|---|---|
| Original R6E avec une clé locale substituée | Ancien libellé « six signatures valides » ; admission toujours `BLOCK` |
| Correctif additif R3, mêmes preuves | `R3_SIGNATURE_ATTRIBUTION_QUARANTINED` |
| R3 avec les six signatures et clés correctes | Six véritables signatures liées aux clés exactes, mais toujours `BLOCK` |
| Erreur de namespace, signature invalide ou liste comprenant plusieurs clés | Refus, jamais admission |

**24 tests adversariaux R3** : 16 contrôles portables de syntaxe et de liaison de clés et huit expériences OpenSSH/POSIX réelles (Windows omet explicitement les huit natifs, sans inventer une preuve).

## R6F-R3-002 à R6F-R3-005 — Confrontation avec les autres domaines

- **ATLAS/FERRUM** : R4A est un laboratoire de threads Rust non relié à un exécuteur utilisateur. R4M vérifie de vraies applications et signatures macOS/XPC CDHash, mais les clés locales *ad hoc* ne constituent pas un signataire de production indépendant. Le test natif R4M six est relancé en série, sans reprise ni dérogation.
- **BOUNDARY/ADVERSARY** : R6D montre que deux PID/UID et deux clés ne constituent pas deux autorités administratives. R5Q montre qu'un ensemble de preuves locales restauré de façon cohérente peut tromper un examinateur local nouvellement démarré.
- **LEDGER/VERITY** : la chronologie des statuts `PENDING` figés R6B–R6E est conservée séparément des journaux propriétaires fournis ultérieurement. Le nouveau contre-exemple comble une lacune dans les 24 tests R6E originaux, sans déduire de la CI que les vérifications externes sont satisfaites.
- **CAPSTONE/GUARDIAN** : les 12 conditions d'admission native restent `UNMET`. R2 ne sait pas authentifier le véritable auteur d'un log local de revue ; les résultats de l'audit de coordination ne valent pas neuf certifications formelles.

## Commandes d'audit (lecture seule, sauf fichiers temporaires du laboratoire)

```bash
python3 -B scripts/roadmap_guardian.py --audit
python3 -B scripts/bureau_001_r1_audit.py --audit
python3 -B scripts/bureau_001_r2_review_intake.py --audit
python3 -B -m unittest discover -s research -p 'test_g03_r6f_r3_signer_binding.py' -v
python3 -B research/g03_r6f_r3_signer_binding.py --exercise
# L'exercice natif refuse l'admission, retourne intentionnellement code 2
```

La CI dédiée couvre macOS Intel, macOS Apple Silicon, Linux et Windows, plus les régressions R6A–R6E, R5A–R5Z, R4Y/R4Z et R4M natif réel six en série sur macOS. Le Test Gate général doit aussi être vert. Le futur rejeu sur le Mac Intel du propriétaire reste une étape distincte, non inférée du succès GitHub.

```ini
R6F_R3=RESEARCH_CANDIDATE_CROSS_BOUNDARY_AUDIT
R6E_OLD_SIGNER_ATTRIBUTION=INSUFFICIENTLY_BOUND_TO_ALLOWED_SIGNERS
R3_NEW_RESEARCH_BINDING=DENY_ONLY
NINE_REVIEW_PERSPECTIVES=SOURCE_ANCHORED
INDEPENDENT_SPECIALIST_EXECUTIONS=0
BUREAU_FORMAL_CERTIFICATION=PENDING
R6F=IN_PROGRESS
R6G=LOCKED_BY_SEQUENCE
NATIVE_ADMISSION_GATES=12_UNMET
EXTERNAL_GUARDIAN=NOT_ENROLLED
OWNER_OOB_ROOT=NOT_AUTHENTICATED
GLOBAL_MONOTONIC_ANTIROLLBACK=NOT_VERIFIED
PRODUCTION=NOT_AUTHORIZED
```
