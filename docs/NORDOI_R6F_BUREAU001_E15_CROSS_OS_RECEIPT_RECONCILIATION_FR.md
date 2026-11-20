# NORDOI G0.3 — R6F / BUREAU-001 — E15
## Réconciliation contradictoire des quatre archives réelles de tests E14

**Date :** 20 novembre 2026. **Fondateur directeur :** Noureddine Eddars. **Développement, tests, audit et formalités techniques :** IA et automatisation réelle, zéro testeur humain et aucune charge de tests manuels au fondateur.

### RoadMap et séquence C20

E14 est techniquement accepté **pour la recherche seulement**, au commit Git exact `deb34302acdbe18563ab06eb10d852865e3b3c6a`, [PR #147 DRAFT](https://github.com/eddarsglobal/nordoi/pull/147) sans fusion :
- [E14 CI spécialisée 4/4 SUCCESS](https://github.com/eddarsglobal/nordoi/actions/runs/44713831052).
- [Rust Test Gate 5/5 SUCCESS](https://github.com/eddarsglobal/nordoi/actions/runs/44713831212).
- [E8 régression 4/4 SUCCESS](https://github.com/eddarsglobal/nordoi/actions/runs/44713830975).

E14 a réellement lancé **57 cibles historiques par système** (24 Rust + 33 Python), et produit **228 journaux bruts** au total, avec résultat par cas et manifestes. E15 vérifie **les vrais reçus de ces mêmes exécutions E14 archivées par GitHub** ; E15 ne prétend pas avoir réexécuté 228 tests supplémentaires, ni avoir enregistré les PID individuels que le manifeste E14 ne contient pas.

### Provenance figée des quatre archives (source de vérité pour E15)

| Système | Identifiant artefact GitHub | Cas PASS | Cas SKIP | SHA256 du manifeste E14 |
|---|---:|---:|---:|---|
| Mac Intel | 14618857785 | 1 039 | 3 | `5eb067fc94695dc1a99747afedbcfb95ef6ac615e8de7d50d9198f6f14d606c4` |
| Apple Silicon | 14618821978 | 1 039 | 3 | `b84511f0065f440182c4e1f46944e635588691761b29c78f1f86bbeab7fc39c7` |
| Ubuntu Linux | 14617774638 | 881 | 19 | `cb5ce82242c912edc037c3e9678af2f54a2fb4659fbb9bb59f51c44514211977` |
| Windows | 14617984265 | 670 | 226 | `6ee300dd620ff8e3fa7f8b005d12b378f0a46a5551c74e1c2f0539ccae53ddd2` |

Les 4 artefacts appartiennent au **run E14 #44713831052**. GitHub Actions E15 utilise un jeton en lecture seule `actions:read` pour les télécharger par **nom exact et run-id fixe**. Aucune clé cryptographique propriétaire ou accès production n'est requis.

### Contrat d'audit réel

Le vérificateur E15 confronte le SHA256 de chaque `MANIFEST.json` aux quatre empreintes ci-dessus, puis :
1. exige quatre dossiers différents, sans archives manquantes ou injectées ;
2. recalcule les SHA256 des **228 journaux bruts** et confronte chaque ligne du manifeste E14 aux octets authentiquement téléchargés ;
3. confronte **chaque source Git historique** aux 99 pins R1 et aux commandes réellement déclarées, dont les 24 `cargo test` et 33 commandes Python par OS ;
4. reparse les véritables résumés `unittest` et Cargo ; distingue `PASS`, `SKIP` et `FAIL` par fichier, sans compter les 226 SKIP de Windows comme réussites cryptographiques POSIX ;
5. vérifie les OS/architectures **déclarés par les runners E14**, sans les présenter comme des attestations de matériel tiers ;
6. rejette la permutation de deux plateformes, la modification d'un octet de log, le recalcul cohérent mais non épinglé du manifeste, l'absence d'un log ou une source hors de l'inventaire.

La sortie produit une synthèse machine vérifiable et une empreinte de son résultat. Ce mécanisme repose sur les fichiers et les identités enregistrés dans **le même dépôt GitHub**. Il ne démontre donc **ni indépendance administrative globale ni histoire monotone protégée à l'extérieur**. Les fonctions du Bureau demeurent des axes d'analyse machine, pas neuf certificats indépendants.

### Contrôle du Bureau

ATLAS contrôle les 24 chemins Rust ; FERRUM compare les 6 tests XPC/CDHash réels sur les deux macOS et les limites de Linux/Windows ; BOUNDARY vérifie la portée processus enregistrée et les PID non sauvegardés ; CIPHER différencie les validations SSHSIG locales des preuves externes ; LEDGER rapproche 228 journaux des SHA ; CAPSTONE maintient le refus natif ; ADVERSARY exécute les cinq manipulations de copie d'archives ; VERITY contrôle 4 CI E14 et ses 57 fichiers par OS ; GUARDIAN impose C3/C8/C14/C20 et le verrou R6G.

### Qualification attendue

24 tests E15 utilisant **obligatoirement les quatre téléchargements réels**, CI sur Mac Intel, Apple Silicon, Linux et Windows ; régressions source E13/E14 et RoadMapGuardian ; Test Gate Rust général et CI E8 séparés. En cas de rejet, les journaux sont conservés et le jalon reste non qualifié. Les erreurs de données ne sont ni ignorées, ni converties en succès.

```ini
G03_R6F_E15=IN_PROGRESS_UNQUALIFIED_UNTIL_EXACT_HEAD_CI
E15_FROZEN_E14_GITHUB_RUN=44713831052
E15_REAL_E14_ARTIFACTS=4_REQUIRED
E15_E14_RAW_LOGS=228_REQUIRED
E15_NEW_HISTORICAL_TEST_EXECUTIONS=0_NOT_CLAIMED
HUMAN_TESTERS=0_PERMANENTLY_FORBIDDEN
FORMAL_INDEPENDENT_REVIEWS=0
EXTERNAL_CUSTODIAN=NOT_ENROLLED
OWNER_OOB_ROOT=NOT_AUTHENTICATED
NATIVE_ADMISSION_GATES=12_UNMET
R6F=IN_PROGRESS_UNCERTIFIED
R6G=LOCKED_BY_SEQUENCE
PRODUCTION=NOT_AUTHORIZED
MERGE=NOT_AUTHORIZED
```
