# NORDOI APP-021 — Rotation et révocation des autorités de signature

23 novembre 2026 · Recherche IA pour agents IA · Non privilégiée, non fusionnée, non destinée à la production.

## Séquence RoadMap

Parent APP-020 au commit 6f0f5625e282b4cd5a9ad5d527087502f5842f71 : 13/13 GitHub SUCCESS, 3 152 tests applicatifs PASS, Bureau interne PASS_WITH_CONTROLS_RESEARCH_ONLY. Décision distincte APP-021 en PR #172, commentaire 7093093121. Aucun amendement de main ni autorisation de production.

## Fonctionnement

APP-020 vérifie une signature Ed25519/SSHSIG sur le dernier HEAD mais ne gère pas le renouvellement ni la révocation de la clé. APP-021 ajoute une chaîne canonique d'autorités à signature cryptographique :

- BOOTSTRAP : l'empreinte exacte de la clé publique Ed25519 initiale vient du destinataire, pas du paquet. Le premier événement est signé par cette clé.
- ROTATE : l'ancien ET le nouveau signataire attestent exactement le même événement canonique via OpenSSH SSHSIG. L'ancienne clé est révoquée, la nouvelle est admise et le plancher de génération ou de séquence augmente.
- FREEZE : l'autorité sortante signe le gel ; plus aucune signature de nouvelle admission ni aucun effet natif n'est permis. L'état FROZEN reste consultable.
- Chaque événement possède une époque strictement croissante et le hash SHA-256 exact de l'enregistrement complet précédent, signatures comprises. Les clés révoquées ne peuvent être réintroduites.
- La chaîne complète est épinglée par un SHA-256 attendu, fourni séparément par le destinataire, avec des planchers externes minimum_epoch, minimum_sequence et minimum_generation.

Les déclarations HEAD continuent d'utiliser APP-020 et le vrai OpenSSH avec le namespace nordoi-head@eddarsglobal ; la chaîne d'autorités utilise un second namespace distinct nordoi-authority@eddarsglobal. Aucun mock cryptographique ou nouvelle bibliothèque de signatures n'est introduit.

Le programme conserve les contrôles APP-018, APP-017, APP-016 et APP-015, avec binaire et sources Rust véritables. Avant restore et resume, il revérifie l'intégralité du registre d'autorités, la déclaration SSHSIG, les politiques, le HEAD et le journal sous le même verrou local, après le rejeu natif et avant les effets APP-014/011.

## Interface

Programme : examples/apps/app021_authority_rotation/receiver_app021.py

Lecture sans moteur Rust :

    python3 -B examples/apps/app021_authority_rotation/receiver_app021.py status \
      --anchor-root /receiver/private/freshness \
      --authority-chain-file /receiver/trusted/current-authorities.json \
      --expected-chain-sha256 TRUSTED_CURRENT_CHAIN_SHA256 \
      --expected-root-key-sha256 TRUSTED_INITIAL_ROOT_KEY_SHA256 \
      --authority-id receiver-authority \
      --receiver-id receiver-agent \
      --minimum-epoch 2 \
      --minimum-sequence 2 \
      --minimum-generation 2 \
      --signed-statement /receiver/trusted/head-statement.json \
      --detached-signature /receiver/trusted/head-statement.sshsig

Pour inspect, ajouter --receiver-roster, --receiver-policy, --portable-dir. Pour verify/restore/resume, ajouter aussi --engine et --source-root. Pour restore et resume, ajouter --restore-root. inspect ne lance pas Rust ; verify refait les calculs natifs ; restore crée les vrais fichiers APP-014 ; resume renvoie uniquement le suffixe JSONL natif de APP-011. Une autorité FROZEN refuse inspect, verify, restore et resume.

## Structure du registre

Le fichier current-authorities.json est un objet JSON strict, avec les champs schema, records, release et production. Le schema vaut NORDOI_RECEIVER_AUTHORITY_ROTATION_CHAIN_V1. Les records forment une liste de 1 à 16 objets avec les champs event, outgoing_signature_b64 et incoming_signature_b64.

Chaque event a exactement les champs schema, epoch, previous_record_sha256, authority_id, receiver_id, state, active_public_key_b64, retired_public_key_sha256, minimum_sequence, minimum_generation, transition, release et production. Son schema vaut NORDOI_RECEIVER_AUTHORITY_TRANSITION_V1.

Tous les JSON doivent être canoniques : clés triées, sans espaces inutiles, octets ASCII et un seul LF final. Le key b64 est le base64 strict des octets d'un fichier ssh-ed25519 canonique sur une seule ligne LF. Le base64 des signatures SSHSIG porte sur les octets exacts des fichiers de signatures OpenSSH. Pour signer l'événement, utiliser la forme :

    ssh-keygen -Y sign -n nordoi-authority@eddarsglobal -f PRIVATE_SIGNING_KEY < event.json > event.sshsig

Lors d'un ROTATE, faire signer le même événement successivement par la clé sortante et par la clé entrante, en conservant les deux signatures. Les tests de test_app021.py contiennent un générateur de fixtures canoniques entièrement déterministe du point de vue des contrôles, avec clés de test éphémères. Ne jamais archiver ces clés privées.

## Limites de sécurité

Une signature conforme n'établit pas la garde réellement indépendante de la clé. Le destinataire doit préserver en dehors de la portée de l'adversaire sa racine publique attendue, le hash récent du registre et les minima d'époque et de fraîcheur. Si l'adversaire peut restaurer simultanément le journal, le registre, ses anciens hash et tous les anciens minima, une ancienne autorité et son ancienne signature peuvent être rejouées. Ce risque est expressément démontré dans les tests et dans la preuve native.

APP-021 ne fournit aucun TPM/TEE, quorum externe, signataire indépendant certifié, sandbox contre administrateur hostile, témoin de fraîcheur hors de l'hôte, garantie de panne électrique, transaction globale ou autorisation de production. Les rôles ATLAS/FERRUM/BOUNDARY/CIPHER/LEDGER/CAPSTONE/ADVERSARY/VERITY/GUARDIAN sont neuf perspectives de revue interne, pas neuf certifications indépendantes.

## Qualification obligatoire

40 tests APP-021 nouveaux par OS et 788 régressions par OS, soit 828 tests par plateforme, 3 312 sur Mac Intel, Apple Silicon, Ubuntu et Windows. Total 23 suites par OS, plus cinq jobs Rust et quatre jobs E8 (13 GitHub CI au SHA exact). La démonstration demo_app021.py doit vérifier 14 assertions par OS avec signatures authentiques Ed25519, vraie source NOI, binaire Rust natif, reprise APP-011 et audit APP-010 final.

Aucun verdict PASS_WITH_CONTROLS_RESEARCH_ONLY tant que ces 13 jobs, les 3 312 tests et les artefacts ne sont pas vérifiés au même commit.

    APP021_CERTIFICATION=PENDING_GITHUB_CI_AND_BUREAU
    APP021_ED25519_SSHSIG=REAL
    APP021_EXTERNAL_CUSTODY=NOT_PROVEN
    APP021_GLOBAL_ANTI_ROLLBACK=NOT_PROVIDED
    R6F=UNCERTIFIED
    R6G=LOCKED
    EXTERNAL_NATIVE_ADMISSION_GATES=12_UNMET
    HUMAN_TESTERS=0
    PR=DRAFT
    MERGE=NONE
    PRODUCTION=NOT_AUTHORIZED
