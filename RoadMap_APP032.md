# NORDOI APP-032 — signature cryptographique des transferts de preuves
Candidat de recherche, 24 novembre 2026. Parent exact cca414d69a1467f06306c939af8c4eee3e89990f, PR #184, C20 22/22 SUCCESS, 3992 tests et 32 assertions, neuf contrôles machine non indépendants.
Décision : https://github.com/eddarsglobal/nordoi/pull/184#issuecomment-7140516657

Le destinataire fournit hors paquet les identités attendues, le pin SHA-256 de la clé publique Ed25519, les deux pins du transfert et de l'archive, et un nonce de 256 bits attendu. Déclaration canonique, signature SSHSIG détachée, namespace propre à APP-032. Signature par OpenSSH externe; aucun secret accepté par le programme. Vérification via OpenSSH et validation APP-031 sur les octets capturés, après contrôle de la signature. Aucune publication en cas d'incohérence. Aucune autorisation de reprendre, libérer ou exécuter.

Limites : preuve de possession de la clé autorisée seulement, pas d'enrôlement réel des agents ni de garde indépendante des clés; clés de tests éphémères. Nonce comparé mais non consommé, donc anti-rejeu non garanti. Pas de transport réseau, chiffrement, durabilité après crash, ni protection contre hôte hostile. OpenSSH local de confiance requis. Aucun changement des APP antérieurs ou du noyau Rust.

Gate : 12 nouveaux tests avec véritables signatures et preuves Rust, 998 régressions, 1010 par OS, 4040 attendus, 8 assertions par OS, 32 attendues, neuf fonctions machine et Rust/E8.
APP032_FORMAL_BUREAU_CERTIFICATION=NOT_CERTIFIED
R6F=IN_PROGRESS_UNCERTIFIED
R6G=LOCKED_BY_SEQUENCE
NATIVE_ADMISSION_GATES=12_UNMET
PRODUCTION=NOT_AUTHORIZED
MERGE=NONE
PR=DRAFT
