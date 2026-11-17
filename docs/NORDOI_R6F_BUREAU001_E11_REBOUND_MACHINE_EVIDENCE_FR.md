# NORDOI G0.3 — R6F / BUREAU-001 — E11
## Dossier d'exécution machine E10 : réécriture cohérente et contrôle aux vrais octets Git

**Propriétaire fondateur :** Noureddine Eddars. **Ingénierie, programmation, tests, documentation, formalités : IA et automatisation réelle. Aucun testeur humain.**

### Prérequis C20 réellement vert

Le parent exact E10 FIX1 est `ae84f34795c680a6929453f0d4723bec47189a4e` ; GitHub Actions [E10 spécialisée 4/4](https://github.com/eddarsglobal/nordoi/actions/runs/44067965884) et [Rust général 5/5](https://github.com/eddarsglobal/nordoi/actions/runs/44067966070) ont terminé SUCCESS. Le Mac Intel a obtenu 24/24 tests E10 et 6/6 tests R4M XPC/CDHash natifs en série. E10 est techniquement qualifié pour la recherche, **pas certifié par un organisme indépendant**.

### Nouvel adversaire E11

E10 crée des vrais rapports en neuf processus machines distincts et un manifeste contenant le SHA256 de chaque rapport. Son vérificateur local vérifie le format, l'ordre, les identités de rôle, les PID, les empreintes calculées des rapports et l'état non autorisé. Cependant il **ne recalcule pas, lors d'une vérification différée, chaque valeur `source_sha256` du rapport par rapport aux véritables octets Git**.

Un seul administrateur peut donc, dans une copie de laboratoire E10 :
1. produire les neuf vrais rapports machines ;
2. changer la prétendue empreinte d'un fichier source dans le rapport ATLAS ;
3. recalculer le SHA256 du rapport entier dans le manifeste ;
4. présenter les deux fichiers cohérents au vérificateur E10 ;
5. constater que l'ancien vérificateur accepte ce dossier *localement cohérent* tout en maintenant `BLOCK` ;
6. appliquer le garde E11 qui confronte tous les fichiers source revendiqués aux **vrais octets du dépôt au HEAD exact**, et constater le rejet du mensonge ;
7. restaurer exactement les deux fichiers originaux pour confirmer le contrôle positif.

Il s'agit d'une preuve négative **réelle sur le disque** : les fichiers temporaires sont effectivement modifiés. Le correctif E11 est **additif** ; E10, E9 et le runtime Rust restent inchangés. Le test `LEDGER` vérifie également une autre modification cohérente.

### Limites fondamentales

Le succès du nouveau garde ne signifie pas qu'une preuve issue du **même administrateur** est une attestation extérieure. La possession du dépôt et du vérificateur sous le même domaine n'assure pas le contrôle externe d'une histoire monotone, d'un quorum ou d'un dépositaire indépendant. Des copies inconnues ou supprimées peuvent encore manquer. E11 ne prétend pas régler ces problèmes.

Les neuf fonctions du Bureau conservent le rôle de vérification technique automatisée, sans compter neuf certificats indépendants. Les formalités de production sont préparées par IA mais **non réputées accomplies** sans preuves réelles.

### Qualification par machines

24 nouveaux tests E11 sur Windows, Ubuntu, Mac Intel et Apple Silicon, expérience de falsification réelle à chaque CI, E2–E10 régressions en lecture seule, RoadMapGuardian, audit des sources R1, et sur macOS six contrôles natifs R4M XPC/CDHash en série. Rust Test Gate général distinct, contrôlé au même SHA Git PR HEAD. Tous les chemins positifs de laboratoire terminent en code `2` avec `BLOCK` ; erreur ou preuve absente termine code `3`.

Les preuves sur Mac Intel sont publiées dans GitHub Actions avec SHA256 du résultat machine. Pas de fusion, de connexion externe ni d'élévation native.

```ini
E11=IN_PROGRESS_UNQUALIFIED_UNTIL_EXACT_HEAD_CI
E11_LOCAL_REPORT_REHASH_PROOF=TO_BE_TESTED
E11_SOURCE_REBIND_GUARD=TO_BE_TESTED
E11_INDEPENDENT_WITNESS=NONE
HUMAN_TESTERS=0_PERMANENTLY_FORBIDDEN
FOUNDER_MANUAL_TESTS=NONE
BUREAU_FORMAL_INDEPENDENT_REVIEWS=0
R6F=IN_PROGRESS_UNCERTIFIED
R6G=LOCKED_BY_SEQUENCE
NATIVE_ADMISSION_GATES=12_UNMET
PRODUCTION=NOT_AUTHORIZED
PR_MERGE=NOT_AUTHORIZED
```
