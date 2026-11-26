# NORDOI APP-038 — journal permanent des contradictions cryptographiques
Candidat de recherche, 26 novembre 2026. Parent exact 1d8ff5694a45f9e46d43452e4533a587cc65c02e, PR #190 FIX1, C20 22/22 SUCCESS, 4280 tests, 32 assertions, neuf contrôles machine non indépendants.
Décision : https://github.com/eddarsglobal/nordoi/pull/190#issuecomment-7184349927

Le journal est distinct du registre APP-037. L'appelant fournit les attentes de confiance et un journal local préexistant. Relecture et revérification de l'accusé accepté, capture bornée et vérification APP-036 du nouvel accusé. Un document identique n'est pas une contradiction même si la signature diffère. Une déclaration différente et valide pour le même contexte est conservée par nonce et SHA-256 du document contradictoire. Création exclusive, copies exactes des deux documents, signatures et clés publiques; reçu canonique final liant contexte, premier reçu et toutes les empreintes. Audit ciblé par empreinte, revérification des deux signatures et comparaison au registre APP-037. Une entrée incomplète reste bloquée et aucune réactivation automatique n'est proposée. Une seconde publication de la même contradiction est refusée. Plusieurs contradictions distinctes peuvent être enregistrées.

Périmètre : processus coopérants partageant un journal et registre locaux stables de confiance. Aucun changement des anciens APP ou du moteur Rust. Journal permanent au sens de conservation sans suppression par APP-038, non journal inviolable contre un hôte hostile, restauration, effacement, panne électrique ou modification concurrente hostile. La collecte est explicite, pas interception automatique des appels APP-037. Audit par entrée, pas preuve d'exhaustivité ni journal global ordonné. Les preuves d'une contradiction attestent des déclarations signées incompatibles, pas l'état réel distant ni une intention malveillante. Le registre APP-037 original reste nécessaire à l'audit; aucune attestation indépendante de la garde des clés. Aucun droit de réexécution, libération, reset, transport ou production.

Gate : 12 nouveaux tests, signatures réelles et preuves natives, concurrence, panne d'écriture injectée, 1070 régressions, 1082 par OS, 4328 attendus, 8 assertions par OS, 32 attendues, neuf contrôles machine et Rust/E8.
APP038_FORMAL_BUREAU_CERTIFICATION=NOT_CERTIFIED
R6F=IN_PROGRESS_UNCERTIFIED
R6G=LOCKED_BY_SEQUENCE
NATIVE_ADMISSION_GATES=12_UNMET
PRODUCTION=NOT_AUTHORIZED
MERGE=NONE
PR=DRAFT
