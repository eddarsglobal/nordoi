# NORDOI — G0.3 R6F / BUREAU-001 E2

## Directive propriétaire : aucun humain ne testera NORDOI

**Décision E2 : tous les tests de NORDOI sont exécutés par des logiciels et des machines ; aucun testeur humain n'est recruté, chargé d'exécuter une suite ou sollicité pour donner un verdict expérimental.**

Cette règle prend effet dans le périmètre de recherche de NORDOI. Le Bureau demeure constitué de neuf **fonctions d'analyse IA** et de suites automatiques ; il ne se transforme pas en neuf personnes physiques, ni en neuf signataires administrativement indépendants.

Une approbation **propriétaire** pour un changement privilégié peut demeurer obligatoire en vertu de la Constitution et de RoadMapGuardian : **autoriser une décision de gouvernance n'est pas effectuer un test**. E2 ne supprime donc pas les verrous existants.

### 1. Correction de trajectoire de E1

Le dossier E1 a préparé neuf mandats sources et un exemple d'invitation pour un tiers humain. **Ces invitations n'ont jamais été envoyées et ne doivent jamais servir à engager des humains pour tester NORDOI.** Elles restent conservées comme trace historique de la proposition abandonnée, mais E2 les déclare **supplantées** pour toute opération de test.

Aucun candidat humain, organisme de contrôle humain ou relecteur externe ne sera recherché dans le cadre d'E2. La documentation de contact E1 ne constitue pas une démarche active. Les données d'identité, signatures privées, certificats et autres secrets ne doivent pas être envoyés dans des issues GitHub.

### 2. Neuf fonctions d'analyse, neuf véritables exécutions machines

Le programme `scripts/bureau_001_e2_machine_only.py` fournit deux opérations distinctes :

- `--audit` : vérification source, sans lancement de test ; il vérifie le registre E1, les **neuf fichiers et empreintes Git** du registre R2, les **99 sources gelées R1**, la directive de non-intervention humaine et les douze conditions natives toujours bloquées.
- `--run --output /chemin/hors/depot` : exécute **sur macOS réel** neuf commandes de test préexistantes, capture stdout/stderr et les empreintes SHA-256 des journaux dans un dossier créé avec permissions `0700`, et écrit `E2_MACHINE_EVIDENCE.json` avec la plateforme, le commit Git, le nombre effectivement observé de tests, leurs codes de sortie et la distinction entre exécution machine et revue spécialisée indépendante. Aucune opération réseau ni clé externe n'est demandée par le lanceur.

| Fonction | Contrôle réellement exécuté par l'automate |
|---|---|
| ATLAS | Rust natif `g03_r4a_structured_native_lab` : 10 tests |
| FERRUM | Vrai `g03_r4m_pinned_code_identity_lab` sur macOS : 6 tests XPC/CDHash, en série |
| BOUNDARY | `test_g03_r6f_r4_fd_race.py` : 24 tests de race réelle entre processus |
| CIPHER | `test_g03_r6f_r6_external_custody_feasibility.py` : 24 tests OpenSSH/SSHSIG réels |
| LEDGER | `test_g03_r6f_r5_inplace_aba.py` : 24 tests de restauration et limite inter-sessions |
| CAPSTONE | `test_bureau_001_r8_closure_gate.py` : 24 tests de refus de clôture |
| ADVERSARY | `test_bureau_001_r9_final_evidence_decision.py` : 24 tests de faux certificats |
| VERITY | `cargo fmt --all -- --check` : vérification de formatage réellement exécutée |
| GUARDIAN | `test_roadmap_guardian.py` : 24 tests de refus natif/gouvernance |

**Ce ne sont pas neuf agents IA autonomes réellement lancés**, ni neuf relectures indépendantes. Ce sont neuf fonctions du Bureau **associées à des commandes machines exécutables et auditées**. Les rapports des vrais tests doivent être produits par CI ou une machine, jamais inventés à partir des rôles. Une future couche d'agents IA pourra ajouter des analyses distinctes et tracées ; E2 ne prétend pas en lancer.

Le fait que toutes les suites soient vertes ne démontrera ni une séparation administrative, ni un gardien OOB, ni une histoire externe monotone, ni une certification native. Une machine/macOS signée locale ne devient pas un acteur extérieur par son nom.

### 3. Qualification et conservation des preuves

La CI E2 doit lancer les 24 tests portables d'intégrité et de politique sur Mac Intel, Apple Silicon, Linux et Windows. Une tâche supplémentaire sur Mac Intel exécute **réellement** les neuf suites ci-dessus, avec journaux et hachages vérifiables ; les deux cas Rust natifs doivent vérifier **10 et 6 tests PASS** et **zéro test ignoré** ; les suites Python doivent compter effectivement **24 tests sans échec ni saut**. Les logs sont stockés hors du dépôt et peuvent être conservés dans les artefacts de CI.

Ne présenter le résultat **9/9 machine suites PASS** qu'après avoir lu le journal du runner et vérifié toutes les suites ; les tests unitaires de parsing ne sont pas cette preuve. Les Windows/Linux CI ne doivent pas prétendre avoir exécuté XPC/CDHash.

La CI GitHub et le Mac du propriétaire sont des équipements réels ; **la sécurité d'administration indépendante** reste non démontrée sous contrôle du même propriétaire. Les résultats doivent rester classés `MACHINE_TECHNICAL_TEST_EVIDENCE_ONLY`.

### 4. Conservation des sources historiques

`RoadMap.md` et `Bureau.md` figurent parmi les **99 blobs Git historiquement gelés par R1**. E2 ne réécrit pas ces fichiers ; la nouvelle directive est enregistrée dans `governance/NORDOI_R6F_BUREAU001_E2_NO_HUMAN_TESTING_POLICY.json` et cette note, sans altérer les anciennes assertions à leurs commits historiques.

**État de gouvernance maintenu :**

```ini
E2_HUMAN_TESTERS=0_FORBIDDEN
E2_HUMAN_TESTER_OUTREACH=FORBIDDEN
E2_MACHINE_ROLES=9_SUITE_MAPPINGS
E2_INDEPENDENT_SPECIALIST_REPORTS=0
E2_REAL_EXTERNAL_GUARDIAN=NOT_ENROLLED
E2_OWNER_OOB_ROOT=NOT_AUTHENTICATED
E2_GLOBAL_MONOTONIC_ANTIROLLBACK=NOT_VERIFIED
R6F=IN_PROGRESS_UNCERTIFIED
R6G=LOCKED_BY_SEQUENCE
NATIVE_ADMISSION_GATES=12_UNMET
PRODUCTION=NOT_AUTHORIZED
```

Cette directive de travail ne déverrouille pas R6G, n'ouvre pas la production et ne fusionne aucune PR.
