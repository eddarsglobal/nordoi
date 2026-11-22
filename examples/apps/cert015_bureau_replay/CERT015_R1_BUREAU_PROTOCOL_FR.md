# NORDOI — CERT-015-R1 | Audits supplémentaires avant APP-016

**Date : 22 novembre 2026 · Revue Bureau / Guardian · AI-only · Recherche non privilégiée**

## Décision de séquence

**APP-016 = HOLD**, sans développement, PR, mutation ni exécution d'APP-016 dans cette révision. La certification supplémentaire demandée porte exclusivement sur **APP-015**, au parent immuable **`7db397628d50a6e7484b89313516e68b3645d951`**, qui dispose déjà d'une qualification CI antérieure **13/13 SUCCESS** (4 tests applicatifs, 5 Rust, 4 E8). PR #166 toujours DRAFT et non fusionnée. Ce dossier n'est pas un raccourci de gouvernance : il impose **un second adversaire méthodologique** avant de décider la suite.

## Risque non résolu par l'ancien test

APP-014 vérifie l'intégrité interne d'un paquet portable, **sans recalcul indépendant**. APP-015 rejoue réellement la logique native avec le moteur et les sources .noi **épinglés**. Pour tester la distinction, CERT-015-R1 crée sans mock de règles métier **deux historiques P2.7 entièrement cohérents** avec deux sources de règles Rust/NOI différentes :
- Paquet **ORIGINAL** : fichier `rules.noi` avec décision `score >= 60`, deux tâches historiques via Rust NORDOI.
- Paquet **ALTERNATE** : fichier `rules.noi` de la même base, mais décision `score >= 1000`, deux tâches historiques réellement recalculées par **le même moteur Rust**.
- Dans chaque cas : **APP-012 → APP-013 → APP-014** exécutés sur de vrais fichiers P2.7, avec empreintes cohérentes et exactes de chaque source et paquet.
- L'audit de **cohérence APP-014 doit accepter les deux**. La **qualification APP-015 avec la source originale indépendamment épinglée doit accepter ORIGINAL et refuser ALTERNATE** en comparant le reçu natif entier et les deux fichiers. Si l'agent destinataire choisit explicitement la **source ALTERNATE** et son empreinte, le rejeu d'ALTERNATE peut réussir : **cela prouve pourquoi l'authenticité EXTERNE des pins reste une obligation**.

Ce test expose honnêtement la frontière de confiance, sans inventer de signature ni de seconde implémentation cryptographique des témoins P2.7.

## Qualification indépendante R1

**20 nouveaux tests** (non dérivés du fixture de `test_app015.py`) exécutent les vrais programmes natifs. Ils couvrent :
- Distinction entre paquet valide et paquet cohérent mais produit par **une autre source métier**.
- Refus avant tout effet des pins faux, manifests altérés, fichiers P2.7 modifiés, marqueurs tronqués et dossiers natifs supplémentaires.
- Aucune restauration d'un paquet incohérent avec la source épinglée ; aucune écriture dans la destination si le replay échoue.
- Restauration seulement après `PASS` ; fichiers historiques recopiés octet pour octet ; second `restore` refusé sans écrasement.
- Continuation des seules tâches futures via Rust véritable, conservation de l'historique et audit APP-010 du graphe final `PASS`.
- Double reprise refusée, aucune modification des originaux et du paquet sous vérification.
- Déclaration correcte de **`native_engine_sandbox=NOT_PROVIDED`**, **`external_pin_authentication=NOT_PERFORMED`**, **`production_admission=NOT_AUTHORIZED`**.

La CI certifiante conserve également l'entièreté des **568 tests APP-001…APP-015 par OS**. Objectif : **588 tests / plateforme, 2 352 exécutions applicatives** sur Mac Intel, Apple Silicon, Linux, Windows, plus les jobs Rust Test Gate **5** et E8 adversarial **4** : **13 jobs** au même commit candidat. Les journaux du nouveau laboratoire sont archivés dans les artifacts GitHub avec un manifeste du SHA-256 du transcript.

## Bureau — conditions de certification (neuf axes documentaires)

1. **ATLAS** : le parent APP-015 qualifié est identique, les 20 tests CERT-015 testent des chemins non redondants et APP-016 reste HOLD.
2. **FERRUM** : le véritable Rust produit les deux historiques cohérents et les replays, sans substitution d'une décision simulée.
3. **BOUNDARY** : aucune restauration ni reprise de fichier natif si la qualification est `REJECTED` ou `INCONCLUSIVE`.
4. **CIPHER** : la confrontation source originale / source modifiée révèle la dépendance réelle à une autorité externe des pins, **sans signature ni sandbox inventés**.
5. **LEDGER** : l'inventaire, les reçus JSON complets et les fichiers P2.7 sont comparés, les chemins de preuve sont épinglés et bornés.
6. **CAPSTONE** : `PASS` signifie exacte reproduction native uniquement. Un manifest cohérent ne donne **pas** une preuve d'identité, une provenance externe ou une production autorisée.
7. **ADVERSARY** : des historiques cohérents contradictoires, et pas seulement des octets arbitrairement corrompus, doivent être refusés par le moteur/source originellement épinglés.
8. **VERITY** : 20/20 tests nouveaux sur quatre OS, 568/568 régressions par OS, 5/5 Rust et 4/4 E8 au **même SHA**. Tout échec bloque la décision.
9. **GUARDIAN** : zéro humain attendu pour développement ou tests ; aucune fusion, privilège R6G, activation production, ni faux organisme certificateur.

**Ces neuf axes ne signifient pas neuf agents ou auditeurs externes indépendants**. Les contrôles peuvent être qualifiés par le Bureau comme revue technique interne ; aucune **certification de sécurité indépendante** n'est revendiquée.

## Barrières et décisions

Au départ :
```ini
CERT015_R1=NOT_YET_CERTIFIED
BUREAU_VERDICT=PENDING_EVIDENCE
APP016=HOLD
PR=DRAFT
MERGE=NONE
R6F=IN_PROGRESS_UNCERTIFIED
R6G=LOCKED_BY_SEQUENCE
EXTERNAL_NATIVE_ADMISSION_GATES=12_UNMET
PRODUCTION=NOT_AUTHORIZED
```

Après qualification machine et revue des preuves, un commentaire de décision du Bureau pourra émettre `PASS_WITH_CONTROLS_RESEARCH_ONLY` **strictement sur le SHA qualifié**. Un PASS ne change pas à lui seul la politique R6F/R6G et **n'entraîne pas le lancement d'APP-016** : sa séquence doit recevoir une décision distincte. Ne jamais appeler cette décision une signature ou une attestation indépendante de sécurité.
