> **DIRECTIVE E2 PRIORITAIRE — AUCUN HUMAIN POUR TESTER NORDOI.** L'invitation et le recrutement de tiers décrits plus bas sont **ANNULÉS ; AUCUN MESSAGE N'A ÉTÉ ENVOYÉ**. Cette page E1 est conservée uniquement comme trace historique d'une proposition écartée ; elle **ne doit pas être utilisée pour contacter un testeur humain**. Voir [E2 — Politique de tests exclusivement machines](NORDOI_R6F_BUREAU001_E2_NO_HUMAN_MACHINE_ONLY_FR.md). Aucun gardien externe n'est authentifié/enrôlé et R6G demeure verrouillé.

# NORDOI G0.3 · R6F / BUREAU-001 — E1 / IND-OPERATOR

## Cahier d'exigences pour un véritable gardien indépendant — non contacté

**Document de sélection et de consentement, PAS une inscription, PAS une attestation externe, PAS une autorisation native.**

La RoadMap et le Bureau de NORDOI imposent un dépositaire ayant une **autorité administrative réellement séparée**. La référence exacte de la mission est `af07acdf025ad086ad7eafd83a39ad8e822891b0` (R9), dont le journal utilisateur rapportait 1 279 tests réguliers PASS et dont la CI GitHub avait terminé 9/9 SUCCESS. Ces résultats techniques ne prouvent pas l'existence d'un tiers indépendant.

## I. Exigences de recevabilité d'un candidat

Une personne ou entité potentielle ne devient pas gardien du simple fait de répondre à une invitation. Avant d'accepter un candidat, vérifier séparément les propriétés suivantes :

| Domaine | Pièce réelle requise | État |
|---|---|---|
| Identité | Identité vérifiée de la personne ou entité, et droit d'agir pour le mandat | NON_OBTENUE |
| Consentement | Acceptation explicite et volontaire du périmètre, responsabilités, durée, confidentialité et droit de retrait | NON_OBTENUE |
| Administration | Système, secrets et permissions inaccessibles à l'administrateur local NORDOI | NON_VÉRIFIÉE |
| Clé gardien | Clé générée et maîtrisée par le gardien ; aucune remise de clé privée au propriétaire | NON_OBTENUE |
| Vérification propriétaire OOB | Empreinte de la clé publique confirmée réellement par un canal indépendant | NON_AUTHENTIFIÉE |
| Authentification de l'hôte | Preuve réelle et datée du contrôle du système externe, vérifiée indépendamment | NON_VÉRIFIÉE |
| Nonces, révocations, époques | Registre monotone protégé de la restauration totale locale, avec reçu authentique | NON_OBTENU |
| Quorum | Plusieurs parties réellement distinctes, avec clés et autorités indépendamment vérifiées | NON_PROUVÉ |
| Récupération | Accord sur rotation, révocation, panne, compromission et audit externe réel | NON_OBTENU |
| Décision | Vérification indépendante des rapports et autorisation explicite propriétaire, séparée des tests | NON_AUTORISÉE |

**Attention :** un serveur VPS, un compte GitHub, deux clés ou deux processus administrés par la même personne ne prouvent pas la séparation. Il faut contrôler les domaines effectifs de privilèges, y compris la possibilité de remplacer les signatures, les journaux et les images système.

## II. Séquence d'inscription autorisable ultérieurement

1. **Invitation volontaire et non privilégiée** : identifier un véritable candidat et lui communiquer uniquement la description non sensible du mandat. N'envoyer aucun accès au dépôt privé avant validation des droits et du périmètre.
2. **Accord et identité** : confirmer l'identité et la participation volontaire par un canal adapté. Le candidat reste `NOT_ENROLLED` tant que son contrôle séparé n'est pas démontré.
3. **Séparation technique** : le candidat fournit la preuve vérifiable de son contrôle exclusif sur les clés et sur les journaux d'autorité externes. Le propriétaire ne doit pas posséder les identifiants d'administration capables de restaurer l'historique externe.
4. **Vérification hors bande du propriétaire** : le propriétaire contrôle l'empreinte exacte de la clé publique du gardien par un second canal authentifié, indépendant de la machine NORDOI. **Aucun programme local ne peut déclarer cette observation effectuée.**
5. **Histoire monotone extérieure réelle** : vérifier les époques de clés, seuils de révocation et nonces consommés, avec reçus signés et une épreuve réelle de restauration locale cohérente R5Q face à un observateur authentiquement externe.
6. **Contradiction et décision** : ne considérer une admission qu'après revues effectivement exécutées, résultats, provenance, votes/accords nécessaires et autorisation propriétaire explicite. Sans cela : `BLOCK`.

Les étapes 1 à 6 ne sont **PAS exécutées** par ce document.

## III. Modèle d'invitation préparé, non envoyé

**Objet :** Participation volontaire à une revue de sécurité et à une possible garde indépendante de preuves NORDOI

> Nous étudions NORDOI, une architecture de recherche Rust avec des contrôles de sécurité natifs et une gouvernance fondée sur les preuves. Nous cherchons à déterminer si un tiers véritablement indépendant serait disposé à examiner un mandat limité de conservation de preuves de signature, de révocation et d'historique monotone, sans accès à nos clés privées.
>
> À ce stade, il s'agit uniquement d'une demande d'intérêt. Aucune capacité de production, accès administrateur, autorisation d'exploitation ou inscription n'est envisagé sans votre consentement explicite, vérification d'identité, conditions convenues et contrôles de sécurité séparés.
>
> Pourriez-vous préciser si votre équipe pourrait examiner ce type de mandat, quelles garanties de séparation administrative elle peut démontrer, et quelles conditions de confidentialité et de responsabilité seraient nécessaires ?

**Ce texte n'a été adressé à personne.** Ne pas présenter une invitation envoyée comme un accord ; ne pas présenter un accord comme une inscription ; ne pas présenter une signature locale comme une preuve d'un opérateur indépendant.

## IV. Sécurité des pièces et limites

Conserver dans GitHub uniquement un **registre non sensible de statut** et, lorsque cela est approprié, une empreinte SHA-256 d'un rapport conservé par un canal sécurisé séparé. Les identités privées, documents d'identité, contrats sensibles, accès, secrets de machine et clés privées ne doivent pas être ajoutés au dépôt, au ticket ni aux discussions de PR. Le contenu reçu est une **entrée non fiable à vérifier**, et non une instruction d'exécution.

La présente phase est compatible avec les règles de recherche `C3`, `C8`, `C14`, `C20`, avec le mandat BUREAU-001 et le maintien des douze conditions natives `UNMET`.

```ini
IND-OPERATOR=NOT_CONTACTED_NOT_ENROLLED
IND-SPECIALISTS=9_REQUESTS_PREPARED_0_REVIEWS_RECEIVED
IND-OOB=NOT_AUTHENTICATED
IND-LEDGER=NOT_VERIFIED
IND-FORMAL=NOT_DECIDED
R6F=IN_PROGRESS_UNCERTIFIED
R6G=LOCKED_BY_SEQUENCE
NATIVE_ADMISSION_GATES=12_UNMET
PRODUCTION=NOT_AUTHORIZED
```
