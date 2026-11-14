# NORDOI G0.3 R6F — BUREAU-001 / R1 : audit transversal de provenance

**Statut : contrôle de coordination fondé sur les sources, pas neuf audits indépendants.** Ce rapport prolonge `RoadMap.md`, `Bureau.md` et `RoadMapGuardian` sur le parent immuable `a81bf04576a9e9b3f43c2ef56a1a5eb19fd731ae` (PR #123). Le jalon reste **R6F IN_PROGRESS / BUREAU FORMAL CERTIFICATION PENDING / R6G LOCKED**.

## Mandat et méthode

Les neuf domaines du Bureau sont utilisés pour confronter les preuves existantes. Aucun outil n'a lancé neuf agents autonomes, aucun dépositaire externe n'a fourni de certificat de confiance, et aucun nouveau jalon natif n'est autorisé. Le vérificateur `scripts/bureau_001_r1_audit.py` vérifie de manière **locale et reproductible** des empreintes **Git blob SHA-1** déjà connues depuis l'arbre GitHub du commit parent précis. Le référentiel des empreintes provient de `governance/NORDOI_R6F_BUREAU001_R1_PINNED_SOURCES.json`, établi à partir du véritable arbre de la branche R6F, et non d'empreintes reconstituées sans base source.

Ce contrôle couvre les **99 objets originaux** suivants : **24 tests Rust R4A–R4X**, **33 couples source/test Python R4Y–R6E = 66 fichiers**, **quatre fichiers de garde R6B–R6E** et **cinq fichiers de gouvernance R6F**. Une concordance Git blob est une preuve de cohérence avec le manifeste de référence, **pas une preuve de signature indépendante du commit**, ni une analyse sémantique complète des 99 fichiers. La CI multi-système complète cette vérification, mais ne l'élève pas en certification externe.

## Constatations reproductibles

| Référence | Axe du Bureau | Constat | Traitement |
|---|---|---|---|
| **R6F-R1-001** | LEDGER / VERITY | Les originaux gelés R6B, R6C, R6D et R6E affichent tous `owner_mac_intel_replay=PENDING`, alors que les journaux du propriétaire ont été fournis après ces commits : respectivement 967, 991, 1 015 et 1 039 tests PASS | **Réconciliation chronologique additive**, pas réécriture des quatre fichiers historiques |
| **R6F-R1-002** | GUARDIAN | Les neuf fonctions du premier handoff sont encore `NOT_SUBMITTED` ; un intitulé d'agent ou une CI verte ne valide pas une revue indépendante | Maintien de `R6F=IN_PROGRESS` et de la certification en attente |
| **R6F-R1-003** | CIPHER / CAPSTONE / BOUNDARY | Les signatures R6B–R6E et les processus sous le même administrateur ne prouvent ni racine hors bande ni gardien externe ; les douze gates des quatre fichiers de preuve sont `UNMET` | R6G **LOCKED**, aucun déblocage natif |
| **R6F-R1-004** | VERITY / ADVERSARY | La première archive propriétaire R6D appelait un fichier R6C inexistant ; le kit R6D FIX1 a corrigé ce défaut et le nouveau rejeu Mac Intel a réussi | Exiger désormais un test d'intégration **shell → Python** avant chaque kit propriétaire |
| **R6F-R1-005** | ATLAS / LEDGER | Aucun manifeste antérieur ne pinne *ensemble* l'ensemble ciblé de 99 fichiers de recherche, preuves et gouvernance à ce parent R6F | Nouveau manifeste Git exact, contrôle `hashlib.sha1(b"blob "+len+NUL+data)`, tests de falsification |
| **R6F-R1-006** | GUARDIAN / ADVERSARY | RoadMapGuardian v1 bloque justement toute revendication de revue déjà réalisée, mais impose actuellement **neuf états `NOT_SUBMITTED` immuables** : le passage futur à des rapports de revue réels nécessitera un protocole v2 distinct | Conserver v1 pour l'enregistrement initial ; concevoir en R6F R2 un registre de revue vérifiable sans auto-certification |

Ces six constatations décrivent des **limites ou incohérences de documents/intégration**. Aucune vulnérabilité native indépendante n'est certifiée à partir des seules preuves disponibles. Tous les défauts observables doivent être corrigés dans le périmètre déjà autorisé, puis contrôlés par régression. Les signatures réelles des laboratoires ne sont pas des preuves de maîtrise d'une autorité extérieure.

## Neuf perspectives de revue : état documenté

| Fonction | Périmètre vérifié par cette coordination | Travail spécialisé restant |
|---|---|---|
| ATLAS | Pin des 24 tests natifs Rust, Constitution C3/C8/C14/C20, règle Future-Native | Revue sémantique exhaustive des contrats Rust |
| FERRUM | Présence et empreinte du test réel R4M XPC/CDHash, résultat CI déjà observé | Nouveau témoin matériel et examen signé du contexte natif |
| BOUNDARY | Limites R6D sur PID/UID, garde R6B–R6E | Audit complet des transferts d'autorité et du sandboxing |
| CIPHER | Contrats signés R6B–R6E et absence reconnue de gardien indépendant | Audit exhaustif des clés, révocations, compromis et récupérations |
| LEDGER | Sources historiques `PENDING` et preuves Terminal ultérieures, 99 pins | Obtention d'un archivage externe effectivement indépendant |
| CAPSTONE | Douze gates `UNMET` sur quatre générations de preuve | Vérification complète du chemin autorisation/capacité |
| ADVERSARY | Nouveaux tests de falsification du manifeste et des revendications de certificat | Campagne contradictoire interversions plus profonde |
| VERITY | R6E propriétaire déclaré 1 039 PASS /188 blobs, CI R6E et R6F 9/9 déjà vérifiées | Reproduction sur environnement physique indépendant et revue des kits |
| GUARDIAN | R6F attribué mais non certifié ; R6G verrouillé | Définir une transition documentée de revues réelles, avec décision autorisée |

Le fait qu'une perspective soit examinée ici ne transforme **aucun** des neuf éléments `review_status=NOT_SUBMITTED` de `NORDOI_BUREAU_R6F_HANDOFF_v1.json` en rapport d'agent exécuté. Il s'agit d'un **audit de coordination** avec preuves statiques et tests exécutables.

## Protocole de réconciliation des statuts

Les quatre dossiers originaux étaient correctement au statut `PENDING` au **moment de leur création**, avant l'arrivée des journaux du Mac Intel. Le nouvel `governance/NORDOI_R6F_BUREAU001_R1_CHRONOLOGY.json` garde les valeurs historiques intactes et consigne séparément le nombre de tests et d'empreintes, les identifiants des actions GitHub et la portée **OWNER_SUPPLIED_CHAT_TERMINAL_OUTPUT**. Cette provenance n'est pas une signature de machine signée par un tiers ni un certificat de dépositaire externe.

Les résultats GitHub précédemment contrôlés directement sont consignés par numéro de workflow, mais le contrôleur local **ne se connecte pas à GitHub**. Il refuse uniquement les modifications non conformes aux valeurs observées et au manifeste ; il ne prouve pas à lui seul que des jobs GitHub sont toujours valides après une modification de branche.

## Contrôles automatiques

```bash
python3 -B scripts/roadmap_guardian.py --audit
python3 -B scripts/bureau_001_r1_audit.py --audit
python3 -B -m unittest discover -s tests -p 'test_bureau_001_r1_audit.py' -v
python3 -B scripts/bureau_001_r1_audit.py --attempt-unlock-r6g
# Dernière commande : BLOCK et code de sortie intentionnel 2.
```

Les 24 nouveaux tests adversariaux attaquent l'intégrité des fichiers Python et Rust, les empreintes, les suppressions, les liens symboliques, les chemins traversants, les faux résultats du propriétaire, les faux identifiants CI, la promotion de statut, le faux gardien indépendant et la modification du registre ancien. Ils opèrent exclusivement sur des **copies temporaires**. Ces tests sont ajoutés aux 24 tests RoadMapGuardian précédents et aux régressions R4Y–R6E.

## Verdict provisoire

```ini
G03_R6F_BUREAU001_R1=COORDINATOR_SOURCE_ANCHORED_AUDIT
FROZEN_PARENT=a81bf04576a9e9b3f43c2ef56a1a5eb19fd731ae
FROZEN_GIT_BLOBS=99_SELECTED_SOURCE_AND_TEST_PATHS
NEW_ADVERSARIAL_CASES=24
CHRONOLOGY_R6B_R6E=4_ADDENDUM_EVENTS
BUREAU_REVIEW_FUNCTIONS=9
INDEPENDENT_SPECIALIST_REVIEWS_COMPLETED=0
R6F=IN_PROGRESS_UNCERTIFIED
R6G=LOCKED_BY_SEQUENCE
EXTERNAL_GUARDIAN=NOT_ENROLLED
OWNER_OOB_ROOT=NOT_AUTHENTICATED
GLOBAL_MONOTONIC_ANTIROLLBACK=NOT_VERIFIED
NATIVE_ADMISSION_GATES=12_UNMET
PRODUCTION=NOT_AUTHORIZED
```

**Prochaine sous-mission R6F R2 :** registre progressif de véritables comptes rendus de revue et campagne contradictoire approfondie. Ni le présent audit, ni la CI, ni les empreintes locales ne peuvent élever un agent IA à une autorité externe indépendante, fusionner la PR ou modifier une admission native.
