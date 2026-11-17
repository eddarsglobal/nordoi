# NORDOI G0.3 / R6F — BUREAU-001 E12
## Réception automatique des preuves : 12 conditions natives, zéro attestation inventée

**Date :** 17 novembre 2026. **Fondateur/directeur :** Noureddine Eddars. La conception, le développement, la vérification, les tests et la préparation des formalités techniques sont réalisés par IA et outils automatisés. **Aucun testeur humain** et aucun test manuel exigé du propriétaire.

### Objectif officiel RoadMap

La RoadMap G0.3 R6F requiert un **contrat de réception de preuves externes réelles** qui distingue les reçus cryptographiques, les attestations venant de la même administration et les preuves effectivement indépendantes, avant tout examen de R6G. E12 livre ce contrat et le vérificateur automatique **en refus par défaut**, pas une admission native.

Le prédécesseur E11 exact `96e56f954721eb96f3526e529a27b2456bd2e8ce` a été qualifié dans GitHub : [CI spécialisée 4/4](https://github.com/eddarsglobal/nordoi/actions/runs/44077394937), [Rust 5/5](https://github.com/eddarsglobal/nordoi/actions/runs/44077394930), [régression E8 4/4](https://github.com/eddarsglobal/nordoi/actions/runs/44077394817). La PR #144 est DRAFT non fusionnée. C20 permet de commencer E12, **pas de certifier R6F**.

### Douze portes exactes

Les douze identifiants sont repris **sans invention** du `scripts/roadmap_guardian.py` et leur état est verrouillé dans `governance/NORDOI_R6F_BUREAU001_E12_TWELVE_GATE_INTAKE.json`.

| Porte | Preuve extérieure réelle nécessaire |
|---|---|
| 01 · Owner root OOB | Racine propriétaire authentifiée par voie indépendante |
| 02 · Custodian identity | Identité attestée d'un dépositaire externe réel |
| 03 · Separate administration | Domaines d'administration effectivement séparés |
| 04 · Guardian key OOB | Clé de gardien épinglée hors du domaine déclarant |
| 05 · OS attestation | Attestation de système signée indépendamment |
| 06 · Fresh challenge/history | Défi frais signé et rattaché à l'historique exact |
| 07 · Independent host | Attestation machine/OS hors du domaine source |
| 08 · Monotonic epoch | Registre d'époques monotone protégé extérieurement |
| 09 · Revocation floor | Plancher de révocation antirollback protégé |
| 10 · Spent nonce floor | Registre global des nonces consommés protégé |
| 11 · Independent quorum | Quorum constitué de domaines réellement indépendants |
| 12 · Recovery/rotation | Audit indépendant du recouvrement et de la rotation |

**Toutes les douze restent `UNMET_INDEPENDENT_EXTERNAL_EVIDENCE`**. Aucun dépôt de clés secrètes, aucun certificat extérieur et aucune troisième administration ne sont inventés. Les simples champs JSON `claimed_* = true` ou les douze lignes `MET` sont considérés comme déclarations non fiables.

### Recherche adversariale vérifiable

L'attaque volontaire `unsafe_naive_self_promotion` montre comment un évaluateur incorrect pourrait accepter douze `MET` signifiés par le même administrateur. Le véritable contrôle `intake_untrusted_json` confirme la structure, les douze IDs exacts, l'absence de duplication, l'identité du commit et la limite de provenance, puis retourne **BLOCK**, 12 portes non satisfaites.

Sur Mac Intel, Apple Silicon et Linux : l'expérience génère un **vrai SSHSIG Ed25519 OpenSSH** avec une clé de laboratoire locale, vérifié dans un processus OS distinct par les contrôles E6 préexistants. Même si la signature est cryptographiquement valide et les douze états sont revendiqués `MET`, aucun témoin extérieur n'est authentifié et le refus est obligatoire. Windows exécute les **18 tests portables** ; les six cas POSIX sont explicitement SKIP, jamais déclarés réussis sous Windows.

Le contrôleur n'est pas une plateforme de preuve indépendante. Il ne doit pas pouvoir ouvrir un droit natif en modifiant son propre JSON. Une future attestation réellement indépendante nécessiterait une infrastructure externe et une racine de confiance vérifiable, configurées séparément sous les permissions applicables. La présence de fonctions du Bureau ne remplace pas ces preuves.

### Contrôles du Bureau

ATLAS vérifie l'architecture du contrat ; FERRUM la preuve OS native et les limitations Windows ; BOUNDARY les domaines distincts ; CIPHER distingue validité SSHSIG et contrôle externe ; LEDGER contrôle antirollback/provenance ; CAPSTONE impose 12 refus ; ADVERSARY soumet les déclarations forgées ; VERITY vérifie HEAD exact, suites et journaux ; GUARDIAN maintient C3, C8, C14, C20 et la séquence.

Ces fonctions sont des **axes d'analyse exécutés par machines**, et non neuf identités de personnes, neuf certificats ou neuf agents indépendants formellement attestés.

### Qualification exigée

24 tests E12 + exercice cryptographique POSIX réel, régressions E2–E11, RoadMapGuardian, R1 audit des sources épinglées, 6 tests R4M XPC/CDHash macOS natifs en série, et CI Rust générale. Exact HEAD du pull request, sans substituer un commit de fusion synthétique. Les rapports machines sont conservés comme artefacts Github. Tous les contrôles positifs de laboratoire retournent `BLOCK` / code 2 ; toute erreur retourne code 3.

**Statut pré-CI : `E12=IN_PROGRESS_UNQUALIFIED`.**

```ini
PROJECT=NORDOI_G03_AI_ENGINEERED_LANGUAGE
E12_NATIVE_GATE_INTAKE=12_UNMET_EXTERNAL_EVIDENCE
E12_TRUE_LOCAL_ED25519=CI_PENDING
E12_EXTERNALLY_VERIFIED_AUTHORITY=0
HUMAN_TESTERS=0_PERMANENTLY_FORBIDDEN
OWNER_MANUAL_TEST_DUTY=NONE
FORMAL_INDEPENDENT_BUREAU_REVIEWS=0
R6F=IN_PROGRESS_UNCERTIFIED
R6G=LOCKED_BY_SEQUENCE
NATIVE_ADMISSION_GATES=12_UNMET
PRODUCTION=NOT_AUTHORIZED
MERGE=NOT_AUTHORIZED
```
