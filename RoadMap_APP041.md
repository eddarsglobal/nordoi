# NORDOI APP-041 — moteur d'unicité des successeurs cryptographiques
Candidat de recherche, 26 novembre 2026. Parent exact 8ff0654a157dd054b471e42e30f05a5062854bf7, PR #193, C20 22/22 SUCCESS, 4424 tests, 32 assertions, neuf contrôles machine non indépendants.
Décision : https://github.com/eddarsglobal/nordoi/pull/193#issuecomment-7186537528

Registre local de confiance distinct du registre APP-037, du journal APP-038 et des fichiers de référence. Chaque candidat reconstruit et vérifie la transition APP-040 sous empreinte externe avant de réserver exclusivement un dossier nommé par empreinte du sceau prédécesseur. START, transition canonique et COMPLETE écrits exclusivement, contrôle de relecture. Un seul successeur accepté par prédécesseur dans le registre partagé. Même candidat : doublon bloqué; autre successeur vérifié : alternative bloquée. Une réservation incomplète bloque définitivement la réattribution. Audit en lecture seule avec attentes externes et journal correspondant au successeur accepté; reconstruction de la transition et comparaison exacte aux octets persistés. Deux processus et deux états de journal réellement vérifiés testent la concurrence entre alternatives légitimes.

Périmètre : processus coopérants utilisant exclusivement APP-041 et partageant le même registre local stable de confiance. Pas de contrôle des anciens points d'entrée, pas de consensus distribué, tête de chaîne globale, protection contre restauration ou effacement hostile, durabilité électrique, ni exhaustivité des contradictions. Audit de la décision nécessite les preuves correspondant au sceau successeur; pas d'audit automatique historique d'un journal ayant évolué. La décision est locale, non signée par une autorité indépendante. Aucun changement des anciens APP ou du moteur Rust. Aucun reset, libération, exécution ou production.

Gate : 12 nouveaux tests, 1106 régressions, 1118 par OS, 4472 attendus, 8 assertions par OS, 32 attendues, neuf contrôles machine et Rust/E8.
APP041_FORMAL_BUREAU_CERTIFICATION=NOT_CERTIFIED
R6F=IN_PROGRESS_UNCERTIFIED
R6G=LOCKED_BY_SEQUENCE
NATIVE_ADMISSION_GATES=12_UNMET
PRODUCTION=NOT_AUTHORIZED
MERGE=NONE
PR=DRAFT
