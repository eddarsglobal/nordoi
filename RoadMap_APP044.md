# NORDOI APP-044 — moteur d'authentification cryptographique des archives
Candidat de recherche, 26 novembre 2026. Parent exact b1a295911a978ede0adaf2fd5629201feb6e7928, PR #196, C20 22/22 SUCCESS, 4568 tests, 32 assertions, neuf contrôles machine non indépendants.
Décision : https://github.com/eddarsglobal/nordoi/pull/196#issuecomment-7191229319

Déclaration canonique liée à l'émetteur, au destinataire, au nonce attendu, à l'empreinte de la clé publique Ed25519 et aux empreintes archive, plan original, ancre et tête. Préparation exclusive après rejeu APP-043. Signature détachée produite hors du moteur par OpenSSH SSHSIG sous namespace dédié nordoi-chain-archive@eddarsglobal. Vérification de la clé publique indépendamment épinglée, déclaration exacte, signature réelle, puis rejeu APP-043 des octets capturés et comparaison de la tête attendue. Relecture finale des entrées. Aucun fichier de clé privée accepté. Douze tests, dont mauvaise clé, mauvais namespace, substitution d'identité, nonce, archive et tête, relocalisation et redémarrage.

Périmètre : authenticité cryptographique relative à une clé et des attentes externes de confiance, pas preuve d'identité civile ou d'enrôlement indépendant, de garde des clés, de fraîcheur, de consommation de nonce ou d'antirollback global. Le moteur ne signe pas directement et n'implémente pas de chiffrement ni transport. Environnement local stable de confiance, binaire OpenSSH de confiance. Aucune mutation des anciens APP ni du moteur Rust, aucun reset, libération, exécution ou production.

Gate : 12 nouveaux tests, 1142 régressions, 1154 par OS, 4616 attendus, 8 assertions par OS, 32 attendues, neuf contrôles machine et Rust/E8.
APP044_FORMAL_BUREAU_CERTIFICATION=NOT_CERTIFIED
R6F=IN_PROGRESS_UNCERTIFIED
R6G=LOCKED_BY_SEQUENCE
NATIVE_ADMISSION_GATES=12_UNMET
PRODUCTION=NOT_AUTHORIZED
MERGE=NONE
PR=DRAFT
