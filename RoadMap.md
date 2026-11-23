# NORDOI — RoadMap officiel et Bureau de gouvernance (G0.3)

**Statut : candidat de gouvernance G0.3 R6F, PR DRAFT, non fusionné.** Ce document n'autorise aucune nouvelle capacité native. Source de vérité vérifiable : [NORDOI_BUREAU_PLAN_v1.json](governance/NORDOI_BUREAU_PLAN_v1.json). Procédure et neuf fonctions de revue : [Bureau.md](Bureau.md). Contrôle local sans écriture : [scripts/roadmap_guardian.py](scripts/roadmap_guardian.py).

## 1. Constitution et hiérarchie

Les lois [CONSTITUTION.md](CONSTITUTION.md) C3 (refus par défaut), C8 (IA utilisable sans dépendance obligatoire à l'inférence), C14 (preuve avant évolution) et C20 (la version précédente doit être verte dans la CI GitHub avant le démarrage de la suivante) prévalent. Les six exigences [Future-Native Gate](governance/future_native_gate_v1.tsv) FNG1–FNG6 sont obligatoires. `docs/GOVERNANCE_PROGRESS.md` conserve les déclarations de référence certifiées ; la présente RoadMap ne les remplace pas.

Les jalons de laboratoire G0.3 R4A–R6E restent **recherche et contrôles négatifs**, et non admissions natives ou preuves d'une autorité externe réelle. Un `PASS` des tests certifie uniquement leurs assertions vérifiables, jamais une gouvernance administrative indépendante.

## 2. Bureau permanent des neuf fonctions spécialisées

Le Bureau NORDOI est une **organisation de revue par fonctions spécialisées d'IA** ; un intitulé de fonction ne signifie pas qu'un agent indépendant a réellement exécuté un audit ni qu'il dispose de sa propre clé, identité, machine ou autorité.

| Fonction | Spécialité obligatoire | Mission |
|---|---|---|
| ATLAS | Architecture Rust | Architecture, NAIR/NAM, invariants et transversalité |
| FERRUM | Sécurité native | XPC/CDHash macOS, Linux, Windows et limites système |
| BOUNDARY | Isolation des processus | Sandbox, FD, PID, frontières de propriété |
| CIPHER | Cryptographie | SSHSIG/Ed25519, clés, nonces et revocation |
| LEDGER | Gestion des preuves | Source SHA, provenance, témoins, cohérence historique |
| CAPSTONE | Sécurité des capacités | Contrats d'effets, autorisations et admissions |
| ADVERSARY | Analyse adversariale | Menaces, contournements et tests négatifs |
| VERITY | Qualification des versions | Tests, CI, rejeu propriétaire, Rustfmt, Clippy |
| GUARDIAN | Gouvernance RoadMap | Ordonnancement, refus des transitions indues, décisions tracées |

La matrice de responsabilité, le protocole contradictoire et les limites de cette organisation sont définis dans `Bureau.md`.

## 3. Processus RoadMap → Bureau → RoadMapGuardian

1. **RoadMap** définit l'objectif du jalon, son parent exact, ses risques, les critères de qualification et les dépendances.
2. **Bureau** répartit les travaux aux neuf fonctions. Chaque fonction produit des observations, contre-exemples, erreurs et éléments probants référencés ; les rubriques non exécutées demeurent `NOT_SUBMITTED`.
3. **Développement** corrige de façon autonome les défauts reproductibles à l'intérieur du périmètre de recherche déjà autorisé, sans toucher aux données protégées, aux signatures externes, au système natif ou à la production.
4. **VERITY + ADVERSARY + LEDGER** confrontent la CI, les tests natifs, les identités Git et les artefacts propriétaires. Le Bureau doit distinguer `replay propriétaire rapporté`, `vérification CI indépendante` et `certification d'autorité externe`.
5. **GUARDIAN** exécute un contrôleur déterministe, lecture seule et sans modèle nécessaire, refuse la disparition des douze interdictions et rejette les statuts non justifiés.
6. Une transition de statut exige une **décision explicite et traçable**, des preuves vérifiées et un contrôle de séquence. Aucune réussite CI ou auto-déclaration d'agent ne vaut admission native, fusion, production ou certification indépendante.

États autorisés de travail : `PENDING → IN_PROGRESS → TESTING → CERTIFIED_RESEARCH` pour un jalon réellement audité ; `REMEDIATE` si défaut ; `LOCKED_BY_SEQUENCE` lorsqu'un préalable manque. Le statut `TECHNICAL_VALIDATED_RESEARCH_ONLY` constate des tests réussis **sans affirmer CERTIFIED_RESEARCH**. Un statut de qualification technique et la permission d'admission native sont deux axes distincts.

## 4. Situation vérifiée et ordre officiel

| Jalon | Objet | État | Preuves / blocages |
|---|---|---|---|
| G0.3 R4A–R5Z | Laboratoires de frontières de confiance et gouvernance | Revue rétrospective demandée | Aucun nouveau certificat global accordé |
| G0.3 R6A–R6D | Défis et signatures réellement exercés sous même administrateur | Contrôles techniques réalisés | Pas de dépositaire indépendant |
| **G0.3 R6E** | Origine de la prétendue preuve hors bande | **TECHNICAL_VALIDATED_RESEARCH_ONLY** | Rejeu propriétaire fourni : **1 039 tests réguliers PASS**, **188/188 blobs Git PASS**, `SCRIPT_EXIT=0` ; CI GitHub **9/9 SUCCESS**, aucune attestation OOB indépendante |
| **G0.3 R6F** | **BUREAU-001 : revue transversale et contrat des preuves externes** | **IN_PROGRESS — reviews NOT_SUBMITTED** | Mission déléguée aux neuf fonctions, certification PENDING |
| **G0.3 R6G** | Vérification future d'un véritable dépositaire externe | **LOCKED_BY_SEQUENCE** | R6F non certifié et preuves administratives externes réelles absentes |

L'évidence R6E provient du journal Terminal communiqué par le propriétaire ; les neuf jobs GitHub ont été vérifiés séparément. Ce journal ne démontre pas l'indépendance administrative ni une certification formelle du Bureau.

## 5. Mission confiée au Bureau : G0.3 R6F / BUREAU-001

**Mandat immédiat :** examiner rétrospectivement les travaux R4A–R6E, approfondir les tests, trouver les erreurs reproductibles, conflits entre codes/rapports/règles et corrections non régressives. En parallèle, formaliser les preuves nécessaires pour le futur R6G : opérateur indépendant réel, vérification propriétaire hors bande, contrôle de clé séparé, racines de confiance, preuves OS/host distinctes, registres externes monotones et procédures de compromission/récupération.

**Livrables attendus, encore non réalisés :**
- un rapport par fonction avec référence de source, preuves et verdict `PASS | REMEDIATE | BLOCK | NOT_EXECUTED` ;
- une matrice de contradictions et de dépendances R4A–R6E, chaque correction reliée à ses tests ;
- un registre de couverture adversariale incluant R4M XPC/CDHash natif et les tests SSHSIG R6A–R6E ;
- un contrat de réception d'éléments externes qui distingue les preuves réelles, celles fournies par l'opérateur et les auto-déclarations locales ;
- un rapport contradictoire du Bureau et une décision explicite du responsable autorisé avant tout changement d'admission ou de statut certifié.

Handoff structuré : [NORDOI_BUREAU_R6F_HANDOFF_v1.json](governance/NORDOI_BUREAU_R6F_HANDOFF_v1.json). L'attribution de fonctions constitue une **instruction d'organisation**, pas l'exécution autonome de neuf agents en arrière-plan.

## 6. Verrous inaltérés

```ini
R6E_OWNER_MAC_INTEL=1039_REGULAR_PASS_OWNER_LOG
R6E_SOURCE_GIT_BLOBS=188/188_PASS_OWNER_LOG
R6E_GITHUB_CI=9/9_SUCCESS
R6F_BUREAU_HANDOFF=ASSIGNED_REVIEWS_NOT_SUBMITTED
R6F_CERTIFICATION=PENDING
R6G=LOCKED_BY_SEQUENCE
REAL_EXTERNAL_GUARDIAN=NOT_ENROLLED
OWNER_OOB_ROOT=NOT_AUTHENTICATED
INDEPENDENT_QUORUM=NOT_PROVEN
GLOBAL_MONOTONIC_ANTIROLLBACK=NOT_VERIFIED
NATIVE_ADMISSION_GATES=12_UNMET
PRODUCTION=NOT_AUTHORIZED
```

Ce jalon ne modifie ni le noyau Rust ni les versions déjà certifiées, ni une branche principale, et ne fusionne aucune PR. Une PR DRAFT de gouvernance doit rester DRAFT jusqu'à vérification et décision explicite.


## 7. Branche applicative IA — RoadMap officiel APP-016 à APP-021 (GOV-FIX1)

La source structurée officielle de ce parcours est désormais le champ research_application_track dans governance/NORDOI_BUREAU_PLAN_v1.json, avec l'annexe inspectable governance/NORDOI_APP_RESEARCH_TRACK_V1.json. Ce parcours est une branche de RECHERCHE APPLICATIVE sans privilège. Il ne réordonne ni R6E, ni R6F, ni R6G et ne lève pas les barrières d'admission natives.

| Jalon APP | Objectif | Commit source exact | PR DRAFT | CI au HEAD | Certification formelle |
|---|---|---|---|---|---|
| APP-016 | Politique de confiance du destinataire et rejeu Rust | e988b214c8daa2b37c6abc36e351ce988bea7880 | #168 | 13/13 SUCCESS | NOT_CERTIFIED |
| APP-017 | Révocation/rotation du roster du destinataire | c048f1e17590b8d863d38a067fe6ac00d78e61c2 | #169 | 13/13 SUCCESS | NOT_CERTIFIED |
| APP-018 | Journal local à fraîcheur monotone | 0aa1fe01fe9391ed67a0fa20f8af7280af56a1ad | #170 | 13/13 SUCCESS | NOT_CERTIFIED |
| APP-019 | HEAD épinglé par le destinataire hors paquet | fa89752c1a62f3577134017d6a90b62d46a25cf7 | #171 | 13/13 SUCCESS | NOT_CERTIFIED |
| APP-020 | HEAD signé Ed25519/SSHSIG | 6f0f5625e282b4cd5a9ad5d527087502f5842f71 | #172 | 13/13 SUCCESS | NOT_CERTIFIED |
| APP-021 | Rotation à double signature, révocation et gel | 6e9c300922348d0ebab4318c7f9c2b4dd4cd76b4 | #173 | 13/13 SUCCESS | NOT_CERTIFIED |

Les six PR #168–173 restent DRAFT, ouvertes et non fusionnées. Le parent exact de chaque source est épinglé dans l'annexe. Les 78 contrôles CI GitHub observés donnent une qualification technique rétrospective TECHNICAL_VALIDATED_RESEARCH_ONLY : pas CERTIFIED_RESEARCH, pas d'admission native. Les neuf rapports du Bureau R6F restent NOT_SUBMITTED ; les commentaires antérieurs intitulés Bureau PASS_WITH_CONTROLS sont des synthèses techniques du coordinateur, pas des certifications autonomes.

GOV-FIX1 répare le décalage entre RoadMap et recherche APP. RoadMapGuardian doit valider strictement les six jalons, leurs parents C20, leur état de recherche et les interdictions d'auto-certification. Un workflow neuf fonctions exécute des CONTROLES MACHINE vérifiables au HEAD ; ses sorties ne valent ni neuf agents IA distincts ni attestations indépendantes.

APP-022 reste LOCKED_BY_GOVERNANCE_REVIEW jusqu'à rapprochement et décision documentée. Le travail de correction non privilégié reste permis et n'exige pas d'intervention humaine. Les 12 conditions natives restent non satisfaites ; R6F uncertified et R6G locked.
