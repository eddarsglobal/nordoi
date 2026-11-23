# NORDOI APP-019 — Empreinte externe du dernier HEAD de confiance

23 novembre 2026 · Recherche IA → IA · C20 strict · Non privilégié

Parent exact APP-018 : 0aa1fe01fe9391ed67a0fa20f8af7280af56a1ad, 13/13 jobs GitHub SUCCESS, 2 832 tests applicatifs PASS, PR #170 DRAFT. Le Bureau a consigné une décision distincte de démarrage de la recherche APP-019 en PR #170. Aucune fusion dans main, ni admission native privilégiée ni production.

## Objectif scientifique

APP-018 vérifie le journal local SHA-256 et protège les générations tant que la racine locale demeure intacte. Un attaquant qui remplace intégralement HEAD.json et history par une vieille copie cohérente peut contourner ce contrôle.

APP-019 oblige le destinataire à fournir le SHA-256 exact de HEAD.json depuis un canal séparé du répertoire du journal. Le programme vérifie le journal complet, le HEAD et le pin reçu sous le même verrou APP-018, puis contrôle roster, policy, source .noi, moteur Rust, plan DAG et checkpoint avant replay/reprise.

La comparaison des empreintes NE prouve PAS l'authentification du canal. Si un attaquant remplace à la fois la racine locale ET la référence externe par leurs anciennes versions cohérentes, l'ancien état reste accepté. Le test 38 et la démonstration le confirment explicitement. Un simple fichier de copie sur la même machine n'est pas une autorité indépendante.

## Commandes disponibles

Programme : examples/apps/app019_external_head_pin/receiver_app019.py

Initialiser un journal APP-018 à partir du SHA-256 externe d'un roster de confiance, puis recevoir un CANDIDAT de hash de HEAD à ancrer réellement ailleurs :

    python3 -B examples/apps/app019_external_head_pin/receiver_app019.py bootstrap --anchor-root /receiver/private/anchor --receiver-roster /receiver/trusted/roster-v1.json --expected-roster-sha256 TRUSTED_ROSTER_SHA256

Capture non authentifiée, sans moteur Rust, d'un candidat d'ancrage :

    python3 -B examples/apps/app019_external_head_pin/receiver_app019.py capture --anchor-root /receiver/private/anchor

Lecture contrôlée du journal, sans Rust :

    python3 -B examples/apps/app019_external_head_pin/receiver_app019.py status --anchor-root /receiver/private/anchor --expected-head-sha256 TRUSTED_HEAD_SHA256

Inspection du paquet sans lancer Rust :

    python3 -B examples/apps/app019_external_head_pin/receiver_app019.py inspect --anchor-root /receiver/private/anchor --expected-head-sha256 TRUSTED_HEAD_SHA256 --receiver-roster /receiver/trusted/roster-v1.json --receiver-policy /receiver/trusted/policy-v1.json --portable-dir /received/portable

Rejeu authentique du Rust choisi par le destinataire :

    python3 -B examples/apps/app019_external_head_pin/receiver_app019.py verify --anchor-root /receiver/private/anchor --expected-head-sha256 TRUSTED_HEAD_SHA256 --receiver-roster /receiver/trusted/roster-v1.json --receiver-policy /receiver/trusted/policy-v1.json --portable-dir /received/portable --engine /receiver/pinned/nordoi --source-root /receiver/pinned/src

Restauration : remplacer verify par restore et ajouter --restore-root /receiver/new-empty-native-root. Reprise : remplacer restore par resume. La sortie de resume reste strictement le suffixe JSONL réellement produit par APP-011, sans preuve artificielle ajoutée.

Rotation avec verrou et pin externe de l'état PRÉCÉDENT :

    python3 -B examples/apps/app019_external_head_pin/receiver_app019.py advance --anchor-root /receiver/private/anchor --expected-head-sha256 TRUSTED_PREVIOUS_HEAD_SHA256 --receiver-roster /receiver/trusted/roster-v2.json --expected-roster-sha256 TRUSTED_NEW_ROSTER_SHA256 --receiver-minimum-generation 2

Le nouveau SHA-256 de HEAD retourné par advance est un candidat NON authentifié. Un agent doit le réancrer via un canal de confiance réellement externe avant d'autoriser un nouvel effet. Tant que le pin attendu n'a pas été mis à jour, le journal avancé est refusé.

## Codes de sortie et limites

- 0 : assertion positive dans le périmètre local, sans certificat externe.
- 2 : REJECTED / publication NOT_STARTED lorsqu'une référence/politique n'est pas admissible avant tout nouvel effet.
- 3 : INCONCLUSIVE / publication UNKNOWN si un effet ou une publication est indéterminé.

Les émissions sont explicites : external_pin_authentication=NOT_PERFORMED ; global_anti_rollback=NOT_PROVIDED ; signature=NOT_PRESENT ; production=NOT_AUTHORIZED ; external_security_certification=NOT_CLAIMED.

Les opérations sous contrôle sont status, inspect, verify, restore, resume. capture ne garantit ni fraîcheur, ni identité, ni provenance. bootstrap et advance ne certifient pas la publication externe de leurs nouvelles empreintes.

Pas de sandbox OS, TPM/TEE, signature, protocole de témoin hors bande authentifié, défense contre l'hôte malveillant, transactions multi-agent globales, garanties de panne électrique ou récupération native in-flight. La protection contre le rollback total dépend strictement d'une référence récente que le destinataire conserve indépendamment de la racine compromise.

## Vérifications obligatoires

40 tests APP-019 nouveaux par plateforme, avec vraie production/reprise Rust, nouvelles politiques et sources NOI, pin stale, rotation, FROZEN et tests de rollback cohérent. 708 régressions par plateforme : APP-018, APP-017, APP-016, CERT-015-R1 et APP-001…015. Total attendu : 748 tests par OS, 2 992 tests applicatifs, 21 suites sur Mac Intel, Apple Silicon, Linux et Windows ; Rust Test Gate 5 jobs + E8 4 jobs = 13 jobs au HEAD exact.

Le script demo_app019.py produit les preuves natives et 12 assertions, notamment un véritable audit APP-010 complet et la limite ancienne racine + ancien pin. Les artefacts sont collectés par OS.

Bureau : ATLAS, FERRUM, BOUNDARY, CIPHER, LEDGER, CAPSTONE, ADVERSARY, VERITY, GUARDIAN sont neuf angles documentaires de revue interne, pas neuf signatures cryptographiques ni neuf organismes indépendants. Aucune promotion du candidat avant 13/13 jobs success au nouveau HEAD, confrontation de preuves et décision expresse du Bureau.

## Verrous

    APP019_STATUS=PENDING_GITHUB_CI_AND_BUREAU
    APP019_C20_PARENT=0aa1fe01fe9391ed67a0fa20f8af7280af56a1ad
    APP019_EXTERNAL_PIN_AUTHENTICATION=NOT_PERFORMED
    APP019_GLOBAL_ANTI_ROLLBACK=NOT_PROVIDED
    APP019_PRODUCTION=NOT_AUTHORIZED
    R6F=IN_PROGRESS_UNCERTIFIED
    R6G=LOCKED_BY_SEQUENCE
    EXTERNAL_NATIVE_ADMISSION_GATES=12_UNMET
    HUMAN_TESTERS=0
    PR=DRAFT
    MERGE=NONE
