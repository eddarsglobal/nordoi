# NORDOI APP-033 — consommation locale unique des nonces
Candidat de recherche, 25 novembre 2026. Parent exact 2210c7029ccc5c8d23a4e8161c78434b15ee94ae, PR #185, C20 22/22 SUCCESS, 4040 tests, 32 assertions, neuf contrôles machine non indépendants.
Décision : https://github.com/eddarsglobal/nordoi/pull/185#issuecomment-7153315986

Le destinataire émet un nonce de 256 bits (secrets.token_hex), lié aux identités et pins attendus, enregistré exclusivement dans un registre local préexistant et de confiance. Les entrées non émises ou incohérentes sont refusées. La signature et les preuves natives APP-032 sont vérifiées dans un espace temporaire avant consommation. Un mkdir exclusif sur le marqueur permanent de consommation départage les processus concurrents avant publication. Même un marqueur vide bloque. La publication écrit le marqueur d'archive en dernier puis vérifie les preuves et écrit le reçu final de consommation. Tout échec après consommation laisse le nonce bloqué, sans reprise automatique. Aucun droit d'exécution ou de libération.

Périmètre : processus coopérants APP-033 partageant le même registre stable sur un hôte local de confiance. Pas de sécurité contre un hôte hostile, déplacement ou restauration du registre, appels directs aux anciens APP, réémission volontaire d'un nouveau défi, anti-rejeu distribué, durabilité après perte d'alimentation, transport chiffré, expiration automatique, enrôlement ou garde indépendante des clés. La persistance après redémarrage de processus est testée; pas de garantie de redémarrage système brutal. Le test de défaillance injectée concerne la publication locale, pas une panne matérielle réelle. Aucun changement des APP précédents ni du moteur Rust.

Gate : 12 nouveaux tests (signatures et preuves natives réelles, processus concurrents, marqueur incomplet, échec de publication injecté), 1010 régressions, 1022 par OS, 4088 attendus, 8 assertions par OS, 32 attendues, neuf fonctions machine et Rust/E8.
APP033_FORMAL_BUREAU_CERTIFICATION=NOT_CERTIFIED
R6F=IN_PROGRESS_UNCERTIFIED
R6G=LOCKED_BY_SEQUENCE
NATIVE_ADMISSION_GATES=12_UNMET
PRODUCTION=NOT_AUTHORIZED
MERGE=NONE
PR=DRAFT
