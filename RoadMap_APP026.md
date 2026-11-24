# NORDOI APP-026 — reconstruction vérifiée du checkpoint portable
Candidat de recherche, 24 novembre 2026. Parent C20 exact f56ca58bcd164075ec4a088696b5a278862289f9, PR #178, 22/22 SUCCESS, 3704 tests applicatifs, 56 assertions, neuf revues machine, aucune certification indépendante.
Décision: https://github.com/eddarsglobal/nordoi/pull/178#issuecomment-7129735660

APP-026 ré-audite APP-025 et accepte exclusivement CLEAN_PREFIX_NEEDS_NEW_CHECKPOINT. Il vérifie les pins fournis par l'hôte, les octets joints et les preuves historiques. Il scelle via APP-013, prépare via APP-014 dans un dossier privé temporaire, vérifie la continuité du préfixe et des fichiers natifs, puis publie dans une destination neuve par APP-014 avec marqueur final. Les sources et le moteur Rust restent intacts. Le résultat fournit un nouveau SHA-256 à transmettre par un canal indépendant; aucune reprise automatique ni nouveau processus Rust. Un état ambigu, complet, inchangé, orphelin ou une destination existante est refusé. En cas d'incertitude de publication, STOP_NO_AUTOMATIC_NATIVE_REPLAY; ne pas supposer un rollback.

Limites : hôte de confiance quiescent, pas de résistance à la concurrence hostile, pas d'authentification externe attestée, pas d'antirollback global ni garantie de durabilité post-crash. Les neuf contrôles machine du Bureau ne sont pas des agents indépendants.

Gate: 12 nouveaux tests sur preuves Rust réelles, 926 régressions, 938 par OS (3752 attendus), huit assertions par OS (32 attendues), quatre OS, Rust Test Gate, E8, neuf fonctions machine. C20 avant toute suite.

APP026_FORMAL_BUREAU_CERTIFICATION=NOT_CERTIFIED
R6F=IN_PROGRESS_UNCERTIFIED
R6G=LOCKED_BY_SEQUENCE
NATIVE_ADMISSION_GATES=12_UNMET
PRODUCTION=NOT_AUTHORIZED
MERGE=NONE
PR=DRAFT
