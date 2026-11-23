# NORDOI APP-020 — Provenance SSHSIG Ed25519 de HEAD

23 novembre 2026 · Recherche IA → IA · Aucune admission de production

## Parent RoadMap C20

APP-019 PR #171 au commit fa89752c1a62f3577134017d6a90b62d46a25cf7 : 13/13 jobs GitHub SUCCESS, 2 992 tests applicatifs PASS, Bureau interne PASS_WITH_CONTROLS_RESEARCH_ONLY. Décision distincte d'ouverture APP-020 : commentaire 7092822671 de PR #171. Aucune fusion vers main.

## Fonctionnalité

APP-019 impose un hash de HEAD fourni par le destinataire, mais ne prouve pas qui lui fournit ce hash. APP-020 ajoute une déclaration canonique de HEAD signée Ed25519/SSHSIG via le vrai OpenSSH ssh-keygen -Y verify. Le destinataire impose la clé publique Ed25519 et son SHA-256 indépendamment du paquet, une identité d'autorité, une identité de récepteur et un minimum de génération et séquence.

Domaine exact : nordoi-head@eddarsglobal. Le contenu signé lie les champs head_sha256, roster_sha256, roster_generation, journal_sequence, latest_entry_sha256, authority_id, receiver_id, release et production. Les champs additionnels sont refusés et les octets doivent être canoniques.

La vérification est faite sous le verrou APP-018 et contrôle la chaîne complète du journal, les politiques APP-017/016 puis le véritable rejeu Rust APP-015. Pour restore et resume, la signature, la tête et la politique sont revérifiées sous le même verrou avant les effets APP-014/011.

Une **signature valide n'établit PAS la garde réellement indépendante de la clé**. Si un ancien état, une ancienne signature valide et un plancher de fraîcheur régressé sont fournis ensemble, un retour en arrière demeure possible. Aucune anti-rollback globale, aucun TPM/TEE, aucune preuve indépendante d'administration, sandbox OS, durabilité de panne électrique ou transaction inter-agents ne sont démontrés.

## Programme et commandes

Fichier : examples/apps/app020_signed_head_provenance/receiver_app020.py

La commande propose renvoie les octets JSON canoniques d'une DÉCLARATION NON SIGNÉE :

    python3 -B examples/apps/app020_signed_head_provenance/receiver_app020.py propose --anchor-root /receiver/private/anchor --authority-id lab-authority --receiver-id agent-dest > statement.json

Sur une machine distincte contrôlée par l'autorité réellement désignée, signer les octets exacts :

    ssh-keygen -Y sign -n nordoi-head@eddarsglobal -f /signer/private/ed25519 < statement.json > statement.sshsig

Exemple d'audit signé (le hash de clé doit venir d'une référence du destinataire distincte) :

    python3 -B examples/apps/app020_signed_head_provenance/receiver_app020.py status --anchor-root /receiver/private/anchor --authority-id lab-authority --receiver-id agent-dest --trusted-public-key /receiver/keys/trusted.pub --expected-public-key-sha256 TRUSTED_KEY_FILE_SHA256 --signed-statement /receiver/trusted/statement.json --detached-signature /receiver/trusted/statement.sshsig --minimum-sequence 1 --minimum-generation 1

Le fichier de clé publique doit contenir une unique ligne canonique de la forme ssh-ed25519 BASE64 avec un saut de ligne final et aucune annotation. L'empreinte SHA-256 attendue porte sur ces octets exacts.

Actions :

- propose : déclaration non signée, jamais une autorité
- status : signature + chaîne de journal validées sans moteur Rust
- inspect : ajoute roster, policy et paquet et refuse sans lancer Rust
- verify : vrai rejeu historique du Rust épinglé par le destinataire
- restore : revérifie signature et politiques puis restaure les fichiers APP-014 de façon exclusive
- resume : revérifie les mêmes contrôles et ne retourne que le suffixe JSONL natif authentique

Pour inspect, verify, restore et resume, fournir aussi --receiver-roster, --receiver-policy et --portable-dir. Pour verify/restore/resume ajouter --engine et --source-root. Pour restore/resume ajouter --restore-root sur un dossier de destination conforme au contrat APP-014.

## Codes d'exécution et sécurité

- Retour 0 : résultat local positif, sans attestation indépendante de garde.
- Retour 2 : REJECTED et publication=NOT_STARTED avant nouvelle admission native.
- Retour 3 : INCONCLUSIVE et publication=UNKNOWN pour effet ou publication indéterminé.

Les réponses indiquent crypto_signature=SSHSIG_ED25519_VERIFIED si OpenSSH valide vraiment les octets signés et les pins. Elles précisent independent_key_custody=NOT_PROVEN, global_anti_rollback=NOT_PROVIDED, external_security_certification=NOT_CLAIMED et production=NOT_AUTHORIZED.

L'appel du vrai ssh-keygen est obligatoire : absence du binaire ou signature invalide => refus. Les fichiers clé publique, signature et statement doivent être des fichiers réguliers bornés, séparés du checkpoint, du journal et de la destination native. Les tests créent une clé privée temporaire dans le runner et l'effacent, sans l'archiver. Cela ne constitue pas une identité ou autorité tierce vérifiée.

## Qualification de recherche

40 tests APP-020 nouveaux par OS, dont vraie signature et contrôle d'identité/domaine, falsifications cohérentes, mauvaise clé, plancher de fraîcheur, rollback complet signé avec ancien plancher, vrai Rust, restore, resume, FROZEN.

748 régressions par OS : APP-019 (40), APP-018 (40), APP-017 (40), APP-016 (40), CERT-015-R1 (20), APP-001 à APP-015 (568). Total 788 tests par système, 3 152 tests applicatifs sur Mac Intel, Apple Silicon, Ubuntu, Windows, 22 suites par OS. Rust Test Gate 5 + E8 adversarial 4 : 13 jobs au SHA exact. Démonstration séparée demo_app020.py : 13 assertions dont vrai Rust, signature OpenSSH, DAG final APP-010, falsifications, rejet de séquence rétrograde et limite de l'ancien couple état + signature.

Bureau : ATLAS, FERRUM, BOUNDARY, CIPHER, LEDGER, CAPSTONE, ADVERSARY, VERITY, GUARDIAN sont neuf perspectives de revue technique INTERNE, pas neuf signataires ou organes indépendants. Aucun verdict PASS avant 13/13 jobs GitHub SUCCESS et inspection de leurs artefacts.

    APP020_STATUS=PENDING_GITHUB_CI_AND_BUREAU
    APP020_SSHSIG=REAL_OPENSSH_ED25519_VERIFICATION
    APP020_INDEPENDENT_KEY_CUSTODY=NOT_PROVEN
    APP020_GLOBAL_ANTI_ROLLBACK=NOT_PROVIDED
    R6F=IN_PROGRESS_UNCERTIFIED
    R6G=LOCKED_BY_SEQUENCE
    EXTERNAL_NATIVE_ADMISSION_GATES=12_UNMET
    HUMAN_TESTERS=0
    PR=DRAFT
    MERGE=NONE
    PRODUCTION=NOT_AUTHORIZED
