# NORDOI APP-047 — signature des preuves d'acceptation par le destinataire
Candidat de recherche, 26 novembre 2026. Parent exact 2ff01d93c456a58cd5b33ed3e40ad2b5ac4eecd3, PR #199, C20 quatre workflows completed/success, 22 conclusions SUCCESS, 4712 tests, 32 assertions, neuf contrôles machine. ATLAS/FERRUM : champ individuel in_progress incohérent avec conclusion success et workflow terminé; journaux au SHA exact confirmant PASS et nettoyage final.
Décision : https://github.com/eddarsglobal/nordoi/pull/199#issuecomment-7198730340

Préparation d'une déclaration canonique liée au dossier APP-046, reçu, défi, identités et clé publique du destinataire distincte de la clé de l'émetteur. Audit du registre actif APP-045 avant publication exclusive, comparaison exacte au dossier APP-046, relecture. Signature détachée externe SSHSIG Ed25519, namespace nordoi-archive-admission-ack@eddarsglobal. Vérification de la clé indépendamment épinglée, déclaration exacte et signature puis rejeu APP-046 sur octets capturés. Aucun accès au registre distant requis pour vérifier. Aucun paramètre de clé privée. Tests réels, mauvaise clé, mauvais namespace, substitution d'identités ou contexte, dossier ou déclaration falsifiés, reçu actif corrompu, relocalisation et redémarrage.

Limites : déclaration du destinataire, pas attestation indépendante de l'acceptation historique, de l'état actuel du nonce, de l'enrôlement ou garde de clé indépendants. Pas de fraîcheur, consommation du reçu, antirollback global ni chiffrement. Empreintes et identités attendues de confiance externes. Aucune modification des anciens APP, noyau Rust, registre actif ou autorisations. C3/C14/C20 maintenus.

Bureau : ATLAS composition, FERRUM isolation et vérification SSHSIG, BOUNDARY séparation des données, CIPHER signatures/domaines, LEDGER liens de preuves, CAPSTONE refus des effets, ADVERSARY falsifications, VERITY tests quatre OS, GUARDIAN règles. Contrôles machine déterministes non indépendants.
Gate : 12 nouveaux tests, 1178 régressions, 1190 par OS, 4760 attendus, 8 assertions par OS, 32 attendues.
APP047_FORMAL_BUREAU_CERTIFICATION=NOT_CERTIFIED
R6F=IN_PROGRESS_UNCERTIFIED
R6G=LOCKED_BY_SEQUENCE
NATIVE_ADMISSION_GATES=12_UNMET
PRODUCTION=NOT_AUTHORIZED
MERGE=NONE
PR=DRAFT
