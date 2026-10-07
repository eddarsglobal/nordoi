# NOI Dynamic Branch Bodies & Selective Runtime Evaluation — V0.9

Status: candidate vertical slice.

## Source surface

V0.9 accepts the V0.8 dynamic input/control surface and permits checked pure expressions inside dynamic branch bodies.

```noi
input key_code;

entry main returns if key_code > 40 {
    key_code + 100
} else {
    key_code + 200
};
```

With canonical key-code `41`, the result is `INT(141)`. With `39`, the result is `INT(239)`.

## Required semantic properties

- both arms are validated and must have the same result kind;
- only the selected dynamic arm is evaluated at runtime;
- the unselected arm has zero observable runtime expression work;
- selected checked arithmetic fails closed on overflow;
- an invalid selected runtime computation cannot silently wrap or coerce;
- static conditions erase before NAIR;
- dynamic branches with two static values preserve V0.8 `BRANCH_VALUE` / NAIR 0.10;
- selective dynamic branch bodies use `BRANCH_EVAL` / NAIR 0.11;
- nested dynamic `if/else` inside a branch body is deferred beyond V0.9 and fails closed;
- no runtime calls, recursion, general jump graph, host authority, or hidden I/O is introduced.

## Certification evidence

The V0.9 certification candidate must prove:

- complete local Release Gate PASS;
- 8 NAIR selective-branch tests PASS;
- 8 source/lowering/runtime tests PASS;
- 4 CLI tests PASS;
- true-path smoke returns `INT(141)`;
- false-path smoke returns `INT(239)`;
- `runtime-branches=1`;
- `runtime-calls=0`;
- `selected-branch-instructions=3` for the canonical smoke;
- `discarded-branch-instructions=0`;
- `nair-minor=0.11`;
- certified public version string remains unchanged;
- exact-SHA five-job CI PASS before tag creation.
