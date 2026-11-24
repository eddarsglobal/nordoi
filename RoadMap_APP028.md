# NORDOI APP-028 — réservation locale exclusive interprocessus
Candidat de recherche, 24 novembre 2026. Parent C20 exact d7f0d3ab84c2f98eb8789c9e2c985353d203ac72, PR #180, 22/22 SUCCESS, 3800 tests, 32 assertions, neuf fonctions machine non indépendantes.
Décision : https://github.com/eddarsglobal/nordoi/pull/180#issuecomment-7136413371

Une réservation persistante unique par identité locale du dossier (st_dev/st_ino), dans son parent canonique, est créée exclusivement par mkdir avant tout effet natif. Le nom ne dépend ni des pins ni d'un identifiant de tentative choisi par l'appelant. Même si le propriétaire meurt ou si l'écriture du reçu échoue, le marqueur n'est jamais supprimé ou repris automatiquement. APP-027 vérifie à nouveau toute la chaîne et effectue la continuation réelle APP-024. START.json et COMPLETE.json bornent les observations locales; la présence d'un reçu final n'est pas une preuve indépendante. Un dossier vide, un fichier obstacle ou une réservation existante bloque. Aucun changement des APP antérieurs ni du noyau Rust.

Limites explicites : seulement les appels coopératifs APP-028 sur le même dossier stable et un système de fichiers local fiable; les anciennes interfaces ne sont pas modifiées et ne respectent pas cette réservation. Pas de protection contre renommage/déplacement, substitution de dossier, suppression manuelle du marqueur, administrateur hostile, restauration de copie, autre hôte, ni crash-durabilité garantie. Aucune récupération automatique, même après succès. Une réservation persistante n'est pas un verrou OS libéré à la mort du processus. Ce choix conservateur sacrifie la disponibilité en cas d'incertitude. Les tests d'arrêt tuent le processus propriétaire de réservation, pas un moteur Rust au milieu d'une transaction.

Gate : 12 nouveaux tests dont véritables processus concurrents et continuation Rust réelle, 950 régressions, 962 par OS, 3848 attendus, 8 assertions par OS, 32 attendues, neuf fonctions machine et Rust/E8. C20 avant toute suite.
APP028_FORMAL_BUREAU_CERTIFICATION=NOT_CERTIFIED
R6F=IN_PROGRESS_UNCERTIFIED
R6G=LOCKED_BY_SEQUENCE
NATIVE_ADMISSION_GATES=12_UNMET
PRODUCTION=NOT_AUTHORIZED
MERGE=NONE
PR=DRAFT
