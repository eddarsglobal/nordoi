# NORDOI APP-006 — Audit autonome des lots d'agents, **en lecture seule**

**22 novembre 2026 — Développement par agents IA pour agents IA.**

## Fonctionnalité nouvelle

NORDOI APP-005 peut désormais exécuter plusieurs requêtes JSONL via le véritable binaire Rust. APP-006 apporte une **contre-vérification indépendante du chemin d'exécution** : un agent peut examiner a posteriori le JSONL produit par le moteur et les **fichiers réellement créés**, sans invoquer de nouveau le compilateur, sans exécuter de programme `.noi`, et sans disposer d'une permission FileWrite.

Implémentation : `audit_app006.py` (Python 3, bibliothèque standard seulement, gratuit, hors ligne), entièrement autonome. Elle n'importe pas de moteur NORDOI, ne lance aucun processus enfant et ne modifie ni les reçus ni les fichiers observés. Elle peut être exécutée par un autre agent ou sur une autre machine ayant les deux éléments d'entrée.

Commande :

```bash
python3 -B examples/apps/app006_readonly_audit/audit_app006.py \
  --batch-root /chemin/vers/la/racine/du/lot \
  --receipts /chemin/vers/les/reponses_natives.jsonl
```

**Préparation des preuves** : APP-005 doit avoir réellement produit le dossier contenant `job-0001/quality-decision/`, `job-0002/quality-decision/`, etc., et sa sortie `stdout` JSONL doit être capturée dans un fichier **extérieur** au dossier du lot. Le vérificateur refuse les fichiers supplémentaires dans ce dossier.

Le résultat est un **seul objet JSON** sur `stdout`, facile à exploiter par tout agent :

```json
{
  "schema": "NORDOI_AGENT_BATCH_AUDIT_V1",
  "status": "PASS",
  "committed_jobs_verified": 2,
  "input_receipts_sha256": "empreinte réelle des octets du fichier JSONL",
  "jobs": [
    {
      "job": "job-0001",
      "key_code": 73,
      "decision": 1,
      "bundle_receipt_sha256_unverified": "empreinte fournie par le moteur",
      "files": [
        {"name": "decision.txt", "size_bytes": 33, "observed_sha256": "sha256 réel du contenu"},
        {"name": "report.txt", "size_bytes": 45, "observed_sha256": "sha256 réel du contenu"}
      ]
    }
  ],
  "attestation": "READ_ONLY_CONTENT_CONSISTENCY_NOT_AUTHENTICATED_SIGNATURE",
  "p27_proof_verification": "SHAPE_ONLY_NOT_CRYPTOGRAPHICALLY_RECOMPUTED",
  "crash_durability": "UNCLAIMED",
  "concurrent_host_adversary": "NOT_COVERED"
}
```

Les valeurs ci-dessus sont **illustratives**, les véritables empreintes, tailles et contenus sont déterminés par la lecture des fichiers, jamais inventés.

## Contrôles

1. Le JSONL natif APP-005 contient **1 à 8 reçus de publication**, suivis éventuellement d'un unique refus `ERROR`. Les objets sont analysés strictement en UTF-8, sans doublons de clés ni valeurs non finies. Le reçu doit terminer par `\n`.
2. Chaque résultat `COMMITTED` conserve le contrat APP-004 : deux fichiers exacts, le module, la décision 0/1, la branche correspondante, le schéma, la forme des empreintes, les capacités `EXPLICIT`, l'autorité ambiante `NONE`, la politique `overwrite=DENIED` et la durabilité `UNCLAIMED`.
3. Pour chaque résultat, un sous-dossier `job-NNNN/quality-decision/` réel et non symbolique est attendu ; seuls `decision.txt` et `report.txt` peuvent s'y trouver. La taille et **tous les octets du fichier** doivent correspondre au contenu annoncé dans le reçu.
4. APP-006 recalcule, avec la bibliothèque cryptographique standard, **SHA-256 des octets réellement lus** de chaque fichier et SHA-256 du flux JSONL fourni. Ces valeurs servent à identifier le matériau observé, pas à créer une signature.
5. Toute disparition, altération, injection de nom de fichier, modification de permission déclarée, mauvaise décision, doublon, sortie supplémentaire ou lien symbolique provoque `REJECTED` avec code de sortie **2**.
6. Si la sortie native indique un échec après un ou plusieurs `COMMITTED`, APP-006 vérifie le préfixe déjà publié et retourne `PARTIAL` avec code de sortie **3**. Un échec avant publication retourne `ERROR` et le même code. **Le préfixe confirmé n'est pas une transaction globale**.

**Limites de sécurité exactes :** APP-006 prouve la **cohérence entre les fichiers observés et les reçus fournis**, à l'instant de leur lecture. Il ne réimplémente **pas** le calcul cryptographique interne `proof_sha256` propre à P2.7 ; il ne peut pas détecter une modification coordonnée des reçus **et** des fichiers sans une preuve externe ancrée ; il n'authentifie pas l'émetteur des reçus. Il ne garantit pas la durabilité électrique ou la protection contre un attaquant local qui change les chemins pendant l'audit. Aucun statut `CERTIFIED` ni preuve de production n'est revendiqué.

## Validation machine

**40 tests** sont écrits pour APP-006. Chaque scénario démarre par une **vraie exécution du binaire Rust NORDOI APP-005** sur le système réel, puis les scénarios négatifs modifient un **artefact réellement produit** afin de vérifier que l'auditeur le refuse. Parmi eux : 2 et 1 jobs, fichiers altérés, manquants ou supplémentaires, falsification de décision, permissions, prétendue durabilité, nom de fichier traversant `..`, JSON dupliqué ou corrompu, liens symboliques, refus avant effet, publication partielle réelle après mutation de la règle `.noi`, et preuve que l'auditeur ne modifie aucun fichier.

Matrice GitHub **Mac Intel, Apple Silicon, Linux, Windows** : 40 tests APP-006, plus les 40 APP-005, 40 APP-004/R2, 31 APP-004/R1, 38 APP-003, 26 APP-002 et 16 APP-001 (soit **231 tests applicatifs par système**), Test Gate Rust et E8.

**Parent C20 qualifié :** APP-005 commit `2348c6484f8ea787957dc9e64c0f3d9279c7986d`, **25/25 jobs GitHub SUCCESS**. Toute conclusion concernant APP-006 devra provenir de ses propres résultats au SHA exact.

**Gouvernance :** zéro testeur humain, développement agentique continu ; R6F non certifié, R6G verrouillé, 12 conditions natives d'indépendance sans preuves réelles, aucune production privilégiée ni fusion protégée autorisée. Ces conditions de certification ne bloquent pas la recherche ou les vérifications en lecture seule.
