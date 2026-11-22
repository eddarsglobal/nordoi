# NORDOI APP-015 — Qualification par rejeu natif indépendant d'un checkpoint portable

**22 novembre 2026 · Développement AI-only · Recherche non privilégiée**

## Séquence constitutionnelle RoadMap / Bureau / Guardian

La source C20 est **APP-014** au SHA `1920279523ee77d379a06353baba5dbb1c71b292`, qualifiée **13/13 jobs SUCCESS** (4 jobs applicatifs, 5 Rust, 4 E8 ; 2 112 exécutions applicatives PASS). La PR #165 est DRAFT, non fusionnée. APP-015 dérive de ce commit exact. Aucune fusion, aucun élargissement de permissions, aucune admission native ni production.

## Pourquoi APP-015

APP-014 garantit qu'un paquet de checkpoint contient un plan, des événements et des fichiers historiques **cohérents** avec leurs reçus natifs et leur SHA-256 externe. Cependant, cette cohérence **n'est pas la même chose qu'un recalcul du travail passé**. Le Bureau identifie cette lacune : un paquet reçu peut contenir des octets apparemment cohérents sans démontrer qu'un moteur Rust épinglé les reproduit.

APP-015 ajoute une **qualification par rejeu déterministe du moteur NORDOI authentique**, pour chaque tâche historiquement `COMMITTED`. Il part d'un paquet APP-014 autonome, valide tous ses octets à travers APP-014/013/010/006, puis utilise un **vrai exécutable Rust** et un **arbre de sources .noi** épinglés **séparément**. Dans un répertoire temporaire neuf, il relance les requêtes natives originales `agent-request-json`, compare **les reçus JSON natifs complets** (dont les champs de témoins P2.7, de preuve et de publication) et compare **octet pour octet** les fichiers `decision.txt` et `report.txt`.

Les branches `SKIPPED` sont validées par l'audit indépendant APP-010 et **ne sont jamais exécutées**. Les inputs, le binaire et les sources sont à nouveau contrôlés après le rejeu. Le répertoire privé de rejeu est éliminé automatiquement ; ni le paquet ni les fichiers d'origine ne sont modifiés.

### Trois commandes agent-à-agent

**1 — `verify-replay` : contrôle natif sans écrire dans les données du destinataire**

```bash
python3 -B examples/apps/app015_independent_native_replay/replay_app015.py verify-replay \
  --portable-dir /received/portable-checkpoint \
  --expected-portable-sha256 TRUSTED_EXTERNAL_PORTABLE_SHA256 \
  --engine /trusted/nordoi \
  --source-root /trusted/noi/src \
  --expected-engine-sha256 TRUSTED_ENGINE_SHA256 \
  --expected-source-sha256 TRUSTED_NOI_SOURCE_SHA256
```

Code 0 : `PASS` signifie **reproduction déterministe exacte par un moteur Rust épinglé**, et non signature, preuve formelle, cryptographie indépendante ou certification de production. Le rapport précise le nombre de jobs et fichiers reproduits ainsi que les limitations.

**2 — `restore-replay` : rejeu indépendant obligatoire avant restauration**

La commande reprend les arguments ci-dessus et ajoute `--restore-root /existing/empty/native-root`. Elle n'écrit dans cette racine **qu'après PASS du rejeu**. APP-014 contrôle encore le paquet et copie ensuite les vrais fichiers avec création exclusive dans une destination vide. Une restauration partielle ou ambiguë reste `ERROR/UNKNOWN`, sans rollback inventé.

**3 — `resume-replay` : rejeu indépendant obligatoire avant toute nouvelle tâche**

Avec la même `--restore-root`, mais une racine déjà restaurée et contrôlée, APP-015 **rejoue d'abord chaque tâche historique**, puis APP-014/APP-011 effectuent la vérification de la racine et les seuls effets restants. La sortie de la commande est **uniquement le suffixe JSONL natif** des nouvelles tâches, concaténable à l'original. Une reprise en doublon est refusée.

### Architecture de preuve

1. **APP-014/013/010/006** — validation lecture seule du conteneur portable, manifestes, événements, statuts, reçus et fichiers P2.7, avec empreintes externes.
2. **APP-007 (concept et algorithmes existants)** — principe de rejeu épinglé de calcul natif pour les agents, adapté aux graphes partiels APP-012 et non aux seules listes JSONL APP-005.
3. **APP-015** — pour chaque nœud `COMMITTED`, invocation réelle et fraîche de `nordoi agent-request-json` avec le `key_code` du plan vérifié, dans un répertoire temporaire neuf ; vérification APP-006 du reçu recalculé et des vrais fichiers.
4. **Comparaison forte dans ce modèle** — objet de reçu entier égal et deux fichiers P2.7 identiques octet pour octet ; les nœuds `SKIPPED` ne produisent rien.
5. **Contrôles après replay** — binaire et source inchangés, paquet vérifié une seconde fois, événements et inventaires identiques. Puis seulement : restauration ou reprise selon l'action.

```text
SHA-256 du paquet (canal externe de confiance)
              |
       APP-014/013/010/006
              |
  Exécutable Rust + source .noi (pins distincts)
              |
     REJEU NATIF indépendant
              |
   reçus exacts + fichiers P2.7 exacts ?
              |
        PASS      REJECTED / INCONCLUSIVE
          |             |
   restore/resume      AUCUN nouveau travail autorisé
```

**Codes :** `0` = qualificatif PASS, restauration vérifiée ou suffixe de reprise native ; `2` = `REJECTED`, preuve/pin/correspondance native contradictoire avant toute restauration/reprise ; `3` = `INCONCLUSIVE/UNKNOWN`, rejeu natif non terminé ou un effet de restauration/reprise incertain. Une défaillance native n'est jamais considérée comme preuve de validité.

### Limites strictes

APP-015 n'est **pas une seconde implémentation indépendante des primitives cryptographiques P2.7**. Il utilise un **second calcul par le même type de moteur Rust épinglé** : cela renforce la confiance dans la reproductibilité, pas dans l'absence de bogue partagé. L'authenticité externe des pins, l'identité de l'émetteur et la provenance de l'exécutable/source doivent être établies hors de ce module ; une signature indépendante n'est **pas** créée. Le répertoire temporaire privé **n'est pas un sandbox OS** : un exécutable arbitraire malveillant peut dépasser son FileWrite accordé via les permissions ambiantes de l'hôte. Le moteur doit donc être qualifié avant exécution.

Le rejeu ne démontre ni durabilité électrique, ni atomicité globale, ni sécurité contre les modifications concurrentes malveillantes, ni reprise arbitraire d'une tâche potentiellement engagée et sans reçu. Le développement n'ajoute aucun droit natif privilégié et reste interdit en production.

## Bureau — neuf perspectives, pas neuf signatures indépendantes

**ATLAS** : trajectoire APP-012 → 013 → 014 → 015 → 011 → 010 ; **FERRUM** : vrai binaire Rust contrôlé ; **BOUNDARY** : rejeu dans scratch, aucune restauration avant PASS ; **CIPHER** : pins externes obligatoires et absence de fausse garantie crypto ; **LEDGER** : reçus entiers et deux fichiers natifs par job ; **CAPSTONE** : `SKIPPED` n'est pas un commit, `ERROR/UNKNOWN` non récupérable automatiquement ; **ADVERSARY** : moteur/source modifié, faux pins, fichier altéré, reprise dupliquée, restauration non vide ; **VERITY** : quatre OS, vrais tests et démonstration avec données natives ; **GUARDIAN** : C20, AI-only, limites natives inchangées.

Ces neuf noms sont des **axes d'examen documentaire** ; aucune signature indépendante de neuf agents n'est revendiquée.

## Qualification GitHub

**40 nouveaux tests APP-015** basés sur les preuves natives réelles APP-012→013→014, plus **528 tests de régression APP-001…APP-014** par OS, soit **568 tests par OS, 2 272 exécutions applicatives attendues** sur Mac Intel, Apple Silicon, Linux et Windows. Rust Test Gate **5 jobs** + adversarial E8 **4 jobs** = **13 jobs**.

Démonstration exigeant : vrai checkpoint → vrai paquet portable déplacé → qualification `verify-replay` des deux jobs historiques → restauration `restore-replay` dans une racine vide → `resume-replay` des tâches nouvelles → audit APP-010 du DAG complet `PASS` ; source native modifiée mais épinglée doit être rejetée par différence de résultat, sans avoir écrit dans la destination.

**Aucun PASS final avant 13/13 jobs GitHub SUCCESS au SHA exact.**

```ini
R6F=IN_PROGRESS_UNCERTIFIED
R6G=LOCKED_BY_SEQUENCE
EXTERNAL_NATIVE_ADMISSION_GATES=12_UNMET
HUMAN_TESTERS=0
PR=DRAFT
MERGE=NONE
PRODUCTION=NOT_AUTHORIZED
```
