# NORDOI — G0.3 R6F BUREAU-001 R4

## Audit adversarial réel : fichiers de signataires, descripteurs ouverts, clés SSHSIG et provenance

**Périmètre : RECHERCHE UNIQUEMENT — DENY-ONLY — PR DRAFT.** Parent gelé exact : `2dd46dc07568bf382c49b1c8bca0cba8900b7b1f`, R6F R3, qualifié dans la CI et par le propriétaire sur Mac Intel (1 135 tests réguliers PASS selon son journal, 188 empreintes sources, 99 empreintes de l'audit R1, 24 tests R3). R4 n'écrase pas les preuves historiques ni le moteur Rust.

## Constat R6F-R4-001 : correction R3 encore vulnérable à une fenêtre TOCTOU

Le correctif R3 `research/g03_r6f_r3_signer_binding.py` apporte la comparaison exacte entre la clé annoncée et le fichier local `allowed_signers`. Pourtant, la fonction `bind()` ouvre le fichier, le vérifie et **ferme son descripteur avant** que le code original `research/g03_r6e_oob_origin_intake.py` ne lance le véritable `ssh-keygen -Y verify -f <chemin>`.

Cela sépare deux instants distincts :

1. **Contrôle** : le fichier `allowed_signers` contient effectivement la clé publique candidate B, donc R3 valide la correspondance.
2. **Course réelle** : un **deuxième processus OS**, sous le même administrateur, effectue un véritable `os.replace` atomique de ce chemin par un fichier autorisant une troisième clé Ed25519 C. La coordination de laboratoire déclenche le remplacement **après le contrôle et avant l'ouverture par OpenSSH**, sans utiliser de temporisation aléatoire.
3. **Utilisation** : un SSHSIG véritable sur les octets originaux R6E, émis par C, devient valide pour le nouveau contenu `allowed_signers` que lit OpenSSH. Le code R3 pouvait alors émettre le libellé `R3_SIX_LOCAL_SSHSIGS_BOUND_TO_EXACT_DECLARED_KEYS_NO_EXTERNAL_CUSTODY`, bien que la sixième signature ne provienne pas de B.

Il s'agit d'un **défaut d'attribution et de continuité de preuve dans un laboratoire à refus systématique**, **pas** d'une admission native accordée. R3, même trompé par cette course, renvoie toujours `BLOCK / NOT_AUTHORIZED`.

## Correction expérimentale R4

La nouvelle recherche `research/g03_r6f_r4_fd_race.py` maintient les descripteurs ouverts pour les clés publiques et pour chaque liste `allowed_signers` durant la chaîne de vérification. Elle compare les octets lus depuis les fichiers ouverts aux identités attendues avant les contrôles OpenSSH. Sur POSIX, **la véritable commande `/usr/bin/ssh-keygen -Y verify` reçoit `/dev/fd/N`** à la place du chemin de liste modifiable, avec passage contrôlé du descripteur au sous-processus. La réouverture du chemin utilisateur ne décide plus quelle clé est admise à l'instant de la sixième signature. À la fin, la garde confronte de nouveau les descripteurs et les chemins.

Une substitution pendant la vérification doit donc entraîner un **refus** (`R4_SIGNER_FILE_RACE_OR_PROVENANCE_QUARANTINED`). L'absence de substitution mène seulement à `R4_SIX_LOCAL_SSHSIGS_VERIFIED_WITH_HELD_ALLOWED_FDS_NO_EXTERNAL_CUSTODY`, qui reste également un **refus d'admission**.

L'expérimentation `--exercise` utilise de **véritables clés Ed25519 et des signatures OpenSSH réelles**, un vrai deuxième processus Python/OS utilisant `os.replace`, et compare le résultat R3 historique et celui de R4 avec la même clé C. Le code de l'instrumentation synchronise l'ordre des appels ; il ne remplace ni l'opération cryptographique ni la course de fichiers par une simulation.

**Limites essentielles :**
- un descripteur ouvert stabilise l'identité des octets effectivement lus malgré le remplacement du chemin, mais n'établit **pas** la confiance en une racine cryptographique externe ni un registre monotone anti-retour arrière ;
- cela n'est **pas** une preuve complète contre l'écriture *en place*, la restauration des mêmes fichiers avant une relecture (effet ABA), le contrôle de la mémoire ou de la session du processus vérificateur par un même administrateur malveillant ;
- `/dev/fd/N` exige un OS POSIX avec un OpenSSH installé et la transmission explicite du FD : sur Windows, l'exercice natif est marqué **non réalisé**, jamais remplacé par une preuve inventée ;
- aucune preuve ne porte sur l'activation du code de laboratoire dans la production : le chemin R4 est **additif**, séparé du runtime et du compilateur NORDOI.

## Couverture des neuf domaines du Bureau

| Domaine | Contradiction ou preuve observée | Conclusion limitée |
|---|---|---|
| ATLAS — Rust | R4A utilise des threads Rust réels mais non intégrés au runtime public | Les laboratoires ne créent pas automatiquement de nouveaux opcodes |
| FERRUM — sécurité native | R4M XPC/CDHash repose sur de vraies signatures locales et des contrôles natifs | Ne vaut pas racine de confiance de production |
| BOUNDARY — isolation | Un FD ouvert reste associé à un inode lorsque le nom est remplacé | Une identité de fichier n'est pas une autorité administrative séparée |
| CIPHER — cryptographie | R3 lie les clés au chemin contrôlé, mais pas à l'objet effectivement ouvert ensuite | R4 utilise le FD déjà ouvert et refuse les changements détectés |
| LEDGER — preuves | Les journaux et empreintes Git sont des preuves locales de cohérence | Pas d'historique externe immuable démontré |
| CAPSTONE — capacités | Les douze portes d'admission natives sont toujours `UNMET` | Aucun privilège nouveau |
| ADVERSARY — attaques | Remplacement atomique réel entre le contrôle R3 et OpenSSH | Contre-exemple reproductible recherché |
| VERITY — qualification | 24 nouveaux cas R4 prévus, plus les régressions anciennes et R4M natif | CI et rejeu propriétaire à constater |
| GUARDIAN — gouvernance | Les neuf fonctions ne constituent pas neuf signataires réellement indépendants | R6F reste ouvert ; R6G verrouillé |

Source structurée : `governance/NORDOI_R6F_BUREAU001_R4_RACE_AUDIT.json`. Il s'agit de **neuf axes d'audit documentés par une revue de coordination**, et non de neuf agents exécutés séparément avec des attestations indépendantes.

## Tests et critères d'acceptation

```bash
python3 -B scripts/roadmap_guardian.py --audit
python3 -B scripts/bureau_001_r1_audit.py --audit
python3 -B scripts/bureau_001_r2_review_intake.py --audit
python3 -B -m unittest discover -s research -p 'test_g03_r6f_r4_fd_race.py' -v
python3 -B research/g03_r6f_r4_fd_race.py --exercise
# Le dernier exercice refuse toute autorisation et retourne volontairement 2.
```

Seize contrôles portables et huit contrôles POSIX réels vérifient l'identité de la clé, la liste de signataires, le remplacement du chemin, l'inode, le refus du nouveau garde, les signatures invalides et la répétition de l'exercice complet. La CI multi-OS doit confirmer les huit natifs sur macOS Intel, macOS Apple Silicon et Linux ; sur Windows leur absence doit être **explicitement indiquée**. Les six vrais tests XPC/CDHash R4M sont exécutés en série sur macOS sans reprise ni dérogation.

Les résultats doivent être documentés **après** les vérifications CI et le rejeu propriétaire, jamais présumés. Aucun remplacement, suppression ou modification d'un ancien fichier R3/R6E, d'un composant Rust ou du RoadMap initial n'est autorisé dans la PR R4.

```ini
R6F_R4=RESEARCH_AUDIT_CANDIDATE
LEGACY_R3_ALLOWLIST_TO_SSH_VERIFY_RACE=UNDER_REAL_TEST
R4_HELD_FD_VERIFICATION=UNDER_REAL_TEST
NINE_SPECIALIST_INDEPENDENT_REVIEWS=0
BUREAU_FORMAL_CERTIFICATION=PENDING
R6F=IN_PROGRESS
R6G=LOCKED_BY_SEQUENCE
EXTERNAL_GUARDIAN=NOT_ENROLLED
OWNER_OOB_ROOT=NOT_AUTHENTICATED
GLOBAL_MONOTONIC_ANTIROLLBACK=NOT_VERIFIED
NATIVE_ADMISSION_GATES=12_UNMET
PRODUCTION=NOT_AUTHORIZED
```
