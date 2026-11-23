# NORDOI APP-023 — Ordonnancement concurrent borné de graphes IA

**23 novembre 2026 · Périmètre AI-only, recherche non privilégiée, aucune production**

## Décision RoadMap et Bureau

Parent exact APP-022 \`1fd98f87a776ef41a2367df0857c1a075e4a5d1a\` : 22/22 GitHub checks SUCCESS, 3 416/3 416 vrais tests sur 4 OS, 56/56 démonstrations natives, neuf fonctions de revue machine vérifiées au SHA. Qualification technique dans [PR #175](https://github.com/eddarsglobal/nordoi/pull/175#issuecomment-7109534873).

Le démarrage APP-023 est consigné séparément dans [PR #175, décision de recherche](https://github.com/eddarsglobal/nordoi/pull/175#issuecomment-7109571196). Cela ne constitue **ni un certificat formel indépendant du Bureau, ni une autorisation de modifier l'ancien noyau, ni une fusion**. Les 99 sources historiques figées et les douze verrous natifs restent protégés.

## Fonctionnalité réellement nouvelle

APP-009 exécute déjà un graphe de 1 à 8 tâches mais toujours **séquentiellement**. APP-023 réutilise exactement le format de graphe NORDOI_AGENT_DAG_V1 et son validateur : l'agent n'apprend aucun langage de déploiement ni une syntaxe d'effets nouvelle.

Un hôte choisit explicitement \`--max-in-flight=1\` ou \`2\`. Chaque tâche admise utilise un **véritable processus Rust NORDOI** \`agent-request-json\`, des sources \`.noi\` authentiques et une destination exclusive \`job-NNNN\` ; les sorties P2.7 sont revérifiées avec APP-006. Le plan JSON de l'agent ne peut fournir ni binaire, ni source, ni permission, ni chemin de destination.

L'ordonnanceur utilise la ThreadPoolExecutor standard bornée. Il admet en parallèle seulement les tâches indépendantes et attend que tous les parents nécessaires aient **terminé et réussi la vérification native** avant de lancer un enfant. Les décisions de branche \`approved\` ou \`rejected\` viennent uniquement des reçus Rust ; les tâches \`SKIPPED\` ne créent aucun dossier. Les événements JSONL sont toujours émis dans l'ordre canonique du graphe, **pas dans l'ordre d'arrivée des processus**.

Si une tâche est incertaine ou échoue, les processus de la vague déjà démarrée terminent et sont constatés avant le retour, et **aucune tâche supplémentaire n'est admise**. Les fichiers réellement publiés restent en place ; ni transaction globale ni restauration fictive. \`ERROR/publication=UNKNOWN\` doit conduire à un audit avant reprise.

## CLI agent

    python3 -B examples/apps/app023_bounded_concurrent_dag/run_app023.py \
      --plan /trusted/host/selected-agent-dag.json \
      --engine /trusted/nordoi \
      --source-root /trusted/noi/src \
      --grant-output-dir /host/explicit/existing-empty-dir \
      --expected-engine-sha256 TRUSTED_REAL_ENGINE_SHA256 \
      --expected-source-sha256 TRUSTED_REAL_SOURCE_SHA256 \
      --max-in-flight 2

Les pins proviennent de l'hôte/destinataire, **pas** du plan hostile. Le traitement respecte les mêmes contrats de chemins, octets et effets que APP-009. Les comptes des travailleurs et les acquittements sont bornés par le graphe.

## Gate Future-Native

- FNG1 : C5/C9/C11/C13/C19 ; amélioration réelle du modèle de tâche et de dépendance.
- FNG2 : aucun portage POSIX ou duplication de syntaxe héritée.
- FNG3 : une seule commande et une seule limite de concurrence, pas de configuration système complexe.
- FNG4 : effets explicites et résultats vérifiés, refus fermé d'un plan invalidé.
- FNG5 : règles de graphe portables sur les quatre OS.
- FNG6 : NORDOI Rust, NAIR, CLI, sources APP-001→022 et 99 blobs R6F non modifiés.

## Limites

La concurrence est réelle au niveau des **processus de tâches NORDOI**, mais le langage n'a **pas encore** de syntaxe native de graphes concurrents et aucun gain de débit n'est établi par cette démonstration. Pas de distribution multi-hôte, de scheduling équitable garanti, de durabilité après panne, de sandbox contre un administrateur hostile ou d'atomicité multi-tâches. Aucune admission privilégiée ni production.

## Qualification obligatoire

24 nouveaux tests APP-023 et 854 régressions = **878 tests par OS, 3 512 sur 4 OS**. Une démonstration de 14 assertions natives sur chaque OS, plus Rust Test Gate, E8, neuf examens machine Bureau spécifiques. Les preuves sont vérifiées au SHA exact de la PR DRAFT, sans exiger de testeurs humains.

APP023_FORMAL_BUREAU_CERTIFICATION=NOT_CERTIFIED
APP023_INDEPENDENT_ADMIN_CUSTODY=NOT_PROVEN
R6F=IN_PROGRESS_UNCERTIFIED
R6G=LOCKED_BY_SEQUENCE
NATIVE_ADMISSION_GATES=12_UNMET
MERGE=NONE
PRODUCTION=NOT_AUTHORIZED
