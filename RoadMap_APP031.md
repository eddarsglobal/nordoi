# NORDOI APP-031 — transfert portable de preuves épinglées
Candidat de recherche, 24 novembre 2026. Parent C20 exact 19504a25c614ce063904b17adad6ef1642aa601e, PR #183, 22/22 SUCCESS, 3944 tests, 32 assertions, neuf contrôles machine non indépendants.
Décision : https://github.com/eddarsglobal/nordoi/pull/183#issuecomment-7140100684

Export canonique JSON/Base64 non compressé d'une archive APP-030 vérifiée. Limites : 16 MiB paquet, 8 MiB décodés, 82 entrées au maximum et noms de fichiers strictement autorisés; pas de chemins arbitraires, liens ou doublons. Réception avec deux SHA-256 attendus fournis hors paquet (archive et transfert), validation syntaxique et sémantique en espace temporaire privé, puis publication exclusive dans une destination neuve, marqueur final en dernier. Réaudit APP-030 du DAG natif après publication. Aucun accès à l'archive d'origine requis. Aucune autorisation de libération ou d'exécution. Les tests exécutent le destinataire dans un autre processus local sur chaque OS, pas sur deux machines physiques reliées entre elles. La portabilité intermachines est visée, pas encore démontrée par un transfert réseau réel.

Limites : pas de chiffrement ni d'authentification de l'émetteur, de canal réseau implémenté, de distribution fiable des pins, d'anti-rejeu distribué ni de durabilité après crash. Base64 n'est pas du chiffrement. Hôte local stable de confiance; pas de concurrence hostile sur les dossiers. Les archives partielles ou destinations existantes ne sont pas écrasées. Le pin du paquet ne remplace pas le pin indépendant de l'archive. Aucun changement des APP précédents ni du moteur Rust.

Gate : 12 nouveaux tests avec véritables preuves Rust, 986 régressions, 998 par OS, 3992 attendus, 8 assertions par OS, 32 attendues, neuf fonctions machine et Rust/E8.
APP031_FORMAL_BUREAU_CERTIFICATION=NOT_CERTIFIED
R6F=IN_PROGRESS_UNCERTIFIED
R6G=LOCKED_BY_SEQUENCE
NATIVE_ADMISSION_GATES=12_UNMET
PRODUCTION=NOT_AUTHORIZED
MERGE=NONE
PR=DRAFT
