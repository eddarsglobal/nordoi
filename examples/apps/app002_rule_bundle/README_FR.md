# NORDOI APP-002 — Pont natif Rust : module importé → sortie P2.7

**20 novembre 2026. Recherche uniquement ; ni intégration en production, ni certification R6F/R6G.**

## Ce qui a réellement été développé

APP-001 possédait deux parcours séparés :
- un module pur V1.3 qui calculait un résultat 0/1 ;
- une sortie P2.7 calculant indépendamment sa propre décision.

**APP-002 ajoute une commande au véritable exécutable Rust NORDOI :**

```text
nordoi module-bundle-write <source-root> <entry-module> <approved.noi> <rejected.noi> <key-code> [--grant-output-dir <dir>]
```

C'est **le moteur NORDOI V1.3** qui exécute réellement un graphe de deux modules .noi, valide `INT(0)` ou `INT(1)`, sélectionne l'un des deux programmes de sortie P2.7 et publie un bundle de deux fichiers en conservant le score initial comme entrée P2.7.

L'**exécutable Rust unique** réalise toute la séquence, avec réutilisation directe du runtime de modules existant et de la bibliothèque de publication P2.7 réelle. Python ne choisit **jamais** la branche. En changeant la règle importée de seuil 60 vers seuil 90, un score 73 passe effectivement de la branche `approved` à `rejected`, sans changer le code Rust et sans logique de décision Python.

### Protection par défaut

- Une autorisation explicite `--grant-output-dir` est obligatoire avant calcul de la règle.
- Les **deux** programmes de sortie sont lus et compilés avant toute exécution à effet.
- Les noms de bundle et les listes **exactes de capacités FileWrite** doivent être identiques dans les deux branches. Différence ou source invalide : refus avant publication.
- Un résultat de règle autre que exactement `INT(0)` ou `INT(1)` est refusé, y compris `INT(2)` et `BOOL(true)`.
- L'exécution P2.7 sélectionnée n'obtient que les capacités des **deux fichiers** explicitement vérifiés, et réutilise le score de départ.
- Écrasement refusé, sortie partielle interdite à travers le chemin P2.7, durabilité après coupure non garantie.
- Sources, importations et limites V1.3/V1.4 d'origine inchangées.

**Portée précise** : il s'agit d'une **composition native de deux profils sous une commande NORDOI**, pas encore de `import` ou d'effets P2.7 arbitraires **dans un unique module source .noi**. Il ne s'agit pas non plus d'une preuve cryptographique nouvelle d'atomicité entre le calcul pur et la publication. Les reçus sous-jacents sont exposés séparément. Aucune nouvelle primitive NAIR, aucune permission ambiante ou sortie réseau.

## Lancer la première application

Depuis la racine de la branche APP-002 :

```bash
python3 examples/apps/app002_rule_bundle/run_app002.py --score 73
```

Le lanceur n'est qu'un adaptateur facultatif : il crée un dossier nouveau dans `~/Downloads`, transmet le score au vrai moteur Rust, et revérifie les octets des deux fichiers. Le moteur Rust invoqué par Cargo utilise `--offline`.

Pour un score 73, les sorties attendues sont :

```text
NORDOI_APP002=PASS
SCORE=73
RULE_MODULES=2
RULE_RESULT=1
SELECTED_BRANCH=approved
decision.txt=approved=true, score=73, next=74
report.txt=module-decision=approved, input=73, next=74
```

Pour 59 : `RULE_RESULT=0`, `SELECTED_BRANCH=rejected`, `approved=false`.
Le score 60 est accepté. Le lanceur refuse une entrée hors 0..100 avant de créer le moindre dossier.

Utilisation native directe, sans Python, en accordant explicitement un dossier de sortie **déjà existant et neuf** :

```bash
cargo run --offline --bin nordoi -- module-bundle-write \
  examples/apps/app002_rule_bundle/rules/src app.main \
  examples/apps/app002_rule_bundle/approved.noi \
  examples/apps/app002_rule_bundle/rejected.noi \
  73 --grant-output-dir "$OUT"
```

## Preuves et gouvernance

26 contrôles Python automatisés **sur le vrai binaire Rust**, Mac Intel, Apple Silicon, Linux, Windows : seuils 0/59/60/73/100 ; mutation réelle de la règle dans une copie ; résultat INT(2) ou BOOL, absence d'import, absence de grant, branches divergentes, source inactive invalide, même source pour les deux branches, refus d'écrasement, reçus, lancement autonome. Les tests APP-001 et les suites Rust générales restent exécutés.

Le fondateur dirige les choix de produit, et l'automatisation réalise les tests : aucun testeur humain ni obligation de tests manuels du fondateur.

**C20 préalable :** APP-001 SHA `ec80edd47cf861315976289baa725b4e64fabe0e`, CI 4/4 + Rust 5/5 + E8 4/4. APP-002 reste DRAFT et NON AUTORISÉ en production, indépendamment du succès CI. R6F non certifié, R6G verrouillé, douze conditions natives non satisfaites, aucune signature externe inventée.
