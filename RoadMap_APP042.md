# NORDOI APP-042 — moteur de vérification intégrale de la chaîne cryptographique
Candidat de recherche, 26 novembre 2026. Parent exact e65ed00d642cf71eeff19e963eaffa8025fd6e6d, PR #194, C20 22/22 SUCCESS, 4472 tests, 32 assertions, neuf contrôles machine non indépendants.
Décision : https://github.com/eddarsglobal/nordoi/pull/194#issuecomment-7187245363

Audit en lecture seule d'une chaîne non vide (1 à 64 transitions), registre APP-041 dédié, ancre et plan canonique épinglés indépendamment. Chaque étape indique des chemins relatifs sous racine de preuves de confiance, sans traversée ni liens symboliques, et les empreintes des sceaux, contextes et transition. Validation de continuité des sceaux et contextes, refus des cycles et comparaison exacte entre prédécesseurs attendus et inventaire du registre. Chaque décision est réauditée par APP-041/040/039/038/037/036 avec instantané historique correspondant et signatures réelles; relecture du registre et du plan. Sortie déterministe : ancre, tête locale, nombre de transitions, empreinte des décisions et du plan. Tests sur deux transitions réelles et adversaires : ancre erronée, décision manquante/orpheline, cycle, rupture, contexte substitué, signature historique ou décision altérée, plan modifié, chemin hors racine.

Périmètre : registre local stable de confiance et instantanés historiques fournis par l'appelant. Aucun archivage automatique, aucune preuve de fraîcheur de la tête, exhaustivité de collecte antérieure, consensus distribué, antirollback global, résistance à un écrivain hostile concurrent ou panne électrique. L'audit ne transforme pas une ancre non qualifiée en ancre de confiance. Le plan attendu doit être fourni indépendamment; l'audit reconstruit sa chaîne et refuse les décisions étrangères au plan. Pas de mutation des données auditées ni du moteur Rust. Aucun reset, libération, exécution ou production.

Gate : 12 nouveaux tests, 1118 régressions, 1130 par OS, 4520 attendus, 8 assertions par OS, 32 attendues, neuf contrôles machine et Rust/E8.
APP042_FORMAL_BUREAU_CERTIFICATION=NOT_CERTIFIED
R6F=IN_PROGRESS_UNCERTIFIED
R6G=LOCKED_BY_SEQUENCE
NATIVE_ADMISSION_GATES=12_UNMET
PRODUCTION=NOT_AUTHORIZED
MERGE=NONE
PR=DRAFT
