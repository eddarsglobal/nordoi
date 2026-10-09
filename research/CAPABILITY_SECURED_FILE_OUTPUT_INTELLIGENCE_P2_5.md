# Capability-Secured File Output Intelligence — P2.5

P2.5 deliberately opens the filesystem in the smallest useful form rather than turning filesystem access into ambient language authority.

The capability model already contains `FileWrite(String)`. P2.5 therefore reuses that exact-capability vocabulary instead of creating a parallel authority system. The source chooses one bounded ordinary file name; the host chooses one already-existing output directory; authorization grants exactly the source target name.

The host directory is intentionally excluded from canonical receipts. This keeps receipts reproducible across machines while still requiring an explicit host grant before any side effect. The security boundary is enforced by allowing only one direct child file, rejecting all path separators and traversal syntax, canonicalizing the host directory, and using create-new semantics.

Create-new is constitutionally important. Overwrite would add a second class of authority because destruction or replacement of existing host data is stronger than creating a new bounded artifact. P2.5 therefore proves only fresh-file creation. Overwrite, append, nested directories and dynamic content remain future additive milestones.

The design keeps Profile 1 and P2.1-P2.4 immutable. It adds no NAIR opcode, no runtime filesystem primitive, no kernel authority and no ambient filesystem capability. Materialization remains a tooling/host boundary after deterministic planning and exact authorization.
