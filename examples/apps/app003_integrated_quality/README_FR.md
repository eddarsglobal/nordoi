# NORDOI APP-003 — Un seul programme d'entrée `.noi`, imports et sorties P2.7

**20 novembre 2026 — Recherche contrôlée uniquement ; DRAFT, aucune production.**

## Résultat du développement

APP-001 exécute séparément une règle Profile 1 et une sortie Profile 2.
APP-002 offre un pont Rust qui demande deux programmes de publication.
**APP-003 apporte un vrai sous-profil de compilation de source unique :**
la déclaration d'entrée dans `src/app/main.noi` contient **à la fois** :
1. l'import de modules de règles purs ;
2. la sélection explicite `rules.approved(key_code)` ;
3. les deux blocs de fichiers de sortie P2.7, `approved` et `rejected`.

Les fonctions importées demeurent dans leurs véritables modules séparés,
comme le veut le mécanisme `import` ; **un seul fichier est la source de
l'entrée avec effets**, non un seul fichier pour toute l'application.

Un sous-compilateur natif **Rust du binaire `nordoi`** effectue une analyse
lexicale de cette entrée, contrôle sa grammaire limitée, crée en mémoire
deux plans de fichiers P2.7 et un graphe de modules V1.3, avant tout calcul
à effets. L'exécutable exécute ensuite la règle importée V1.3 (résultat
obligatoirement INT(0) ou INT(1)), puis publie seulement le bras sélectionné
avec le véritable moteur P2.7. **Le lanceur Python ne prend jamais la
décision.** Aucun accès Internet, aucun test humain requis.

## Syntaxe NORDOI réellement développée

```noi
module app.main;
import app.rules;

effect FileWrite;
input key_code;

entry main selects rules.approved(key_code) writes "quality-decision" {
    approved {
        "decision.txt" emits "approved=true, score=" + key_code + ", next=" + (key_code + 1) + "\n";
        "report.txt" emits "module-decision=approved, input=" + key_code + ", next=" + (key_code + 1) + "\n";
    }
    rejected {
        "decision.txt" emits "approved=false, score=" + key_code + ", next=" + (key_code + 1) + "\n";
        "report.txt" emits "module-decision=rejected, input=" + key_code + ", next=" + (key_code + 1) + "\n";
    }
};
```

Le fichier `app/rules.noi` définit la fonction `approved(score)` avec
seuil 60. Si le seuil change pour 90, **le score 73 est rejeté** par
l'exécution réelle du module NORDOI. Les fichiers écrits reflètent la
décision retenue, sans interpolation de logique par Python.

**Périmètre grammatical APP-003 strict :**
- déclarations dans cet ordre : `module`, 1–8 `import`, `effect FileWrite`,
  `input key_code`, `entry main selects alias.function(key_code)` ;
- un bundle dont les noms et les deux blocs de fichiers passent le compilateur
  P2.7 historique ; deux blocs obligatoires `approved` puis `rejected` ;
- les deux bras exigent **exactement les mêmes noms/capacités FileWrite** ;
- entrée de règle pure vérifiée par le compilateur/modules V1.3 historique ;
- résultat obligatoire INT(0) ou INT(1) ; tous les autres résultats refusés ;
- aucun effet ambiant, import arbitraire à effets, récursion ni nouveau opcode
  NAIR ; aucun accès aux fichiers autres que les deux FileWrite accordés.

Cette version **n'instaure pas** un système général `effect FileWrite` +
`import` dans tout le langage NORDOI ni une attestation cryptographique
globale réunissant les preuves V1.3 et P2.7. Elle est **un profil borné**
de compilation, testé et présenté comme tel.

## Lancer l'application sur le Mac Intel

Depuis la racine de cette branche GitHub :

```bash
python3 examples/apps/app003_integrated_quality/run_app003.py --score 73
```

Le programme Python valide uniquement l'entrée utilisateur 0..100, crée un
nouveau dossier privé sous Downloads, appelle le **vrai moteur Rust**
en mode Cargo hors-ligne et vérifie les octets des deux fichiers après
l'exécution. Il ne génère **aucun code .noi**, ne choisit **aucune branche**
et n'exécute **aucune simulation**.

Commande Rust directe, en fournissant un dossier vide déjà existant :

```bash
cargo run --offline --bin nordoi -- integrated-bundle-run \
  examples/apps/app003_integrated_quality/src app.main 73 \
  --grant-output-dir "$OUT"
```

Sortie du lanceur attendue :

```text
NORDOI_APP003=PASS
SOURCE_PROFILE=ONE_NOI_ENTRY_IMPORTS_PURE_RULE
SCORE=73
RULE_MODULES=2
RULE_RESULT=1
SELECTED_BRANCH=approved
decision.txt=approved=true, score=73, next=74
report.txt=module-decision=approved, input=73, next=74
APP003_NATIVE_COMPILER=V13_TO_P27_SINGLE_ENTRY
```

Un score 59 produit `RULE_RESULT=0`, `SELECTED_BRANCH=rejected`
et `approved=false`. Les sorties se trouvent sous un nouveau dossier
`NORDOI_APP003_.../quality-decision`, deux fichiers par exécution.

## Contrôles et gouvernance

**38 cas de validation automatique** sur le véritable exécutable NORDOI :
module source unique, règles importées, seuils 0/59/60/73/100,
mutation de la vraie règle, refus sans permission, refus de
réécriture, mauvais alias/fonction, import manquant ou ambigu, résultat
INT(2)/Bool, source inactive invalide, divergence de cibles, sortie
du bloc, injection lexicale dans une chaîne et reçus séparés authentiques.

Exécution CI : Mac Intel, Apple Silicon, Linux, Windows, plus
non-régressions APP-002 (26 tests), APP-001 (16 tests), Test Gate Rust
et régressions adversariales E8. Tous les tests et développements sont
automatisés ; **aucun testeur humain**.

Préalable C20 : APP-002 SHA `ac5a284cdedaea303ebdfc23b129459c872958df`,
suite exacte 4/4 APP-002, 5/5 Rust et 4/4 E8 SUCCESS. APP-003 ne peut
prétendre une qualification complète qu'après ses propres jobs
successifs exact-head. Aucune nouvelle signature ni autorité externe
fictive. R6F non certifié, R6G verrouillé et douze conditions natives
non satisfaites. **PR DRAFT, production non autorisée.**

**Limites de P2.7 conservées :** écriture create-new, 2 à 8 fichiers,
publication par renommage du dossier dans le même filesystem, pas de
durabilité après panne électrique, de transaction distribuée ou de
protection contre un adversaire local modifiant concurremment la cible.
