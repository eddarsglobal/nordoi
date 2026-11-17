# NORDOI G0.3 — R6F / BUREAU-001 E4

## Démonstration réelle : falsification cohérente d'un journal et de son manifeste

**Aucun humain ne testera NORDOI. Aucun testeur humain contacté. Aucun gardien externe fictif.**

### Base réelle E3 et limites

Le résultat E3 réellement observé est [CI #43949772851](https://github.com/eddarsglobal/nordoi/actions/runs/43949772851), sur le commit exact `655ea42e5d4c7a62fb1506243c15706044b6ae29` : **9/9 véritables suites machines Mac Intel PASS**, et neuf journaux SHA-256 contrôlés. Le manifeste publié portait le SHA-256 `4e910cfa9fc1f8aad64027148b68dc621abeec3915207e30877af5daac039e4a`. Les **cinq** tests généraux Rust sont PASS. Pour la CI spécialisée E3, Mac Intel, Linux, Windows et le job de neuf suites ont terminé SUCCESS ; le dernier job Apple Silicon a été **CANCELLED, et non PASS**. Il est interdit de convertir ce résultat historique en 10/10 ou en 5/5 spécialisés.

Cette référence prouve qu'un programme GitHub a exécuté des suites Rust/Python et que ses propres journaux se correspondent au commit. Elle ne prouve pas l'identité d'un administrateur extérieur, un historique monotone ou la disponibilité permanente des artefacts.

### Hypothèse falsifiable

*Si un seul administrateur peut écrire à la fois les neuf journaux et leur manifeste JSON, il peut modifier un journal réel, recalculer son SHA-256 et modifier le manifeste correspondant. Le vérificateur E3 peut accepter ce nouveau couple comme cohérent, même si le contenu diffère d'un état antérieur.* Les contrôles E3 de vrai git HEAD et de résultat de test resteraient cohérents avec le même SHA.

Un fichier et son hash ont une **propriété de cohérence**, pas une propriété d'authenticité indépendante. E4 recherche la preuve concrète de ce défaut, sans déclarer que des adversaires externes ont été présents ou qu'une attaque de production s'est produite.

### Exercice E4 : vraies données machines, vrais fichiers et processus OS

Le nouveau `scripts/bureau_001_e4_coherent_evidence_tampering.py` :

1. Vérifie le HEAD exact de la PR par le contrôle E3 et l'identité des sources gelées R1/E2/E3.
2. Lance sur **Mac Intel réellement exécuté** les neuf suites machines d'E2, avec leurs log files et manifeste de la référence E4. Il ne fabrique pas le résultat d'un test.
3. Relance le vérificateur E3 **dans un autre processus OS** sur les neuf journaux originaux, pour obtenir une première vérification.
4. Retient une **empreinte préalable locale** de ces neuf journaux et du manifeste.
5. Copie le manifeste et les neuf journaux **dans un répertoire temporaire différent**, en préservant les originaux.
6. Effectue un remplacement de fichier `os.replace` atomique dans la **copie** : ajoute un marqueur d'expérience au journal Rust ATLAS, conserve la sortie Rust réellement produite et ses vrais compteurs, recalcule le SHA-256 et réécrit de manière cohérente le manifeste copié.
7. Lance E3 dans un **nouveau processus OS** sur la copie altérée. Le résultat positif attendu pour l'expérience est qu'E3 **accepte à tort comme non modifiée** la preuve cohérente réécrite.
8. Compare la copie avec l'**empreinte préalable inchangée** : E4 doit détecter la discordance.
9. Réécrit ensuite **uniquement l'empreinte préalable dans le laboratoire** pour démontrer que cette protection locale cesse de détecter la modification si la même autorité peut également modifier l'ancre.
10. Contrôle l'intégrité des neuf fichiers originaux et du HEAD Git après l'expérience ; aucune mutation n'est appliquée au dépôt, à la véritable archive E3 ni à un système de production.

Le jeu de données est réel (tests OS/macOS), mais l'altération est **une expérience adversariale volontaire sur copie**. Les 24 tests portables ajoutés incluent des opérations sur fichiers réels et des *fixtures* de parsing clairement distinguées des véritables sorties natives. Le job Mac Intel de CI est seul habilité à déclarer la démonstration complète sur les véritables journaux.

### Deux conclusions de sécurité distinctes

**Contrôle E3 d'une preuve sans référence antérieure protégée :** si la copie modifiée et son manifeste concordent, le contrôle peut donner un verdict PASS. Ce ne sera pas une vraie preuve que le journal est originel.

**Contrôle E4 avec empreinte obtenue avant modification :** la copie modifiée échoue tant que l'empreinte préalable demeure inchangée. En revanche, si la même administration peut réécrire **aussi** cette empreinte, la différence de génération historique est de nouveau indémontrable sur la seule base des fichiers locaux.

En conséquence, E4 n'annoncera **aucun remède cryptographique complet** à l'antirollback ou à la falsification par même administrateur. Une garantie plus forte exigerait un témoin effectivement séparé et authentifié, dont l'historique de clés/nonce/révocation reste inaccessible à l'administrateur local. Cette preuve **n'existe pas encore** dans NORDOI.

### Politique et décision

```ini
E4_HYPOTHESIS=COHERENT_LOCAL_LOG_AND_MANIFEST_REWRITE
E4_ACTUAL_NINE_MACHINE_LOGS=REQUIRED_ON_REAL_CI_MAC_INTEL
E4_PRIOR_SNAPSHOT=LOCAL_UNAUTHENTICATED_NOT_EXTERNAL_WITNESS
E4_SAME_ADMIN_REWRITE_LIMIT=NOT_ELIMINATED
HUMAN_TESTERS=0_FORBIDDEN
INDEPENDENT_SPECIALIST_REVIEWS=0
REAL_EXTERNAL_CUSTODIAN=NOT_ENROLLED
OWNER_OOB_ROOT=NOT_AUTHENTICATED
GLOBAL_MONOTONIC_ANTIROLLBACK=NOT_VERIFIED
R6F=IN_PROGRESS_UNCERTIFIED
R6G=LOCKED_BY_SEQUENCE
NATIVE_ADMISSION_GATES=12_UNMET
PRODUCTION=NOT_AUTHORIZED
```

Le laboratoire fonctionne **sans recrutement, sans intervention humaine de test, sans clés de tiers et sans fusion de PR**. Le propriétaire conserve la responsabilité distincte d'autoriser une transition privilégiée selon la Constitution ; cette autorisation administrative n'est pas un test humain.
