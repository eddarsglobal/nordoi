# NORDOI G0.3 R6F / BUREAU-001 E10
## Dossier de neuf contrôles machines réellement exécutés — IA uniquement

**Direction de projet :** Noureddine Eddars. **Ingénierie, vérification, rapports et formalités techniques :** IA et outils automatisés. Zéro testeur humain. Le fondateur n'a pas de devoir de test manuel.

### Prérequis C20
E9 `c789f02c637d9d8215772a67ca2e85cef35e64af` : [4/4 CI spécialisée](https://github.com/eddarsglobal/nordoi/actions/runs/44056262617) et [5/5 Test Gate Rust](https://github.com/eddarsglobal/nordoi/actions/runs/44056262413). E9 demeure DRAFT non fusionné. E10 ne modifie aucune version Rust précédente ni la RoadMap.

### Pourquoi E10 ?
La RoadMap exige des rapports de vérification, une matrice de contradictions, un registre de couverture et un contrat de réception de preuves d'autorité externe avant de pouvoir considérer la clôture R6F. Les recherches E6–E9 ont produit des contrôles négatifs, mais leurs résultats ne constituent pas à eux seuls neuf rapports d'exécution attribués et vérifiables. E10 réalise neuf **contrôles réels en processus machine distincts** et archive chaque constat avec SHA256, rôle, PID et commit exact.

Les neufs fonctions sont :
- **ATLAS** : manifeste et frontière Rust source ; **FERRUM** : code-source du test natif XPC/CDHash ; **BOUNDARY** : garde locale de correspondance événement/SQLite.
- **CIPHER** : parser canonique du paquet Ed25519 E6 (les vraies signatures sont qualifiées séparément via les régressions E9) ; **LEDGER** : cinq contradictions reliées à des fichiers réels ; **CAPSTONE** : blocage d'une tentative de promotion.
- **ADVERSARY** : différence entre indicateur de conflit et refus d'autorité ; **VERITY** : HEAD Git exact et worktree propre ; **GUARDIAN** : contrôle des neuf fonctions toujours `NOT_SUBMITTED` formellement et des douze conditions natives.

Chaque fonction génère **un fichier JSON différent**. L'orchestrateur lance les neuf contrôles en sous-processus OS réels, vérifie les empreintes sources, les PID et leurs états, puis produit un `MANIFEST.json`. Toute falsification d'un fichier ou perte d'un élément déclenche le refus du vérificateur. L'artefact est temporaire, hors dépôt Git. Aucun réseau extérieur ni clé privée du propriétaire n'est nécessaire pour ces neuf contrôles.

### Ce que les rapports prouvent et ne prouvent pas

Ils prouvent les **assertions automatiques exécutées**, pas l'identité d'auditeurs externes ou de neuf agents IA autonomes indépendants. Chaque rapport porte le statut `MACHINE_CHECK_PASS`, et `formal_independent_review=NOT_SUBMITTED`. Ce sont des tests de sources et de politique, **pas des revues externes d'une personne ou d'une institution**.

Le contrat machine comprend la matrice de contradictions suivante :

| Référence | Risque contrôlé | Limite |
|---|---|---|
| E6 | Signature Ed25519 valide sous même administration | Absence d'autorité tierce |
| E7 | Atomicité locale des nonces | Restauration du journal |
| E8 | Deux bases locales divergentes | Branches cachées non dénombrables |
| E9 | Événement SHA local décorrélé de SQLite | Correction uniquement dans laboratoire E9 |
| Bureau R6F | Confusion neuf fonctions machines / neuf attestations indépendantes | Rapports formels toujours non remis |

La réception automatique future de preuves extérieures exige des **domaines administratifs réellement séparés**, une racine propriétaire authentifiée OOB, un historique monotone protégé, des attestations signées réellement indépendantes, une vérification d'identité et un quorum prouvé. L'IA peut préparer et vérifier ces formalités **si l'infrastructure indépendante existe réellement** ; E10 ne la prétend pas présente.

### Tests attendus

24 nouveaux tests portables : vérification des neuf fonctions, source exacte, refus des fausses promotions, falsification de rapports et exécution de neuf vrais processus. CI Mac Intel, Apple Silicon, Linux, Windows ; régressions complètes E2–E9 ; contrôle R1 99 blobs ; RoadMapGuardian ; six vrais R4M XPC/CDHash en série sur macOS et Rust général. Les essais POSIX réels Ed25519 restent effectués dans la régression E9 (Windows : SKIP explicite). Ne jamais compter un test source E10 comme une signature Ed25519 indépendante.

Les contrôles attendus retournent `BLOCK` / code **2** lorsqu'ils réussissent ; erreur scientifique ou de source = code **3**. Aucun changement de droits, ni fusion, ni admission native.

```ini
E10=IN_PROGRESS_UNQUALIFIED_UNTIL_EXACT_HEAD_CI
MACHINE_FUNCTION_REPORTS=9_EXPECTED_NOT_YET_CI_QUALIFIED
ACTUAL_INDEPENDENT_SPECIALIST_REVIEWS=0
HUMAN_TESTERS=0_PERMANENTLY_FORBIDDEN
FOUNDER_MANUAL_TEST_DUTY=NONE
EXTERNAL_CUSTODIAN=NOT_ENROLLED
OWNER_OOB_ROOT=NOT_AUTHENTICATED
GLOBAL_MONOTONIC_ANTIROLLBACK=NOT_VERIFIED
R6F=IN_PROGRESS_UNCERTIFIED
R6G=LOCKED_BY_SEQUENCE
NATIVE_ADMISSION_GATES=12_UNMET
PRODUCTION=NOT_AUTHORIZED
```
