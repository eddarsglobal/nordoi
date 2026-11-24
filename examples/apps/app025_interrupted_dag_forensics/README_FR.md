# NORDOI — APP-025 : audit de récupération après interruption concurrente

**24 novembre 2026 · RoadMap candidate de recherche IA pour IA, sans privilège de production.**

## Chronologie C20 et décision du Bureau

Parent exact **APP-024** `0cd342d32c4dac9ac99ace569244df0c682a9af6`, PR #177 DRAFT. Les **22/22 contrôles GitHub au HEAD** sont SUCCESS ; **3 608/3 608 vrais tests natifs** et 56/56 assertions de démonstration sur Mac Intel, Apple Silicon, Linux et Windows. La décision de recherche non privilégiée APP-025 est consignée dans [PR #177](https://github.com/eddarsglobal/nordoi/pull/177#issuecomment-7126127019).

Les neuf fonctions de contrôle MACHINE du Bureau sont des contrôles déterministes et **non** neuf signatures d'agents IA indépendants. La certification formelle du Bureau reste **NOT_CERTIFIED**. Les 99 sources historiques R6F, Rust/NAIR/CLI, RoadMap.md et Bureau.md restent octet à octet inchangés.

## Fonction réelle

APP-024 pouvait créer de nouveaux dossiers natifs P2.7 pendant la reprise parallèle de tâches indépendantes, mais pas **auditer séparément une trace capturée après un arrêt**. Une tâche peut avoir publié son résultat avant qu'un reçu ait pu être capturé. APP-025 est un **auditeur de récupération en lecture seule**, et ne lance **aucun** processus Rust.

L'hôte fournit : (1) le checkpoint portable APP-014 et son SHA-256 de confiance obtenu hors du paquet ; (2) le dossier accordé contenant les publications réelles restaurées et les éventuels nouveaux dossiers ; (3) le suffixe JSONL APP-024 capturé, éventuellement vide, et son SHA-256 externe.

L'auditeur vérifie complètement le paquet APP-014, son plan, les données historiques et sa signature **absente** (SHA-256 de cohérence, pas identité authentifiée). Il reconstitue exactement `original-prefix.jsonl + suffix.jsonl` sans normaliser les octets, et lance l'audit indépendant APP-010 pour vérifier les **vrais reçus produits par NORDOI Rust**, les fichiers P2.7 correspondants, la topologie du DAG, les conditions, les décisions et **l'absence de dossiers orphelins**.

### Verdicts stricts

| État | Signification et action |
|---|---|
| `COMPLETE_VERIFIED` | Toutes les tâches sont vérifiées ; **ne pas rejouer** le DAG terminé |
| `ORIGINAL_CHECKPOINT_REUSABLE` | Aucun nouveau job ; le checkpoint APP-014 et les fichiers restaurés originaux sont cohérents |
| `CLEAN_PREFIX_NEEDS_NEW_CHECKPOINT` | Un suffixe partiel authentiquement cohérent a été observé ; **construire/sceller un nouveau checkpoint vérifié** avant toute reprise ultérieure, ne pas réutiliser l'ancien |
| `UNCERTAIN_BLOCKED` | Journal non reçu, tronqué, faux, fichiers orphelins, erreur native, pin manquant ou altération : **arrêter et ne pas relancer automatiquement** |

Le statut `CLEAN_PREFIX_NEEDS_NEW_CHECKPOINT` **n'est pas** une certification que le checkpoint original autorise la continuation : sa racine contient déjà des publications plus récentes. Aucune fonction ne recrée un reçu manquant ni ne suppose qu'un journal effacé prouve l'absence d'effet.

### CLI

```bash
python3 -B examples/apps/app025_interrupted_dag_forensics/audit_app025.py \
  --portable-dir /host/pinned/app014-checkpoint \
  --expected-portable-sha256 HOST_TRUSTED_SHA256 \
  --restore-root /host/granted/real-native-jobs \
  --suffix-events /host/captured/app024-suffix.jsonl \
  --expected-suffix-sha256 HOST_TRUSTED_CAPTURE_SHA256
```

Retour **0** si l'un des trois états cohérents est confirmé ; retour **3** et `UNCERTAIN_BLOCKED` en cas de preuve incomplète ou incohérente. Sortie un seul objet JSON canonique, sans chemin absolu. Tous les fichiers d'origine demeurent en lecture seule ; un fichier temporaire hors des dossiers du projet sert uniquement à l'audit APP-010.

## Critères Future-Native FNG1 à FNG6

FNG1 — C5/C9/C11/C13/C19 et authentification du résultat agent. FNG2 — pas de simple parité POSIX. FNG3 — CLI compacte réutilisant les formats APP-014/024. FNG4 — reprise refusée en cas d'effet orphelin, pin erroné ou journal incomplet. FNG5 — Python standard et vrai Rust pour produire les preuves sur quatre OS. FNG6 — zéro modification du code Rust, des anciens APP ou des 99 blobs figés.

## Gate de tests

24 nouveaux tests natifs APP-025 exploitent des checkpoints APP-012/013/014, de vrais jobs Rust APP-024, des copies strictes de leurs octets et des cas adversariaux. **902 régressions** APP-001…024 et 24 nouveaux tests = **926 par système**, soit **3 704 tests** quatre OS ; démonstration de 14 assertions par système. Rust Test Gate, E8 et neuf revues machine spécialisées au même HEAD exact avant verdict. Une preuve Machine CI n'est pas une certification indépendante du Bureau.

## Limites

Aucun vrai antirollback global après redémarrage, authentification indépendante de l'origine des pins, résistance à un administrateur malveillant contrôlant fichiers et pins, transaction multi-tâches ou recouvrement automatique d'une opération in-flight. L'auditeur classe la **cohérence observée** des preuves natives, pas l'historique complet garanti du monde réel. Pas de sandbox ni d'accélération revendiqués.

```ini
APP025_FORMAL_BUREAU_CERTIFICATION=NOT_CERTIFIED
R6F=IN_PROGRESS_UNCERTIFIED
R6G=LOCKED_BY_SEQUENCE
NATIVE_ADMISSION_GATES=12_UNMET
PRODUCTION=NOT_AUTHORIZED
MERGE=NONE
PR=DRAFT
```
