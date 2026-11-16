# NORDOI G0.3 R6F — BUREAU-001 R8

## Audit contradictoire des critères de clôture : travaux techniques qualifiés ≠ certification indépendante

**Parent source exact R7 :** `ee4975e42d4364dd3d2bc78ae2d7a08623042298`. La preuve transmise pour R7 comprend un rejeu intégral propriétaire sur macOS 14.5 Intel x86_64 (`978` Python, `253` Rust, **1 231 tests réguliers PASS**, `SCRIPT_EXIT=0`) et la CI GitHub spécialisée 4/4 + générale 5/5 SUCCESS. Le journal est une observation de l'exécution déclarée par le propriétaire ; il n'est pas une signature externe indépendante, et les succès CI ne prouvent pas la séparation administrative.

**Décision R8 envisagée :** classer les preuves du dossier, vérifier les conditions de clôture et **refuser la clôture formelle de R6F** tant que les attestations indépendantes et les douze conditions natives ne sont pas réellement obtenues. R8 n'est **pas** une revue exécutée par neuf spécialistes indépendants et ne met pas fin à R6F. Il ajoute seulement un contrôle de cohérence et de refus, sans modifier les anciens jalons.

## 1. Registre R1–R7 des travaux techniques

| Mission | Commit exact | Tests réguliers du rejeu propriétaire rapportés |
|---|---|---:|
| R1 | `108548e2021de42e65e8261fadd8184d4d9ff783` | 1 087 |
| R2 | `80010bdddd5e4743f023c0eed888a1c3360339da` | 1 111 |
| R3 | `2dd46dc07568bf382c49b1c8bca0cba8900b7b1f` | 1 135 |
| R4 | `4dd43f37e7a7ac0ca0b0b39fcc88248cd2bad9ad` | 1 159 |
| R5 | `4d3bd3cbff8c67cef98481750270a2c6c0ce7b37` | 1 183 |
| R6 | `17614622acb09440dfe2428b85fe2c89491a0b35` | 1 207 |
| R7 | `ee4975e42d4364dd3d2bc78ae2d7a08623042298` | 1 231 |

Les comptes de tests correspondent à des **journaux utilisateur communiqués**, non à une nouvelle exécution indépendante par R8. Pour R3–R7, les références des deux exécutions GitHub Actions propres à chaque jalon figurent dans le registre R8 ; la réussite CI qualifie les assertions des tests, pas une autorité extérieure. R8 conserve les sept contradictions R7, les 99 empreintes R1 et les 188 empreintes historiques sélectionnées.

## 2. Sept critères de clôture — décision à deux axes

| ID | Contrôle de clôture | Verdict fondé sur les preuves actuellement présentes |
|---|---|---|
| `R8-SOURCE-001` | Dépôt/archives figés et rejeux R1–R7 | Résultats techniques **rapportés** et sources épinglées ; pas d'attestation externe |
| `R8-CI-002` | CI GitHub R7 exacte | **9/9 SUCCESS** précédemment vérifié, capacité technique seulement |
| `R8-REVIEW-003` | Neuf rapports indépendants des rôles du Bureau | **UNMET** : R2 indique zéro rapport soumis |
| `R8-OOB-004` | Racine propriétaire authentifiée hors bande | **UNMET** : aucune vérification indépendante enregistrée |
| `R8-OPERATOR-005` | Gardien juridiquement/administrativement distinct + quorum | **UNMET** : personne réellement enrôlée, aucun quorum indépendant |
| `R8-LEDGER-006` | Historique global monotone, récupération et révocation | **UNMET** : limite de retour arrière total R5Q encore effective |
| `R8-DECISION-007` | Certification formelle du Bureau et autorisation distincte du propriétaire | **UNMET** |

**Verdict de clôture :** les deux premières catégories de preuves techniques existent avec des limites explicites. **Les cinq catégories externes manquent.** Le dossier de recherche peut être **consolidé pour examen**, mais **ne peut pas être déclaré formellement clos, certifié ou autorisé**.

Cette classification est une vérification des pièces présentes dans le dépôt, effectuée par un coordinateur. Elle **ne se substitue pas** à un audit indépendant réel.

## 3. Les douze conditions natives historiques sont conservées sans falsification

La liste exacte provient de `research/G03_R6E_OOB_ORIGIN_NO_EXTERNAL_AUTHORITY.json` et `scripts/roadmap_guardian.py`. R8 exige les mêmes douze identifiants, le même ordre et l'état exact `UNMET_INDEPENDENT_EXTERNAL_EVIDENCE`. Chaque champ `candidate_external_evidence`, `independent_signer` et `authenticated_custodian` reste `null` ; chaque `admitted` reste `false`.

Les neuf rôles du Bureau ATLAS, FERRUM, BOUNDARY, CIPHER, LEDGER, CAPSTONE, ADVERSARY, VERITY et GUARDIAN sont conservés **sans signature, journal indépendant ou verdict exécuté**. Les neuf positions ne sont pas neuf actes d'expertise indépendants.

## 4. Attaque adversariale du protocole de clôture

Le nouveau garde de recherche `scripts/bureau_001_r8_closure_gate.py` ouvre, **en lecture seule**, les sources réelles R1–R7 via les audits R1/R2/R7, vérifie les commits, les sept contradictions, la liste exacte des douze portes et les sept critères. Il refuse toute contradiction dans les sources de preuve, tout statut inventé d'une porte ou toute soumission faussement signée d'un spécialiste.

Le format `candidate_template()` permet de tester une revendication **locale non authentifiée** : « le CI est vert, le propriétaire a 1 231 tests, neuf réviseurs indépendants ont approuvé, le gardien externe est présent, le quorum est formé, les douze portes sont satisfaites ». **Même avec tous ces champs positifs, le vérificateur répond `BLOCK`**, car ce paquet ne contient ni racine indépendante authentifiée ni signatures externes vérifiées. Les champs du candidat ne sont jamais promus en preuves d'admission.

Sur POSIX, `--exercise` rejoue réellement l'expérience R7 fondée sur de **vraies signatures locales OpenSSH Ed25519** et sur la **restauration cohérente réelle de trois fichiers** R5Q, sans créer de fausse autorité externe. Le résultat natif positif attendu pour le laboratoire est un **refus intentionnel avec code 2** ; une erreur de source ou d'exécution est codée **3**, jamais acceptée comme un test réussi.

`scripts/test_bureau_001_r8_closure_gate.py` couvre **24 contrôles** : 16 portables (source, JSON canonique, double clé, parent SHA, rôles, non-promotion, autorisations falsifiées) et huit POSIX réels (OpenSSH, R7 et R5Q). Les contrôles POSIX sont explicitement ignorés sous Windows.

## 5. Conditions de clôture future, sans les présumer réalisées

Les éléments encore nécessaires sont : vérification humaine et autorisation d'un véritable opérateur séparé ; empreinte exacte du gardien confirmée par le propriétaire via un canal indépendant ; signatures et contrôles de domaine administratif indépendants ; historique monotone externe des époques de clés, nonces consommés et révocations ; quorum distinct et audit de récupération ; neuf comptes rendus spécialisés avec exécution et provenance attestées ; **décision de gouvernance et autorisation du propriétaire séparées** si le dossier devient un jour éligible.

**R8 ne contacte personne, n'enrôle pas de gardien et ne crée pas de clé de tiers.** Il n'envoie aucune preuve sur le réseau. Il n'intègre pas de mécanisme de production. Aucun privilège, droit d'exécution native ni verrou RoadMap ne change.

## 6. Tests et qualification indépendante du propriétaire

```bash
python3 -B scripts/roadmap_guardian.py --audit
python3 -B scripts/bureau_001_r1_audit.py --audit
python3 -B scripts/bureau_001_r2_review_intake.py --audit
python3 -B scripts/bureau_001_r8_closure_gate.py --audit
python3 -B -m unittest discover -s scripts -p 'test_bureau_001_r8_closure_gate.py' -v
python3 -B scripts/bureau_001_r8_closure_gate.py --exercise
```

`--audit` et `--exercise` retournent **2 volontairement** pour la décision `BLOCK`, même après vérification réussie. Chaque runner CI et Mac Intel doit vérifier les champs JSON exacts et distinguer une exécution du refus d'une erreur du laboratoire.

La CI doit conserver tous les anciens tests R1–R7, la suite Rust, Rustfmt et Clippy strict, ainsi que **six contrôles natifs signés XPC/CDHash R4M en série sur macOS sans reprise ni dérogation**. Le rejeu Mac Intel sera un contrôle technique propre à l'équipement du propriétaire, non une certification externe.

```ini
G03_R6F_R8=RESEARCH_CLOSURE_EVIDENCE_AUDIT_CANDIDATE
R1_TO_R7_RESEARCH_TECHNICAL_EVIDENCE=CONSOLIDATED_WITH_LIMITATIONS
R6F_FORMAL_CLOSURE=BLOCKED_MISSING_INDEPENDENT_EVIDENCE
INDEPENDENT_SPECIALIST_REVIEWS=0
EXTERNAL_GUARDIAN=NOT_ENROLLED
OWNER_OOB_ROOT=NOT_AUTHENTICATED
GLOBAL_MONOTONIC_ANTIROLLBACK=NOT_VERIFIED
NATIVE_ADMISSION_GATES=12_UNMET
R6F=IN_PROGRESS_UNCERTIFIED
R6G=LOCKED_BY_SEQUENCE
PRODUCTION=NOT_AUTHORIZED
```

**Aucune certification externe, fusion ou admission native n'est effectuée par R8.**
