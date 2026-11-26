# NORDOI APP-039 — scellement vérifiable du journal des contradictions
Candidat de recherche, 26 novembre 2026. Parent exact 80d03ed6eb83eb3ac1afd2722821c487cc3d5650, PR #191, C20 22/22 SUCCESS, 4328 tests, 32 assertions, neuf contrôles machine non indépendants.
Décision : https://github.com/eddarsglobal/nordoi/pull/191#issuecomment-7184974571

Inventaire canonique borné de toutes les entrées visibles du journal fourni, jusqu'à 64 contradictions et 64 contextes. Les attentes indépendantes sont fournies par fichier canonique et empreinte externe; leur ensemble de nonces doit correspondre exactement aux dossiers présents. Chaque contradiction est auditée par APP-038, avec vérification des deux signatures et de l'accusé APP-037 accepté. Le manifeste inclut les huit empreintes de fichiers par entrée, les empreintes des reçus, le nombre d'entrées et l'empreinte de l'inventaire. Deux lectures et audits successifs identiques sont exigés. Le sceau est publié exclusivement hors du registre et journal; vérification en lecture seule sous empreinte du sceau conservée indépendamment, reconstruction et comparaison des octets. Détection de suppression, ajout, substitution ou altération par rapport au sceau épinglé.

Périmètre : instantané du journal visible, environnement local stable et de confiance, attentes externes fiables. Ne garantit ni collecte exhaustive des contradictions, ni absence de suppressions avant scellement, ni résistance à un écrivain hostile concurrent, ni fraîcheur de l'empreinte externe, ni antirollback global, ni durabilité après panne. Le sceau est une empreinte SHA-256, pas une signature d'une autorité indépendante. Les ajouts légitimes nécessitent un nouveau sceau, sans écrasement automatique. Les anciens APP et moteur Rust restent inchangés. Aucun reset, exécution native, libération ou production.

Gate : 12 nouveaux tests avec signatures et preuves natives, 1082 régressions, 1094 par OS, 4376 attendus, 8 assertions par OS, 32 attendues, neuf contrôles machine et Rust/E8.
APP039_FORMAL_BUREAU_CERTIFICATION=NOT_CERTIFIED
R6F=IN_PROGRESS_UNCERTIFIED
R6G=LOCKED_BY_SEQUENCE
NATIVE_ADMISSION_GATES=12_UNMET
PRODUCTION=NOT_AUTHORIZED
MERGE=NONE
PR=DRAFT
