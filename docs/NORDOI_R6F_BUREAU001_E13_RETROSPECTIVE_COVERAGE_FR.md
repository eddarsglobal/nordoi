# NORDOI G0.3 — R6F / BUREAU-001 / E13
## Audit rétrospectif R4A–R6E : intégrité des sources ≠ exécution des tests

**Date :** 20 novembre 2026. **Direction :** Noureddine Eddars. L'IA et les outils automatisés réalisent le travail technique et préparent les formalités ; **aucun testeur humain, aucune charge de tests manuels pour le fondateur**.

### Pourquoi E13 est requis par la RoadMap

Le mandat R6F demande un registre rétrospectif des tests adversariaux R4A–R6E, des rapports de neuf fonctions et une matrice de contradictions entre sources, essais et conclusions. E10 a prouvé neuf contrôles machines réels ; E12 a qualifié le refus automatique des douze déclarations d'admission native. Ils n'établissent cependant **pas** que toutes les campagnes historiques R4A–R6E ont été de nouveau exécutées.

Préalable **C20** au parent exact E12 FIX2 `4ce07de0ee0478ee05794303bf528d071ef07488` :
- [CI spécialisée E12 #44080133471 — 4/4 SUCCESS](https://github.com/eddarsglobal/nordoi/actions/runs/44080133471).
- [Test Gate Rust #44080133579 — 5/5 SUCCESS](https://github.com/eddarsglobal/nordoi/actions/runs/44080133579) après **relance ciblée** du job macOS initialement annulé. Les journaux de la première annulation restent historiques et ne sont pas effacés.
- [Régression E8 #44080133473 — 4/4 SUCCESS](https://github.com/eddarsglobal/nordoi/actions/runs/44080133473).
- [PR #145](https://github.com/eddarsglobal/nordoi/pull/145) DRAFT et non fusionnée.

### Nouveauté scientifique vérifiable

La preuve R1 contient **99 identités de blobs Git**, réparties dans quatre ensembles : 24 fichiers de tests Rust R4A–R4X, 33 sources et 33 fichiers de tests Python R4Y–R6E, quatre snapshots R6B–R6E et cinq bases de gouvernance.

E13 relit les **99 sources réelles** de la copie de travail au HEAD Git exact, exécute `git ls-tree -rz HEAD` par sous-processus réel et confronte les identités du tree Git au manifeste ancien. Il vérifie aussi l'appariement des 33 sources/tests et les neuf fonctions du mandat R6F, toutes marquées `NOT_SUBMITTED` sur le plan des rapports indépendants formels.

L'expérience négative crée alors une **déclaration mensongère mais structurellement cohérente** : « les 24 fichiers Rust et les 33 tests Python ont tous été exécutés avec succès ; les neuf fonctions sont des auditeurs indépendants ; un dépositaire externe et les douze portes natives sont qualifiés ». Un agrégateur naïf pourrait compter ces 57 chemins comme 57 exécutions. Le nouveau contrôleur qualifie uniquement les **chemins effectivement observés et leurs empreintes**, rejette cette promotion vers une preuve d'exécution, ne crédite aucun certificat externe et maintient `BLOCK`.

E13 ne prétend **pas** que les 57 tests n'ont jamais été exécutés auparavant : il indique seulement que la vérification des 99 blobs n'est pas une preuve de **réexécution pendant E13**. Les preuves historiques issues de GitHub et les résultats du Terminal du propriétaire conservent leur attribution exacte, sans devenir rétroactivement des signatures indépendantes.

### Contrôle des neuf fonctions du Bureau

| Fonction | Vérification/réserve dans E13 |
|---|---|
| ATLAS | 24 fichiers natifs Rust source-ancrés ; pas 24 tests runtime E13 |
| FERRUM | R4M natif six tests réexécutés dans la CI E13 macOS, à qualifier sur le HEAD exact |
| BOUNDARY | Les frontières processus/UID historiques exigent des logs propres, pas seulement des sources |
| CIPHER | SSHSIG local éprouvé par régressions E6/E12 ; aucune racine extérieure créée |
| LEDGER | 99 objets du Git tree, manifestes R1, quatre snapshots et attribution des résultats |
| CAPSTONE | Une déclaration `PASS` sans vrai reçu d'exécution ne déverrouille aucune capacité |
| ADVERSARY | Suppression d'un chemin Rust, doublon Python, faux test injecté, faux quorum |
| VERITY | HEAD exact, CI spécialisée, Rust général, distinction des SKIP et jobs annulés |
| GUARDIAN | C3/C8/C14/C20, neuf rapports formels toujours `NOT_SUBMITTED` et douze portes natives verrouillées |

Cette grille ne constitue pas neuf rapports d'experts indépendants effectivement signés.

### Livrables et vérifications exigés

Cinq fichiers E13, exclusivement additifs : contrat, vérificateur de provenance, 24 tests négatifs, ce rapport, CI GitHub multi-OS. La CI conserve les régressions E2–E12, les tests natifs macOS R4M en série, les 99 pins R1, RoadMapGuardian et un Test Gate Rust général séparé. Aucune suppression de test ni dérogation.

Le programme E13 retourne code `2` + `BLOCK` quand l'expérience de classification réussit et code `3` lorsque la preuve ou la source est incorrecte.

**Limite :** la comparaison du tree Git est une preuve de cohérence de la copie git, pas une attestation externe du fournisseur GitHub, une revue formelle indépendante, ni une autorisation d'admission.

```ini
E13=IN_PROGRESS_UNQUALIFIED_UNTIL_EXACT_HEAD_CI
HISTORICAL_GIT_BLOBS=99_SOURCE_VERIFICATION_EXPECTED
HISTORICAL_RUST_FILES=24_SOURCE_ONLY_NOT_RUNTIME_ATTESTED_BY_E13
HISTORICAL_PYTHON_TEST_PAIRS=33_SOURCE_ONLY_NOT_RUNTIME_ATTESTED_BY_E13
HUMAN_TESTERS=0_PERMANENTLY_FORBIDDEN
FORMAL_INDEPENDENT_BUREAU_REVIEWS=0
EXTERNAL_TRUST_ROOTS=0
R6F=IN_PROGRESS_UNCERTIFIED
R6G=LOCKED_BY_SEQUENCE
NATIVE_ADMISSION_GATES=12_UNMET
PRODUCTION=NOT_AUTHORIZED
MERGE=NOT_AUTHORIZED
```
