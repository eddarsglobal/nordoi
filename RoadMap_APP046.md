# NORDOI APP-046 — conservation autonome des preuves d'acceptation
Candidat de recherche, 26 novembre 2026. Parent exact d2b8dde03613707dd149153a5f5a6a8b939d7cdb, PR #198, C20 22/22 SUCCESS, 4664 tests, 32 assertions, neuf contrôles machine non indépendants.
Décision : https://github.com/eddarsglobal/nordoi/pull/198#issuecomment-7197774600

RoadMap.md et Bureau.md consultés : C3 refus par défaut, C14 preuves avant évolution, C20 parent vert. Aucun changement de statut certifié ou d'admission.

Dossier JSON canonique borné à 48 Mio, sept membres nommés exactement : archive, déclaration, signature détachée, clé publique, défi, START et COMPLETE. Capture bornée; audit APP-045 des sources; rejeu du dossier dans un registre temporaire isolé; correspondance exacte du reçu épinglé extérieurement; relecture des sources avant publication exclusive. Vérification autonome sous empreintes externes du dossier et du reçu et attentes APP-044. Aucun fichier de clé privée. Tests réels d'acceptation, relocalisation des cinq entrées originales, déterminisme, reçu ou signature modifiés, membres manquants ou supplémentaires, marqueur incomplet, redémarrage et absence d'écrasement.

Périmètre : conservation de preuves, pas attestation indépendante de l'acceptation passée. Un registre reconstruit n'est jamais un registre actif de consommation; le nonce courant et la fraîcheur ne sont pas vérifiés. Le reçu local doit être épinglé par une source de confiance extérieure au dossier. Pas de preuve de garde indépendante, consensus distribué, antirollback global, durabilité électrique ou résistance à un écrivain hostile. Dossier non chiffré, production non autorisée. Ancien code et moteur Rust inchangés.

## Assistance Bureau — périmètres de revue machine
ATLAS : composition et rejeu APP-045. FERRUM : isolation temporaire et absence de mutation des sources. BOUNDARY : liste fermée des membres et budgets. CIPHER : authentification et empreintes indépendantes. LEDGER : provenance exacte du reçu. CAPSTONE : absence d'effets autorisés. ADVERSARY : falsification et interruption. VERITY : qualification quatre OS et régressions. GUARDIAN : séquence C14/C20 et verrous. Ces contrôles sont déterministes, pas neuf audits indépendants.

Gate : 12 nouveaux tests, 1166 régressions, 1178 par OS, 4712 attendus, 8 assertions par OS, 32 attendues, neuf contrôles machine et Rust/E8.
APP046_FORMAL_BUREAU_CERTIFICATION=NOT_CERTIFIED
R6F=IN_PROGRESS_UNCERTIFIED
R6G=LOCKED_BY_SEQUENCE
NATIVE_ADMISSION_GATES=12_UNMET
PRODUCTION=NOT_AUTHORIZED
MERGE=NONE
PR=DRAFT
