# NORDOI APP-027 — continuation native depuis un checkpoint reconstruit
Candidat de recherche, 24 novembre 2026. Parent C20 exact e61ae006d6bab8ae3e139c8992d12e12e7cad96e, PR #179, 22/22 SUCCESS, 3752 tests et 32 assertions, neuf revues machine non indépendantes.
Décision : https://github.com/eddarsglobal/nordoi/pull/179#issuecomment-7134407054

APP-027 exige cinq pins SHA-256 fournis par l'hôte (ancien paquet, suffixe observé, nouveau paquet, moteur, source), un consentement explicite et un espace de tentative neuf. Ré-audit APP-025 du préfixe étendu, vérification APP-014 des deux paquets, correspondance du plan, du préfixe, de l'inventaire historique et du dossier de travail. La continuation se fait dans le dossier partiellement exécuté déjà audité, distinct des deux paquets, et non dans une restauration automatique supplémentaire. Les seuls nouveaux effets natifs proviennent de l'exécuteur APP-024 existant, borné à un ou deux workers. Capture du suffixe dans une tentative exclusive, puis audit APP-025 complet. START.json avant exécution et COMPLETE.json seulement après confirmation. Un échec ou interruption après création de tentative signifie résultat inconnu, sans relance automatique. Aucun nouveau privilège Rust, pas de modification des anciens APP.

Limites : hôte de confiance quiescent, pas de registre global anti-rejeu, verrou global interprocessus, résistance à concurrence hostile, authentification externe attestée ni durabilité post-crash. Le reçu local ne prouve pas une autorité indépendante et n'empêche pas un hôte de restaurer volontairement une copie ancienne ailleurs.

Gate : 12 nouveaux tests utilisant les vrais résultats Rust (états partiels construits à partir de preuves authentiques), 938 régressions, 950 par OS, 3800 attendus, 8 assertions par OS, 32 attendues, 9 fonctions machine et Rust/E8. C20 avant toute suite.
APP027_FORMAL_BUREAU_CERTIFICATION=NOT_CERTIFIED
R6F=IN_PROGRESS_UNCERTIFIED
R6G=LOCKED_BY_SEQUENCE
NATIVE_ADMISSION_GATES=12_UNMET
PRODUCTION=NOT_AUTHORIZED
MERGE=NONE
PR=DRAFT
