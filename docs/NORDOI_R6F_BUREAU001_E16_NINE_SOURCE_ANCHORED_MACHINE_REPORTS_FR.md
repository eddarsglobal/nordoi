# NORDOI G0.3 — R6F / BUREAU-001 / E16
## Neuf rapports machines fondés sur des sources et des journaux réels — rapport contradictoire

**Date :** 20 novembre 2026. **Direction :** Noureddine Eddars. Le travail technique, les tests, la documentation et la préparation des formalités relèvent de l'IA et des outils automatisés. **Aucun testeur humain, aucune obligation de tests manuels pour le fondateur.**

### Préalable C20

Parent E15 au SHA exact `8af265158bfbeb9c1406d2c310d505281dc98180`, [PR #148 DRAFT](https://github.com/eddarsglobal/nordoi/pull/148), sans fusion. [CI E15 spécialisée 4/4 SUCCESS](https://github.com/eddarsglobal/nordoi/actions/runs/44716501372), [Rust général 5/5 SUCCESS](https://github.com/eddarsglobal/nordoi/actions/runs/44716501419) et [régression E8 4/4 SUCCESS](https://github.com/eddarsglobal/nordoi/actions/runs/44716501426). Le rapport final E15 est publié dans la [PR #148](https://github.com/eddarsglobal/nordoi/pull/148#issuecomment-7036849099).

**Objet :** produire neuf **rapports de vérification machine réellement exécutée**, liés au même mandat BUREAU-001 et aux preuves exactes E14/E15, sans changer frauduleusement les neuf rapports formels `NOT_SUBMITTED` du handoff.

### Réalisations techniques prévues pour E16

La CI E16 télécharge cinq archives GitHub préexistantes avec permission `actions:read` uniquement : les **quatre artefacts E14** ([run #44713831052](https://github.com/eddarsglobal/nordoi/actions/runs/44713831052), contenant 228 journaux bruts) et **l'artefact E15 Mac Intel** ([run #44716501372](https://github.com/eddarsglobal/nordoi/actions/runs/44716501372), résultat JSON SHA256 `fc457cce0ad9004d66467f85be97376549c653f464cfa03427457646c97ca576`). L'auditeur refuse toute archive absente, altérée ou incohérente au HEAD source.

Chaque fonction est réellement lancée par l'orchestrateur comme **un processus OS distinct**. Chaque processus recalcule la preuve des 228 journaux via le vérificateur E15, puis produit un JSON canonique contenant : fonction, PID observé lors du lancement, ID de la tâche dans le handoff, fichiers historiques spécifiques réellement vérifiés, SHA de source réel, identités blobs Git, 4 compteurs E14 par fichier, SHA des journaux, limites et statut d'autorité. Un manifeste final attache les neuf empreintes de rapports ainsi qu'une matrice de **sept contradictions**.

Les PIDs prouvent neuf exécutions machine distinctes dans ce laboratoire, **pas neuf agents IA autonomes indépendants**, ni neuf institutions. Aucun rapport ne devient une attestation signée extérieure du seul fait de la présence d'un rôle ou d'un hash.

### Neuf fonctions et véritables pièces

| Fonction | Revue machine ancrée aux sources | Limite essentielle |
|---|---|---|
| ATLAS | Fichiers natifs Rust R4A, R4V ; architecture et admission simulée par laboratoire | Absence d'audit complet du compilateur certifié |
| FERRUM | R4M XPC/CDHash, R4F Linux, R4G macOS ; résultats réels par OS | Six cas Mac natifs ne sont pas des garanties Windows/Linux |
| BOUNDARY | R4E, R4K et R6D ; tests de frontières et domaines | Une séparation de PID/UID ne garantit pas des administrations distinctes |
| CIPHER | R5X, R6A, R6E ; vrais journaux de tests SSHSIG et provenance | Signatures locales ≠ témoin indépendant |
| LEDGER | R5Q et R5Z ; preuve de restauration/rejeu et intégrité | Antirollback global non attesté |
| CAPSTONE | R4V et R5F ; refus d'admission | Douze conditions externes toujours non satisfaites |
| ADVERSARY | R5Q et R6C ; rollback cohérent et faux quorum | Les cas négatifs ne prouvent pas l'absence de toute attaque |
| VERITY | R4M et R6E ; exactitude de tests, OS et SKIP | Les rapports CI ne sont pas des certificats externes |
| GUARDIAN | R6E et R4Q ; séquence et refus de quorum | R6F non certifié ; R6G verrouillé |

**Chaque lien de source ci-dessus** est contrôlé dans les vrais fichiers `tests/` et `research/` du manifeste des 99 blobs R1. Le contrat JSON E16 identifie les chemins complets.

### Sept contradictions et limites explicitement suivies

1. **Portabilité :** Mac Intel et Apple Silicon comptent chacun 3 SKIP, Linux 19, Windows 226. Aucun SKIP n'est assimilé à un PASS natif.
2. **Identité des processus E14 :** les 228 journaux sont réels, mais E14 n'a pas conservé un PID individuel pour chaque cible. E16 n'invente pas rétrospectivement ces PID.
3. **SSHSIG local :** les signatures cryptographiquement valides sous même administration ne prouvent pas un dépositaire externe.
4. **GitHub :** quatre runners et des manifestes SHA ne sont pas quatre administrateurs réellement indépendants.
5. **Bureau :** neuf fonctions machines de vérification ne constituent pas neuf rapports indépendants formellement signés.
6. **Inventaire versus exécution :** E13 a prouvé les 99 sources, E14 leur réexécution, E15 a réconcilié les 228 journaux ; chaque niveau a sa propre portée.
7. **Admission native :** dépositaire, racine propriétaire OOB, quorum et registres externes monotones toujours absents ; 12 verrous maintenus.

### Essais adversariaux E16

24 nouveaux tests exigent les vrais artefacts téléchargés. Ils reproduisent des altérations sur **copies des rapports effectivement produits** : modification d'un octet, modification cohérente d'un rapport et recalcul de son SHA dans le manifeste, suppression d'un rôle, falsification de la matrice. La validation compare les rapports non seulement à leur manifeste mais également aux vrais octets sources et aux journaux antérieurs, afin de refuser une falsification auto-cohérente.

### Ce qui reste à accomplir dans R6F

Le handoff `governance/NORDOI_BUREAU_R6F_HANDOFF_v1.json` demeure formellement **`ASSIGNED_REVIEWS_NOT_SUBMITTED`**. Les comptes rendus machines E16 sont des **preuves préparatoires source-ancrées**, pas une substitution à un véritable processus indépendant ni à une décision d'autorité. Une décision formelle ne peut être proclamée que si ses critères sont réellement satisfaits, sans créer des signatures fictives.

Aucun changement du noyau Rust, du dépôt principal, des sources historiques ou des versions antérieures ; aucun droit production. CI spécialisée prévue quatre OS, plus Rust général et régressions. Avant fin de CI au HEAD exact, le verdict E16 reste `IN_PROGRESS_UNQUALIFIED`.

```ini
E16=IN_PROGRESS_UNQUALIFIED_UNTIL_EXACT_HEAD_CI
E16_NINE_ACTUAL_MACHINE_REPORTS=EXPECTED
E16_PRIOR_ACTUAL_E14_LOGS=228_REQUIRED
E16_E15_REAL_ANCHOR=SHA256_PIN_REQUIRED
E16_CONTRADICTION_MATRIX=7_SOURCE_ANCHORED
FORMAL_INDEPENDENT_REVIEWS=0_NOT_SUBMITTED
HUMAN_TESTERS=0_PERMANENTLY_FORBIDDEN
FOUNDER_MANUAL_TEST_DUTY=NONE
EXTERNAL_WITNESSES=0
R6F=IN_PROGRESS_UNCERTIFIED
R6G=LOCKED_BY_SEQUENCE
NATIVE_ADMISSION_GATES=12_UNMET
PRODUCTION=NOT_AUTHORIZED
MERGE=NOT_AUTHORIZED
```
