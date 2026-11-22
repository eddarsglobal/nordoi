# NORDOI APP-016 — Politique de confiance propre au destinataire

**22 novembre 2026 · Recherche IA → IA · Zéro testeur humain requis**

## RoadMap / C20 / Bureau

APP-016 est **démarré uniquement après** la qualification supplémentaire CERT-015-R1 `f8455e602076ffabc49abf4f123cb8d1e70bc639` : **13/13 jobs GitHub SUCCESS**, 2 352/2 352 tests applicatifs, dont 80 tests indépendants de falsification cohérente. La PR #167 demeure DRAFT, ouverte et non fusionnée. Le commentaire de libération de séquence est archivé dans la conversation de la PR #167 ; **la libération porte exclusivement sur un développement de recherche APP-016**, pas sur une certification, une fusion, une permission privilégiée ou la production.

## Problème vérifié et solution

CERT-015-R1 a montré que deux ensembles natifs P2.7 produits par le véritable Rust avec des règles .noi différentes pouvaient **tous deux être cohérents** et passer APP-014. APP-015 refuse le faux historique seulement lorsqu'il rejoue avec **la source correcte et épinglée séparément**. Si un expéditeur malveillant convainc le récepteur d'utiliser sa propre source et son propre SHA-256, le rejeu peut réussir.

**APP-016 retire à l'expéditeur le pouvoir de proposer les références de confiance.** Le destinataire définit son **propre fichier de politique de réception**, séparé du paquet portable, avec son **empreinte SHA-256 attendue obtenue par un canal distinct**. Le paquet n'est admis que s'il concorde avec cette politique préexistante, qui fixe explicitement les quatre empreintes :
- `engine_sha256` : binaire Rust que le récepteur accepte d'exécuter ;
- `source_tree_sha256` : arbre de règles .noi que le récepteur accepte ;
- `portable_sha256` : **un seul checkpoint portable approuvé** ;
- `plan_sha256` : plan DAG métier attendu.

La politique précise également **les actions autorisées** ; elle n'est jamais chargée depuis le paquet de l'expéditeur. Un paquet portant des hashes cohérents ne peut modifier cette politique ni obtenir des droits supplémentaires.

### Exemple de fichier de politique canonique

La politique contient exactement ces champs, sans alias ni clés supplémentaires :

```json
{
  "schema": "NORDOI_AGENT_RECEIVER_POLICY_V1",
  "policy_id": "receiver-lab-016",
  "engine_sha256": "SHA256_VERIFIE_DU_MOTEUR_64_HEX",
  "source_tree_sha256": "SHA256_VERIFIE_DES_SOURCES_NOI_64_HEX",
  "portable_sha256": "SHA256_VERIFIE_DU_MANIFESTE_PORTABLE_64_HEX",
  "plan_sha256": "SHA256_VERIFIE_DU_PLAN_64_HEX",
  "allowed_actions": ["inspect", "restore", "resume", "verify"],
  "release": "RESEARCH_ONLY",
  "pin_origin": "RECEIVER_SUPPLIED_NOT_EXTERNALLY_ATTESTED",
  "signature": "NOT_PRESENT",
  "production": "NOT_AUTHORIZED"
}
```

*Illustration de structure seulement* : les valeurs de SHA-256 ci-dessus sont des noms explicatifs et **ne constituent pas** des données utilisables. Le véritable fichier doit être **JSON canonique** (clés triées, séparateurs compacts, une fin de ligne LF), sans doublon, avec des SHA-256 réels sur 64 caractères hexadécimaux minuscules. La liste des actions doit être triée sans doublons. Le récepteur obtient **indépendamment** le hash de ce fichier : `--expected-policy-sha256`. Cette empreinte n'est pas une signature et le module n'en certifie pas la provenance.

### Les quatre actions du destinataire

**1. `inspect`** — valide l'ancrage externe de la politique, ses permissions, les empreintes du checkpoint portable et les fichiers P2.7 via APP-014/013/010/006, **sans exécuter Rust** et sans écrire dans le dossier de réception :

```bash
python3 -B examples/apps/app016_receiver_trust_policy/receiver_app016.py inspect \
  --receiver-policy /receiver/trusted/policy.json \
  --expected-policy-sha256 RECEIVER_TRUSTED_POLICY_SHA256 \
  --portable-dir /received/portable-checkpoint
```

**2. `verify`** — même barrière, puis **rejeu authentique APP-015** dans un répertoire temporaire privé, avec les **pins récupérés uniquement de la politique**, et comparaisons exactes des reçus P2.7 et fichiers natifs :

```bash
python3 -B examples/apps/app016_receiver_trust_policy/receiver_app016.py verify \
  --receiver-policy /receiver/trusted/policy.json \
  --expected-policy-sha256 RECEIVER_TRUSTED_POLICY_SHA256 \
  --portable-dir /received/portable-checkpoint \
  --engine /receiver/trusted/nordoi \
  --source-root /receiver/trusted/noi/src
```

**3. `restore`** — nécessite l'action `restore` dans la politique, rejoue le passé par APP-015, revérifie la politique et le paquet **avant tout effet dans une racine native existante, vide et distincte**, puis délègue la création exclusive à APP-014.

**4. `resume`** — nécessite l'action `resume` dans la politique, reprend la même vérification/rejeu, puis délègue au véritable Rust via APP-014/011 uniquement les tâches futures ; ne produit **que le suffixe JSONL natif**. Une seconde reprise ou une racine altérée est refusée.

Pour `restore` et `resume`, ajouter `--restore-root /receiver/empty-or-previously-restored/native-root` aux paramètres de `verify`.

### Sémantique et frontières de sécurité

- **Code 0** : `POLICY_MATCH` pour `inspect`, `PASS` pour `verify`, `RESTORED` pour `restore`, ou suffixe natif réel pour `resume`. `POLICY_MATCH` ne signifie pas que Rust a été rejoué ; `PASS` ne signifie ni signature ni certification de production.
- **Code 2, `REJECTED`, `publication=NOT_STARTED`** : politique ou empreinte invalide, paquet altéré, action interdite, moteur/source non autorisé, reçu recalculé contradictoire, destination non conforme. Aucun nouvel effet sur la **destination**.
- **Code 3, `INCONCLUSIVE`, `publication=UNKNOWN`** : moteur natif incapable de conclure ou publication/restauration au résultat incertain. Aucune fausse conclusion `PASS` ou annulation globale.
- Les chemins de l'exécutable et des sources sont **sélectionnés par le récepteur**, puis leurs octets doivent correspondre à la politique. Les chemins embarqués dans le paquet ne deviennent jamais des arguments d'exécution ou des permissions.
- La politique doit être un vrai fichier borné (4 096 octets maximum) et non un symlink, situé **à l'extérieur du paquet portable et de la racine restaurée**. L'empreinte externe de la politique est obligatoire.
- **Limites non résolues** : qui authentifie la politique et son SHA-256 ? Le récepteur/son autorité de confiance en dehors de ce programme. Un attaquant qui remplace **à la fois** la politique et la valeur externe attendue peut changer la source autorisée. Il n'y a **aucune signature, identité tierce, sandbox OS, protection complète contre les races locales ou coupures électriques**, ni transaction globale. La vérification utilise le **même type de moteur Rust épinglé**, pas un second vérificateur cryptographique P2.7 indépendant.
- Le module ne met pas en œuvre une autorité réseau, un contrôle multi-agents distribué, une politique d'expiration, une révocation durable ou un système de single-use global. Aucun de ces attributs ne peut être présumé.

## Bureau — neuf axes de revue (non neuf signatures)

**ATLAS** : séquence parent CERT-015-R1 et limite APP-016 ; **FERRUM** : vrai Rust pour chaque rejeu ; **BOUNDARY** : policy séparée, aucune écriture avant admission ; **CIPHER** : source et binaire imposés par destinataire, hash externe et absence de fausse signature ; **LEDGER** : plan, manifeste portable, reçus natifs et fichiers P2.7 ; **CAPSTONE** : permissions explicites et erreur `UNKNOWN` en cas de publication ambiguë ; **ADVERSARY** : checkpoint et source alternatifs cohérents, source/policy remplacées, privilèges non autorisés, symlink, duplications et faux claims ; **VERITY** : matrice native réelle quatre OS, toutes régressions ; **GUARDIAN** : C20, AI-only, PR DRAFT, non-production.

Ces axes sont une **revue interne technique** ; ils ne créent **pas neuf auditeurs externes ni neuf signatures indépendantes**.

## Qualification à exécuter

**40 nouveaux tests APP-016**, **20 tests indépendants CERT-015-R1** et **568 régressions APP-001…APP-015**, soit **628 tests par OS / 2 512 exécutions** sur Mac Intel, Apple Silicon, Linux et Windows. Les jobs Rust Test Gate **5** et E8 adversarial **4** complètent la matrice **13/13**.

La démonstration de chaque OS doit prouver : source native authentique APP-012 → paquet APP-013/014 portable → politique récepteur externe → `inspect` sans exécution native → `verify` avec rejeu natif exact → `restore` dans une destination vide → `resume` réel → APP-010 `PASS` complet ; une autre source et un autre checkpoint **tous deux cohérents** sont refusés par la politique d'origine, même si leur propre pin est valide. Une reprise en doublon est refusée.

**Aucun verdict APP-016 avant 13/13 jobs SUCCESS au commit exact et commentaire du Bureau.**

```ini
APP016_START=RESEARCH_ONLY
APP016_QUALIFICATION=PENDING_CI_AND_BUREAU
R6F=IN_PROGRESS_UNCERTIFIED
R6G=LOCKED_BY_SEQUENCE
EXTERNAL_NATIVE_ADMISSION_GATES=12_UNMET
HUMAN_TESTERS=0
PR=DRAFT
MERGE=NONE
PRODUCTION=NOT_AUTHORIZED
```
