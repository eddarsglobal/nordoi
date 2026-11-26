# NORDOI APP-037 — registre sécurisé des accusés de réception
Candidat de recherche, 25 novembre 2026. Parent exact d96ac64488aed42bf194cda3c409463e0a02b492, PR #189, C20 22/22 SUCCESS, 4232 tests, 32 assertions, neuf contrôles machine non indépendants.
Décision : https://github.com/eddarsglobal/nordoi/pull/189#issuecomment-7170563877

Registre local de confiance côté émetteur. Préenregistrement exclusif du contexte attendu sous un nonce (identités, empreintes du transfert, archive, défi et clés). Capture bornée des octets de l'accusé, signature et clé publique, vérification APP-036 en espace privé, puis création exclusive du marqueur accepted. Écriture exclusive des octets vérifiés et du reçu final. Audit en lecture seule des preuves persistées, signature revérifiée, contexte et empreintes contrôlés. Un marqueur incomplet reste bloquant. Même accusé : DUPLICATE_ACK_BLOCKED. Accusé différent mais valide pour le même contexte : CONTRADICTORY_SIGNED_ACK_BLOCKED, avec empreintes dans la réponse, sans modification de l'enregistrement initial. La contradiction n'est pas persistée. Les appels concurrents peuvent obtenir UNCERTAIN_BLOCKED pendant une écriture, jamais une seconde acceptation. Aucun reset ou réessai automatique.

Limites : processus coopérants APP-037 partageant un registre stable sur filesystem local de confiance; aucun antirollback global ou protection contre restauration, effacement ou modification hostile du registre, ancien point d'entrée, panne électrique, ou processus malveillant. Persistance entre processus testée, durabilité après crash non garantie. Détection de contradiction à réception d'un nouvel accusé valide, pas surveillance globale ou journal persistant de conflits. L'accusé reste une déclaration signée, non une attestation indépendante de la machine distante. Pas de réseau, chiffrement, exécution ou libération. Aucun changement des anciens APP ou du moteur Rust.

Gate : 12 nouveaux tests, véritables signatures et preuves natives, 1058 régressions, 1070 par OS, 4280 attendus, huit assertions par OS, 32 attendues, neuf contrôles machine et Rust/E8.
APP037_FORMAL_BUREAU_CERTIFICATION=NOT_CERTIFIED
R6F=IN_PROGRESS_UNCERTIFIED
R6G=LOCKED_BY_SEQUENCE
NATIVE_ADMISSION_GATES=12_UNMET
PRODUCTION=NOT_AUTHORIZED
MERGE=NONE
PR=DRAFT
