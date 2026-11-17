# NORDOI G0.3 — État consolidé de recherche, RoadMap et Bureau
**Date :** 17 novembre 2026. **Source :** dépôt `eddarsglobal/nordoi`, RoadMap, Bureau et runs GitHub Actions observés. **Portée :** état technique vérifié, sans certification indépendante.

## Synthèse de direction
NORDOI est un projet Rust de recherche en sécurité/isolation et admission native sous contrôle strict. Des protections techniques et contre-exemples reproductibles ont été qualifiés dans la recherche G0.3. **NORDOI n'est pas autorisé pour la production.** Le nombre de jobs verts ne résout pas la question de l'autorité extérieure et n'autorise aucune transition native.

La branche principale GitHub reste au commit `277d9a1af9aeed4dbd4afb1306e2931bc4439cb6`, issu de l'intégration gouvernée antérieure de R3 ; les candidats R6F sont sur branches et PR DRAFT, non intégrés.

## Tableau de progression

| Échelon | Résultat contrôlable | Statut |
|---|---|---|
| R3 / branche principale | Code Rust intégré antérieurement, sans droit de production ; études de performance et tests documentés dans l'historique GitHub | Base existante, sans autorisation production |
| R4A–R5Z | Laboratoires de sécurité, code identity, provenance, rollback et garde | Recherche historique à revoir transversalement |
| R6E | RoadMap : 1039 tests réguliers PASS et 188/188 blobs Git d'après le journal Mac Intel propriétaire ; CI GitHub 9/9 | TECHNICAL_VALIDATED_RESEARCH_ONLY |
| R6F / BUREAU-001 R1–R9 | Recherches et audits négatifs source-anchored, sans preuve indépendante universelle | IN_PROGRESS_UNCERTIFIED |
| E6 / PR #139 | Véritable reçu local signé Ed25519 ; rejeu interprocessus et re-signature même administrateur ; CI 9/9 | PASS_WITH_CONTROLS_RESEARCH_ONLY |
| E7 / PR #140 | TOCTOU nonce ; SQLite BEGIN IMMEDIATE et PK : une consommation locale, une refusée ; rollback possible ; CI 9/9 | PASS_WITH_CONTROLS_RESEARCH_ONLY |
| E8 / PR #141 | Deux SQLite forkées : deux consommations locales du même nonce ; conflit détecté uniquement si deux branches fournies ; CI 9/9, sur Mac Intel 24/24 E8 et R4M 6/6 | TECHNICAL_VALIDATED_RESEARCH_ONLY |
| E9 / nouvelle branche | Contradiction entre événement local réécrit et vraie ligne SQLite ; tests et CI à qualifier | IN_PROGRESS_UNQUALIFIED |
| R6G | Réel dépositaire externe indépendant et historique monotone hors même administration | LOCKED_BY_SEQUENCE |

**E8 exact :** `bf4c0d66256ad3a871e44cae39808af5ea1418d4` ; [spécialisée 4/4](https://github.com/eddarsglobal/nordoi/actions/runs/44036977230) et [Rust 5/5](https://github.com/eddarsglobal/nordoi/actions/runs/44036977539). Les quatre systèmes sont Mac Intel, Apple Silicon, Ubuntu et Windows. Les six tests POSIX de l'étude E8 sont explicitement SKIP sous Windows et ne sont jamais présentés comme tests Windows réussis.

## Risques non levés
1. Administrateur unique pouvant signer avec une clé authentique sans être un tiers indépendant.
2. Rejeu interprocessus ou restauration d'historique ; atomicité d'une SQLite locale ne protège pas contre une restauration de toutes ses copies.
3. Divergence entre plusieurs copies de registre ; accès à une copie ne prouve pas l'existence ou l'absence d'autres copies.
4. Empreintes et événements locaux cohérents pouvant être réécrits par celui qui contrôle le stockage.
5. Absence d'un dépositaire externe enrôlé, d'une racine propriétaire OOB authentifiée, d'un registre monotone réellement protégé, d'une preuve de quorum/autorité indépendante.
6. Les **douze conditions d'admission native sont toujours non satisfaites** ; aucune capacité ni production n'est permise.

## Bureau et autorité
ATLAS, FERRUM, BOUNDARY, CIPHER, LEDGER, CAPSTONE, ADVERSARY, VERITY, GUARDIAN sont neuf fonctions d'expertise documentées. Leur existence dans les contrats n'est pas la preuve de neuf audits indépendants réellement exécutés. La RoadMap demande de vrais rapports avec preuves par fonction et une décision contradictoire formelle avant toute certification R6F ; les preuves et attestations indépendantes requises restent manquantes.

**Politique permanente du propriétaire : zéro humain testeur.** Les exécutions d'essais sont uniquement machines. Aucune invitation/recrutement pour test humain.

## Décision et jalon immédiatement suivant
La réussite CI E8 respecte le préalable C20 pour commencer E9 : investigation de fausses déclarations d'événements face au vrai contenu SQLite, sans répéter la simple course de nonces E7 ni la démonstration de deux registres E8. Exiger nouveaux tests négatifs réels, contrôle d'égalité au commit exact, régressions précédentes, six tests natifs Mac R4M en série, et résultats strictement BLOCK. Pas de fusion, runtime productif, enrôlement externe, révocation ni déploiement.

```ini
MAIN_BRANCH=277d9a1af9aeed4dbd4afb1306e2931bc4439cb6
E8_HEAD=bf4c0d66256ad3a871e44cae39808af5ea1418d4
E8_CI=9/9_SUCCESS
E9=IN_PROGRESS_UNQUALIFIED
HUMAN_TESTERS=0_FOREVER_FORBIDDEN
R6F=IN_PROGRESS_UNCERTIFIED
R6G=LOCKED_BY_SEQUENCE
INDEPENDENT_SPECIALIST_REVIEWS=0_CONFIRMED
EXTERNAL_CUSTODIAN=NOT_ENROLLED
OWNER_OOB_ROOT=NOT_AUTHENTICATED
GLOBAL_MONOTONIC_ANTIROLLBACK=NOT_VERIFIED
NATIVE_ADMISSION_GATES=12_UNMET
PRODUCTION=NOT_AUTHORIZED
```
