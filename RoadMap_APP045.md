# NORDOI APP-045 — acceptation unique des archives authentifiées
Candidat de recherche, 26 novembre 2026. Parent exact 0af591eda95db4d397a935c997a97689b43f36c6, PR #197, C20 22/22 SUCCESS, 4616 tests, 32 assertions, neuf contrôles machine non indépendants.
Décision : https://github.com/eddarsglobal/nordoi/pull/197#issuecomment-7193422231

Défi de 256 bits généré par le destinataire, lié aux attentes canoniques APP-044. Enregistrement exclusif du défi; vérification intégrale de l'archive et signature avant consommation atomique du nonce par création exclusive du dossier consumed. Marqueur permanent même si publication interrompue. Reçu canonique lié au défi, vérification, signature, déclaration, archive, tête et décisions. Audit en lecture seule par rejeu des preuves originales fournies et comparaison exacte au reçu conservé. Tests réels de processus concurrents, redémarrage, signature incorrecte avant consommation, nonce non émis, contexte erroné, interruption, falsification du reçu, défi distinct.

Périmètre : processus coopérants sur un registre local stable de confiance. Unicité par nonce, pas par archive indépendamment des défis; un nouveau défi et une nouvelle signature valide peuvent autoriser une autre acceptation de preuve. Pas de preuve de fraîcheur temporelle, consensus distribué, antirollback global, durabilité électrique ou résistance à un écrivain hostile. Le reçu n'est pas une attestation indépendante et les preuves signées ne sont pas recopiées dans le registre. Aucun changement des anciens APP ni moteur Rust, aucune exécution, libération, reset ou production.

Gate : 12 nouveaux tests, 1154 régressions, 1166 par OS, 4664 attendus, 8 assertions par OS, 32 attendues, neuf contrôles machine et Rust/E8.
APP045_FORMAL_BUREAU_CERTIFICATION=NOT_CERTIFIED
R6F=IN_PROGRESS_UNCERTIFIED
R6G=LOCKED_BY_SEQUENCE
NATIVE_ADMISSION_GATES=12_UNMET
PRODUCTION=NOT_AUTHORIZED
MERGE=NONE
PR=DRAFT
