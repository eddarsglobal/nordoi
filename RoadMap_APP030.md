# NORDOI APP-030 — archivage vérifiable des preuves de réservation
Candidat de recherche, 24 novembre 2026. Parent C20 exact ad0c08d2fe718d5fceb976678a846a375c47e707, PR #182, 22/22 SUCCESS, 3896 tests, 32 assertions, neuf revues machine non indépendantes.
Décision : https://github.com/eddarsglobal/nordoi/pull/182#issuecomment-7139041452

Archive additive exclusivement après audit APP-029 réussi : copie des deux paquets portables, du suffixe partiel et final, des quatre reçus et de tous les résultats natifs vérifiés. Journal stderr hors périmètre (non probant). Les sources ne sont jamais déplacées ni supprimées. Destination neuve et disjointe, inventaire borné avec SHA-256, vérification avant marqueur final et second audit des sources, publication exclusive. Une archive partielle reste non admissible; aucune relance automatique. La vérification hors ligne exige le pin externe du manifeste et revérifie l'inventaire, les deux paquets et la chaîne native APP-025. Les reçus sont conservés octet à octet et liés à l'inventaire; le contrôle hors ligne ne réatteste pas l'identité du dossier original ni les reçus d'origine indépendamment du manifeste épinglé.

Limites : hôte local stable et quiescent, sans garantie de crash-durabilité, de résistance à un administrateur hostile, d'antirollback global, d'authentification externe ou de sauvegarde permanente. Aucun pouvoir de libérer la réservation ou d'exécuter, même après succès. Pas de modification des anciens APP ou du moteur Rust. Les neuf revues machine ne sont pas neuf certifications indépendantes.

Gate : 12 nouveaux tests avec résultats Rust authentiques, 974 régressions, 986 par OS, 3944 attendus, 8 assertions par OS, 32 attendues, neuf fonctions machine et Rust/E8. C20 avant toute suite.
APP030_FORMAL_BUREAU_CERTIFICATION=NOT_CERTIFIED
R6F=IN_PROGRESS_UNCERTIFIED
R6G=LOCKED_BY_SEQUENCE
NATIVE_ADMISSION_GATES=12_UNMET
PRODUCTION=NOT_AUTHORIZED
MERGE=NONE
PR=DRAFT
