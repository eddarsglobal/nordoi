# NORDOI APP-001 — Contrôle de seuil et rapport de décision

**Statut :** application expérimentale réelle, **pas** un service de production
ni une décision réglementaire ou de sécurité. Code local, gratuit, hors réseau,
construit sur les profils NORDOI existants.

## Utilité et usage

L'utilisateur fournit un score entier de 0 à 100. La règle de démonstration
est **score supérieur ou égal à 60**. NORDOI calcule approved et rejected
à l'exécution et publie **deux vrais fichiers** dans un nouveau répertoire :

- quality-gate/decision.txt : décision, refus et score observé.
- quality-gate/report.txt : score, valeur suivante et critère atteint.

Pour un score de 73 :

    decision.txt: approved=true, rejected=false, score=73
    report.txt:   score=73, next=74, meets_threshold=true

Pour un score de 59, la décision s'inverse. Le score exact 60 est accepté.

## Commande unique depuis la racine du dépôt

    python3 examples/apps/app001_quality_gate/run_app001.py --score 73

Le lanceur Python est **uniquement un adaptateur de lancement** : il valide
l'intervalle utilisateur, prépare un répertoire de sortie privé et unique
dans Downloads (ou temporaire si Downloads manque), invoque l'exécutable
Rust réel avec bundle-write, lui accorde explicitement --grant-output-dir,
puis vérifie les octets des deux fichiers et le reçu NORDOI. Cargo est
exécuté --offline. Il ne remplace ni ne simule le programme .noi.

Sortie attendue : NORDOI_APP001=PASS ; APPROVED=true ;
NORDOI_REAL_BUNDLE=... Les deux fichiers restent sur disque, hors dépôt,
dans leur répertoire propre.

Pour choisir un autre parent déjà existant :

    python3 examples/apps/app001_quality_gate/run_app001.py --score 59 --output-parent "$HOME/Downloads"

Après compilation, on peut aussi passer --engine ./target/debug/nordoi
pour utiliser directement l'exécutable sans Cargo.

**Limites :** l'intervalle 0..100 est appliqué par le lanceur Python,
pas par une nouvelle primitive NORDOI. Le code .noi utilise l'entrée
dynamique key_code. Pour P2.7 seules deux capacités explicites de fichiers
sont accordées. Écrasement refusé, publication par renommage de répertoire
sur le même système de fichiers, durabilité après coupure électrique
explicitement NON prouvée.

## Modules et paquet — autre parcours déjà existant

Le sous-projet pure_project/ est un véritable projet NORDOI **Profile 1**
de deux modules : app.main important app.rules. Il retourne un résultat
pur 0 (score inférieur à 60) ou 1 (score supérieur ou égal à 60).

    cargo run --offline --bin nordoi -- module-graph-run examples/apps/app001_quality_gate/pure_project/src app.main 73

Pour construire le paquet .npkg, copier pure_project dans un répertoire
temporaire (sans écrire dans les sources du dépôt), puis :

    cargo run --offline --bin nordoi -- build /chemin/vers/copied_pure_project
    cargo run --offline --bin nordoi -- build /chemin/vers/copied_pure_project --locked

La CI exécute réellement ces opérations sur une copie temporaire.

**Les modules Profile 1 et l'écriture groupée Profile 2 sont deux parcours
distincts.** Le paquet pur n'exécute pas les écritures P2.7, et le source
P2.7 n'importe pas app.rules. Aucune intégration native unique inventée.

## Tests réels et gouvernance

16 contrôles automatiques vérifient le véritable moteur NORDOI compilé,
sans simulation : seuils et scores frontières, permission absente, refus
d'écrasement, reçu explicite, exécutions du lanceur, saisies interdites,
modules/imports réels et paquet déterministe construit avec lock.

CI quatre plateformes : Mac Intel, Apple Silicon, Linux, Windows.
C20 préalable : E17 FIX3 9/9 succès au SHA exact
5f1a06d8541a2a76e6d2bb574b8569712092c0d0.
La RoadMap/Bureau R6F garde R6G verrouillé et 12 conditions natives
non satisfaites. Aucune fusion ni production autorisée.

**Zéro testeur humain imposé, aucune vérification manuelle exigée du
fondateur.** Aucun changement du runtime historique ni de la Constitution.
