# NORDOI APP-040 — chaînage cryptographique des sceaux successifs
Candidat de recherche, 26 novembre 2026. Parent exact 22891c1bcd4677cfc43069e2ff76fbecabd341ef, PR #192, C20 22/22 SUCCESS, 4376 tests, 32 assertions, neuf contrôles machine non indépendants.
Décision : https://github.com/eddarsglobal/nordoi/pull/192#issuecomment-7185982256

Transition append-only entre deux manifestes APP-039 épinglés. Validation canonique, bornée, des manifestes et de leurs contextes indépendants. Chaque ancienne ligne (nonce, empreinte de contradiction, empreinte du reçu et huit empreintes de fichiers) et chaque ancienne attente doivent être présentes à l'identique dans le successeur. Au moins une nouvelle contradiction est obligatoire. Le nouveau manifeste est revérifié contre le journal courant via APP-039. Le reçu de transition lie les deux empreintes de sceaux, les deux contextes, les inventaires et les entrées nouvelles; publication exclusive hors registre et journal. Revérification en lecture seule sous empreinte externe du reçu. Aucun ancien fichier écrasé.

Périmètre : preuve d'une transition locale entre deux états de confiance, pas une chaîne complète à tête globale unique. Le sceau historique doit avoir été préalablement qualifié et son empreinte conservée indépendamment; son ancien état physique n'est pas rejoué. Pas de preuve de fraîcheur, protection contre branches concurrentes, rollback global, suppression antérieure au premier sceau, exhaustivité de collecte, panne électrique ou écrivain hostile. Les limites APP-039 de 64 entrées et 64 contextes sont conservées. Le nouveau journal doit correspondre exactement au sceau successeur fourni. Aucun changement des anciens APP ou du moteur Rust, aucune exécution, libération, reset ou production.

Gate : 12 nouveaux tests, signatures et preuves natives, 1094 régressions, 1106 par OS, 4424 attendus, 8 assertions par OS, 32 attendues, neuf contrôles machine et Rust/E8.
APP040_FORMAL_BUREAU_CERTIFICATION=NOT_CERTIFIED
R6F=IN_PROGRESS_UNCERTIFIED
R6G=LOCKED_BY_SEQUENCE
NATIVE_ADMISSION_GATES=12_UNMET
PRODUCTION=NOT_AUTHORIZED
MERGE=NONE
PR=DRAFT
