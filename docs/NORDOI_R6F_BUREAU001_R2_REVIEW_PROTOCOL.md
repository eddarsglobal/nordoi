# NORDOI G0.3 R6F — BUREAU-001 R2 : registre de revues et contrôle des preuves

**Statut : R6F EN COURS, R6G VERROUILLÉ, PR DRAFT DE RECHERCHE.** Ce jalon est enfant exact de R6F R1 `108548e2021de42e65e8261fadd8184d4d9ff783`, déjà qualifié techniquement sur le Mac Intel du propriétaire : **1 087 tests réguliers PASS, 188/188 empreintes historiques et 99/99 empreintes de l'audit R1**, `SCRIPT_EXIT=0`, CI GitHub R1 **9/9 SUCCESS**. Ces résultats sont des observations transmises du terminal et des vérifications GitHub distinctes, pas des signatures d'un opérateur indépendant.

## Pourquoi le Bureau a besoin de R2

Le premier `scripts/roadmap_guardian.py` bloque correctement toute déclaration trompeuse de neuf revues déjà terminées. Il ne permet pas encore de **recevoir** un nouveau rapport de revue sans confondre trois concepts différents :

1. **Source réellement inspectée** par le coordinateur : une observation et une citation que le contrôleur peut comparer aux octets Git d'un fichier précis.
2. **Compte rendu soumis et journal local vérifié** : cohérence vérifiable entre un dossier et un fichier de log local ; leur auteur et leur origine restent déclaratifs.
3. **Audit spécialisé exécuté avec provenance indépendante et décision formelle** : exige des preuves d'exécution et d'identité effectivement authentifiées, et une procédure de validation distincte ; il n'est pas obtenu en R2.

R2 crée `governance/NORDOI_R6F_BUREAU001_R2_REVIEW_REGISTER.json` : neuf *perspectives* spécialisées ATLAS, FERRUM, BOUNDARY, CIPHER, LEDGER, CAPSTONE, ADVERSARY, VERITY et GUARDIAN, chacune reliée à **un vrai fichier de recherche ou de gouvernance, son SHA de blob Git et un fragment littéral vérifiable**, avec une observation et une lacune à explorer. C'est **une inspection source de coordination**, pas neuf agents IA exécutés indépendamment. Les neuf états de revue spécialisée restent `NOT_SUBMITTED`.

R2 ne modifie pas le registre initial `governance/NORDOI_BUREAU_R6F_HANDOFF_v1.json` : la porte v1 garde sa fonction de refuser toute auto-certification, tandis que la nouvelle porte accepte de **contrôler en lecture seule** un dossier de preuve local, sans promotion de statut.

## Contrôles et nouveau format de revue

`scripts/bureau_001_r2_review_intake.py --audit` : relance intégralement la porte R1, ses **99 blobs Git épinglés**, la chronologie R6B–R6E et les portes du Bureau ; vérifie les neuf rôles, leurs SHA Git et leurs passages exacts dans les sources, et interdit les neuf fausses signatures ou rapports indépendants.

`--check-submission governance/review_logs/nom.json` : le contrôleur accepte un dossier JSON canonique, strict, associé à un seul rôle et un seul commit de référence R1. Il vérifie le SHA-256 réel d'un journal de lecture local dans `governance/review_logs/`. Il n'exécute aucune commande figurant dans le dossier et n'écrit aucun fichier. Même un dossier correctement formé retourne **`STAGED_UNVERIFIED_LOCAL_REVIEW_LOG_ONLY`** : ni l'identité, ni la sincérité du journal, ni l'indépendance du relecteur ne peuvent être établies par cette lecture locale.

Format de dossier recevable, après création d'un vrai journal et calcul de son empreinte (ici valeurs indicatives, **pas une preuve existante**) :

```json
{
  "schema": "NORDOI-R6F-R2-REVIEW-SUBMISSION-V1",
  "project": "eddarsglobal/nordoi",
  "parent_head": "108548e2021de42e65e8261fadd8184d4d9ff783",
  "role": "CIPHER",
  "source_git_blob_sha1": "d2d3da085bbcf46b3d8a9a743b6e2ff0f820b0aa",
  "reviewer_claim": "local reviewer",
  "provenance": "LOCAL_SELF_REPORTED_ONLY",
  "execution_command_record": "python -B -m unittest -v",
  "review_result_claim": "PASS",
  "evidence_log_relpath": "governance/review_logs/cipher.log",
  "evidence_log_sha256": "replace-with-actual-sha256",
  "requested_decision": "STAGE_ONLY",
  "independent_operator_attestation": null,
  "independent_signer_signature": null
}
```

Le contrôleur exige le JSON **canonique** (ordre lexicographique des clés, sans espaces, un saut de ligne final). L'empreinte de log doit être réelle. Si l'auteur affirme une autorisation, un signataire externe, un certificat ou une élévation native, la demande est **refusée**.

Les 24 tests adversariaux R2 introduisent : faux nombre de rôles ; rôles dupliqués ; faux SHA Git ou fragment source ; réutilisation abusive de la même preuve ; faux agent exécuté ; fausse certification ; fausse signature externe ; promotion R6G ; changement du compte des 12 gates ; dossier hors base ; empreinte de log altérée ; chemins traversants ou symlink ; doublons JSON ; vérification que toutes les tentatives restent sans écriture. Ils s'ajoutent aux 48 tests RoadMapGuardian et BUREAU-001 R1, ainsi qu'aux régressions scientifiques R4Y–R6E et aux six tests natifs R4M serialisés sur macOS.

## Objectifs de développement après R2

Le Bureau doit poursuivre l'audit Rust, des frontières OS/processus et de la cryptographie ; les neuf perspectives source ne suffisent pas à conclure à une revue contradictoire indépendante. La suite de R6F doit faire accepter **des comptes rendus véritablement étayés**, relier les correctifs à leurs tests, puis décider explicitement et sans faux quorum si une qualification de recherche plus large est justifiée. Le véritable dépositaire externe et la racine propriétaire hors bande resteront nécessaires avant toute évolution des douze admissions natives.

```ini
BUREAU001_R2=COORDINATOR_SOURCE_INSPECTION_WITH_CONTROLS
PARENT_R1=108548e2021de42e65e8261fadd8184d4d9ff783
SOURCE_PERSPECTIVES=9
INDEPENDENT_AGENT_EXECUTIONS=0
SPECIALIST_CERTIFICATIONS=0
STAGED_LOCAL_LOG=NOT_INDEPENDENT_ATTESTATION
G03_R6F=IN_PROGRESS
G03_R6G=LOCKED_BY_SEQUENCE
NATIVE_ADMISSION_GATES=12_UNMET
OWNER_OOB_ROOT=NOT_AUTHENTICATED
INDEPENDENT_GLOBAL_ANTIROLLBACK=NOT_VERIFIED
PRODUCTION=NOT_AUTHORIZED
```

**Aucun code Rust, profil certifié, programme .noi ou moteur d'exécution n'est modifié par R2.** Le but est de rendre les prochains audits réels plus vérifiables, sans fabriquer l'autorité des agents.
