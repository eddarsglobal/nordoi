# Runtime Upgrade Intelligence 0.1

K1.15 was designed after comparing several mature stateful-upgrade models.

## Apache Flink

Flink treats state evolution as a restore/migration problem and explicitly documents compatibility
limits. Its documentation is especially useful in showing why some identity-bearing state cannot be
silently rewritten without risking nondeterminism.

Reference:
https://nightlies.apache.org/flink/flink-docs-master/docs/dev/datastream/fault-tolerance/serialization/schema_evolution/

## Temporal

Temporal requires deterministic workflow execution and provides explicit versioning strategies when
workflow code changes over time. The important lesson for NORDOI is that changing executable logic
must not be treated as if historical state had always been produced by the new code.

Reference:
https://docs.temporal.io/workflow-definition

## Kubernetes storage-version migration

Kubernetes distinguishes the API representation from the stored representation and uses explicit
storage-version migration to rewrite durable objects when their storage version changes. This
reinforces the separation between ordinary recovery and an intentional version transition.

References:
https://kubernetes.io/docs/tasks/manage-kubernetes-objects/storage-version-migration/
https://kubernetes.io/docs/concepts/overview/working-with-objects/storage-version/

## NORDOI conclusion

NORDOI K1.15 adopts the strictest common principle: state migration is an explicit transition with
bounded deterministic rules and a durable lineage. It does not infer compatibility merely because
two programs have similarly shaped atoms, and it does not weaken K1.14's exact-program recovery
law.
