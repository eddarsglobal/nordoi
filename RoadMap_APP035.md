# NORDOI APP-035 — liaison vérifiable réception/destination
Candidat de recherche, 25 novembre 2026. Parent exact c85657b5cdf91d7d6d078ab40b2a31c3c4d2d95a, PR #187, C20 22/22 SUCCESS, 4136 tests, 32 assertions, neuf contrôles machine non indépendants.
Décision : https://github.com/eddarsglobal/nordoi/pull/187#issuecomment-7154180640

Le destinataire lie le défi au chemin canonique attendu et à l'identité locale du dossier parent (st_dev, st_ino). Nouvelle entrée de registre incompatible avec l'ancien contexte APP-033. Vérification réelle SSHSIG et native en staging; seconde vérification du parent, consommation exclusive permanente du nonce, création exclusive de la destination, capture de son identité, publication et reçu de fin. L'audit revérifie le défi, les reçus, les identités, la signature et tous les octets des preuves. Refus d'une substitution par une copie identique conservant l'original, du déplacement, du parent remplacé, de la falsification des reçus et du rejeu. Aucune suppression, réactivation ou relance. L'échec après consommation conserve le blocage.

Périmètre : hôte et registre locaux de confiance, filesystem stable, identité disponible et non réutilisée; aucune garantie contre une modification concurrente hostile, une restauration du registre, le recyclage d'inodes, une panne électrique, un administrateur hostile ou un contournement par les anciens APP. La destination est liée par le registre local du destinataire, pas par la signature de l'émetteur. Pas de preuve d'identité matérielle cryptographique, de durabilité après crash ou d'anti-rejeu distribué. Aucun lien rétroactif prétendu pour APP-033. Aucun changement des anciens APP ni du moteur Rust.

Gate : 12 nouveaux tests (preuves et signatures réelles, processus concurrents, substitution identique, échec de publication injecté), 1034 régressions, 1046 par OS, 4184 attendus, 8 assertions par OS, 32 attendues, neuf contrôles machine et Rust/E8.
APP035_FORMAL_BUREAU_CERTIFICATION=NOT_CERTIFIED
R6F=IN_PROGRESS_UNCERTIFIED
R6G=LOCKED_BY_SEQUENCE
NATIVE_ADMISSION_GATES=12_UNMET
PRODUCTION=NOT_AUTHORIZED
MERGE=NONE
PR=DRAFT
