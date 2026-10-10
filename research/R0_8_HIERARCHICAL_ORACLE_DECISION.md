# R0.8 decision — extend independent oracle to a three-generation scope chain

**Parent:** certified `r0.7` / `8452ce32a27ea93c4eb343dec7165ae4b6be704a`. **Disposition:** TEST-ONLY candidate, not promoted, not an implementation of NORDOI runtime concurrency.

R0.6 introduced a separate single-scope oracle; R0.7 expanded it to one root and one child. R0.8 extends the **separately specified oracle** (not the SUT semantics) to a root -> child -> grandchild. The R0.2 source, r0.7 history, kernel, NAIR and compiler remain unchanged. The scope is deliberately capped at three total scopes and two cumulative tasks/resources per scope so exact transition-level conformance can be reviewed.

### Independent evidence vs. reusing implementation

The oracle tracks its own scopes, effect declarations, grants, resources, tasks, outcomes and accepted-event count. It may import shared public enum value types for comparison but must not query the R0.2 model for predicted results. The SUT provides actual receipts/errors only after oracle prediction. Rejects must leave both candidate states unchanged. A deliberately altered oracle result is tested only as synthetic mismatch detection; not a real R0.2 defect.

### Acceptance gate

1. 26/26 bounded Rust tests and governance witnesses map exactly.
2. Generated 5,120 attempted comparisons, 7,776 bounded words and 90 legal three-pair orders pass.
3. Existing r0.2.1 through r0.7 conformance checks and complete local release gate pass.
4. `git diff --exit-code r0.7 -- .` reports no modifications to certified tracked files.
5. CI 5/5 on the commit exact SHA precedes an annotated `r0.8` tag.

### Future-native and non-claim

The experiment cannot be used as a source of host authority; there is no runtime task executor or real parallelism in these tests. Production integration is **not** approved. Proving arbitrary-depth hierarchy or real-time cancellation remains open. No formal or universal security claim follows from a passing test suite.
