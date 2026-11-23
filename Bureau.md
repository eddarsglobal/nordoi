# NORDOI — Bureau des neuf fonctions spécialisées

## Mission, nature et autorité

Le Bureau est une **organisation de fonctions d'expertise par agents IA** destinée à préparer, vérifier et remettre en cause le développement de NORDOI. Les neuf fonctions ne sont pas neuf personnes physiques, opérateurs indépendants, identités cryptographiques certifiées, ni neuf processus IA effectivement lancés par le seul fait de ce document. L'exécution effective d'une revue doit être appuyée par un rapport de travail et par des preuves contrôlables.

Le Bureau contrôle le code, les preuves, la sécurité, la cohérence entre versions et les tests, approfondit les cas négatifs et peut demander des corrections **dans le périmètre de développement déjà autorisé**. Il n'a aucun pouvoir implicite de fusionner une PR, d'installer ou d'activer une capacité privilégiée, d'enrôler une autorité externe, de déroger aux lois, de certifier des signatures qu'il contrôle lui-même ou d'autoriser la production.

La Constitution [C3, C8, C14, C20](CONSTITUTION.md), la loi principale, le Future-Native Gate et les douze conditions de garde native prévalent. Les fonctions sont des **axes de revue** : ne jamais présenter une synthèse provenant d'une seule exécution du modèle comme neuf audits indépendants.

## Sièges permanents et devoir de contestation

| Siège | Domaine | Examens contradictoires obligatoires |
|---|---|---|
| **ATLAS** | Rust / architecture | ABI et sémantique NAIR/NAM, changements d'API, portabilité, absence de dette structurelle |
| **FERRUM** | Sécurité OS native | macOS XPC/CDHash/entitlements, frontières Linux et Windows, signatures de code et contexte natif |
| **BOUNDARY** | Isolation des processus | PID, UID/EUID, descripteurs, permissions, sandbox, fuite d'autorité et confused deputy |
| **CIPHER** | Cryptographie | OpenSSH SSHSIG, Ed25519, clé racine, empreintes, namespace, rejeu, révocation, quorum |
| **LEDGER** | Provenance et conservation | sources exactes, SHA Git, chaîne des reçus, chronologie, rejet des états restaurés |
| **CAPSTONE** | Contrats d'effets et de capacités | permission explicite, refus par défaut, absence d'admission native implicite |
| **ADVERSARY** | Recherche de failles | tests négatifs, injection, falsification, dégradation, fausses preuves et conflits |
| **VERITY** | Tests et livraisons | matrices de CI, portée des réussites, Mac Intel réel, Rustfmt, Clippy, traces reproductibles |
| **GUARDIAN** | RoadMap et règles | séquence, constitution, preuve d'achèvement et transitions autorisées |

Chaque siège doit produire son `source_sha`, les commandes réellement exécutées, les tests et limites vérifiées, les erreurs et conflits, les corrections proposées, les dépendances et le verdict, ou bien indiquer explicitement **NOT_EXECUTED**. **Une attribution n'est pas un compte rendu et une signature locale n'est pas une signature d'une autorité indépendante.**

## Protocole transversal

1. **Inventaire / exactitude** : confirmer les commits source, fichiers et journaux réellement disponibles. Aucune reconstruction silencieuse des preuves manquantes.
2. **Revue adversariale croisée** : confronter Rust/Python, macOS/Linux/Windows, données de preuve et CI. Chercher les contradictions, les contournements et les cas où une réussite locale est interprétée abusivement comme preuve externe.
3. **Correction contrôlée** : résoudre sans demande répétée les bugs reproductibles dans le périmètre déjà autorisé ; produire les tests de régression et relancer les contrôles ciblés. Aucune décision privilégiée sans autorisation distincte.
4. **Décision qualifiée** : récapituler les neuf rapports, séparer preuves observées et affirmations non vérifiées, publier les limitations et recommander un statut. Une fonction ne peut pas voter au nom d'un autre contrôle non exécuté.
5. **RoadMapGuardian** : machine de vérification déterministe, sans dépendance à une inférence IA pour l'exécution ordinaire, qui bloque les promotions non documentées, les faux certificats et toute disparition des douze conditions natives.
6. **Maintien de la traçabilité** : toutes les preuves et les décisions sont ancrées à un commit exact. Lorsqu'une PR change, vérifier de nouveau la CI et les empreintes.

### Barrières de certification

- **TECHNICAL_VALIDATED_RESEARCH_ONLY** : tests et contrôle technique documentés ; ne signifie ni `CERTIFIED_RESEARCH` ni admission.
- **CERTIFIED_RESEARCH** : exige un véritable audit du Bureau, les rapports de toutes les fonctions, les preuves de tests et une décision documentée. Pas de vote ou signature d'agent imaginaire.
- **NATIVE_ADMISSION / PRODUCTION** : requiert en outre les douze preuves d'autorité externe réelle, une racine propriétaire vérifiée hors bande, un opérateur et un témoin indépendants, une chronologie monotone protégée, les décisions et autorisations nécessaires. Aucune certification technique de laboratoire ne suffit.

Une revue de neuf fonctions sous un même modèle ou administrateur est **un processus d'analyse**, et non un quorum cryptographique de neuf opérateurs indépendants. La prévention de la confusion « signature ≠ autorité » est un devoir explicite de CIPHER, LEDGER, CAPSTONE et GUARDIAN.

## Affectation active : G0.3 R6F / BUREAU-001

**Décision d'organisation : mission assignée, aucune revue déclarée terminée.** Le Bureau prend la responsabilité de la suite du **développement de recherche de NORDOI** à travers un audit intégral R4A–R6E, l'inventaire des conflits et un contrat de réception des preuves externes en vue de R6G.

Le mandat est structuré par `governance/NORDOI_BUREAU_R6F_HANDOFF_v1.json`, avec neuf sous-missions `NOT_SUBMITTED`. Pour chaque sous-mission, conserver la différence entre travail réellement exécuté et travail assigné. Le jalon R6G reste `LOCKED_BY_SEQUENCE` tant que R6F n'a pas satisfait ses critères et que la véritable preuve externe manque.

R6F doit systématiquement vérifier l'absence de changement des versions certifiées, la réussite du contrôle CI C20, le test natif R4M XPC/CDHash six en série, les expériences réelles SSHSIG R6A–R6E, et le statut exact des douze conditions externes. Le Bureau ne doit pas transformer les 1 039 tests R6E et les 188 empreintes Git rapportés sur Mac Intel en attestation de confiance extérieure.

## Exécution reproductible de la garde

```bash
python3 scripts/roadmap_guardian.py --audit
python3 -m unittest discover -s tests -p 'test_roadmap_guardian.py' -v
```

La garde est **lecture seule, sans réseau, sans inférence et sans droit de modifier l'état du dépôt**. Ses contrôles prouvent la cohérence formelle des registres et documents qu'elle connaît ; elle ne remplace pas l'audit technique ni une autorité extérieure.

## Verrous définitifs du candidat de gouvernance

```ini
BUREAU_ROLE_COUNT=9
BUREAU_R6F_REVIEWS=9_NOT_SUBMITTED
R6F=IN_PROGRESS_UNCERTIFIED
R6G=LOCKED_BY_SEQUENCE
EXTERNAL_GUARDIAN=NOT_ENROLLED
OWNER_OOB_ROOT=NOT_AUTHENTICATED
GLOBAL_MONOTONIC_ANTIROLLBACK=NOT_VERIFIED
NATIVE_ADMISSION_GATES=12_UNMET
PRODUCTION=NOT_AUTHORIZED
```


## Complément GOV-FIX1 — preuves APP et véritables revues

Une fonction ATLAS, FERRUM, BOUNDARY, CIPHER, LEDGER, CAPSTONE, ADVERSARY, VERITY ou GUARDIAN n'est réputée exécutée que si ses opérations et ses preuves (source exacte, commandes effectivement lancées, journaux, contrôles négatifs et conclusion bornée) sont observables. De simples intitulés dans un commentaire ne constituent ni des rapports de neuf agents ni des certifications indépendantes.

La branche applicative APP-016 à APP-021 est enregistrée dans governance/NORDOI_APP_RESEARCH_TRACK_V1.json et référencée par governance/NORDOI_BUREAU_PLAN_v1.json. Ses 78 contrôles GitHub historiques vérifiés au commit de chaque jalon fondent seulement TECHNICAL_VALIDATED_RESEARCH_ONLY, et pas CERTIFIED_RESEARCH. Les commentaires de synthèse précédemment postés par le coordinateur ne sont pas des signatures du Bureau : FORMAL_BUREAU_CERTIFICATION=NOT_CERTIFIED.

Le workflow GOV-FIX1 réalise neuf contrôles automatisés spécialisés avec traces au SHA courant. Ces rapports de MACHINE peuvent être réellement exécutés par CI, mais ne constituent ni neuf agents IA indépendants ni quorum ni preuve de dépositaire externe. Les neuf rapports historiques R6F demeurent explicitement NOT_SUBMITTED jusqu'à exécution et pièces propres.

RoadMapGuardian doit bloquer suppression, désordre et fausse promotion de la chaîne APP, changement de parent/SHA, faux compteur de revues exécutées et disparition des douze interdictions natives. R6F reste IN_PROGRESS_UNCERTIFIED, R6G LOCKED_BY_SEQUENCE, APP-022 LOCKED_BY_GOVERNANCE_REVIEW. Aucun test humain n'est nécessaire, mais aucune autorité indépendante n'est inventée.
