# NORDOI G0.3 — R6F / BUREAU-001 R9

## Audit contradictoire final des preuves manquantes et des conditions de décision

**Périmètre : recherche, lecture seule, sans admission native.** Parent immuable R8 : `cad5140206725ed45e84a445176360be13ae8a36`. R8 a été techniquement qualifié par un journal propriétaire Mac Intel (1 255 tests réguliers PASS, `SCRIPT_EXIT=0`), et par la CI GitHub (4/4 spécialisée + 5/5 générale). Le journal est une **preuve rapportée par le propriétaire**, non une attestation matérielle ou humaine émise par une autorité externe.

R9 **ne prétend pas avoir exécuté neuf revues indépendantes**. C'est un contrôle contradictoire préparatoire portant sur les sources gelées R1–R8, les preuves reçues et les préconditions vérifiables d'une décision administrative ultérieure.

## 1. Décision principale : HOLD, et pas CERTIFIED

Le registre `governance/NORDOI_R6F_BUREAU001_R9_FINAL_EVIDENCE_DECISION.json` conserve les sept critères de R8. Deux catégories de preuves techniques sont présentes et qualifiées avec les limitations de provenance indiquées :

- **R8-SOURCE-001 — preuves source et rejeux propriétaire** : les sources figées et les résultats R1–R8 rapportés (R8 : 1 002 Python + 253 Rust = 1 255 tests réguliers), sans signature indépendante attestant l'équipement ;
- **R8-CI-002 — CI GitHub** : les exécutions R8 #43874942792 et #43874942657 sont **9/9 SUCCESS** sur le commit exact, mais ne constituent ni une authentification de gardien ni une preuve de quorum.

**Cinq critères restent non satisfaits :**

| Critère | Condition réelle manquante | Conséquence |
|---|---|---|
| R8-REVIEW-003 | Neuf rapports effectivement réalisés et contrôlables par spécialité ; R2 enregistre toujours **zéro soumission** | Aucune certification des neuf fonctions |
| R8-OOB-004 | Empreinte exacte du gardien vérifiée par le propriétaire sur un véritable canal hors bande indépendant | Pas de racine de confiance propriétaire |
| R8-OPERATOR-005 | Identité, consentement et administration réellement séparée du gardien ; vrais signataires de quorum | Ni opérateur indépendant ni quorum |
| R8-LEDGER-006 | Époques de clés, révocations, nonces consommés et récupération protégés dans un historique externe, résistant au retour arrière R5Q | Pas d'antirollback global indépendant |
| R8-DECISION-007 | Décision formelle documentée du Bureau avec les preuves ci-dessus, puis autorisation propriétaire distincte | Pas de clôture formelle de R6F, ni d'admission native |

Ces pièces sont **intrinsèquement extérieures** à un exercice local exécuté par le même administrateur. Dix nouvelles séries de tests locales ne peuvent pas les produire. Ne pas confondre **deux clés distinctes**, **deux machines gérées par une seule personne** ou **deux identifiants d'agent IA** avec deux domaines de confiance indépendants.

## 2. Graphe des dépendances : ordre logique avant toute décision

Le dossier R9 impose cinq blocs de preuve :

1. `IND-SPECIALISTS` — réception et contrôle réel des revues spécialisées, distinctes des neuf vues du coordinateur ;
2. `IND-OPERATOR` — identité, accord et contrôle administratif authentifié du gardien extérieur ;
3. `IND-OOB` — validation par le propriétaire de la clé exacte du gardien hors du domaine compromis, après identification de l'opérateur ;
4. `IND-LEDGER` — conservation réellement indépendante des époques, nonces, révocations et récupération ; mise à l'épreuve par restauration locale complète ;
5. `IND-FORMAL` — décision contradictoire authentifiée et autorisation propriétaire après obtention et vérification des quatre précédentes catégories.

Chaque bloc indique les éléments de preuve futurs mais ses champs **`evidence_sha256=null` et `verified_by_independent_operator=false` restent inchangés**. Le graphe n'accepte pas un « certificat » autoémis par le même administrateur comme substitut.

Pour les douze portes natives, la liste est exactement celle de `scripts/roadmap_guardian.py` et `research/G03_R6E_OOB_ORIGIN_NO_EXTERNAL_AUTHORITY.json`. Les douze sont encore `UNMET_INDEPENDENT_EXTERNAL_EVIDENCE`, avec `admitted=false`. R6G reste `LOCKED_BY_SEQUENCE`.

## 3. Huit attaques contradictoires couvertes

| ID | Stratégie adversariale | Verdict exigé |
|---|---|---|
| A1 | Transformer 1 255 tests propriétaire et CI 9/9 en « certificat indépendant » | Refus |
| A2 | Afficher neuf fonctions sources comme neuf revues indépendantes achevées | Refus |
| A3 | Signer localement une prétendue racine OOB avec deux vraies clés Ed25519 du même administrateur | Refus |
| A4 | Utiliser un autre hôte/VPS sous la même administration et prétendre à un gardien séparé | Refus |
| A5 | Restaurer trois fichiers cohérents et prétendre qu'ils établissent un registre global monotone | Refus |
| A6 | Réutiliser un ancien journal propriétaire PASS en changeant son commit source | Refus |
| A7 | Inscrire `formal_certificate=true` ou douze portes `MET` dans du JSON local | Refus |
| A8 | Déduire du bon résultat des audits une autorisation humaine ou une décision formelle inexistante | Refus |

Ces scénarios sont **des hypothèses de falsification encodées dans la matrice**, et non huit attaques indépendantes exécutées contre un vrai gardien extérieur. Les tests réels R9 reprennent de véritables signatures OpenSSH Ed25519 locales, la course R4 entre processus, la mutation et restauration R5, ainsi que le contre-exemple historique de restauration cohérente R5Q. Aucun gardien extérieur fictif n'est joué comme un véritable opérateur indépendant.

## 4. Garde R9 et comportement de refus

`scripts/bureau_001_r9_final_evidence_decision.py` :

- exécute les audits originaux R8 → R7 → R1/R2, préserve les 99 empreintes sources R1, les neuf fiches `NOT_SUBMITTED`, les sept contradictions documentées et les douze portes ;
- maintient une **distinction temporelle** : le dossier source R8 est figé avec `owner_mac_intel_r8=PENDING` à sa création, tandis que R9 référence séparément le journal propriétaire PASS reçu ultérieurement, sans réécrire R8 ;
- exige exactement cinq dépendances externes en état `NOT_RECEIVED`/`NOT_ENROLLED`/`NOT_AUTHENTICATED`/`NOT_VERIFIED`/`NOT_DECIDED` et des champs d'attestation nuls ;
- refuse toutes les affirmations autoémises de certification, y compris un paquet qui proclame les neuf revues, un gardien, une racine OOB, un historique externe et douze portes déjà satisfaites ;
- propose seulement une **décision administrative de recherche `HOLD_FORMAL_R6F_CLOSURE_MISSING_INDEPENDENT_EVIDENCE`**, sans mettre à jour la RoadMap, fusionner une PR ou activer une capacité.

`--audit` effectue un contrôle source réel **en lecture seule** et retourne intentionnellement le **code 2** pour signaler un refus. `--exercise` réutilise les laboratoires POSIX/OpenSSH/R5Q **réels** de R8, avec code 2 si la preuve locale reste strictement refusée. Une incohérence ou un incident expérimental retourne **code 3** : cette erreur ne doit jamais être transformée en succès.

24 nouveaux tests : 16 portables et huit cas réels POSIX (Mac Intel/Apple Silicon/Linux), non exécutés sous Windows. Les originaux R3–R8 et R6A–R6E/R5A–R5Z/R4Y–R4Z restent inchangés. Les six tests natifs signés R4M XPC/CDHash doivent rester exécutés en série sur macOS, sans reprise ni dérogation.

## 5. Décision de programmation : arrêter les boucles sans nouveau fait

R9 recommande de **ne pas multiplier des R10/R11 locaux qui ne feraient que confirmer de nouveau l'absence d'autorité externe**. Les travaux suivants doivent répondre à l'un des deux critères :

- **Nouvelle hypothèse de sécurité falsifiable**, avec risque concret, contre-exemple mesurable et test réel qui apporte une preuve technique nouvelle, en conservant le statut `RESEARCH_ONLY`.
- **Nouvelle preuve du monde réel**, provenant d'un opérateur réellement séparé, d'un canal hors bande authentifié, d'une conservation antirollback indépendante ou d'une revue spécialisée effectivement réalisée, avec provenance vérifiable et consentement.

Le présent audit ne contacte personne, n'enrôle aucun opérateur, n'exporte aucune clé secrète et n'effectue aucun appel réseau vers un tiers. Toute décision future d'admission exige les contrôles et autorisations de la RoadMap, de la Constitution et du Bureau ; une PR DRAFT réussie n'autorise rien.

## 6. Rejeu et statut

```bash
python3 -B scripts/roadmap_guardian.py --audit
python3 -B scripts/bureau_001_r8_closure_gate.py --audit
python3 -B scripts/bureau_001_r9_final_evidence_decision.py --audit
python3 -B -m unittest discover -s scripts -p 'test_bureau_001_r9_final_evidence_decision.py' -v
python3 -B scripts/bureau_001_r9_final_evidence_decision.py --exercise
# Audit/exercice R9 : exit 2 intentionnel si le dossier reste BLOCK, 3 si erreur.
```

```ini
G03_R6F_R9=RESEARCH_FINAL_CONTRADICTORY_AUDIT_CANDIDATE
G03_R6F_TECHNICAL_EVIDENCE=OWNER_LOGS_AND_CI_WITH_SCOPE_LIMITS
R6F_FORMAL_CLOSURE=HOLD_MISSING_INDEPENDENT_EVIDENCE
INDEPENDENT_SPECIALIST_REVIEWS=0
EXTERNAL_CUSTODIAN=NOT_ENROLLED
OWNER_OOB_ROOT=NOT_AUTHENTICATED
GLOBAL_MONOTONIC_ANTIROLLBACK=NOT_VERIFIED
NATIVE_ADMISSION_GATES=12_UNMET
R6F=IN_PROGRESS_UNCERTIFIED
R6G=LOCKED_BY_SEQUENCE
PRODUCTION=NOT_AUTHORIZED
```
