# NORDOI G0.3 R6F — BUREAU-001 R5 : mutations en place, ABA et cohérence multifichier

**Candidat de recherche seulement, refus systématique de toute admission native.** Parent R4 exact : `4dd43f37e7a7ac0ca0b0b39fcc88248cd2bad9ad`. Le rejeu Mac Intel R4 soumis par le propriétaire a rapporté **1 159 tests réguliers PASS**, et sa CI GitHub **9/9 SUCCESS**. Ce document R5 décrit un nouvel audit : les résultats R5 doivent être vérifiés séparément, sans réinterpréter les expériences antérieures.

## La question testée

Un descripteur ouvert (FD) empêche que le nom d'un fichier remplacé par un autre soit confondu avec l'objet déjà ouvert. **Il ne rend pas immuables les octets du même fichier.** Un administrateur peut écrire des octets différents *dans le même inode*, puis remettre les octets d'origine. Plusieurs fichiers peuvent aussi être modifiés/restaurés ensemble. Une suite A → B → A peut laisser des octets et un inode final identiques au début, mais les métadonnées de changement (`ctime`) peuvent révéler les opérations pendant une session de contrôle.

Il faut séparer trois garanties différentes :

1. **Intégrité pendant un examen en cours :** le processus qui conserve un FD observe aussi le `ctime`, la taille, l'identité de l'inode et les octets ; il peut refuser après une mutation en place.
2. **Cohérence entre fichiers :** un fichier public `.pub` et un `allowed_signers` peuvent être temporairement modifiés ensemble, puis restaurés. Une comparaison des octets finaux ne prouve pas qu'aucun état intermédiaire n'a existé.
3. **Historique global au-delà du processus :** dans le laboratoire historique R5Q, une restauration cohérente de trois vrais fichiers SQLite/JSON suivie d'un **nouveau processus** efface la connaissance locale d'un état plus récent ; seules une conservation et une autorité indépendantes peuvent éventuellement établir l'anti-retour-arrière global.

### Trois expériences R5 réellement effectuées par le code, si elles passent

`research/g03_r6f_r5_inplace_aba.py --exercise` crée **des dossiers privés temporaires uniquement** ; n'écrit pas dans le dépôt original, les clés de production ni les données de l'utilisateur.

- **R5-A, modification en place :** un véritable processus OS secondaire modifie les octets d'un `allowed_signers` ouvert, force l'écriture, restaure les octets d'origine et remet l'heure `mtime`. Le même inode et les mêmes octets reviennent ; la garde R4 doit **refuser** l'historique qu'elle a pu constater, notamment grâce à `ctime`.
- **R5-B, deux fichiers :** le processus secondaire modifie le fichier candidat `.pub` et sa liste `allowed_signers`, puis restaure **les deux**. Le contrôle R4 doit refuser une session contenant des changements réels même si les deux fichiers semblent cohérents à la fin.
- **R5-C, restauration ABA du chemin :** le processus secondaire renomme le fichier original, substitue une deuxième identité puis replace le **même inode original**. Même octets et même inode à la fin ne doivent pas être pris pour une preuve de continuité ininterrompue. Le refus R4 est exigé.
- **R5-D, restauration d'époque :** rejoue réellement le laboratoire existant `research/g03_r5q_coherent_local_rollback.py`, avec les trois fichiers SQLite/JSON restaurés et un nouveau processus examinateur qui ne peut pas déterminer l'existence d'une époque avancée.

**Important :** R5 n'affirme pas avoir contourné R4 dans ces scénarios de la même session. L'objectif est d'observer ce que R4 détecte déjà et de documenter précisément où sa garantie **s'arrête**. Toute conclusion d'un succès expérimental dépendra des résultats CI et Mac Intel. Une restauration globale entre sessions est déjà un contre-exemple historique R5Q : ce n'est pas une nouvelle admission accordée.

## Mesures, tests, correction et limites

Les expériences n'utilisent aucune simulation de signature ou de processus. Les modifications de fichiers et les restaurations se font via un véritable processus `python3` distinct, `os.open`/`write`/`fsync`, `os.rename`/`os.replace`, suivi d'observations `os.stat`. Les vérifications de signature restent celles de l'OpenSSH installé.

Le correctif R5 est **une nouvelle garde de recherche adversariale et sa matrice de régression**, non une modification silencieuse du R4 qualifié. Si R4 refuse déjà les trois mutations sous les conditions expérimentales, il serait inexact de présenter R5 comme la correction d'une faille d'admission native. Les résultats sont explicitement `BLOCK / NOT_AUTHORIZED`.

**Limites non levées :** un attaquant contrôlant la mémoire du vérificateur, le noyau, un système de fichiers malveillant, l'origine de toutes les preuves, ou capable de restaurer une époque entière entre deux exécutions, n'est pas réfuté par un test local réussi. Le `ctime` est une métadonnée locale, non un horodatage indépendant authentifié, et n'est pas garanti uniformément sur tous systèmes de fichiers. La définition d'un mécanisme de confiance externe reste nécessaire.

## Bureau : neuf angles de contrôle, aucune fausse signature

La matrice `governance/NORDOI_R6F_BUREAU001_R5_ABA_MULTIFILE_AUDIT.json` confronte ATLAS (Rust), FERRUM (natif/macOS), BOUNDARY (FD/processus), CIPHER (signatures), LEDGER (preuves), CAPSTONE (capacités), ADVERSARY (ABA), VERITY (tests) et GUARDIAN (contrôles). **Ce sont des perspectives documentées par un audit de coordination**, pas neuf agents indépendants ayant réellement effectué des revues authentifiées.

## Qualifications requises

```bash
python3 -B scripts/roadmap_guardian.py --audit
python3 -B scripts/bureau_001_r1_audit.py --audit
python3 -B scripts/bureau_001_r2_review_intake.py --audit
python3 -B -m unittest discover -s research -p 'test_g03_r6f_r5_inplace_aba.py' -v
python3 -B research/g03_r6f_r5_inplace_aba.py --exercise
# La dernière commande retourne intentionnellement le code 2 : aucune admission native.
```

La CI doit réaliser **24 nouveaux contrôles R5** (16 portables et huit POSIX avec de vrais processus), les régressions R3/R4, R6A–R6E, R5A–R5Z, R4Y/R4Z, le RoadMapGuardian, les audits R1/R2 et **six véritables contrôles natifs R4M XPC/CDHash en série sans reprise ni dérogation sur macOS**. Windows n'hérite d'aucune fausse preuve POSIX. Le rejeu propriétaire Mac Intel est un contrôle distinct.

```ini
R6F_R5=RESEARCH_AUDIT_CANDIDATE
R4_HELD_FD_MUTATION_GUARD=UNDER_REAL_NATIVE_TEST
CROSS_EPOCH_COHERENT_ROLLBACK=R5Q_PRIOR_REAL_COUNTEREXAMPLE
INDEPENDENT_BUREAU_REVIEWS=0
BUREAU_FORMAL_CERTIFICATION=PENDING
EXTERNAL_GUARDIAN=NOT_ENROLLED
OWNER_OOB_ROOT=NOT_AUTHENTICATED
INDEPENDENT_GLOBAL_ANTIROLLBACK=NOT_VERIFIED
NATIVE_ADMISSION_GATES=12_UNMET
R6F=IN_PROGRESS_UNCERTIFIED
R6G=LOCKED_BY_SEQUENCE
PRODUCTION=NOT_AUTHORIZED
```
