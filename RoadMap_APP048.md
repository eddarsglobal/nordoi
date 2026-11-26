# NORDOI APP-048 — registre sécurisé des accusés de réception authentifiés
Candidat de recherche, 26 novembre 2026. Parent exact 874ced03c3603935935b21ae5e8eec8ed0bd5ddb, PR #200, C20 22/22 SUCCESS, 4760 tests, 32 assertions, neuf contrôles machine non indépendants.
Décision : https://github.com/eddarsglobal/nordoi/pull/200#issuecomment-7200912196

Registre émetteur préenregistré par nonce et contexte stable (identités, clé des deux parties, archive, plan, ancre et tête). Les empreintes du dossier APP-046 et du reçu sont fournies séparément à la vérification APP-047 par le contexte de confiance appelant, et non acceptées automatiquement du message entrant. Capture bornée des quatre fichiers, vérification des octets capturés, relecture des entrées puis création exclusive du marqueur accepted, START, preuves, COMPLETE. Audit avec rejeu exact des preuves conservées et comparaison du reçu canonique. Refus des doublons et des autres déclarations authentiques pour le même contexte, sans écrasement de la première décision. Marqueur incomplet bloquant. Le test contradictoire utilise deux registres de laboratoire distincts avec le même défi, deux signatures SSHSIG véritables utilisant des algorithmes de hachage différents, deux dossiers et deux accusés authentifiés; aucune indépendance administrative revendiquée.

Périmètre : unicité par nonce dans un registre local stable de confiance partagé par processus coopérants. La contradiction désigne deux déclarations canoniques authentifiées différentes sous même contexte stable, pas une preuve de fraude ni deux décisions logiquement opposées. Les preuves contradictoires ne sont pas conservées par APP-048. Audit sous confiance dans le registre local et les attentes enregistrées; pas de protection contre son remplacement ou sa restauration malveillante, ni durabilité électrique, anti-rejeu distribué, fraîcheur ou preuve indépendante de l'état distant. Aucune exécution, libération, réinitialisation, production ou fusion. Anciennes sources et RoadMap centrale inchangées.

Bureau : ATLAS composition; FERRUM exclusivité et interruption; BOUNDARY séparation des entrées; CIPHER authentification; LEDGER conservation; CAPSTONE refus des effets; ADVERSARY concurrence et contradictions; VERITY tests; GUARDIAN séquence C14/C20 et verrous. Contrôles machine déterministes, non indépendants.
Gate : 12 nouveaux tests, 1190 régressions, 1202 par OS, 4808 attendus, 8 assertions par OS, 32 attendues.
APP048_FORMAL_BUREAU_CERTIFICATION=NOT_CERTIFIED
R6F=IN_PROGRESS_UNCERTIFIED
R6G=LOCKED_BY_SEQUENCE
NATIVE_ADMISSION_GATES=12_UNMET
PRODUCTION=NOT_AUTHORIZED
MERGE=NONE
PR=DRAFT
