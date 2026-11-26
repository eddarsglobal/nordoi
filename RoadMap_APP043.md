# NORDOI APP-043 — moteur d'archivage autonome des preuves cryptographiques
Candidat de recherche, 26 novembre 2026. Parent exact bbe7e464614cb41c3a4c98beeebaf2395cc8172d, PR #195, C20 22/22 SUCCESS, 4520 tests, 32 assertions, neuf contrôles machine non indépendants.
Décision : https://github.com/eddarsglobal/nordoi/pull/195#issuecomment-7189521977

Archive JSON canonique, membres base64, liste fermée de chemins normalisés et budgets de 32 Mio pour archive, 20 Mio de membres décodés, 16384 membres, 256 Kio par membre. Audit APP-042 avant collecte, sélection des seuls fichiers de preuves attendus par les manifestes, contextes et décisions. Rejeu dans un répertoire temporaire isolé sous plan relocalisé dérivé du plan d'origine épinglé. Recollecte exacte et comparaison des membres, comparaison de l'ancre, de la tête, de l'empreinte des décisions et du nombre de transitions. Nouvelle vérification des sources avant publication exclusive. Audit portable sous trois empreintes de confiance externes : archive, plan original, ancre. Aucun répertoire original requis au rejeu. Tests réels de relocalisation, déterminisme, suppression, falsification de signature, traversée de chemin, membre supplémentaire et exclusion de fichiers étrangers.

Périmètre : archivage déclenché explicitement, pas automatique à chaque transition; archive de preuves signées non chiffrée, pas garantie générale de confidentialité du contenu des preuves. Aucun fichier de clé privée sélectionné par le collecteur. Les empreintes doivent être fournies indépendamment et ne prouvent pas la fraîcheur ni le consensus distribué. Pas de protection antirollback globale, durabilité électrique, résistance à un écrivain hostile concurrent ni exhaustivité antérieure. Les anciens APP et moteur Rust restent inchangés. Aucun reset, libération, exécution native ou production.

Gate : 12 nouveaux tests, 1130 régressions, 1142 par OS, 4568 attendus, 8 assertions par OS, 32 attendues, neuf contrôles machine et Rust/E8.
APP043_FORMAL_BUREAU_CERTIFICATION=NOT_CERTIFIED
R6F=IN_PROGRESS_UNCERTIFIED
R6G=LOCKED_BY_SEQUENCE
NATIVE_ADMISSION_GATES=12_UNMET
PRODUCTION=NOT_AUTHORIZED
MERGE=NONE
PR=DRAFT
