# NORDOI APP-004 — Contrat JSON natif pour les agents IA

**21 novembre 2026 — Développement par IA pour agents IA, sans testeur humain.** Candidat de recherche borné : **ne pas annoncer une admission native indépendante, une production autorisée ni une durabilité après panne électrique**.

## Utilité : un agent peut parler directement à NORDOI

APP-003 exécute un unique programme d'entrée `.noi`, qui importe une règle et écrit deux fichiers. APP-004 apporte une **interface de réponse machine native** au même exécutable Rust, pour que l'agent appelant n'ait plus à comprendre les textes historiques du terminal.

```text
nordoi agent-bundle-json <source-root> <entry-module> <key-code> --grant-output-dir <dir>
```

La décision est toujours calculée par `execute_module_graph_v13`, les effets de fichiers sont toujours exécutés par `execute_atomic_bundle_p27`. Le Rust utilise le **même compilateur APP-003**, non un faux modèle Python. La sortie `stdout` est **exactement un objet JSON terminé par un saut de ligne**.

Exemple de commande native (répertoire cible neuf existant) :

```bash
cargo run --offline --bin nordoi -- agent-bundle-json \
  examples/apps/app003_integrated_quality/src app.main 73 \
  --grant-output-dir "$OUT"
```

La version facile pour un agent utilise un répertoire nouveau automatiquement et vérifie les fichiers sur disque :

```bash
python3 examples/apps/app004_agent_json/run_app004.py --score 73
```

### Contrat v1 en cas de réussite

Le JSON natif contient les champs :

```json
{
  "schema":"NORDOI_AGENT_BUNDLE_RESULT_V1",
  "status":"COMMITTED",
  "entry_module":"app.main",
  "key_code":73,
  "module_count":2,
  "decision":1,
  "selected":"approved",
  "bundle":"quality-decision",
  "files":[
    {"name":"decision.txt","bytes":33,"content":"approved=true, score=73, next=74\n","proof_sha256":"..."},
    {"name":"report.txt","bytes":45,"content":"module-decision=approved, input=73, next=74\n","proof_sha256":"..."}
  ],
  "authority":{"mode":"EXPLICIT","grant_count":2,"ambient":"NONE","overwrite":"DENIED"},
  "publication":"DIRECTORY_RENAME",
  "partial_final_state":"FORBIDDEN",
  "crash_durability":"UNCLAIMED",
  "rule_receipt_hex":"...",
  "bundle_receipt_sha256":"..."
}
```

**Les longueurs ci-dessus sont illustratives** ; seules les longueurs calculées par NORDOI à l'exécution sont probantes. La vraie réponse est écrite par Rust **à partir des objets reçus du moteur**, y compris les octets effectifs des fichiers, le reçu de plan de règle V1.3 et le reçu P2.7. Elle ne contient ni chemin absolu, ni horodatage, ni identifier propre à la machine. L'ordre des champs et fichiers est déterministe ; le même score et les mêmes sources produisent les mêmes octets JSON sur le même binaire.

Pour un score 59, le JSON revient avec `decision:0`, `selected:"rejected"`, `status:"COMMITTED"` et le contenu des vrais fichiers refusés.

### Contrat v1 en cas de refus

Sans `--grant-output-dir` explicite :

```json
{"schema":"NORDOI_AGENT_BUNDLE_RESULT_V1","status":"ERROR","error_code":"MISSING_EXPLICIT_GRANT","exit_code":5,"publication":"UNKNOWN"}
```

Le code de sortie POSIX reste non nul. En cas de syntaxe invalide : `BAD_ARGUMENT_COUNT`, `INVALID_KEY_CODE` ou `INVALID_GRANT_SYNTAX`. Les défauts détectés par le moteur sont classés `NATIVE_OPERATION_FAILED` ; les détails restent sur `stderr`, jamais mélangés au JSON de `stdout`. **Une erreur après un effet ne garantit pas l'absence de fichiers : le protocole dit volontairement `publication:"UNKNOWN"`, jamais `NOT_COMMITTED` par défaut.**

### Garanties et limites

- Les permissions `FileWrite` exactes restent explicitement accordées et l'autorité ambiante reste `NONE`.
- Les branches inactives doivent être valides ; une décision `INT(2)`, un import manquant, des noms de cibles divergents ou une absence d'autorisation provoquent un refus.
- Le client Python n'applique **aucun seuil métier** et ne calcule jamais la branche. Il valide l'intervalle d'entrée seulement, crée un dossier privé, appelle le binaire réel, lit **un seul JSON**, vérifie les contenus réels contre le JSON natif et retransmet le document original à un autre agent.
- Le hash `proof_sha256` est le **reçu d'intégrité P2.7 propre au fichier**, pas à tort un simple SHA256 du contenu. Le reçu complet P2.7 est émis sur `stderr`. La v1 n'est pas une nouvelle attestation cryptographique commune des deux profils.
- **Aucun moteur IA requis à l'exécution normale** (Constitution C8). Pas de réseau, de signature humaine, de testeur humain, ni de jeton externe.
- Il ne s'agit pas d'une API réseau, d'un serveur agent MCP, d'un JSON d'entrée générique, ni d'une nouvelle capacité native privilégiée.
- La publication P2.7 garde `crash_durability=UNCLAIMED`, sans garantie universelle face à une coupure électrique ou un adversaire local concurrent.

## Validation et RoadMap / Bureau

30 tests automatiques APP-004 sur le **vrai exécutable Rust**, sur quatre plateformes GitHub : Mac Intel, Apple Silicon, Windows, Linux. Ils exercent le contenu JSON, ses refus et sa détermination, les vraies permissions, les règles mutées, les fichiers exacts, l'intégrité des preuves, Unicode, l'absence d'écrasement, la continuité de la CLI APP-003 et l'utilisation par le client.

La CI effectue aussi les **38 tests APP-003**, les **26 APP-002**, les **16 APP-001**, les contrôles Rust généraux et l'adversarial E8. L'ensemble de la qualification est exécuté par machines ; aucun humain n'est sollicité pour la débloquer.

Parent C20 exact : `2b5cfa1d8e6bde3ef5372b5753ad7a642136ec8a`, directive permanente AI-only vérifiée **10/10 CI** ; l'implémentation APP-004 doit obtenir **sa propre preuve exact-HEAD** avant un verdict final.

**Statut historique inchangé :** R6F non certifié, R6G verrouillé, 12 conditions d'indépendance natives non satisfaites. Cette piste de certification est séparée du développement agentique, lequel ne doit pas être immobilisé par une attente humaine.
