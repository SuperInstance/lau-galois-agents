# lau-galois-agents

**Galois theory applied to agent capability spaces.**

[![Tests](https://img.shields.io/badge/tests-111-passing-brightgreen)]()
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)]()

---

## What This Does

This crate models an AI agent's **capability space** as an algebraic field. You define what an agent can do, extend it with new capabilities, and then use Galois theory to answer fundamental questions:

- **Which capabilities are constructible?** — Can a capability be built step-by-step, or is it fundamentally unreachable?
- **What are the symmetries?** — Which capabilities can be swapped without changing the agent's behavior?
- **What's the lattice of intermediate states?** — How many ways can an agent exist between two configurations?

The marquee result: just as the insolvability of the quintic proves that some geometric constructions are impossible with ruler and compass, this crate proves that some capability configurations are **fundamentally insolvable** — no sequence of extensions can reach them.

---

## Key Idea

The central metaphor:

| Galois Theory | Agent Capabilities |
|---|---|
| Base field F | Agent's current capabilities |
| Field extension K/F | Adding new capabilities |
| Galois group Gal(K/F) | Symmetries preserving behavior |
| Solvable group | Capabilities that can be built incrementally |
| Insolvable group (S₅) | Capabilities that **cannot** be constructed |
| Fundamental theorem | Subgroups ↔ Intermediate capability sets |

An agent's capabilities form a "field." Extending the field (adding capabilities) creates a "field extension." The symmetries that permute the new capabilities while preserving the base form the "Galois group." The fundamental theorem of Galois theory then establishes an order-reversing bijection between subgroups of the Galois group and intermediate capability sets.

---

## Install

Add to your `Cargo.toml`:

```toml
[dependencies]
lau-galois-agents = { git = "https://github.com/SuperInstance/lau-galois-agents" }
```

Or clone and use locally:

```bash
git clone https://github.com/SuperInstance/lau-galois-agents.git
# In your Cargo.toml:
# lau-galois-agents = { path = "../lau-galois-agents" }
```

**Dependencies:** `serde` (with `derive`), `nalgebra`.

---

## Quick Start

```rust
use lau_galois_agents::{GaloisAgent, GaloisAnalysis};

// Create an agent with base capabilities
let mut agent = GaloisAgent::new("planner", vec![
    ("perceive", 1),
    ("act", 1),
]);

// Extend with new capabilities
agent.extend("reasoning", vec![
    ("plan", 2),
    ("reason", 3),
]);

// Analyze the extension
let analysis = agent.analyze_extension(0).unwrap();
println!("{}", analysis.summary());
// Extension 'reasoning': degree=2, Galois group order=1 (abelian),
// normal=true, Galois=true, solvable=true

// Check constructibility
assert!(agent.is_constructible(&["plan", "reason"]));
```

### Analyzing Symmetries

```rust
use lau_galois_agents::{
    CapabilityField, Capability, CapabilityExtension,
    GaloisGroup, FundamentalTheorem,
};

// Two interchangeable capabilities → non-trivial Galois group
let base = CapabilityField::new("base");
let ext = CapabilityExtension::new("symmetric", base, vec![
    Capability::new("skill_a", 1),
    Capability::new("skill_b", 1),  // same power → symmetric
]);

let gal = GaloisGroup::compute(&ext);
assert_eq!(gal.order(), 2); // S₂: can swap a ↔ b

// The fundamental theorem gives us subgroups ↔ intermediate fields
let ft = FundamentalTheorem::compute(&ext);
for corr in &ft.correspondences {
    println!("Subgroup (order {}) ↔ Field {:?}",
        corr.subgroup.order(),
        corr.intermediate_field.capability_names(),
    );
}
```

### The Insolvability Result

```rust
use lau_galois_agents::constructible::{is_constructible_degree, is_constructible_polygon};

// Classical Galois theory: constructible iff degree is a power of 2
assert!(is_constructible_degree(4));   // ✓
assert!(!is_constructible_degree(5));  // ✗ — the quintic

// Which regular polygons are constructible?
assert!(is_constructible_polygon(17));  // ✓ — Fermat prime
assert!(!is_constructible_polygon(7));  // ✗ — not constructible
```

---

## API Reference

### Core Types

| Type | Module | Description |
|---|---|---|
| `Poset` | `poset` | Finite partially ordered set with joins, meets, lattice check |
| `GaloisConnection` | `galois_connection` | Adjoint pair of monotone maps between posets |
| `Capability` | `field` | A single named capability with power level and dependencies |
| `CapabilityField` | `field` | Algebraic structure of capabilities (combination + composition) |
| `CapabilityExtension` | `extension` | Adding new capabilities to a base field |
| `Permutation` | `galois_group` | Permutation with compose, inverse, order, sign |
| `PermutationGroup` | `galois_group` | Group of permutations (Sₙ, Aₙ, Cₙ, custom) |
| `GaloisGroup` | `galois_group` | Symmetry group of a capability extension |
| `FundamentalTheorem` | `fundamental_theorem` | The Galois correspondence: subgroups ↔ intermediate fields |
| `FixedField` | `fixed` | Capabilities invariant under a subgroup |
| `ConstructibilityReport` | `constructible` | Full constructibility analysis of an extension |
| `GaloisAgent` | `agent` | High-level agent API tying everything together |

### `GaloisAgent` — High-Level API

```rust
let mut agent = GaloisAgent::new("name", vec![("base_cap", 1)]);
agent.extend("ext_name", vec![("new_cap", 2)]);
agent.analyze_extension(0);    // Option<GaloisAnalysis>
agent.analyze_all();           // Vec<GaloisAnalysis>
agent.is_constructible(&["x"]); // bool
agent.minimal_extension(&["x", "y"]); // CapabilityExtension
```

### `GaloisAnalysis` — Complete Analysis

Returned by `analyze_extension()`:

| Field | Type | Meaning |
|---|---|---|
| `galois_group_order` | `usize` | Number of symmetries |
| `galois_group_abelian` | `bool` | Whether the Galois group is commutative |
| `degree` | `usize` | Number of adjoined capabilities |
| `is_normal` | `bool` | Whether conjugates are complete |
| `is_separable` | `bool` | Always `true` (characteristic 0) |
| `is_galois` | `bool` | Normal + separable |
| `solvability` | `SolvabilityResult` | Can this be built step-by-step? |
| `constructibility` | `ConstructibilityReport` | Per-capability constructibility |

### `PermutationGroup` — Group Theory

```rust
PermutationGroup::symmetric(3);   // S₃ (order 6)
PermutationGroup::cyclic(4);      // C₄ (order 4)
PermutationGroup::alternating(4); // A₄ (order 12)
PermutationGroup::trivial(3);     // {e} (order 1)

group.order();                    // |G|
group.is_abelian();               // commutative?
group.is_solvable();              // solvable?
group.subgroups();                // all subgroups
group.center();                   // Z(G)
group.index_of(&sub);            // [G:H]
```

### `CapabilityField` — The Algebra

```rust
let mut f = CapabilityField::new("my_field");
f.add_capability(Capability::new("read", 1));
f.add_combination("read", "write", "literacy");
f.add_composition("plan", "execute", "agent");

f.combine("read", "write");   // Some("literacy")
f.compose("plan", "execute"); // Some("agent")
f.intersection(&other);       // greatest common subfield
f.compositum(&other);         // least common extension
f.to_poset();                 // dependency-ordered poset
```

---

## How It Works

The crate is organized in layers, each building on the last:

### Layer 1: Order Theory (`poset`, `galois_connection`)

Everything starts with **partially ordered sets** (posets). A `Poset` stores elements and cover relations, computing transitive closure for comparisons. It supports joins (least upper bounds), meets (greatest lower bounds), and can test if it forms a lattice.

A `GaloisConnection` pairs two monotone maps (left and right adjoints) between posets. The fundamental property: `f(p) ≤ q ⟺ p ≤ g(q)`. The closure operator `g ∘ f` is always idempotent and extensive, and its fixed points are the "closed elements."

### Layer 2: The Algebra of Capabilities (`field`, `extension`)

A `CapabilityField` models an agent's capability space as an algebraic structure with two operations:
- **Combination** (addition analogue): merging capabilities
- **Composition** (multiplication analogue): layering capabilities

Capabilities have a `power` level (like a degree) and optional `dependencies` on other capabilities. The field supports subfield checks, intersection (greatest common subfield), and compositum (least common extension).

A `CapabilityExtension` represents adjoining new capabilities to a base field. It tracks degree, can decompose into a tower of simple extensions, and enumerates all intermediate fields (2ⁿ − 1 for n adjoined capabilities).

### Layer 3: Symmetry Groups (`galois_group`)

The `Permutation` type supports identity, transposition, cycle construction, plus compose, inverse, order, and sign. `PermutationGroup` generates the full group from generators via closure, and provides named groups (Sₙ, Aₙ, Cₙ).

The `GaloisGroup` of an extension is the group of all permutations of adjoined capabilities that preserve structure (same power, same dependency count). Capabilities with identical power levels are interchangeable — this is where symmetry emerges.

### Layer 4: The Fundamental Theorem (`fundamental_theorem`)

`FundamentalTheorem::compute()` establishes the Galois correspondence for an extension:
- Maps each subgroup of the Galois group to its fixed field
- Maps each intermediate field to its stabilizer subgroup
- Verifies the bijection
- Builds the subgroup lattice

This is the deep result: the structure of subgroups *is* the structure of intermediate capability configurations.

### Layer 5: Properties (`normal`, `solvable`, `splitting`, `fixed`)

- **Normal extensions**: checking if conjugate capabilities are complete, computing normal closures
- **Solvable groups**: derived series, composition series — determines if capabilities are constructible step-by-step
- **Splitting fields**: "capability polynomials" whose roots are the needed capabilities; the splitting field is the minimal extension containing all roots
- **Fixed fields**: given a subgroup, compute exactly which capabilities are invariant

### Layer 6: Constructibility and Agents (`constructible`, `agent`)

`ConstructibilityReport` classifies each capability as:
- **Trivial** — already in the base field
- **Constructible** — can be built via solvable extensions
- **Insolvable** — requires solving an unsolvable extension (the quintic barrier)

`GaloisAgent` ties it all together with a clean API: create agents with base capabilities, extend them, and analyze the resulting Galois-theoretic structure.

---

## The Math

### Galois Theory (Recap)

Classical Galois theory studies field extensions K/F through the lens of symmetry. The **Galois group** Gal(K/F) is the group of all field automorphisms of K that fix F pointwise. The **fundamental theorem** states:

> There is an order-reversing bijection between subgroups of Gal(K/F) and intermediate fields between F and K.

An extension is **Galois** if it's both normal (every irreducible polynomial with a root splits completely) and separable (distinct roots). A group is **solvable** if its derived series terminates at the trivial group. The crowning result:

> An extension K/F is solvable (can be built by successive radical extensions) if and only if Gal(K/F) is solvable. Since S₅ is not solvable, the general quintic has no solution by radicals.

### The Agent-Capability Metaphor

This crate treats an agent's capabilities as elements of a "field":
- The **base field** F is the agent's current capability set
- A **field extension** K/F adds new capabilities
- **Combination** (field addition): merging capabilities to produce new ones
- **Composition** (field multiplication): layering capabilities for compound effects
- The **Galois group** permutes interchangeable capabilities while preserving the base
- **Normal extensions** ensure conjugate capabilities are all present
- **Solvable extensions** correspond to incrementally constructible capabilities

The **insolvability of the quintic** translates to: some capability configurations (those with S₅ symmetry) cannot be reached by any sequence of incremental extensions. This is not a practical limitation — it's a mathematical impossibility.

### Constructibility

Following the classical ruler-and-compass analogy:
- A degree-n extension is constructible iff n is a power of 2
- A regular n-gon is constructible iff n = 2ᵏ · p₁ · p₂ · … where each pᵢ is a distinct Fermat prime
- Fermat primes known: 3, 5, 17, 257, 65537

---

## Test Suite

111 tests across 12 modules:

| Module | Tests | Coverage |
|---|---|---|
| `poset` | 6 | Empty, single, chain, antichain, join/meet, powerset |
| `galois_connection` | 7 | Identity, closure, kernel, closed elements, composition |
| `field` | 9 | Create, combine, compose, subfield, intersection, compositum |
| `extension` | 9 | Simple, degree, trivial, minimal, compose, intermediate, tower |
| `galois_group` | 19 | Permutations, groups (Sₙ, Aₙ, Cₙ), subgroups, center, Galois computation |
| `fundamental_theorem` | 8 | Correspondence, bijection, lattice, normal subgroups |
| `normal` | 7 | Normality check, closure, separability, Galois |
| `solvable` | 10 | Abelian, S₃, S₄, S₅, derived subgroup, composition series |
| `splitting` | 9 | Polynomials, splitting field, verification, degree |
| `fixed` | 7 | Full Galois, trivial, subgroup fixed, Artin index |
| `constructible` | 9 | Analysis, degrees, polygons, insolvable detection |
| `agent` | 11 | Create, extend, analyze, constructibility, summaries |

Run all tests:

```bash
cargo test
```

---

## License

MIT
