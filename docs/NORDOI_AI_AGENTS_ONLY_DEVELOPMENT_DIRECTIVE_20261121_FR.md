# NORDOI — Directive permanente de développement par agents IA, pour agents IA

**Date : 21 novembre 2026. Instruction du fondateur.** Source structurée : [NORDOI_AI_AGENTS_ONLY_CONTINUATION_V1.json](../governance/NORDOI_AI_AGENTS_ONLY_CONTINUATION_V1.json).

## 1. Sens impératif

**NORDOI est développé par des agents IA pour des agents IA. Aucun humain ne doit être recruté, attendu ni sollicité pour débloquer la suite de la conception, du code, des tests, de la revue technique, des corrections, du packaging et de la qualification automatisable.**

Cette directive étend à **l'ensemble du développement** la politique E2 existante d'absence définitive de testeurs humains. Le fondateur dirige les objectifs et peut communiquer des choix volontaires ; les agents ne doivent pas transformer une action manuelle de sa part en prérequis technique répété.

Une assignation à un rôle IA ne prouve pas qu'un agent indépendant a effectivement travaillé. Produire les journaux réels ou marquer `NOT_EXECUTED`.

## 2. Workflow attendu (sans attente d'humains)

1. Sélectionner le prochain objectif fonctionnel concret conforme à la RoadMap, en privilégiant des fonctionnalités utiles aux **agents IA** : API et CLI lisibles par machine, diagnostics structurés, reproductibilité, contrats d'effets, packages, interopérabilité.
2. Vérifier le parent Git exact et la CI GitHub (Constitution C20) avant de commencer un nouveau jalon.
3. Les agents conçoivent, codent, compilent, exécutent les tests sur **de vrais binaires et plateformes** ; ils créent des contre-exemples adversariaux puis corrigent les défaillances jusqu'à un résultat vérifié.
4. Le Bureau exerce ses neuf **fonctions techniques** ; chaque preuve porte son SHA, son périmètre, ses résultats et ses manques. Un rapport du même agent n'est pas une signature tierce indépendante.
5. Les artefacts et commandes de laboratoire doivent être utilisables par un agent directement, sans clic, signature ni lecture de Terminal imposés à un humain.
6. Après `PASS` technique documenté, poursuivre **le jalon de développement utile suivant**. Ne pas réexécuter indéfiniment des laboratoires dont la seule conclusion est l'absence d'une autorité extérieure.

**Cette politique ne prétend pas fournir une exécution autonome en arrière-plan lorsque aucun agent ou système d'automatisation n'est réellement actif.** La possibilité technique et les autorisations réelles des outils restent observables, non inventées.

## 3. Pourquoi les verrous R6F / R6G existent

La RoadMap historique `RoadMap.md` distingue la **qualification technique de recherche** de l'**admission native indépendante**.

Les **douze barrières externes R6G** conservées dans `governance/NORDOI_R6F_BUREAU001_E12_TWELVE_GATE_INTAKE.json` incluent : racine d'autorité hors bande, identité d'un dépositaire réellement distinct, séparation administrative, attestation OS indépendante, défi signé neuf, protection des époques, nonces et révocations contre rollback, quorum et récupération indépendante.

**Ce ne sont pas douze interventions humaines de test à demander.** Ce sont des garanties de séparation et de confiance à observer dans des systèmes réellement distincts. Des agents IA peuvent rechercher et implémenter les mécanismes, mais **ne peuvent pas créer l'indépendance administrative par déclaration ou en lançant neuf rôles dans le même compte**. À défaut, cet axe de certification spécifique reste `UNMET` et on continue les autres développements.

Les **douze exigences R11-G01 à G12** de `governance/r11_native_readiness_gates_v1.tsv` sont quant à elles des défis de maturité du runtime natif : scheduling, cycle de vie, ownership, capacités, cleanup, I/O, crash, sécurité, portabilité, compilateur, ressources et revue. La plupart sont de **l'ingénierie vérifiable par machines** ; elles ne sont pas automatiquement satisfaites par les 12 preuves de R6G et ne doivent pas être confondues avec elles.

## 4. Deux pistes parallèles, sans faux succès

**A. Développement agentique continu :** écrire et publier des candidats de recherche exécutables, faire évoluer les fonctions NORDOI de manière contrôlée, qualifier techniquement les versions après succès des tests machine, produire leurs preuves et poursuivre selon C20. **Aucun humain requis.**

**B. Certification de production et activation native privilégiée :** conserver les contrôles d'autorité C3/C11/C12/C13 et la séparation des domaines, vérifier les racines, les autorisations et la résistance aux attaques avec des preuves réelles. Si impossible, **ne pas déclarer certifiée la capacité**. Le manque ne bloque pas A.

La directive du fondateur **n'est pas** une signature fabriquée d'un dépositaire extérieur, une désactivation des contrôles Guardian, ni une autorisation implicite de fusion des chaînes historiques ou de production. Elle **supprime la dépendance humaine du processus de développement**, sans falsifier des certificats.

## 5. Application aux jalons APP-001 à APP-003

APP-001, APP-002, APP-003 ont démontré des programmes `.noi` réels, sur Mac Intel et en CI quatre OS, sans avoir obtenu une certification native globale. **Cette distinction reste valide, mais elle n'interdit pas APP-004 et les jalons fonctionnels suivants.** Le choix du prochain jalon suit la RoadMap et sa preuve C20 ; les agents privilégient une valeur réelle pour les utilisateurs **agents IA**, et non une nouvelle répétition des motifs `R6G=LOCKED`.

## 6. Préservation des fichiers historiques

Cette directive est **additive**. Elle ne réécrit pas les sources figées `RoadMap.md`, `Bureau.md`, `CONSTITUTION.md` ou les contrats R6F/E2/E12. La garde historique conserve sa portée et les dossiers APP précédents ne sont pas fusionnés.

**Règle finale :** `HUMAN_DEPENDENCY_FOR_DEVELOPMENT=NONE`, `AGENTIC_DEVELOPMENT_CONTINUES_AFTER_GREEN_C20=YES`, `FAKE_EXTERNAL_AUTHORITY=FORBIDDEN`, `NATIVE_PRODUCTION_CERTIFICATION=SEPARATE`.
