# NORDOI G0.3 · R6F/BUREAU-001 — E3

## Correction d'une ambiguïté de provenance réelle dans les tests machines E2

**Directive propriétaire permanente : ZÉRO testeur humain.** E3 est un nouveau test de cohérence de preuve CI, pas une certification native ni une revendication d'authentification externe.

### Observation réelle de E2 (GitHub Actions)

La PR E2 #135 avait comme source de référence `33a677224feddc580b9507d6c3fa826cabe27004`. Le journal du job GitHub E2 [43932867642 / 133530096908](https://github.com/eddarsglobal/nordoi/actions/runs/43932867642) indique explicitement que le lanceur de neuf suites réelles a exécuté le checkout `4736162aee852e7b9bbe43cfe186121c632ec4bb`, c'est-à-dire la **référence de fusion synthétique de la PR**.

Le résultat E2 n'était pas inventé : **9/9 suites machines PASS** sur Mac Intel, logs de neuf rôles archivés et empreintes SHA-256 vérifiées, CI E2 spécialisée 5/5 et général Rust 5/5. Toutefois, **la référence effectivement testée n'était pas l'exact HEAD de la PR**. On ne doit pas recopier ce verdict en prétendant avoir directement testé le commit candidat.

GitHub a archivé l'artefact E2 `14258877979` et annoncé le SHA-256 de son ZIP `a7a884239cb8ef2979169ecba9e44578507738b9794a80ed5e80b736b7cc15ff`. Le registre E3 contient les **neuf empreintes de logs publiées dans le journal du job**. Une empreinte représente des octets, pas une signature de gardien indépendant. Le ZIP n'est pas secrètement authentifié par le fait que GitHub l'héberge, et sa durée de conservation est limitée.

### Nouvelle exigence de qualification machine E3

Le nouveau workflow E3 appelle `actions/checkout@v4` avec le champ `ref` explicitement fixé à `${{ github.event.pull_request.head.sha || github.sha }}`. Cette valeur, issue de l'événement de CI, est transmise séparément au contrôle `scripts/bureau_001_e3_exact_head_provenance.py` qui compare :
- `git rev-parse --verify HEAD^{commit}` au HEAD exact exigé ;
- `git status --porcelain` à une chaîne vide ;
- le champ `source_head_before_after` des preuves E2 produites lors de **ce nouveau job** au même HEAD exact ;
- les neuf rôles aux commandes réellement attendues, au compte de tests Rust/Python par rôle et à leurs sorties non nulles ou nulles ;
- le SHA-256 des **neuf vrais fichiers de journaux**, lus sans suivi de lien symbolique et avec vérification de l'inode avant lecture, aux empreintes déclarées dans la preuve E2 ;
- les limites obligatoires : **zéro testeur humain, zéro revue indépendante certifiée, douze portes non remplies, R6G verrouillé, production non autorisée**.

Le contrôle de vérification ne crée aucune signature externe. Il **ne peut pas empêcher un administrateur qui contrôle les deux sources d'éditer ensemble un journal et son manifeste** avant leur vérification. Le SHA-256 n'est pas un témoin externe. Son objectif est d'empêcher l'**attribution erronée** d'une preuve machine à un autre commit et de détecter les corruptions et substitutions simples de fichiers, pas d'inventer un antirollback global.

### Vingt-quatre tests adversariaux et qualification réelle

`scripts/test_bureau_001_e3_exact_head_provenance.py` comporte **24 tests portables** sur Mac Intel, Apple Silicon, Linux et Windows. Ils incluent de vraies opérations filesystem temporaires (fichiers réguliers, lien symbolique, remplacement atomique de journal, absence de journal), détection de double champ JSON, SHA erroné, commande falsifiée, faux spécialiste indépendant, compte de tests falsifié et autorité étrangère fictive. Les chaînes de tests utilisées par les tests de parsing sont **des fixtures contrôlées**, clairement distinctes des neuf suites réellement exécutées par un runner de CI.

Le workflow comprend un job **Mac Intel réel** qui relance séparément `scripts/bureau_001_e2_machine_only.py --run`, sur le **checkout explicitement épinglé à la tête E3**, puis le vérificateur E3 analyse la sortie des neuf rôles et leurs neuf journaux. Le job dépose ses résultats dans les artefacts GitHub uniquement après vérification. Les autres jobs n'ont pas le droit de se présenter comme détenteurs de neuf traces natives macOS.

La CI générale Rust continue en parallèle. Les originaux `RoadMap.md`, `Bureau.md`, le registre des 99 blobs R1 et les sources R1–E2 ne sont pas modifiés.

### Verdict gouvernance attendu

```ini
E2_REAL_MACHINE_SUITES=9/9_PASS_ON_OBSERVED_SYNTHETIC_MERGE
E2_CANDIDATE_HEAD=33a677224feddc580b9507d6c3fa826cabe27004
E2_ACTUAL_RUN_HEAD=4736162aee852e7b9bbe43cfe186121c632ec4bb
E3_NEW_SOURCE_HEAD=TO_VERIFY_FROM_REAL_PR_EVENT
E3_NINE_MACHINE_SUITES=NOT_YET_RUN_AT_DOCUMENT_CREATION
HUMAN_TESTERS=0
INDEPENDENT_SPECIALIST_REVIEWS=0
REAL_EXTERNAL_CUSTODIAN=NOT_ENROLLED
OWNER_OOB_ROOT=NOT_AUTHENTICATED
GLOBAL_MONOTONIC_ANTIROLLBACK=NOT_VERIFIED
R6F=IN_PROGRESS_UNCERTIFIED
R6G=LOCKED_BY_SEQUENCE
NATIVE_ADMISSION_GATES=12_UNMET
PRODUCTION=NOT_AUTHORIZED
```

Cette correction apporte un **nouveau fait falsifiable** au programme, et non une boucle R10 de répétition. La certification indépendante reste absente.
