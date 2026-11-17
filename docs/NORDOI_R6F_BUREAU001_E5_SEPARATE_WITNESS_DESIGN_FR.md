# NORDOI G0.3 — R6F / BUREAU-001 E5

## Architecture de témoin de preuves séparé — contrat et impossibilités sous une seule administration

**Directive propriétaire : zéro testeur humain, toujours.** Il ne s'agit ni de recrutement de spécialistes humains, ni d'enrôlement d'un gardien, ni d'une admission native. E5 prépare un **contrat de preuves externe** et un audit de refus, uniquement dans le périmètre de recherche NORDOI.

### Point de départ réel, et ce que E4 a démontré

Le laboratoire E4 sur vrai Mac Intel et sur le commit `0c4a4f5b1b349edb445871c4980daba57e936a59` a réellement exécuté neuf suites machines PASS. L'exercice a modifié sur **copies** un vrai journal ATLAS et le SHA-256 correspondant dans le manifeste. Un nouveau processus a accepté la paire falsifiée via E3 ; une empreinte antérieure locale non altérée détectait la divergence, mais l'empreinte réécrite par le même administrateur ne protégeait plus l'histoire. [CI E4](https://github.com/eddarsglobal/nordoi/actions/runs/44023579163) **5/5 SUCCESS**, [Test Gate Rust](https://github.com/eddarsglobal/nordoi/actions/runs/44023579257) **5/5 SUCCESS**. Ce résultat constitue une **limitation de confiance mesurée**, pas une défaillance cryptographique de SHA-256.

### Architecture candidate, NON déployée

<escape>
PRODUCTEUR / NORDOI LOCAL
  ├─ code source (Git HEAD exact)
  ├─ tests machines réels / journaux
  ├─ manifeste SHA256 / instantanés locaux
  └─ déclaration de défi et de témoin (NON FIABLE)
           │  preuve source + objet canonique + défi
           ▼
VÉRIFICATEUR DE TÉMOIN (CANDIDAT, NON ENRÔLÉ)
  ├─ clef publique vérifiée hors bande PAR UNE SOURCE INDÉPENDANTE
  ├─ signature réellement vérifiée pour un namespace et un défi frais
  ├─ admin séparé, contrôle exclusif de la clef privée
  ├─ état durable hors contrôle du producteur
  ├─ époque de clef et seuils nonce/révocation monotones
  └─ récupération/rotation et quorum à autorités séparées
           │  reçu externe authentique, si réellement obtenu
           ▼
GUARDIAN / ROADMAP (N'ACCEPTE PAS UNE SIMPLE AUTO-DÉCLARATION)
  └─ décision autorisée seulement avec les 12 preuves exigées,
     les vérifications et approbations formelles distinctes
</escape>

**Aucun des éléments « témoin » extérieurs ci-dessus n'est attesté en E5.** Le schéma décrit ce qu'il serait nécessaire de vérifier ; il n'affirme pas qu'un tiers possède déjà une clé ou un historique.

### Matrice de six obligations non satisfaites

| ID | Preuve réelle exigée | Limite actuelle |
|---|---|---|
| `independent_root` | Empreinte publique précise du gardien authentifiée hors bande indépendamment de l'administrateur local | NON_AUTHENTICATED |
| `operator_domain` | Gardien réellement habilité, contrôle administratif distinct et secret non accessible localement | NOT_ENROLLED |
| `freshness` | Signature liée au défi unique frais, au namespace, au commit exact et à l'historique vérifié | NOT_VERIFIED |
| `external_monotonicity` | Époques, nonces et révocations conservés dans une histoire durable indépendante et résistante à restauration R5Q | NOT_VERIFIED |
| `threshold_and_recovery` | Quorum de domaines véritablement distincts, rotation/récupération vérifiables | NOT_PROVEN |
| `formal_approval` | Décision du Bureau et autorisation propriétaire séparée après contrôle réel | NOT_DECIDED |

La notion de domaine séparé ne découle pas d'une seconde clé, machine virtuelle, VPS, processus ou agent IA. **Une signature authentique prouve l'usage d'une clé, pas la propriété administrative de cette clé.**

### Contenu du nouveau contrat E5

`governance/NORDOI_R6F_BUREAU001_E5_SEPARATE_WITNESS_CONTRACT.json` conserve la qualification technique E4, les cinq domaines de confiance et ces six obligations, toutes **non vérifiées**. Il recopie exactement les douze identifiants de conditions natives avec `promoted=false`.

`scripts/bureau_001_e5_separate_witness_contract.py` expose :

- `--audit --expected-head SHA` : vérifie la chaîne de sources E4/E3/E2/R1, le vrai HEAD Git et les valeurs non autorisées ; retourne **2** pour refus attendu ou **3** en cas d'erreur.
- `parse` / `assess` : lit une **proposition non fiable** au format JSON strictement canonique. Les champs de déclaration locale de séparation, empreinte OOB, quorum ou autorisation ne sont **jamais** convertis automatiquement en droits. Rejette les champs inconnus, faux booléens, horloges/nombres non canoniques, SHA erronés, doublons et réutilisation sous un autre commit.
- `--exercise --expected-head SHA` : sur POSIX/OpenSSH réel, reprend **deux signatures Ed25519 localement produites et réellement vérifiées**, et le contre-exemple R5Q réel de restauration cohérente des trois fichiers depuis un autre processus. Vérifie que l'affirmation mensongère de séparation et de quorum n'ouvre aucun droit. Ce contrôle ne « simule » jamais un gardien indépendant.
- **24 tests** : 20 vérifications portables + 4 tests POSIX réels (OpenSSH et R5Q). Sur Windows, ces quatre cas réels sont explicitement ignorés ; aucun PASS natif fictif n'est déclaré.

La CI E5 effectuera les tests sur macOS Intel, macOS Apple Silicon, Linux et Windows avec checkout épinglé à la tête exacte de PR. Elle analysera séparément les codes **2 = DENY attendu** et **3 = incohérence/erreur**, et conservera les originaux R1–E4 et la garde native inchangés.

### Critères de futurs reçus réels (pas d'implémentation de réception positive en E5)

Un véritable dossier ultérieur nécessiterait, au minimum : identifiant du domaine autorisé, preuve du contrôle exclusif de clé, racine publique effectivement vérifiée hors bande, engagement cryptographique sur l'empreinte source et l'archive des preuves, défi unique frais reçu dans une fenêtre admissible, signature dans un namespace exact, liaison à l'époque et à l'historique antérieur non restaurable, seuil de révocation/nonce, procédure de récupération et quorum réellement indépendant. Chaque pièce devrait être vérifiée **hors du domaine administratif du producteur** et par un mécanisme dont celui-ci ne peut pas réécrire l'histoire.

E5 ne développe **aucune fonction d'admission positive** et n'accepte aucune prétendue signature externe : il ne dispose pas de l'origine indépendante qu'il faudrait authentifier. Ce choix est un garde-fou contre le risque de fabriquer une fausse identité de témoin en laboratoire.

### Décision de gouvernance

```ini
E5_PROTOCOL_DESIGN=SOURCE_BOUND_DENY_ONLY
E5_EXTERNAL_PROOF_OBLIGATIONS=6_UNMET
E5_LOCAL_TWO_REAL_SSHSIG=NOT_EXTERNAL_AUTHORITY
E5_REAL_R5Q_COHERENT_ROLLBACK=STILL_UNRESOLVED_GLOBALLY
E5_REAL_EXTERNAL_WITNESS_RECEIPTS=0
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

**E5 est une étude de protocole avec tests négatifs réels**, ni une intégration réseau de témoin, ni la solution au défaut E4. L'autorisation propriétaire de gouvernance (si un jour demandée) n'est pas un test humain. Aucune personne n'est sollicitée pour tester NORDOI, aucun secret privé n'est exporté, aucun service externe n'est enrôlé et aucune fusion de PR n'est autorisée par ce travail.
