# NORDOI APP-005 — Lots JSONL natifs pour agents IA

**21 novembre 2026 — Projet développé par agents IA pour agents IA. Aucun testeur ni signature humaine requis.**

## Ce qui fonctionne

NORDOI ajoute, au **véritable binaire Rust existant**, une commande de lots de
requêtes machine :

\`\`\`text
nordoi agent-batch-jsonl <source-root> <entry-module> --grant-output-dir <existing-dir>
\`\`\`

Un agent envoie **1 à 8 documents JSON de type APP-004/R2**, un par ligne sur
\`stdin\`. Le moteur Rust lit le flux complet sous une limite maximale
de 32 776 octets, vérifie **toutes les requêtes avant de créer le moindre
répertoire de sortie**, puis exécute chaque document en ordre croissant :

\`\`\`jsonl
{"schema":"NORDOI_AGENT_REQUEST_V1","key_code":73}
{"schema":"NORDOI_AGENT_REQUEST_V1","key_code":59}
{"schema":"NORDOI_AGENT_REQUEST_V1","key_code":60}
\`\`\`

Pour une requête numéro 1, le moteur crée exclusivement le sous-répertoire
\`job-0001/\`, pour la suivante \`job-0002/\`, etc. Les dossiers sont
fixés par Rust ; **aucune valeur du JSON ne détermine les chemins**.
Un nouveau sous-répertoire est créé par l'opération système \`create_dir\`,
qui refuse toute cible déjà existante. L'autorité d'écriture vient du
\`--grant-output-dir\` **explicitement fourni hors JSON**, jamais des
données envoyées par l'agent. Les liens symboliques à la racine accordée
ou au sous-répertoire déjà existant sont refusés.

Pour chaque job, le véritable moteur \`execute_module_graph_v13\` calcule la
règle importée, le véritable moteur \`execute_atomic_bundle_p27\` publie
les deux fichiers de cette requête avec FileWrite exact, et le binaire écrit
**un objet JSON APP-004 V1 sur une ligne de \`stdout\`**. Résultat normal :
une ligne de réponse pour chaque requête, en ordre déterministe.

## Démonstration sur une machine

De la racine Git de cette branche :

\`\`\`bash
python3 examples/apps/app005_agent_batch/run_app005.py --scores 73 59 60
\`\`\`

L'adaptateur Python vérifie l'intervalle des entrées, crée un dossier de
sortie privé et neuf sous \`Downloads\`, appelle le **binaire Rust réel hors-ligne**
et vérifie pour chaque job les fichiers effectivement écrits par P2.7
contre le JSON émis par Rust. Il ne calcule **aucune décision métier**.

La ligne de commande directe nécessite un répertoire accordé existant et vide :

\`\`\`bash
printf '%s\n' \
  '{"schema":"NORDOI_AGENT_REQUEST_V1","key_code":73}' \
  '{"schema":"NORDOI_AGENT_REQUEST_V1","key_code":59}' \
| cargo run --offline --bin nordoi -- agent-batch-jsonl \
    examples/apps/app003_integrated_quality/src app.main \
    --grant-output-dir "$OUT"
\`\`\`

Sorties réelles attendues, sous \`$OUT\` :

\`\`\`text
job-0001/quality-decision/decision.txt
job-0001/quality-decision/report.txt
job-0002/quality-decision/decision.txt
job-0002/quality-decision/report.txt
\`\`\`

Les deux lignes JSON de \`stdout\` incluent les mêmes reçus cryptographiques
V1.3 et P2.7 que les opérations individuelles APP-004.

## Sécurité et obligations de preuve

Le protocole **valide la totalité du lot avant toute publication**, avec
limite par document de 4 096 octets et limite globale, refus des doublons,
des champs de permission en JSON, des erreurs de grammaire, d'une 9e demande
et d'une ligne vide. Un lot avec une requête finale invalide n'engendre
**aucune création** dans le dossier accordé. Une destination \`job-NNNN\`
préexistante dans ce dossier provoque un refus **avant l'exécution du
premier job**, afin d'interdire les écrasements.

La publication demeure **atomique par bundle P2.7**, mais
**n'est pas atomique à l'échelle du lot**. Si le premier job est publié
et si le second échoue (par exemple règle \`INT(2)\`), la première ligne
de reçu \`COMMITTED\` et les premiers fichiers restent présents ; le dernier
enregistrement est \`ERROR\` avec \`publication:"UNKNOWN"\` et la commande
sort avec un code non nul. **Aucun rollback automatique, aucune prétention
à une transaction multi-jobs ou à la durabilité électrique après crash**.

Limite d'isolation : ce candidat ne prétend pas résister à un adversaire local
qui modifie les chemins **simultanément** à l'exécution. Pas de sandbox OS
ajouté ni autorisation privilégiée. Pas de réseau, pas de serveur, pas de
système de tâches asynchrone, pas de modèle IA nécessaire au runtime.

## Test Gate et Bureau

**40 tests APP-005** : au moins 1, 2, 3 et 8 jobs, entrées répétées et dans
l'ordre inverse, sorties identiques aux commandes APP-004/R2 individuelles,
règle importée réellement modifiée, lot invalide en deuxième ou troisième
position refusé **avant effet**, erreurs de UTF-8/tailles/clés, refus des
liens symboliques et destinations existantes, échec après premier commit,
client agent sans logique métier, permissions et vrais contenus.

Tests de non-régression sur Mac Intel, Apple Silicon, Linux, Windows :
40 APP-004/R2 + 31 APP-004/R1 + 38 APP-003 + 26 APP-002 + 16 APP-001,
soit **191 tests applicatifs par système** une fois les 40 APP-005 ajoutés.
Tests unitaires Rust du nouveau parseur de lots, Rustfmt/Clippy et E8.

Parent C20 : APP-004/R2, commit
\`da6d53e1b22bda8a13fb4bdc31c46e574ee4f348\`, **21/21 jobs CI
réussis** (quatre systèmes). Toute nouvelle qualification APP-005 doit
provenir de ses propres résultats exact-HEAD.

**R6F IN_PROGRESS_UNCERTIFIED, R6G LOCKED_BY_SEQUENCE, gates natives externes
12 UNMET, production NOT_AUTHORIZED ;** ces axes restent séparés du
développement autonome de recherche. Cette PR reste DRAFT et non fusionnée.
