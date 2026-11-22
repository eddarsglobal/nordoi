# NORDOI APP-018 — Ancrage local persistant de fraîcheur du destinataire

**22 novembre 2026 · Laboratoire Rust/agent-à-agent · Recherche non privilégiée**

## RoadMap / Bureau / Guardian

Parent exact **APP-017** : `c048f1e17590b8d863d38a067fe6ac00d78e61c2`, qualifié **13/13 jobs GitHub SUCCESS** et **2 672/2 672 tests applicatifs PASS** sur quatre OS. PR #169 DRAFT, non fusionnée. Le Bureau a consigné une décision de démarrage APP-018 **recherche uniquement**. Aucune fusion de branches, signature humaine ou autorisation de production.

## Problème scientifique : la fraîcheur non durable d'APP-017

APP-017 peut révoquer une ancienne politique de confiance en exigeant **un nouveau roster et un nouveau SHA-256 externe**. Mais un agent redémarré peut accepter un ancien roster et son ancien SHA-256, tous deux cohérents, si le système ne conserve **aucun état local actuel fiable**.

APP-018 ajoute un **journal persistant chaîné par SHA-256** dans un répertoire local privé du destinataire, indépendant du paquet reçu. Le programme ne choisit plus la « dernière » génération à partir des seules déclarations de l'appelant : il relit toute sa chaîne et en déduit le roster courant. L'appelant ne peut pas rétrograder le journal via une opération normale `advance`.

Ce laboratoire assure **une prévention locale du retour en arrière tant que le répertoire d'ancrage original reste intact, non remplacé et consulté par tous les agents**, y compris après redémarrage du processus. Il ne possède pas de TPM, enclave, signature ou coffre indépendant, et **ne résiste pas à un adversaire contrôlant le stockage lui-même ou choisissant un autre répertoire**.

### Stockage exact

```text
receiver-owned-freshness-anchor/       # création exclusive, mode 0700 sur POSIX
  HEAD.json                             # état canonique courant, remplacement atomique
  history/                              # mode 0700 sur POSIX
    entry-00000001.json                # création exclusive, jamais réécrite
    entry-00000002.json                # génération plus récente, SHA de l'entrée 1
    ...
```

Chaque entrée contient numéro séquentiel, génération du roster, état `ACTIVE` ou `FROZEN`, SHA-256 externe du roster, politique active et liste des politiques révoquées, l'empreinte de l'entrée précédente et des marqueurs explicites de restrictions. **Toutes les entrées sont relues et validées** lors de chaque commande, sans confiance dans le seul `HEAD.json`. Le dernier hash de la chaîne doit être identique à `HEAD.json`; un trou, une écriture incomplète, un fichier supplémentaire ou une altération sont **refusés**.

Les générations sont **strictement croissantes** et chaque transition requiert un SHA de roster différent. Si la politique active change, l'ancienne doit apparaître dans les politiques révoquées. Une politique révoquée à un quelconque stade du journal **ne peut pas être réadoptée** par une génération ultérieure. Les entrées sont plafonnées à **128** : au-delà, le système refuse toute nouvelle transition, sans purge cachée. La liste de révocations reste plafonnée à 32 par roster APP-017.

### Journal et garanties de publication

L'écriture d'une nouvelle entrée est **exclusive** (`O_EXCL`), suivie d'un `fsync` du fichier et d'une tentative de `fsync` du répertoire. L'état `HEAD.json` est ensuite produit via un fichier temporaire, `fsync`, `os.replace` et tentative de `fsync` du répertoire. Sur Windows, si le fsync des répertoires n'est pas disponible, le code ne l'assimile **pas** à une preuve de durabilité électrique. Le marqueur est `FILE_FSYNC_AND_DIRECTORY_FSYNC_WHEN_SUPPORTED`.

Un verrou de répertoire créé de façon exclusive (`.lock`) empêche deux processus APP-018 d'utiliser simultanément le même ancrage, **y compris pendant les calculs Rust et les effets APP-014/011**. Si le processus meurt avant nettoyage du verrou, le système **reste bloqué** ; aucune reprise automatique qui pourrait effacer une opération inconnue n'est autorisée. Une écriture d'entrée réussie mais une mise à jour `HEAD.json` inachevée entraîne une chaîne contradictoire : **fail closed**. Aucun rollback global ou récupération autonome d'une publication native in-flight n'est revendiqué.

### Cinq commandes agent-à-agent

**`bootstrap`** : initialise un **répertoire qui n'existait pas**, depuis un roster APP-017 déjà validé par une empreinte attendue obtenue indépendamment. Aucun second bootstrap ne peut écraser cet ancrage.

```bash
python3 -B examples/apps/app018_local_freshness/freshness_app018.py bootstrap \
  --anchor-root /receiver/private/current-freshness \
  --receiver-roster /receiver/trusted/roster-gen1.json \
  --expected-roster-sha256 TRUSTED_EXTERNAL_ROSTER_SHA256 \
  --receiver-minimum-generation 1
```

**`advance`** : sélectionne un **nouveau roster explicitement épinglé par le destinataire**, exige une génération supérieure à celle du journal, vérifie la continuité des révocations et écrit une nouvelle entrée exclusive et un `HEAD.json` atomiquement remplacé.

```bash
python3 -B examples/apps/app018_local_freshness/freshness_app018.py advance \
  --anchor-root /receiver/private/current-freshness \
  --receiver-roster /receiver/trusted/roster-gen2.json \
  --expected-roster-sha256 NEW_TRUSTED_EXTERNAL_ROSTER_SHA256 \
  --receiver-minimum-generation 2
```

**`status`** : relit la chaîne locale complète, sans consulter le moteur Rust ni charger le paquet ou les sources.

```bash
python3 -B examples/apps/app018_local_freshness/freshness_app018.py status \
  --anchor-root /receiver/private/current-freshness
```

**`inspect`, `verify`, `restore`, `resume`** : ajoutent les contrôles APP-017→016→015. Ils reçoivent obligatoirement le même `--anchor-root`, le fichier roster courant `--receiver-roster`, la politique sélectionnée `--receiver-policy`, et le paquet `--portable-dir`. Aucune valeur `--expected-roster-sha256` n'est acceptée pour ces quatre opérations : **le hash vient du journal local**. `verify`, `restore`, `resume` nécessitent aussi `--engine` et `--source-root` choisis par le destinataire ; les deux dernières nécessitent la destination `--restore-root`.

La commande `inspect` n'exécute pas Rust ; `verify` calcule dans le scratch APP-015. `restore` et `resume` exigent une nouvelle lecture de la chaîne, du roster, des reçus, du moteur et de la politique sous **le même verrou local**, puis délèguent aux fonctions APP-014/011 vérifiées. Le suffixe de `resume` reste **uniquement les nouvelles lignes JSONL natives**.

**États** : code 0 = résultat strictement positif dans le périmètre ; code 2 = `REJECTED` avant tout nouvel effet natif ; code 3 = `INCONCLUSIVE` / `publication=UNKNOWN` pour un effet ou une publication indéterminés. Un verrou resté présent donne un refus fail-closed, **pas** une suppression automatique.

## Évaluation adversariale

- `bootstrap` vérifié avant et après un vrai **redémarrage du processus Python** ; chaîne et premier roster encore présents.
- Deux paquets APP-012/013/014 effectivement calculés avec Rust et des sources .noi originales/alternatives ; APP-015 recalcule leurs fichiers P2.7.
- `advance` remplace une génération 1 par 2, journal durable relu en nouveaux processus ; **l'ancien roster + son authentique ancien SHA-256** sont refusés tant que l'ancrage est consulté.
- La politique initiale est réellement révoquée ; la nouvelle génération autorise les vrais résultats Rust alternatifs et peut restaurer/reprendre le DAG sous audit APP-010.
- Un retour d'une politique précédemment révoquée à la génération 3 est refusé par **l'historique des révocations**, et non par la liste du seul roster courant.
- État `FROZEN` inscrit comme transition persistante, puis `inspect/verify/restore/resume` refusés, sans modifier les fichiers natifs existants.
- Reprises doubles, racines non initialisées, verrou persistant, tête historique falsifiée, entrée corrompue, fichier supplémentaire, trou de séquence et symlink sont refusés.

## Bureau — neuf axes de revue (non neuf certificats indépendants)

ATLAS (séquence C20), FERRUM (Rust réel), BOUNDARY (verrou local et opérations natives exclusives), CIPHER (SHA-256, chaîne, ancrages externes sans signature), LEDGER (relecture de chaque entrée et refus de réadmission d'une politique révoquée), CAPSTONE (verrous et publications incertaines = fail-closed), ADVERSARY (replay d'anciens pins cohérents après redémarrage, corruption et gel), VERITY (quatre OS et regressions historiques), GUARDIAN (PR DRAFT, AI-only, aucune fusion/production).

Ces axes représentent **une revue analytique interne**, jamais une authentification par neuf tiers indépendants.

## Qualification CI visée

**40 nouveaux tests APP-018** + **40 APP-017** + **40 APP-016** + **20 CERT-015-R1** + **568 APP-001…APP-015** = **708 tests par système**, soit **2 832 tests applicatifs** sur Mac Intel, Apple Silicon, Linux et Windows. Rust Test Gate 5 + E8 adversarial 4 = **13 jobs**. Une preuve intégrée sur chaque OS doit vérifier : deux vraies générations de roster, redémarrage, rejet du vieux hash pourtant authentique, app011 vraie reprise native, app010 audit complet, `FROZEN` sans effet, ainsi que stockage journal strict et limites d'autorité explicites.

### Limites non levées

L'ancrage persistant APP-018 est seulement aussi fiable que **le répertoire de stockage local confié au destinataire**. Un adversaire système peut remplacer/supprimer **tous** les fichiers du journal ou persuader le programme de lire une ancienne copie complète. Ni `fsync` ni les hachages non signés ne résolvent ce problème. `fsync` n'est pas un test réel de coupure de courant. Une panne entre deux opérations peut laisser un journal contradictoire ou un verrou orphelin et l'agent reste bloqué ; **aucune restauration automatique dangereuse** n'est prévue. Le contrôle de fichiers sans sandbox ne protège pas contre l'exécution d'un moteur Rust arbitraire malveillant, des races d'hôte, ou des effets native in-flight inconnus. Aucune distribution de roster authentifiée, synchronisation multi-appareils, TPM/TEE, signature, transaction globale ni autorisation de production n'est fournie.

```ini
APP018_STATUS=PENDING_NATIVE_CI_AND_BUREAU
APP018_POWERLOSS_CERTIFICATION=NOT_PROVIDED
APP018_HOST_ADVERSARY_PROTECTION=NOT_PROVIDED
APP018_PR=DRAFT
APP018_MERGE=NONE
APP018_PRODUCTION=NOT_AUTHORIZED
R6F=IN_PROGRESS_UNCERTIFIED
R6G=LOCKED_BY_SEQUENCE
EXTERNAL_NATIVE_ADMISSION_GATES=12_UNMET
HUMAN_TESTERS=0
```
