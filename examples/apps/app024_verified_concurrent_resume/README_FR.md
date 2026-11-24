# NORDOI — APP-024 : reprise concurrente vérifiable après checkpoint réel

**24 novembre 2026 — branche de recherche AI-only non privilégiée.**

## Respect RoadMap / Bureau / Guardian

APP-024 suit APP-023, commit exact \`45bac739f00deb710f8a6065d3ef616c810b8476\`, **22/22 checks GitHub SUCCESS** incluant le E8 Apple Silicon rejoué. **3 512/3 512 tests natifs et 56/56 assertions de démonstration PASS sur quatre OS**. La décision de démarrage est consignée dans le commentaire https://github.com/eddarsglobal/nordoi/pull/176#issuecomment-7125777174. Les neuf fonctions MACHINE de revue ont réussi ; aucune certification indépendante du Bureau n'a été émise.

Cet addendum versionné ne modifie pas les 99 blobs historiques R6F ni la RoadMap originelle. Aucun droit de production ou d'admission native privilégiée. R6F IN_PROGRESS_UNCERTIFIED, R6G LOCKED_BY_SEQUENCE et 12 gates non satisfaites.

## Fonctionnalité réelle pour agents IA

Un agent peut demander de **reprendre en concurrence** un DAG de 1 à 8 tâches depuis un checkpoint portable APP-014 réellement produit par NORDOI Rust, scellé APP-013 et indépendamment vérifié par APP-010. L'hôte doit avoir déjà restauré les fichiers P2.7 avec APP-014 RESTORE dans une racine accordée. APP-024 vérifie :

1. **L'intégralité** du checkpoint APP-014 à partir d'un SHA-256 fourni hors du paquet ; aucune signature/externe authenticité n'est inventée.
2. Le plan, le journal natif historique, tous les reçus APP-010/006 et **tous les vrais octets P2.7 restaurés**, avant toute nouvelle écriture.
3. Les pins séparés du vrai binaire Rust et de la source .noi, l'absence d'ambiguïté du préfixe, des dossiers restants ou de réexécution.
4. Les parents validés du DAG avant tout enfant. Seules les tâches **nouvelles** exécutent un vrai processus Rust \`nordoi agent-request-json\`, limité à **1 ou 2 tâches simultanées**.
5. Les résultats de reprise sont des événements APP-009 JSONL **uniquement pour le suffixe**, triés par index du DAG, que l'auditeur indépendant APP-010 revérifie contre la **concaténation** préfixe original + suffixe réussi.

Aucune copie des anciens jobs dans le chemin de reprise, aucune réexécution des jobs historiques, et aucune permission/path tirée du JSON non fiable.

### Utilisation après APP-014 RESTORE

    python3 -B examples/apps/app024_verified_concurrent_resume/run_app024.py \
      --portable-dir /host/pinned/checkpoint-portable \
      --expected-portable-sha256 SHA256_PINNED_HORS_PAQUET \
      --restore-root /host/authorized/already-restored-root \
      --engine /host/trusted/nordoi \
      --source-root /host/trusted/noi-src \
      --expected-engine-sha256 SHA256_DU_BIN_RUST \
      --expected-source-sha256 SHA256_DES_SOURCES_NOI \
      --max-in-flight 2

Codes de sortie : **0** suffixe complet et audité ; **2** REJECTED avant nouvel effet ; **3** erreur native ou audit inconclusif, fichiers précédemment publiés préservés et effets en cours **UNKNOWN**. En cas d'échec parallèle, les tâches déjà démarrées sont drainées ; aucun faux rollback ni prétention que le suffixe partiel est un checkpoint propre immédiatement réutilisable.

## Gate FNG1–6

FNG1 = C5/C9/C11/C13/C19 et lacune réelle reprise concurrente. FNG2 = aucune copie de POSIX. FNG3 = même format DAG et un choix de concurrence 1|2. FNG4 = pins, grants, audit indépendant avant/après, fail-closed pour états inconnus. FNG5 = standard Python + vrai Rust sur macOS Intel/ARM, Linux, Windows. FNG6 = noyau, Rust, .noi, historique APP-001…023, Constitution/RoadMap/Bureau originels intouchés.

## Preuves et limites

24 nouveaux tests APP-024 + **878** régressions = **902 tests par OS**, **3 608** sur quatre OS, plus **14** assertions de démonstration native par OS. La CI doit contrôler le SHA exact, E8, Rust et les neuf fonctions de revue MACHINE avant verdict. Les jobs machine ne constituent pas neuf agents IA indépendants.

Le système ne garantit pas un sandbox OS, un dépositaire indépendant, la fraîcheur globale après redémarrage, la récupération automatique d'un effet in-flight inconnu, une transaction multi-tâches, la résistance à un administrateur hôte malveillant, ni des gains de vitesse mesurés.

APP024_FORMAL_BUREAU_CERTIFICATION=NOT_CERTIFIED
R6F=IN_PROGRESS_UNCERTIFIED
R6G=LOCKED_BY_SEQUENCE
NATIVE_ADMISSION_GATES=12_UNMET
PRODUCTION=NOT_AUTHORIZED
MERGE=NONE
PR=DRAFT
