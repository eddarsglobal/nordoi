# NORDOI G0.3 — R6F / BUREAU-001 R7

## Rapport contradictoire consolidé R1–R6 et contrat de preuves pour un gardien réellement indépendant

**Verdict de portée : RECHERCHE SOURCE-PINNED / DENY-ONLY.** Parent immuable : `17614622acb09440dfe2428b85fe2c89491a0b35` (R6). R6 : **1 207 tests réguliers PASS** selon le journal du propriétaire Mac Intel macOS 14.5 x86_64, avec `SCRIPT_EXIT=0` ; CI spécialisée 4/4 et générale 5/5 SUCCESS à la même référence. Le journal utilisateur prouve une exécution déclarée, **pas une attestation matériellement ou administrativement indépendante**. R7 ajoute un audit, une matrice de preuves, des tests et une CI ; il ne transforme aucune source historique en nouvelle autorité.

### 1. Matrice des six étapes, sans réécrire le passé

| Étape | Commit exact de référence | Tests réguliers rapportés par le propriétaire | Qualification d’autorité externe |
|---|---|---:|---|
| R1 | `108548e2021de42e65e8261fadd8184d4d9ff783` | 1 087 | Absente |
| R2 | `80010bdddd5e4743f023c0eed888a1c3360339da` | 1 111 | Absente |
| R3 | `2dd46dc07568bf382c49b1c8bca0cba8900b7b1f` | 1 135 | Absente |
| R4 | `4dd43f37e7a7ac0ca0b0b39fcc88248cd2bad9ad` | 1 159 | Absente |
| R5 | `4d3bd3cbff8c67cef98481750270a2c6c0ce7b37` | 1 183 | Absente |
| R6 | `17614622acb09440dfe2428b85fe2c89491a0b35` | 1 207 | Absente |

Les comptes R1–R6 dans ce tableau sont **des résultats de journaux propriétaire communiqués**, pas des mesures produites par un certificateur indépendant. Les rapports GitHub CI R3–R6 ont chacun terminé avec **9/9 jobs SUCCESS** ; succès de tests et provenance humaine indépendante restent des choses différentes. Les sources R1/R2 originales sont inspectées par `scripts/bureau_001_r1_audit.py` et `scripts/bureau_001_r2_review_intake.py`, non modifiées.

La chronologie R1 du dépôt indique **`PENDING_AT_COMMIT_TIME`**, puis un champ séparé décrit le rejeu propriétaire ultérieur de R6B–R6E comme `PASS_WITH_CONTROLS_RESEARCH_ONLY`. Il n'y a pas lieu de « corriger » rétroactivement les sources gelées : l'instant de création et la preuve reçue plus tard ont deux provenances différentes.

### 2. Sept contradictions/limitations — leurs explications falsifiables

| ID | Sujet | Constat et disposition |
|---|---|---|
| R7-TEMPORAL-001 | PENDING ancien / PASS ultérieur | Deux instants : conserver l'instantané historique et joindre le rejeu propriétaire en annexe datée |
| R7-PROVENANCE-002 | R6E / R3 | R6E pouvait mal attribuer une signature valide à la mauvaise clé de `allowed_signers` ; R3 refuse le contre-exemple dans son **nouveau** chemin, pas dans l'ancienne source gelée |
| R7-TOCTOU-003 | R3 / R4 | R3 vérifie puis ferme le FD avant le ré-ouvrir par `ssh-keygen` ; R4 transmet le descripteur déjà ouvert et détecte le remplacement du chemin réel dans les conditions testées |
| R7-ABA-004 | R4 / R5 / R5Q | La session R4 détecte les restaurations ABA observées ; un nouveau processus après restauration cohérente complète des trois fichiers **perd encore** l'histoire plus récente |
| R7-OOB-005 | R6 / gardien | Deux signatures locales Ed25519 véritables peuvent couvrir un texte mensonger affirmant un gardien externe ; possession de clés != identité indépendante |
| R7-REVIEW-006 | Neuf rôles / neuf revues | R2 contient neuf **points de vue du coordinateur**, aucune soumission de relecteur indépendant et aucune attestation signée réelle |
| R7-CI-007 | Succès CI / autorisation native | 1 207 tests du Mac et 9 jobs GitHub SUCCESS ne remplissent aucune des douze conditions de confiance externe |

Ces sept éléments sont détaillés, avec chemins de sources, dans `governance/NORDOI_R6F_BUREAU001_R7_CONTRADICTIONS_AND_GUARDIAN_PLAN.json`. **Le résultat défendable de R7 est une consolidation des preuves et un refus conscient des prétentions excessives**, pas une nouvelle implémentation d'autorité privilégiée.

### 3. Douze exigences externes — le protocole de collecte réel

La liste canonique des douze ID est celle de `research/G03_R6E_OOB_ORIGIN_NO_EXTERNAL_AUTHORITY.json` et de `scripts/roadmap_guardian.py`. La matrice R7 vérifie leur identité et leur ordre exact, sans suppression ni promotion :

| Phase | Exigences | Ce qui doit être produit dans le monde réel |
|---|---|---|
| A — identité et consentement | `owner_oob_authenticated_root`, `actual_external_custodian_identity`, `distinct_administrative_control_domains` | Opérateur distinct dûment autorisé, canal de vérification propriétaire indépendant, séparation administrative réelle |
| B — épinglage OOB | `guardian_key_pinned_out_of_band` | Empreinte exacte et réception confirmée hors de portée du Mac et des comptes du propriétaire |
| C — hôte et signature | `independently_signed_os_attestation`, `signed_fresh_challenge_and_exact_history`, `independent_os_domain_and_host_witness` | Attestations réelles de l’hôte, challenge frais, preuves signées liées aux octets et commits exacts |
| D — anti-retour-arrière | `externally_monotonic_key_epoch`, `protected_global_revocation_floor`, `protected_global_spent_nonce_floor` | Historique à haute-eau des époques, révocations et nonces conservé indépendamment, contrôlé par un test de restauration locale totale |
| E — quorum et récupération | `independent_quorum`, `independent_recovery_rotation_audit` | Signatures réelles des administrateurs indépendants, journal vérifiable de rotation/récupération/révocation |
| F — décision | Aucun ID natif supplémentaire | Relectures spécialisées indépendantes, vote/autorisation explicite de gouvernance, puis seulement une décision distincte sur les étapes ultérieures |

**Statut actuel de toutes les phases A–F : `NOT_STARTED`.** Pour chacune des douze conditions, `external_evidence_sha256=null`, `independent_operator_signature=null`, `owner_out_of_band_verification_receipt=null`, `admitted=false`. Ce sont des **exigences futures**, pas des preuves disponibles.

Un VPS, un compte GitHub, un second processus, deux clés cryptographiques ou un agent IA placés sous l'administration du même propriétaire ne deviennent pas des tiers indépendants du seul fait d'un autre nom. La provenance doit être contrôlée **hors du domaine administratif dont on étudie la compromission**.

### 4. Protection contre les fausses déclarations

Le nouveau `scripts/bureau_001_r7_contradiction_guard.py` :

- exécute les contrôles historiques R1/R2 sur les sources réelles ; le manifeste R1 vérifie notamment les **99 empreintes Git** figées ;
- exige les neuf rôles sources et **zéro signature/revue indépendante**, vérifie les commits parents R3→R6 et les douze portes exactes ;
- classe les sept points de divergence : chaque explication doit avoir un fichier source existant ;
- refuse tout remplacement de `UNMET` par un `admitted=true`, toute fausse revue externe ou tout contournement du verrou R6G ;
- expose un **format strict de proposition non authentifiée** `candidate_template()`. Même si un paquet contient un nom d'opérateur, un prétendu OOB, un prétendu quorum et un prétendu historique monotone, `assess_candidate()` ne délivre **jamais** une autorité externe. Le dépôt ne dispose pas encore d'un protocole pouvant authentifier une vraie signature indépendante ;
- peut relancer l'exercice antérieur **avec deux vraies signatures Ed25519 locales** et l'expérience R5Q de véritable restauration de trois fichiers entre deux sessions. Ces expériences sont négatives et restent `BLOCK`.

Il n’y a aucune inscription externe, clé secrète exportée, requête réseau ni simulation de signature ou d'autorité tierce. Le travail R7 peut être exécuté hors connexion après récupération du code ; la CI du dépôt fournit une preuve additionnelle d'exécution des contrôles, pas de consentement humain.

### 5. Exécution et critères

```bash
python3 -B scripts/roadmap_guardian.py --audit
python3 -B scripts/bureau_001_r1_audit.py --audit
python3 -B scripts/bureau_001_r2_review_intake.py --audit
python3 -B scripts/bureau_001_r7_contradiction_guard.py --audit
python3 -B -m unittest discover -s scripts -p 'test_bureau_001_r7_contradiction_guard.py' -v
python3 -B scripts/bureau_001_r7_contradiction_guard.py --exercise
```

**Important :** R7 `--audit` et `--exercise` retournent **volontairement le code 2** pour certifier qu'aucune admission n'est accordée ; **le code 3 indique une erreur ou une mise en quarantaine** et ne doit jamais être traité comme un succès. Le véritable test CI/owner doit interpréter ces codes et vérifier le JSON attendu. Les 24 nouveaux tests R7 couvrent 16 contrôles portables et huit exercices réels POSIX/OpenSSH (Windows indique les huit natifs comme ignorés).

Pour être qualifiée, la CI R7 devra montrer les régressions R3–R6, les tests natifs macOS signés R4M XPC/CDHash **six en série, sans reprise ni dérogation**, les politiques R1/R2/Guardian, le noyau Rust inchangé et le nouveau test négatif R7. Un rejeu Mac Intel propriétaire séparé reste requis.

### Décision de gouvernance attendue, non encore prouvée à la rédaction

```ini
G03_R6F_R7=RESEARCH_ONLY_CONTRADICTION_AUDIT_CANDIDATE
FROZEN_R1_GIT_SOURCE_BLOBS=99
NINE_BUREAU_SOURCE_PERSPECTIVES=9
INDEPENDENT_SPECIALIST_EXECUTIONS=0
NATIVE_ADMISSION_GATES=12_UNMET
EXTERNAL_OPERATOR=NOT_ENROLLED
OWNER_OOB_ROOT=NOT_AUTHENTICATED
GLOBAL_ANTIROLLBACK=NOT_VERIFIED
R6F=IN_PROGRESS_UNCERTIFIED
R6G=LOCKED_BY_SEQUENCE
PRODUCTION=NOT_AUTHORIZED
```

**Aucune fusion, aucune certification formelle, aucune création d'opérateur indépendant, aucun changement de politique native par R7.**
