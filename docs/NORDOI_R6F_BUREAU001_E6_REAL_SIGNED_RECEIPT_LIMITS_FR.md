# NORDOI G0.3 — BUREAU-001 / R6F E6
## Signatures cryptographiques réelles, reçus de témoin et limite de l'administration unique

**Décision d'exécution du Bureau : recherche seulement. Aucun humain ne testera NORDOI.** Sources normatives gelées et conservées : `RoadMap.md` (C3, C8, C14, C20), `Bureau.md` et les 99 blobs Git historiques R1. E6 reste dans le mandat R6F sans commencer R6G.

### Succession vérifiée, et défaut falsifiable

E5, exact HEAD `ccad109eff3931b7c0417bf5e25e3291617bd5c7`, a qualifié un contrat **deny-only** de six obligations de témoin dans GitHub Actions : [CI spécialisée](https://github.com/eddarsglobal/nordoi/actions/runs/44025229325) **4/4 SUCCESS** et [Test Gate Rust](https://github.com/eddarsglobal/nordoi/actions/runs/44025229249) **5/5 SUCCESS**. Les signatures OpenSSH Ed25519 réellement utilisées sont des **signatures locales sous une même administration**. E4 avait déjà établi qu'un acteur pouvant modifier journal et manifeste peut produire des empreintes cohérentes contrefaites.

La question nouvelle de **E6** est plus précise : *un reçu de témoin authentiquement signé, qui lie un commit exact, un manifeste, un défi et le reçu antérieur, empêche-t-il un même administrateur d'émettre des versions incompatibles et de faire rejouer un ancien reçu après perte d'une mémoire locale des nonces ?*

**Hypothèse : non.** La signature authentifie les octets et la possession d'une clé ; si la clé privée et la liste des signataires permis restent sous la même administration, aucune observation externe ne certifie quelle histoire a été vue auparavant. Un processus vierge sans l'historique des nonces peut accepter de nouveau un ancien reçu cryptographiquement valide.

### Contrat de paquet (strict et sans admission)

`scripts/bureau_001_e6_signed_receipt_rebinding.py` vérifie **13 champs canoniques** dont : identifiant de protocole, projet, SHA40 du commit réellement checkout, empreinte du manifeste, empreinte du reçu précédent, défi SHA256, empreinte de la clé publique, époque, planchers de révocation et nonce, fenêtre temporelle locale, et une déclaration explicite non fiable de garde externe.

L'espace de signature OpenSSH réservé à cette recherche est `nordoi-r6f-e6-local-witness-receipt`. Il empêche l'acceptation d'une signature produite dans un autre espace. Un champ inconnu, double, non canonique, un mauvais type ou un mauvais hachage est rejeté. Tous les chemins de validation retournent **BLOCK**, y compris lorsque la cryptographie réelle valide les octets.

### Protocole d'expérience E6 — machines seulement

Sur POSIX avec OpenSSH réel, les tests créent une clé **Ed25519 locale éphémère** dans un répertoire de laboratoire `0700`, produisent un véritable reçu JSON canonique, le signent réellement et le vérifient dans un second processus OS. Aucun transfert de clé privée, aucune inscription externe, aucun réseau ou test humain.

Ils contrôlent les cas suivants :

1. La **signature locale authentique** est vérifiée pour l'empreinte source et le contexte exact, mais le résultat reste `BLOCK`, sans autorité externe.
2. Une modification des octets signés sans nouvelle signature est rejetée par le vrai vérificateur OpenSSH.
3. Une signature cryptographiquement réelle produite pour **un autre namespace** est rejetée.
4. Un défi, un commit, un manifeste ou un reçu antérieur différent de la requête est refusé même si la signature locale sur son propre paquet est valide.
5. Un **ensemble local conservé** de nonces déjà dépensés rejette le rejeu, mais un autre processus sans ce jeu local accepte de nouveau la signature valide — démonstration de l'absence de mémoire globale.
6. Le **même administrateur peut signer un second reçu au contenu d'historique différent** avec la même clé locale ; la seconde signature est véritable, sans que l'on puisse établir extérieurement laquelle est historiquement canonique.
7. La vraie expérience **R5Q de retour arrière cohérent des trois fichiers** démontre de nouveau la limite d'un plancher local face à une restauration complète.
8. Les fichiers Git historiques, la production et les identifiants des douze portes natives demeurent inchangés.

Les 24 tests `scripts/test_bureau_001_e6_signed_receipt_rebinding.py` comprennent **18 contrôles portables** et **six cas POSIX/OpenSSH réellement exécutés**. Sous Windows, les six cas spécifiques OpenSSH POSIX sont *SKIP*, jamais déclarés `PASS`. La CI vérifie les comptes réels, le commit exact et le contrôle natif signé R4M six en série sur macOS.

### Ce que E6 peut établir, et ce qu'il ne peut pas établir

| Réalisation technique | État de preuve possible |
|---|---|
| Signer puis vérifier les octets exacts avec une clé locale Ed25519 | Oui, **possession locale de clé** |
| Refuser un autre namespace, commit, défi et contexte | Oui, **contrôle cryptographique local** |
| Refuser un nonce déjà connu dans une collection locale non altérée | Oui, **anti-rejeu local conditionnel** |
| Empêcher la restauration complète ou la réécriture du journal et du plancher par la même autorité | **Non**, limite R5Q toujours présente |
| Démontrer qu'une clé et un témoin sont sous un domaine administratif externe et indépendant | **Non**, aucune identité extérieure authentifiée |
| Débloquer une seule condition native, R6G ou la production | **Interdit** |

Le formulaire d'un reçu peut contenir une déclaration `claimed_external_custody=true`. Celle-ci ne crée **aucune indépendance**. Deux processus, deux clés, deux machines ou neuf fonctions du Bureau sous le même administrateur ne sont pas automatiquement neuf autorités distinctes.

### Résultat et contrôle de séquence

```ini
E6=RECEIPT_SIGNATURE_CONTEXT_AND_REPLAY_RESEARCH_DENY_ONLY
E6_TRUE_POSIX_OPENSSH=REQUIRED_FOR_NATIVE_EXERCISE
E6_LOCAL_CONTEXT_BINDING=TESTED_NOT_INDEPENDENT_WITNESS
E6_GLOBAL_NONCE_ANTIROLLBACK=NOT_PROVEN
E6_EXTERNAL_WITNESS_RECEIPTS=0
E6_HUMAN_TESTERS=0_FORBIDDEN
INDEPENDENT_SPECIALIST_REVIEWS=0
EXTERNAL_CUSTODIAN=NOT_ENROLLED
OWNER_OOB_ROOT=NOT_AUTHENTICATED
NATIVE_ADMISSION_GATES=12_UNMET
R6F=IN_PROGRESS_UNCERTIFIED
R6G=LOCKED_BY_SEQUENCE
PRODUCTION=NOT_AUTHORIZED
```

La décision E6 doit rester DRAFT, additive et sans fusion, avec examen contradictoire du Bureau : **CIPHER** vérifie les octets et le namespace, **LEDGER** vérifie le reçu antérieur et R5Q, **ADVERSARY** rejoue et re-signe, **FERRUM** vérifie les véritables outils/macOS, **VERITY** confronte le commit et la CI, **GUARDIAN** préserve l'ordre officiel. Les autres fonctions contrôlent les frontières de capacités et la non-régression. Il ne s'agit pas de neuf rapports exécutés indépendamment.
