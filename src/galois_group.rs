//! Galois groups - symmetry groups of capability extensions.
//!
//! The Galois group of an extension K/F is the group of all symmetries (automorphisms)
//! of K that fix F pointwise. In our context: permutations of capabilities that
//! preserve the base capabilities and the structure of the extension.

use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use crate::field::CapabilityField;
use crate::extension::CapabilityExtension;

/// A permutation represented as a map from element index to element index.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Permutation {
    /// Maps position i -> permuted position.
    pub mapping: Vec<usize>,
}

impl Permutation {
    /// Identity permutation of given size.
    pub fn identity(n: usize) -> Self {
        Self { mapping: (0..n).collect() }
    }

    /// Create from a slice.
    pub fn from_slice(mapping: &[usize]) -> Self {
        Self { mapping: mapping.to_vec() }
    }

    /// A transposition (swap i and j).
    pub fn transposition(n: usize, i: usize, j: usize) -> Self {
        let mut mapping: Vec<usize> = (0..n).collect();
        mapping.swap(i, j);
        Self { mapping }
    }

    /// A cycle from the given elements.
    pub fn cycle(n: usize, elements: &[usize]) -> Self {
        let mut mapping: Vec<usize> = (0..n).collect();
        if elements.is_empty() {
            return Self { mapping };
        }
        for i in 0..elements.len() {
            mapping[elements[i]] = elements[(i + 1) % elements.len()];
        }
        Self { mapping }
    }

    /// Apply this permutation.
    pub fn apply(&self, i: usize) -> usize {
        self.mapping[i]
    }

    /// Compose two permutations: (self ∘ other)(i) = self(other(i)).
    pub fn compose(&self, other: &Permutation) -> Permutation {
        let mapping = other.mapping.iter().map(|&i| self.mapping[i]).collect();
        Permutation { mapping }
    }

    /// The inverse permutation.
    pub fn inverse(&self) -> Permutation {
        let mut mapping = vec![0; self.mapping.len()];
        for (i, &j) in self.mapping.iter().enumerate() {
            mapping[j] = i;
        }
        Permutation { mapping }
    }

    /// The order of this permutation (smallest positive n where σ^n = id).
    pub fn order(&self) -> usize {
        let id = Permutation::identity(self.mapping.len());
        let mut current = self.clone();
        for n in 1..=self.mapping.len().pow(2) + 1 {
            if current == id {
                return n;
            }
            current = current.compose(self);
        }
        self.mapping.len() // fallback
    }

    /// Check if this is the identity permutation.
    pub fn is_identity(&self) -> bool {
        self.mapping.iter().enumerate().all(|(i, &j)| i == j)
    }

    /// The sign of the permutation (+1 or -1).
    pub fn sign(&self) -> i32 {
        let mut visited = vec![false; self.mapping.len()];
        let mut sign = 1i32;
        for start in 0..self.mapping.len() {
            if visited[start] { continue; }
            let mut cycle_len = 0;
            let mut current = start;
            while !visited[current] {
                visited[current] = true;
                current = self.mapping[current];
                cycle_len += 1;
            }
            if cycle_len % 2 == 0 {
                sign *= -1;
            }
        }
        sign
    }
}

/// A group of permutations.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermutationGroup {
    pub name: String,
    pub degree: usize, // permutations act on {0, ..., degree-1}
    pub elements: Vec<Permutation>,
}

impl PermutationGroup {
    /// Create a new permutation group, verifying closure.
    pub fn new(name: &str, degree: usize, generators: Vec<Permutation>) -> Self {
        let elements = Self::generate_group(degree, &generators);
        Self { name: name.to_string(), degree, elements }
    }

    /// Generate the full group from generators using closure.
    fn generate_group(degree: usize, generators: &[Permutation]) -> Vec<Permutation> {
        let mut group = vec![Permutation::identity(degree)];
        let mut changed = true;
        while changed {
            changed = false;
            let current = group.clone();
            for g in &current {
                for gen in generators {
                    let prod = g.compose(gen);
                    if !group.contains(&prod) {
                        group.push(prod);
                        changed = true;
                    }
                    let prod2 = gen.compose(g);
                    if !group.contains(&prod2) {
                        group.push(prod2);
                        changed = true;
                    }
                }
            }
        }
        group
    }

    /// The symmetric group S_n.
    pub fn symmetric(n: usize) -> Self {
        if n == 0 {
            return Self { name: "S_0".to_string(), degree: 0, elements: vec![] };
        }
        let transpositions: Vec<Permutation> = (0..n-1)
            .map(|i| Permutation::transposition(n, i, i + 1))
            .collect();
        Self::new(&format!("S_{}", n), n, transpositions)
    }

    /// The cyclic group C_n.
    pub fn cyclic(n: usize) -> Self {
        if n == 0 {
            return Self { name: "C_0".to_string(), degree: 0, elements: vec![] };
        }
        let cycle = Permutation::cycle(n, &(0..n).collect::<Vec<_>>());
        Self::new(&format!("C_{}", n), n, vec![cycle])
    }

    /// The alternating group A_n (even permutations).
    pub fn alternating(n: usize) -> Self {
        let sn = Self::symmetric(n);
        let even: Vec<Permutation> = sn.elements.into_iter().filter(|p| p.sign() == 1).collect();
        Self { name: format!("A_{}", n), degree: n, elements: even }
    }

    /// The trivial group {e}.
    pub fn trivial(degree: usize) -> Self {
        Self {
            name: "trivial".to_string(),
            degree,
            elements: vec![Permutation::identity(degree)],
        }
    }

    /// Order of the group.
    pub fn order(&self) -> usize {
        self.elements.len()
    }

    /// Check if a permutation is in the group.
    pub fn contains_perm(&self, p: &Permutation) -> bool {
        self.elements.contains(p)
    }

    /// Check if this is a subgroup of another group.
    pub fn is_subgroup_of(&self, other: &PermutationGroup) -> bool {
        self.elements.iter().all(|p| other.contains_perm(p))
    }

    /// Find all subgroups (by checking cyclic subgroups generated by each element).
    pub fn subgroups(&self) -> Vec<PermutationGroup> {
        let mut subgroups = Vec::new();
        // Generate subgroups from each element
        for gen in &self.elements {
            let sg = Self::new(&format!("subgroup"), self.degree, vec![gen.clone()]);
            if !subgroups.iter().any(|existing: &PermutationGroup| {
                existing.elements.len() == sg.elements.len() &&
                    sg.elements.iter().all(|p| existing.contains_perm(p))
            }) {
                subgroups.push(sg);
            }
        }
        // Also generate subgroups from pairs of elements
        for i in 0..self.elements.len() {
            for j in i+1..self.elements.len() {
                let sg = Self::new("subgroup", self.degree, vec![
                    self.elements[i].clone(),
                    self.elements[j].clone(),
                ]);
                if !subgroups.iter().any(|existing| {
                    existing.elements.len() == sg.elements.len() &&
                        sg.elements.iter().all(|p| existing.contains_perm(p))
                }) {
                    subgroups.push(sg);
                }
            }
        }
        subgroups
    }

    /// The center of the group.
    pub fn center(&self) -> PermutationGroup {
        let center_elems: Vec<Permutation> = self.elements.iter()
            .filter(|g| self.elements.iter().all(|h| {
                g.compose(h) == h.compose(g)
            }))
            .cloned()
            .collect();
        PermutationGroup {
            name: format!("Z({})", self.name),
            degree: self.degree,
            elements: center_elems,
        }
    }

    /// Check if the group is abelian (commutative).
    pub fn is_abelian(&self) -> bool {
        self.elements.iter().all(|g| {
            self.elements.iter().all(|h| {
                g.compose(h) == h.compose(g)
            })
        })
    }

    /// Compute the index [self : subgroup].
    pub fn index_of(&self, subgroup: &PermutationGroup) -> Option<usize> {
        if !subgroup.is_subgroup_of(self) {
            return None;
        }
        if subgroup.order() == 0 {
            return None;
        }
        Some(self.order() / subgroup.order())
    }

    /// Apply a permutation to a list of strings.
    pub fn apply_to_strings(&self, perm: &Permutation, items: &[String]) -> Vec<String> {
        items.iter().enumerate().map(|(i, _)| {
            items[perm.apply(i)].clone()
        }).collect()
    }
}

/// The Galois group of a capability extension.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GaloisGroup {
    /// The extension this group is associated with.
    pub extension_name: String,
    /// The permutation group.
    pub group: PermutationGroup,
    /// Which capabilities are being permuted (the adjoined ones).
    pub permuted_capabilities: Vec<String>,
    /// Which capabilities are fixed (the base ones).
    pub fixed_capabilities: Vec<String>,
}

impl GaloisGroup {
    /// Compute the Galois group of an extension.
    /// For capabilities: find all permutations of adjoined capabilities
    /// that preserve the structure (dependencies, compositions, etc.).
    pub fn compute(extension: &CapabilityExtension) -> Self {
        let n = extension.adjoined.len();
        if n == 0 {
            return Self {
                extension_name: extension.name.clone(),
                group: PermutationGroup::trivial(0),
                permuted_capabilities: vec![],
                fixed_capabilities: extension.base.capability_names().into_iter().map(String::from).collect(),
            };
        }

        // Find all permutations that are valid symmetries
        let sn = PermutationGroup::symmetric(n);
        let valid_perms: Vec<Permutation> = sn.elements.into_iter()
            .filter(|perm| Self::is_valid_symmetry(perm, extension))
            .collect();

        let group = PermutationGroup {
            name: format!("Gal({})", extension.name),
            degree: n,
            elements: valid_perms,
        };

        Self {
            extension_name: extension.name.clone(),
            group,
            permuted_capabilities: extension.adjoined.clone(),
            fixed_capabilities: extension.base.capability_names().into_iter().map(String::from).collect(),
        }
    }

    /// Check if a permutation is a valid symmetry of the extension.
    fn is_valid_symmetry(perm: &Permutation, extension: &CapabilityExtension) -> bool {
        let adjoined = &extension.adjoined;
        // Apply permutation to adjoined capabilities
        for i in 0..adjoined.len() {
            let j = perm.apply(i);
            let cap_i = extension.extended.get(&adjoined[i]);
            let cap_j = extension.extended.get(&adjoined[j]);
            match (cap_i, cap_j) {
                (Some(ci), Some(cj)) => {
                    // Symmetry: swapped capabilities must have same power
                    if ci.power != cj.power {
                        return false;
                    }
                    // And same number of dependencies
                    if ci.dependencies.len() != cj.dependencies.len() {
                        return false;
                    }
                }
                _ => {}
            }
        }
        true
    }

    /// The order of the Galois group.
    pub fn order(&self) -> usize {
        self.group.order()
    }

    /// Check if the extension is Galois (|Gal| = degree of extension).
    pub fn is_galois(&self, extension: &CapabilityExtension) -> bool {
        self.group.order() == extension.adjoined.len()
    }

    /// Find subgroups of the Galois group.
    pub fn subgroups(&self) -> Vec<PermutationGroup> {
        self.group.subgroups()
    }

    /// Get the fixed capabilities under the entire group.
    pub fn fixed_field(&self, extension: &CapabilityExtension) -> CapabilityField {
        let mut fixed = CapabilityField::new(&format!("Fix({})", self.extension_name));
        for name in extension.base.capability_names() {
            if let Some(cap) = extension.base.get(name) {
                fixed.add_capability(cap.clone());
            }
        }
        // Adjoined capabilities that are fixed by all group elements
        for (i, cap_name) in self.permuted_capabilities.iter().enumerate() {
            let is_fixed = self.group.elements.iter().all(|perm| {
                perm.apply(i) == i
            });
            if is_fixed {
                if let Some(cap) = extension.extended.get(cap_name) {
                    fixed.add_capability(cap.clone());
                }
            }
        }
        fixed
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::field::Capability;

    #[test]
    fn test_identity_permutation() {
        let p = Permutation::identity(3);
        assert!(p.is_identity());
        assert_eq!(p.apply(0), 0);
        assert_eq!(p.apply(1), 1);
        assert_eq!(p.apply(2), 2);
    }

    #[test]
    fn test_transposition() {
        let p = Permutation::transposition(3, 0, 2);
        assert_eq!(p.apply(0), 2);
        assert_eq!(p.apply(1), 1);
        assert_eq!(p.apply(2), 0);
    }

    #[test]
    fn test_cycle() {
        let p = Permutation::cycle(3, &[0, 1, 2]);
        assert_eq!(p.apply(0), 1);
        assert_eq!(p.apply(1), 2);
        assert_eq!(p.apply(2), 0);
    }

    #[test]
    fn test_compose_permutations() {
        let p1 = Permutation::transposition(3, 0, 1);
        let p2 = Permutation::transposition(3, 1, 2);
        let composed = p1.compose(&p2);
        assert_eq!(composed.apply(0), 1); // 0->0->1
    }

    #[test]
    fn test_inverse_permutation() {
        let p = Permutation::cycle(3, &[0, 1, 2]);
        let inv = p.inverse();
        let id = p.compose(&inv);
        assert!(id.is_identity());
    }

    #[test]
    fn test_permutation_order() {
        let p = Permutation::cycle(3, &[0, 1, 2]);
        assert_eq!(p.order(), 3);
    }

    #[test]
    fn test_permutation_sign() {
        let id = Permutation::identity(3);
        assert_eq!(id.sign(), 1);
        let trans = Permutation::transposition(3, 0, 1);
        assert_eq!(trans.sign(), -1);
    }

    #[test]
    fn test_symmetric_group() {
        let s3 = PermutationGroup::symmetric(3);
        assert_eq!(s3.order(), 6);
        assert_eq!(s3.degree, 3);
    }

    #[test]
    fn test_cyclic_group() {
        let c3 = PermutationGroup::cyclic(3);
        assert_eq!(c3.order(), 3);
        assert!(c3.is_abelian());
    }

    #[test]
    fn test_alternating_group() {
        let a3 = PermutationGroup::alternating(3);
        assert_eq!(a3.order(), 3); // A_3 = C_3
    }

    #[test]
    fn test_trivial_group() {
        let t = PermutationGroup::trivial(3);
        assert_eq!(t.order(), 1);
    }

    #[test]
    fn test_subgroup() {
        let c3 = PermutationGroup::cyclic(3);
        let s3 = PermutationGroup::symmetric(3);
        assert!(c3.is_subgroup_of(&s3));
    }

    #[test]
    fn test_abelian() {
        let c3 = PermutationGroup::cyclic(3);
        assert!(c3.is_abelian());
        let s3 = PermutationGroup::symmetric(3);
        assert!(!s3.is_abelian());
    }

    #[test]
    fn test_center() {
        let c3 = PermutationGroup::cyclic(3);
        let z = c3.center();
        assert_eq!(z.order(), 3); // C_3 is abelian, center = itself
    }

    #[test]
    fn test_galois_group_computation() {
        let base = CapabilityField::new("base");
        let ext = crate::extension::CapabilityExtension::new("test", base, vec![
            Capability::new("a", 1),
            Capability::new("b", 1),
        ]);
        let gal = GaloisGroup::compute(&ext);
        // a and b have same power, so all permutations are valid
        assert_eq!(gal.order(), 2); // S_2 has order 2
    }

    #[test]
    fn test_galois_group_different_powers() {
        let base = CapabilityField::new("base");
        let ext = crate::extension::CapabilityExtension::new("test", base, vec![
            Capability::new("a", 1),
            Capability::new("b", 2),
        ]);
        let gal = GaloisGroup::compute(&ext);
        // Different powers, only identity symmetry
        assert_eq!(gal.order(), 1);
    }

    #[test]
    fn test_fixed_field() {
        let mut base = CapabilityField::new("base");
        base.add_capability(Capability::new("base_cap", 1));
        let ext = crate::extension::CapabilityExtension::new("test", base, vec![
            Capability::new("a", 1),
            Capability::new("b", 2),
        ]);
        let gal = GaloisGroup::compute(&ext);
        let fixed = gal.fixed_field(&ext);
        assert!(fixed.contains("base_cap"));
    }

    #[test]
    fn test_index() {
        let c3 = PermutationGroup::cyclic(3);
        let trivial = PermutationGroup::trivial(3);
        assert_eq!(c3.index_of(&trivial), Some(3));
    }

    #[test]
    fn test_apply_to_strings() {
        let p = Permutation::transposition(3, 0, 2);
        let g = PermutationGroup::trivial(3);
        let result = g.apply_to_strings(&p, &["a".into(), "b".into(), "c".into()]);
        assert_eq!(result, vec!["c", "b", "a"]);
    }
}
