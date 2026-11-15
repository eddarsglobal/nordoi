# NORDOI — Où en est le projet ? (explication pour débutant)

**État de référence : 15 novembre 2026, G0.3 R6F / BUREAU-001 R2 en développement de recherche.** La branche candidate n'est pas une nouvelle version certifiée du langage.

## NORDOI en une phrase

NORDOI est un **langage de programmation expérimental** et une architecture d'exécution visant à rendre les programmes explicites, atomiques et sécurisés. Le compilateur et le moteur de base existent réellement ; le projet final visé est plus large.

## Ce que vous pouvez déjà faire

Le dépôt contient `src/bin/nordoi.rs`, les modules de compilation Rust, les examples `.noi`, les tests natifs et les mécanismes de permissions. Le guide `docs/NORDOI_G02_FIRST_USE_QUICKSTART.md` documente cet exemple :

```noi
module app.main;
effect ConsoleWrite;
entry main emits "Hello from NORDOI P2.1!\n";
```

Commande :

```bash
cargo run --bin nordoi -- run examples/p2_1_console/hello.noi --grant-console
```

Le résultat attendu est `Hello from NORDOI P2.1!`. Le drapeau `--grant-console` est une autorisation **distincte** de la déclaration `effect ConsoleWrite`.

Les profils de sortie P2.1–P2.7 couvrent notamment la console et des sorties vers des fichiers bornés. Le modèle P2.7 peut publier ensemble un répertoire contenant **2 à 8 fichiers**, selon des limites précises. Il ne prétend pas pouvoir lire tous les fichiers, utiliser Internet, garantir la durabilité après panne de courant ou résister à des modifications hostiles concurrentes du répertoire par une autre application. Référence : `docs/NOI_ATOMIC_MULTI_FILE_BUNDLE_OUTPUT_SPEC_P2_7.md`.

## Trois niveaux de maturité à ne pas confondre

1. **Socle certifié :** `docs/GOVERNANCE_PROGRESS.md` enregistre **K1.18**, Production Profile 1 `v1.7` et Production Profile 2 `p2.1`–`p2.7`, avec référence certifiée `p2.7` au commit `0a634d6cab52074e18b42d1ffea00ce02cdba359`.
2. **Développement et recherche G0.3 :** de vrais tests Rust natifs, isolation, signatures SSHSIG, accusés de réception, contrôle des falsifications et conservation de preuves sont examinés. Ces laboratoires ne créent pas automatiquement de nouvelles fonctionnalités du langage accessibles à un programme utilisateur ni une autorisation de production.
3. **Bureau R6F :** neuf fonctions spécialisées structurent les audits. La CI R6F R1 était **9/9 SUCCESS** et le propriétaire a déclaré **1 087 tests réguliers PASS** sur Mac Intel, avec **188/188 empreintes historiques** et **99/99 empreintes de l'audit R1**. Ces réussites ne démontrent pas neuf agents autonomes exécutés. En R2, les neuf sources sont indexées pour contrôle et les vrais journaux locaux peuvent être examinés sans en certifier l'origine.

## Combien du projet complet est fait ?

Le modèle de planification **historique G0.1** estimait **58,35 % de NORDOI v1 réalisé**, avec **41,65 % restant** ; il s'agissait d'une estimation pondérée, **non certifiée**. Les **310 principes sur 322**, soit **96,27 %**, ayant reçu des éléments de preuve dans leur périmètre explicite sont une autre mesure — **cela ne signifie pas que le langage est terminé à 96,27 %**. Aucun pourcentage actualisé et audité n'est établi par R6F R2.

Les travaux importants restants incluent : **langage généraliste et outils développeurs**, mémoire et tâches concurrentes, calcul CPU/GPU/NPU, exécution en réseau, rendu spatial universel 2D/3D/XR, plateformes et écosystème. Voir `governance/nordoi_v1_delivery_model_v1.tsv` et `docs/NORDOI_CONSTITUTIONAL_CONFORMANCE_MATRIX_G0_1.md`.

## Où en sommes-nous sur la sécurité ?

Le Bureau s'intéresse actuellement à un problème précis : **comment savoir qu'une preuve ou une signature provient d'une autorité réellement indépendante plutôt que de deux clés appartenant au même administrateur ?** Les laboratoires R6A–R6E vérifient de vraies opérations cryptographiques, mais l'enrôlement externe indépendant reste **absent**. **Douze conditions d'admission natives ne sont pas encore satisfaites**, l'étape R6G est bloquée et la production n'est pas autorisée.

**Résumé : un langage et un socle sûrs dans un périmètre borné fonctionnent déjà ; une partie considérable du projet NORDOI v1 reste à concevoir, intégrer, tester et qualifier.** Le Bureau R6F vise à protéger cette progression contre les validations insuffisamment prouvées.
