# Logic Prover

A lightweight DPLL SAT solver exposed as a WASM plugin for R.A.I.N..

## Overview

Provides boolean satisfiability solving via a DPLL (Davis-Putnam-Logemann-Loveland) algorithm, compiled to WASM so R.A.I.N. agents can call it as a tool without native compilation requirements.

## Usage

```rust
use logic_prover::{solve, DpllResult};

// CNF: (a ∨ ¬b) ∧ (b ∨ c) ∧ (¬a)
let cnf = vec![
    vec![Lit::Var("a".into()), Lit::Neg("b".into())],
    vec![Lit::Var("b".into()), Lit::Var("c".into())],
    vec![Lit::Neg("a".into())],
];

match solve(cnf) {
    DpllResult::Sat(assignment) => { /* ... */ }
    DpllResult::Unsat => { /* formula is contradictory */ }
}
```

## Build

```bash
cargo build -p logic_prover --release
```

The WASM artifact is at `target/wasm32-wasip1/release/logic_prover.wasm`.

## Registration

Add a thin adapter in `src/tools/` mapping WASM plugin calls to the `Tool` trait, then register in `src/tools/mod.rs` factory under the `"logic_prover"` key.
