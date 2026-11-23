# NORDOI — Extension officielle candidate de RoadMap, APP-016 à APP-022 (GOV-FIX1)

**Addendum versionné non rétrospectif.** L'ancienne RoadMap.md et le plan Bureau originel doivent rester inchangés octet à octet, conformément au registre R6F de 99 blobs Git figés. Le fichier de référence de cet addendum est governance/NORDOI_APP_RESEARCH_ROADMAP_V1.json ; chaque état technique est rapproché de governance/NORDOI_APP_RESEARCH_TRACK_V1.json. Cette extension ne remplace ni les règles constitutionnelles ni R6F/R6G.

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
