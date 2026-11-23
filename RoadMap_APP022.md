# NORDOI — APP-022 : fraîcheur attestée en direct par processus témoin

**23 novembre 2026 — candidate de recherche IA, sans droits de production.**

## Gouvernance et ordre RoadMap

APP-022 succède à GOV-FIX1, commit exact 9f18af45ff64695137e8e7c8d0874034803b521b, PR #174, 22/22 checks SUCCESS. La décision de démarrage de recherche non privilégiée est documentée dans le commentaire GitHub 7100390105 de PR #174. Cette décision de coordination est distincte de la certification formelle indépendante du Bureau ; cette dernière n'est PAS obtenue. Les neuf rapports historiques R6F restent NOT_SUBMITTED. Les 99 blobs protégés et les douze verrous natifs restent inchangés.

La spécification structurée et ses conditions sont dans governance/NORDOI_APP022_RESEARCH_GATE_V1.json. Pas d'altération du noyau Rust ni des sources APP-001 à APP-021.

## Nouvelle fonctionnalité réelle

Un processus distinct, uniquement sur l'adresse de loopback 127.0.0.1, détient la clé PRIVÉE Ed25519 du témoin et fournit une réponse signée au défi aléatoire du destinataire. Le destinataire conserve et vérifie une seconde clé PUBLIQUE, épinglée par SHA-256 et distincte de l'autorité APP-021. Les données signées incluent le nonce aléatoire, le SHA-256 exact du HEAD et de la chaîne APP-021, l'époque, le compteur, la séquence, la génération et les identités destinataire/autorité.

Le contrôle vérifie réellement les signatures OpenSSH SSHSIG, dans le domaine nordoi-freshness@eddarsglobal. Pour RESTORE et RESUME, après le rejeu Rust natif et avant tout effet sur les fichiers, il sollicite une DEUXIÈME nouvelle attestation sous verrou APP-018 et exige que l'état signé reste identique. Le témoin refuse la baisse des compteurs dans la durée du processus et les altérations à compteur inchangé.

Les trois capacités doivent refuser : témoin absent ou indisponible, clé publique erronée, nonce non récent, chaîne ou HEAD incohérent, planchers diminués, état témoin modifié, autorité gelée ou révoquée. La démonstration est une vraie exécution locale, et non une simulation cryptographique.

## Exécution reproductible

Le témoin se lance avec Python 3 et ssh-keygen OpenSSH, en lui fournissant un état local JSON canonique signé pour l'expérience :

    python3 -B -u examples/apps/app022_live_freshness_witness/witness_app022.py \
      --state-file /lab/witness/current.json \
      --private-key /lab/witness/ephemeral_private \
      --port 0

Le récepteur donne les arguments APP-021 habituels, plus --witness-url http://127.0.0.1:PORT/attest, --witness-public-key, --expected-witness-key-sha256 et --minimum-witness-counter. La clé privée du témoin reste dans le processus témoin et n'est PAS passée à l'outil destinataire.

Un témoin séparé en processus n'est PAS un dépositaire indépendant : les tests utilisent un même administrateur et la même machine. Un adversaire contrôlant le serveur, son état, le journal et les éléments de confiance, puis redémarrant le témoin, peut revenir à un état cohérent antérieur : le test 26 montre cette limite réelle. Ni TPM/TEE, ni témoin hors hôte, ni monotonicité garantie après redémarrage, ni antirollback mondial, ni production.

## Qualification C20

26 tests nouveaux APP-022 + 828 régressions précédentes = 854 tests par OS, 3 416 sur quatre OS (Mac Intel, Apple Silicon, Ubuntu, Windows). Démonstration de 14 assertions réelles par plateforme, plus jobs Rust et E8. Toute correction requiert un rejeu intégral au SHA Git exact. Les anciens PR restent DRAFT.

APP022_FORMAL_BUREAU=NOT_CERTIFIED
APP022_EXTERNAL_CUSTODY=NOT_PROVEN
APP022_GLOBAL_ANTIROLLBACK=NOT_PROVEN
R6F=IN_PROGRESS_UNCERTIFIED
R6G=LOCKED_BY_SEQUENCE
NATIVE_ADMISSION_GATES=12_UNMET
PRODUCTION=NOT_AUTHORIZED
PR=DRAFT
MERGE=NONE
