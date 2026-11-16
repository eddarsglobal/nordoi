> **DIRECTIVE E2 PRIORITAIRE — TESTS HUMAINS INTERDITS.** Le propriétaire a interdit définitivement tout test humain de NORDOI. Les démarches de recrutement ou de remise de ce dossier à des personnes pour tester le projet sont **ANNULÉES, NON ENVOYÉES ET NON AUTORISÉES**. Le contenu E1 ci-dessous est archivé comme proposition historique et ne sert plus de mode opératoire. Pour les tests, appliquer [E2 — Exécutions machines uniquement](NORDOI_R6F_BUREAU001_E2_NO_HUMAN_MACHINE_ONLY_FR.md). Les neuf rôles sont des fonctions d'analyse, pas des personnes. Les verrous natifs restent non satisfaits.

# NORDOI G0.3 · R6F / BUREAU-001 — E1

## Dossier de remise aux neuf fonctions pour revues réellement exécutées

**Nature : neuf demandes de revue préparées, aucune demande envoyée, aucune revue réalisée ou certifiée par ce document.** La référence à examiner est le commit R9 exactement figé :

`af07acdf025ad086ad7eafd83a39ad8e822891b0`.

Source des rôles : `governance/NORDOI_BUREAU_R6F_HANDOFF_v1.json`. Source des fichiers, extraits et empreintes Git historiques : `governance/NORDOI_R6F_BUREAU001_R2_REVIEW_REGISTER.json`. Les neuf commandes de travail, références de source et questions contradictoires sont préparées dans `governance/NORDOI_R6F_BUREAU001_E1_INDEPENDENT_REVIEW_REQUESTS.json`.

### Distinctions obligatoires

**Une demande de revue ne constitue pas une revue.** Une analyse rédigée par le coordinateur, même divisée en neuf chapitres, ne vaut pas neuf rapports indépendamment exécutés. Une preuve de source Git identifie des octets, mais ne prouve ni l'identité du relecteur ni l'indépendance du domaine administratif. Les tests CI ne remplacent ni la vérification d'identité ni une autorisation réelle.

Aucune clé privée, jeton d'accès GitHub, adresse personnelle, contrat d'identité ou secret ne doit être envoyé dans une issue GitHub, un commentaire, une PR ou un rapport public. Si une signature externe, un contrat de consentement ou une copie d'identité est requis, la vérification doit être effectuée par un canal séparé et sécurisé, puis son **statut et sa référence non sensible** seulement consignés dans le registre.

### Procédure de revue indépendante — données à obtenir, pas des actions déjà faites

1. Sélectionner un **relecteur authentiquement distinct** du seul coordinateur, capable d'accéder au code autorisé et de procéder à une réelle évaluation. Obtenir son consentement et confirmer le périmètre.
2. Donner un accès limité en lecture à la référence git exacte et aux sources utiles ; ne jamais lui communiquer de clés privées de NORDOI.
3. Faire vérifier **sur son propre environnement** le commit source et les octets. Exemple pour un clone autorisé :

   ```bash
   git fetch origin af07acdf025ad086ad7eafd83a39ad8e822891b0
   git checkout --detach af07acdf025ad086ad7eafd83a39ad8e822891b0
   test "$(git rev-parse HEAD)" = "af07acdf025ad086ad7eafd83a39ad8e822891b0"
   git hash-object -- tests/g03_r4m_pinned_code_identity_lab.rs
   ```

   La dernière commande n'est qu'un exemple pour FERRUM ; **pour chaque rôle**, employer le chemin et l'empreinte attendue issus du registre E1, et vérifier la correspondance réelle. N'interpréter aucun hash comme une signature d'identité.
4. Réaliser la revue en suivant les questions de son rôle et en exécutant réellement les tests pertinents **sur des environnements isolés et autorisés**. Un test non exécutable doit être déclaré `NOT_EXECUTED`, et non `PASS`.
5. Produire un rapport traçable, avec les commandes et journaux **réellement obtenus** ; chaque fichier de preuve peut être identifié par SHA-256, mais un hachage ne remplace pas l'authentification de son auteur.
6. Vérifier la provenance, le consentement et, si une attestation signée est utilisée, la véritable relation entre clé, signataire et domaine administratif. Le propriétaire conserve une décision distincte.
7. **Ne changer aucun verrou** à la réception d'une seule revue : `R6F=IN_PROGRESS_UNCERTIFIED` tant que la décision formelle n'a pas été prise ; `R6G=LOCKED_BY_SEQUENCE` et les douze portes `UNMET`.

### Modèle de compte rendu pour un relecteur réellement intervenu

```markdown
# NORDOI R6F — Revue spécialisée réelle
Rôle Bureau : <ATLAS | FERRUM | BOUNDARY | CIPHER | LEDGER | CAPSTONE | ADVERSARY | VERITY | GUARDIAN>
Exécutant / organisation : <identité vérifiable, non sensible>
Consentement et mandat de revue : <référence externe vérifiable>
Séparation du domaine administratif : <comment vérifiée, ou NON_VÉRIFIÉE>
Commit évalué : af07acdf025ad086ad7eafd83a39ad8e822891b0
Fichier-source et Git blob SHA1 : <chemin et hash réellement calculé>
Machine, système et versions : <constat réel>
Date et lieu d'exécution des tests : <information de contexte non sensible>
Commandes réellement exécutées : <liste>
Résultats : <PASS/FAIL/SKIP par commande>
Hash SHA-256 de chaque journal : <constat réel>
Contre-exemples / failles : <preuve et méthode de reproduction>
Restrictions / données non observées : <faits>
Verdict spécialiste : <PASS | REMEDIATE | BLOCK | NOT_EXECUTED>
Identité cryptographique et canal de vérification : <méthode réellement vérifiée ; ou NONE>
Attestation/signature indépendante : <référence réelle ; ou NONE>
Décision d'admission ou de production : AUCUNE (en dehors du pouvoir de ce rapport)
```

### Couverture de chaque fonction

| Fonction | Revue exigée |
|---|---|
| ATLAS | Rust, architecture, NAIR/NAM, invariants, Constitution |
| FERRUM | macOS XPC/CDHash natif R4M (six tests en série), Linux/Windows, codesign |
| BOUNDARY | UID/PID/FD, sandbox, races et confused deputy |
| CIPHER | SSHSIG/Ed25519, clé revendiquée contre vraie clé, namespace, OOB, révocation |
| LEDGER | Git blobs, chaînage de reçus, chronologie, restauration locale R5Q |
| CAPSTONE | C3/C11–C14, droits d'effet, refus des douze portes |
| ADVERSARY | recherche de **nouvelles** failles falsifiables, injection et faux certificats |
| VERITY | CI, qualification réelle Mac Intel, écarts entre systèmes et tests ignorés |
| GUARDIAN | loi C20, mandat R6F, exigences d'autorité et verrou R6G |

**Critère d'acceptation E1 :** il doit exister une preuve de revue effectivement exécutée, de sa portée, de son auteur et de son indépendance lorsque celle-ci est requise. Sans cela, la fiche reste `NOT_SUBMITTED`. L'exécution de cette préparation par un seul coordinateur ne fait avancer aucune des neuf soumissions.

### État conservé

```ini
E1_REQUESTS_PREPARED=9
E1_REQUESTS_SENT=0
INDEPENDENT_SPECIALIST_REVIEWS=0
IND-OPERATOR=NOT_ENROLLED
OWNER_OOB_ROOT=NOT_AUTHENTICATED
R6F=IN_PROGRESS_UNCERTIFIED
R6G=LOCKED_BY_SEQUENCE
NATIVE_ADMISSION_GATES=12_UNMET
PRODUCTION=NOT_AUTHORIZED
```
