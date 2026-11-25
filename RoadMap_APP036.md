# NORDOI APP-036 — accusé de réception cryptographique
Candidat de recherche, 25 novembre 2026. Parent exact 48e908a93c4266d24563dad4afdc23a07c2951d5, PR #188, C20 22/22 SUCCESS, 4184 tests, 32 assertions, neuf contrôles machine non indépendants.
Décision : https://github.com/eddarsglobal/nordoi/pull/188#issuecomment-7154588470

Préparation exclusive d'un accusé canonique après audit APP-035 réussi et revérifié. L'accusé lie les identités, nonce, pins du transfert et de l'archive, clé de l'émetteur, clé du destinataire, empreinte du défi APP-035 et empreinte du reçu local. Les chemins locaux ne figurent pas dans le document. Signature SSHSIG Ed25519 externe avec la clé du destinataire et domaine nordoi-reception-ack@eddarsglobal. Vérification par l'émetteur sans accès au disque destinataire, avec contexte attendu et clé publique épinglée indépendamment du message. Aucun secret accepté par le programme. Une signature par la clé de l'émetteur au lieu de la clé destinataire autorisée est refusée lorsque les clés sont distinctes, comme dans les tests. Les entrées de confiance doivent être provisionnées par l'application.

Limites : signature d'une déclaration de réception, pas une attestation indépendante de l'état distant ni preuve de présence actuelle du dossier. La signature externe ne prouve pas que le signataire a utilisé le préparateur. Le destinataire de confiance doit respecter son protocole. Pas de réseau implémenté, chiffrement, consommation d'accusé, garde indépendante des clés, antirollback global, durabilité après panne, preuve d'identité matérielle ou sécurité contre un hôte hostile. Aucun reset, libération ou exécution, aucun changement des anciens APP ou du moteur Rust.

Gate : 12 nouveaux tests avec véritables signatures et preuves Rust, 1046 régressions, 1058 par OS, 4232 attendus, 8 assertions par OS, 32 attendues, neuf contrôles machine et Rust/E8.
APP036_FORMAL_BUREAU_CERTIFICATION=NOT_CERTIFIED
R6F=IN_PROGRESS_UNCERTIFIED
R6G=LOCKED_BY_SEQUENCE
NATIVE_ADMISSION_GATES=12_UNMET
PRODUCTION=NOT_AUTHORIZED
MERGE=NONE
PR=DRAFT
