# NORDOI G0.3 R6F — BUREAU-001 R6 : faisabilité de la conservation indépendante et de la racine hors bande

**Statut : recherche négative / dossier de faisabilité uniquement.** Parent exact R5 : `4d3bd3cbff8c67cef98481750270a2c6c0ce7b37`. R5 a été qualifié techniquement avec les résultats GitHub CI **9/9 SUCCESS** et un rejeu propriétaire Mac Intel de **1 183 tests réguliers PASS** transmis par le propriétaire ; ce n'est pas une signature externe indépendante. R6 ajoute de nouvelles pièces sans fusionner R5, changer le compilateur, le noyau Rust, ni ouvrir les douze permissions natives.

## Problème réellement abordé

**Une signature cryptographique valide n'authentifie pas automatiquement l'indépendance administrative de la personne qui signe.** Le projet a déjà obtenu, dans ses laboratoires réels R6C–R6E, des signatures OpenSSH véritables de plusieurs clés locales, de vrais processus distincts et des déclarations de domaine. Pourtant ces clés et ces fichiers de signataires appartiennent au même administrateur.

R6 examine la différence entre quatre faits souvent confondus :

1. **Intégrité des octets et possession de clés privées :** vraies signatures Ed25519 OpenSSH sur un dossier canonique, vérifiées avec des clés publiques épinglées et des descripteurs de `allowed_signers` maintenus ouverts.
2. **Identité réelle de l'opérateur externe :** doit être vérifiée par un canal et une administration hors du contrôle du propriétaire local. Une chaîne JSON signée localement disant « gardien indépendant » ne suffit pas.
3. **Racine propriétaire hors bande :** le propriétaire doit réellement confirmer la clé exacte du gardien par un canal indépendant, en conservant un reçu vérifiable de cette observation. Deux clés locales ou deux PID ne prouvent pas ce canal.
4. **Historique monotone indépendant :** le contre-exemple réel R5Q restaure de manière cohérente trois fichiers de preuves locaux ; un nouvel examinateur peut perdre toute connaissance de l'époque avancée. Aucune signature écrite par le même administrateur ne rend les registres locaux impossibles à restaurer.

## Nouvelle expérience R6

`research/g03_r6f_r6_external_custody_feasibility.py` crée dans des dossiers privés temporaires une preuve **réellement signée par les deux clés Ed25519 locales** du laboratoire R6E, sous namespace dédié `nordoi-r6f-r6-feasibility`, et lie les octets exacts du document signé à l'empreinte de la pièce R6E historique et à son challenge.

Le vérificateur utilise le vrai `/usr/bin/ssh-keygen -Y verify` sur les deux signatures, en réutilisant les contrôles R4 par **descripteurs ouverts et clés publiques épinglées**. Il distingue :
- une preuve locale correctement signée, sans affirmation d'autorité externe ;
- le **même type de preuve cryptographiquement valide** qui affirme frauduleusement que le propriétaire a vérifié une empreinte hors bande, qu'un gardien indépendant existe, qu'un historique distant est monotone et qu'un quorum externe s'est réuni ;
- une signature invalide, un mauvais namespace, un dossier altéré, une clé manquante, un champ JSON dupliqué ou une source historique incompatible.

**Tous les résultats restent `BLOCK / NOT_AUTHORIZED`.** La première pièce ne prouve que la signature locale ; la deuxième permet de démontrer qu'un document local honnêtement signé peut contenir des déclarations externes fausses ; les pièces malformées sont mises en quarantaine. Aucune signature ni simulation d'un véritable tiers indépendant n'est fabriquée.

L'exercice ajoute le **rejeu réel du laboratoire R5Q** de restauration cohérente de trois fichiers SQLite/JSON. Ainsi un vrai contrôle cryptographique local et un historique possiblement restauré sont confrontés dans le même dossier de recherche, sans se faire passer pour une garantie externe.

## Les douze exigences natives, sans promotion artificielle

La liste exacte provenant de `research/G03_R6E_OOB_ORIGIN_NO_EXTERNAL_AUTHORITY.json` est conservée dans `governance/NORDOI_R6F_BUREAU001_R6_EXTERNAL_CUSTODY_FEASIBILITY.json`. Chaque condition garde `UNMET_INDEPENDENT_EXTERNAL_EVIDENCE`.

| Condition | Preuve réellement nécessaire |
|---|---|
| owner_oob_authenticated_root | Confirmation propriétaire par canal hors bande authentifié |
| actual_external_custodian_identity | Identité et autorisation réelles de l'opérateur externe |
| distinct_administrative_control_domains | Administration et stockage clés hors de portée du propriétaire local |
| guardian_key_pinned_out_of_band | Empreinte exacte de la clé gardien confirmée indépendamment |
| independently_signed_os_attestation | Attestation de la plateforme/identité sous provenance externe indépendante |
| signed_fresh_challenge_and_exact_history | Challenge frais signé avec rattachement exact à l'historique |
| independent_os_domain_and_host_witness | Témoin d'hôte et domaine administrativement séparés |
| externally_monotonic_key_epoch | Époque de clés protégée indépendamment des retours arrière |
| protected_global_revocation_floor | Seuil global de révocation hors du pouvoir de restauration local |
| protected_global_spent_nonce_floor | Seuil global des nonces consommés, protégé à l'extérieur |
| independent_quorum | Quorum réel de signataires vérifiés et administrativement indépendants |
| independent_recovery_rotation_audit | Révocation/rotation/récupération authentifiées et archivées indépendamment |

**Un compte GitHub distinct, un VPS du même propriétaire, deux clés générées localement, deux noms de domaine ou deux agents IA partageant le même administrateur ne satisfont pas ces preuves.** La faisabilité future dépend de la participation réelle d'un opérateur séparément autorisé, d'une vérification propriétaire hors bande et d'une conservation indépendante des données : aucun n'a été obtenu par R6.

## Programme de recherche proposé, sans lancement externe

Le Bureau peut préparer un protocole d'inscription future avec six phases : A — identité réelle et accord de l'opérateur ; B — empreinte hors bande vérifiée par le propriétaire ; C — contrôle administratif et hôte séparés avec signatures fraîches ; D — historique monotone externe des époques de clés, nonces et révocations ; E — vrai quorum et tests adversariaux de récupération ; F — revues indépendantes et décision formelle explicite. **Ces étapes sont des exigences non satisfaites, pas des actions déjà exécutées.**

Le Bureau apporte ici neuf **perspectives sources** ATLAS, FERRUM, BOUNDARY, CIPHER, LEDGER, CAPSTONE, ADVERSARY, VERITY et GUARDIAN. Elles ne doivent pas être confondues avec neuf agents effectivement exécutés ou signataires indépendants : **independent_specialist_reviews=0**.

## Tests reproductibles et limites

```bash
python3 -B scripts/roadmap_guardian.py --audit
python3 -B scripts/bureau_001_r1_audit.py --audit
python3 -B scripts/bureau_001_r2_review_intake.py --audit
python3 -B -m unittest discover -s research -p 'test_g03_r6f_r6_external_custody_feasibility.py' -v
python3 -B research/g03_r6f_r6_external_custody_feasibility.py --exercise
# Dernière commande : code de sortie 2 attendu pour le refus d'admission.
```

**24 tests adversariaux** : 16 tests portables de protocole, JSON canonique, liens de source, doubles champs et refus d'élévation ; huit expériences POSIX/OpenSSH véritables, dont signatures locales, assertions d'opérateur contrefaites mais réellement signées, document modifié après signature, preuve manquante et vrai R5Q. Sur Windows, les tests nécessitant OpenSSH et descripteurs POSIX sont **ignorés explicitement** ; cela ne signifie pas qu'ils ont réussi sur Windows.

Les tests et fichiers R1–R5 et les six vérifications macOS XPC/CDHash R4M en série doivent rester inchangés. Le gardien R6F reste `IN_PROGRESS_UNCERTIFIED`, R6G demeure `LOCKED_BY_SEQUENCE`, la certification du Bureau est `PENDING`, aucun tiers n'est enrôlé, aucun historique externe monotone n'est vérifié et `PRODUCTION=NOT_AUTHORIZED`.

```ini
G03_R6F_R6=RESEARCH_FEASIBILITY_CANDIDATE
LOCAL_TWO_ED25519_SIGNATURES=TRUE_KEY_POSSESSION_NOT_EXTERNAL_IDENTITY
OWNER_OOB_ROOT=NOT_AUTHENTICATED
INDEPENDENT_EXTERNAL_CUSTODIAN=NOT_ENROLLED
GLOBAL_MONOTONIC_ANTIROLLBACK=NOT_VERIFIED
INDEPENDENT_SPECIALIST_REVIEWS=0
NATIVE_ADMISSION_GATES=12_UNMET
R6F=IN_PROGRESS_UNCERTIFIED
R6G=LOCKED_BY_SEQUENCE
PRODUCTION=NOT_AUTHORIZED
```
