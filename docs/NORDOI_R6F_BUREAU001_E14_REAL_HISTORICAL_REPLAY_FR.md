# NORDOI G0.3 — R6F / BUREAU-001 — E14
## Réexécution réellement automatisée des 57 cibles historiques R4A–R6E

**Date :** 20 novembre 2026. **Direction du fondateur :** Noureddine Eddars. **Exécution technique, contrôles, rapports et formalités :** IA et outils automatisés. **Zéro testeur humain ; aucune opération manuelle de test imposée au fondateur.**

### Préalable C20 vérifié

E13 : commit exact `4da31db4363931606530f130b4d5330510a88184`. [CI spécialisée E13 4/4 SUCCESS](https://github.com/eddarsglobal/nordoi/actions/runs/44710468140), [Rust Test Gate 5/5 SUCCESS](https://github.com/eddarsglobal/nordoi/actions/runs/44710468035), [régression E8 4/4 SUCCESS](https://github.com/eddarsglobal/nordoi/actions/runs/44710468200). La [PR #146](https://github.com/eddarsglobal/nordoi/pull/146) demeure DRAFT, non fusionnée.

E13 a vérifié **99 sources historiques** contre l'arbre Git réel, dont 24 fichiers de tests Rust R4A–R4X, 33 sources Python et 33 fichiers de tests Python R4Y–R6E, quatre snapshots et cinq dossiers de gouvernance. E13 n'a **pas** présenté ces 57 fichiers de tests comme 57 exécutions effectivement réussies.

### Nouvelle preuve exigée pour E14

E14 exécute les **57 cibles de tests historiques effectivement présentes** en lançant un nouveau processus OS pour chaque fichier, et produit un reçu de résultat indépendant **par fichier**, sur **chaque plateforme GitHub ciblée** :

- **24 tests d'intégration Rust** via `cargo test --offline --release --test <nom_fichier> -- --nocapture --test-threads=1`.
- **33 suites Python** via `python -B -m unittest discover -s research -p <nom_exact.py> -v`.

Les tests et leur code historique ne sont pas modifiés. Les cibles sont dérivées exclusivement du manifeste figé R1, déjà confronté au Git tree par E13. Les commandes ne sont jamais construites à partir de déclarations d'un opérateur non fiable.

Pour **chaque fichier**, l'orchestrateur écrit son vrai journal brut `.log` et enregistre : chemin de source, identité blob Git, commande exécutée, plateforme, code retour réel, nombre de cas rencontrés, réussis, ignorés et échoués, empreinte SHA256 du journal, durée, et statut. Un `MANIFEST.json` exact-HEAD lie les 57 éléments et leurs fichiers. La vérification du manifeste contrôle qu'aucun test absent, doublon, ignoré ou en échec n'est déguisé en réussite.

Les tests spécifiques à macOS (R4M XPC/CDHash), Linux (Landlock), POSIX OpenSSH et Windows conservent leurs frontières natives réelles. Une suite qui ne réalise que des tests de refus portables sur une plateforme non compatible ne doit **jamais** être présentée comme ayant exercé les capacités natives de l'autre système. Pour R4M sur macOS, E14 exige au moins les **six cas natifs réels**.

Les 24 tests nouveaux E14 vérifient séparément : identité C20/source, commandes figées, unicité des 57 cibles, interprétation véridique des compteurs, refus des suites vides, refus des codes d'échec, expérience historique R4Y réelle en sous-processus et détection d'altération d'un journal. **Ces tests unitaires ne remplacent pas l'exercice complet des 57 processus**, exécuté séparément dans chaque CI.

### Bureau et garde

- **ATLAS** : 24 fichiers Rust historiques exacts ; **FERRUM** : macOS XPC/CDHash, Linux et Windows sans fausse équivalence des tests natifs.
- **BOUNDARY** : isolement des 57 processus tests et de leurs journaux ; **CIPHER** : véritables tests cryptographiques historiques (dans les systèmes qui les supportent), aucune attestation externe.
- **LEDGER** : identités Git, journaux bruts, empreintes de chaque reçu ; **CAPSTONE** : aucun résultat ne confère des capacités natives.
- **ADVERSARY** : résultats absents, dupliqués, ignorés, falsifiés ; **VERITY** : exhaustivité des 57 processus, quatre OS, absence de fausse promotion des SKIP ; **GUARDIAN** : C3/C8/C14/C20, les douze conditions externes restent non satisfaites.

Ces neuf perspectives de contrôle ne sont pas neuf opérateurs indépendants, et des résultats CI ne valent pas attestation d'un dépositaire extérieur.

### Règles de qualification

La CI spécialisée est conçue pour Mac Intel, Apple Silicon, Ubuntu et Windows. Chaque job doit réussir les **57 cibles** et archiver les journaux bruts et `MANIFEST.json`. Le Test Gate Rust général reste séparé, et les contrôles régressifs du Bureau et RoadMapGuardian sont obligatoires. Si une ancienne cible échoue, son journal et son statut doivent être conservés ; aucune dérogation par SKIP artificiel ni réduction silencieuse du nombre de cibles n'est autorisée. La prochaine décision ne peut être prise qu'au **HEAD PR exact** et après qualification de tous les jobs.

E14 ne prétend pas reproduire les mesures historiques du Mac personnel du fondateur : il apporte de **nouvelles exécutions GitHub documentées**. Les rapports de fonctions formels indépendants restent non remis ; le projet demeure une recherche.

```ini
E14_STATUS=IN_PROGRESS_UNQUALIFIED_UNTIL_EXACT_HEAD_CI
E14_GENUINE_HISTORICAL_FILE_TARGETS=57_EXPECTED
E14_RUST_FILE_TARGETS=24
E14_PYTHON_FILE_TARGETS=33
E14_EXECUTION_RECEIPTS=CI_PENDING
E14_SKIPS=REPORTED_SEPARATELY_NOT_PASSED
HUMAN_TESTERS=0_PERMANENTLY_FORBIDDEN
INDEPENDENT_SPECIALIST_REVIEWS=0
EXTERNAL_WITNESS_RECEIPTS=0
NATIVE_ADMISSION_GATES=12_UNMET
R6F=IN_PROGRESS_UNCERTIFIED
R6G=LOCKED_BY_SEQUENCE
PRODUCTION=NOT_AUTHORIZED
MERGE=NOT_AUTHORIZED
```
