//! Partially ordered sets (posets) - the foundation for Galois connections.

use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

/// An element in a poset, identified by a string label.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct PosetElement(pub String);

impl PosetElement {
    pub fn new(label: &str) -> Self {
        Self(label.to_string())
    }
}

impl std::fmt::Display for PosetElement {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// A finite partially ordered set.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Poset {
    /// Elements of the poset.
    elements: BTreeSet<PosetElement>,
    /// Cover relations: maps each element to the set of elements strictly greater.
    /// The full order is the transitive closure of cover relations.
    covers: BTreeMap<PosetElement, BTreeSet<PosetElement>>,
}

impl Poset {
    /// Create a new empty poset.
    pub fn new() -> Self {
        Self {
            elements: BTreeSet::new(),
            covers: BTreeMap::new(),
        }
    }

    /// Create a poset from elements and explicit less-than pairs.
    pub fn from_relations(elements: BTreeSet<PosetElement>, relations: &[(PosetElement, PosetElement)]) -> Self {
        let mut covers = BTreeMap::new();
        for e in &elements {
            covers.insert(e.clone(), BTreeSet::new());
        }
        for (a, b) in relations {
            if elements.contains(a) && elements.contains(b) {
                covers.entry(a.clone()).or_default().insert(b.clone());
            }
        }
        Self { elements, covers }
    }

    /// Add an element to the poset.
    pub fn add_element(&mut self, elem: PosetElement) {
        self.elements.insert(elem.clone());
        self.covers.entry(elem).or_default();
    }

    /// Add an order relation a < b.
    pub fn add_relation(&mut self, a: &PosetElement, b: &PosetElement) {
        if self.elements.contains(a) && self.elements.contains(b) {
            self.covers.entry(a.clone()).or_default().insert(b.clone());
        }
    }

    /// Get all elements.
    pub fn elements(&self) -> &BTreeSet<PosetElement> {
        &self.elements
    }

    /// Check if a <= b (reflexive).
    pub fn leq(&self, a: &PosetElement, b: &PosetElement) -> bool {
        if a == b {
            return true;
        }
        self.lt(a, b)
    }

    /// Check if a < b (strict, transitive).
    pub fn lt(&self, a: &PosetElement, b: &PosetElement) -> bool {
        let mut visited = BTreeSet::new();
        let mut stack = vec![a.clone()];
        while let Some(current) = stack.pop() {
            if visited.contains(&current) {
                continue;
            }
            visited.insert(current.clone());
            if let Some(greater) = self.covers.get(&current) {
                if greater.contains(b) {
                    return true;
                }
                for g in greater {
                    stack.push(g.clone());
                }
            }
        }
        false
    }

    /// Check if a and b are comparable.
    pub fn comparable(&self, a: &PosetElement, b: &PosetElement) -> bool {
        self.leq(a, b) || self.leq(b, a)
    }

    /// Compute the join (least upper bound) of two elements, if it exists.
    pub fn join(&self, a: &PosetElement, b: &PosetElement) -> Option<PosetElement> {
        let mut upper_bounds: BTreeSet<PosetElement> = self.elements()
            .iter()
            .filter(|e| self.leq(a, e) && self.leq(b, e))
            .cloned()
            .collect();
        if upper_bounds.is_empty() {
            return None;
        }
        // Find the least upper bound
        let least = upper_bounds.iter().cloned().find(|candidate| {
            upper_bounds.iter().all(|ub| self.leq(candidate, ub))
        });
        least
    }

    /// Compute the meet (greatest lower bound) of two elements, if it exists.
    pub fn meet(&self, a: &PosetElement, b: &PosetElement) -> Option<PosetElement> {
        let mut lower_bounds: BTreeSet<PosetElement> = self.elements()
            .iter()
            .filter(|e| self.leq(e, a) && self.leq(e, b))
            .cloned()
            .collect();
        if lower_bounds.is_empty() {
            return None;
        }
        let greatest = lower_bounds.iter().cloned().find(|candidate| {
            lower_bounds.iter().all(|lb| self.leq(lb, candidate))
        });
        greatest
    }

    /// Check if this poset is a lattice (all pairs have joins and meets).
    pub fn is_lattice(&self) -> bool {
        let elems: Vec<_> = self.elements.iter().collect();
        for a in &elems {
            for b in &elems {
                if self.join(a, b).is_none() || self.meet(a, b).is_none() {
                    return false;
                }
            }
        }
        true
    }

    /// The bottom element (least), if it exists.
    pub fn bottom(&self) -> Option<&PosetElement> {
        self.elements.iter().find(|e| {
            self.elements.iter().all(|other| self.leq(e, other))
        })
    }

    /// The top element (greatest), if it exists.
    pub fn top(&self) -> Option<&PosetElement> {
        self.elements.iter().find(|e| {
            self.elements.iter().all(|other| self.leq(other, e))
        })
    }

    /// The number of elements.
    pub fn len(&self) -> usize {
        self.elements.len()
    }

    /// Is the poset empty?
    pub fn is_empty(&self) -> bool {
        self.elements.is_empty()
    }

    /// Generate the subset lattice (powerset ordered by inclusion).
    pub fn powerset_lattice(items: &[String]) -> Self {
        let n = items.len();
        let mut elements = BTreeSet::new();
        let mut relations = Vec::new();
        for mask in 0u64..(1u64 << n) {
            let subset: BTreeSet<String> = items.iter()
                .enumerate()
                .filter(|(i, _)| (mask >> i) & 1 == 1)
                .map(|(_, s)| s.clone())
                .collect();
            let label: String = subset.iter().cloned().collect::<Vec<_>>().join(",");
            elements.insert(PosetElement::new(&label));
        }
        // Relations: subset < superset (add one element)
        let all_labels: Vec<_> = elements.iter().cloned().collect();
        for a in &all_labels {
            for b in &all_labels {
                if a == b { continue; }
                let a_set: BTreeSet<&str> = a.0.split(',').filter(|s| !s.is_empty()).collect();
                let b_set: BTreeSet<&str> = b.0.split(',').filter(|s| !s.is_empty()).collect();
                // Check if a is a subset of b and |b\| = |a| + 1
                if a_set.is_subset(&b_set) && b_set.len() == a_set.len() + 1 {
                    relations.push((a.clone(), b.clone()));
                }
            }
        }
        Self::from_relations(elements, &relations)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_poset() {
        let p = Poset::new();
        assert!(p.is_empty());
        assert_eq!(p.len(), 0);
    }

    #[test]
    fn test_single_element() {
        let mut p = Poset::new();
        let e = PosetElement::new("a");
        p.add_element(e.clone());
        assert!(p.leq(&e, &e));
        assert!(!p.lt(&e, &e));
        assert_eq!(p.bottom(), Some(&e));
        assert_eq!(p.top(), Some(&e));
    }

    #[test]
    fn test_chain_poset() {
        let mut elems = BTreeSet::new();
        elems.insert(PosetElement::new("a"));
        elems.insert(PosetElement::new("b"));
        elems.insert(PosetElement::new("c"));
        let p = Poset::from_relations(elems, &[
            (PosetElement::new("a"), PosetElement::new("b")),
            (PosetElement::new("b"), PosetElement::new("c")),
        ]);
        assert!(p.lt(&PosetElement::new("a"), &PosetElement::new("c"))); // transitive
        assert!(!p.lt(&PosetElement::new("c"), &PosetElement::new("a")));
        assert_eq!(p.bottom(), Some(&PosetElement::new("a")));
        assert_eq!(p.top(), Some(&PosetElement::new("c")));
    }

    #[test]
    fn test_antichain() {
        let mut elems = BTreeSet::new();
        elems.insert(PosetElement::new("x"));
        elems.insert(PosetElement::new("y"));
        let p = Poset::from_relations(elems, &[]);
        assert!(!p.comparable(&PosetElement::new("x"), &PosetElement::new("y")));
    }

    #[test]
    fn test_join_meet() {
        let mut elems = BTreeSet::new();
        let a = PosetElement::new("a");
        let b = PosetElement::new("b");
        let top = PosetElement::new("top");
        let bot = PosetElement::new("bot");
        for e in [&a, &b, &top, &bot] { elems.insert(e.clone()); }
        let p = Poset::from_relations(elems, &[
            (bot.clone(), a.clone()),
            (bot.clone(), b.clone()),
            (a.clone(), top.clone()),
            (b.clone(), top.clone()),
        ]);
        assert_eq!(p.join(&a, &b), Some(top));
        assert_eq!(p.meet(&a, &b), Some(bot));
        assert!(p.is_lattice());
    }

    #[test]
    fn test_powerset_lattice() {
        let p = Poset::powerset_lattice(&["x".to_string(), "y".to_string()]);
        assert_eq!(p.len(), 4);
        assert!(p.is_lattice());
    }
}
