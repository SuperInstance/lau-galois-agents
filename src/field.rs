//! Capability fields - algebraic structures for agent capabilities.
//!
//! A "field" in our context is the set of capabilities an agent possesses,
//! equipped with combination (addition) and composition (multiplication) operations.

use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use crate::poset::{Poset, PosetElement};

/// A capability that an agent can possess.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Capability {
    pub name: String,
    pub power: u32, // "degree" - how powerful this capability is
    pub dependencies: BTreeSet<String>, // capabilities required as prerequisites
}

impl Capability {
    pub fn new(name: &str, power: u32) -> Self {
        Self {
            name: name.to_string(),
            power,
            dependencies: BTreeSet::new(),
        }
    }

    pub fn with_dependency(mut self, dep: &str) -> Self {
        self.dependencies.insert(dep.to_string());
        self
    }
}

/// A capability field - the algebraic structure of an agent's capabilities.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilityField {
    pub name: String,
    /// Base capabilities (like base field elements).
    capabilities: BTreeMap<String, Capability>,
    /// Combination rules: (cap_a, cap_b) -> result
    combinations: BTreeMap<(String, String), String>,
    /// Composition rules: (cap_a, cap_b) -> result
    compositions: BTreeMap<(String, String), String>,
}

impl CapabilityField {
    /// Create a new empty capability field.
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            capabilities: BTreeMap::new(),
            combinations: BTreeMap::new(),
            compositions: BTreeMap::new(),
        }
    }

    /// Add a capability to the field.
    pub fn add_capability(&mut self, cap: Capability) {
        self.capabilities.insert(cap.name.clone(), cap);
    }

    /// Add a combination rule: a + b = result.
    pub fn add_combination(&mut self, a: &str, b: &str, result: &str) {
        self.combinations.insert((a.to_string(), b.to_string()), result.to_string());
    }

    /// Add a composition rule: a * b = result.
    pub fn add_composition(&mut self, a: &str, b: &str, result: &str) {
        self.compositions.insert((a.to_string(), b.to_string()), result.to_string());
    }

    /// Combine two capabilities (analogous to field addition).
    pub fn combine(&self, a: &str, b: &str) -> Option<String> {
        self.combinations.get(&(a.to_string(), b.to_string()))
            .or_else(|| self.combinations.get(&(b.to_string(), a.to_string())))
            .cloned()
    }

    /// Compose two capabilities (analogous to field multiplication).
    pub fn compose(&self, a: &str, b: &str) -> Option<String> {
        self.compositions.get(&(a.to_string(), b.to_string()))
            .or_else(|| self.compositions.get(&(b.to_string(), a.to_string())))
            .cloned()
    }

    /// Check if a capability is in this field.
    pub fn contains(&self, name: &str) -> bool {
        self.capabilities.contains_key(name)
    }

    /// Get a capability by name.
    pub fn get(&self, name: &str) -> Option<&Capability> {
        self.capabilities.get(name)
    }

    /// Get all capability names.
    pub fn capability_names(&self) -> Vec<&str> {
        self.capabilities.keys().map(|s| s.as_str()).collect()
    }

    /// The degree of the field (number of capabilities).
    pub fn degree(&self) -> usize {
        self.capabilities.len()
    }

    /// Check if this field is a subfield of another.
    pub fn is_subfield_of(&self, other: &CapabilityField) -> bool {
        self.capabilities.keys().all(|k| other.contains(k))
    }

    /// Compute the intersection of two fields (greatest common subfield).
    pub fn intersection(&self, other: &CapabilityField) -> CapabilityField {
        let mut result = CapabilityField::new(&format!("{} ∩ {}", self.name, other.name));
        for (name, cap) in &self.capabilities {
            if other.contains(name) {
                result.add_capability(cap.clone());
            }
        }
        result
    }

    /// Compute the compositum (least common extension) of two fields.
    pub fn compositum(&self, other: &CapabilityField) -> CapabilityField {
        let mut result = CapabilityField::new(&format!("{} ∨ {}", self.name, other.name));
        for (name, cap) in &self.capabilities {
            result.add_capability(cap.clone());
        }
        for (name, cap) in &other.capabilities {
            if !result.contains(name) {
                result.add_capability(cap.clone());
            }
        }
        result
    }

    /// Convert to a poset based on dependency ordering.
    pub fn to_poset(&self) -> Poset {
        let mut poset = Poset::new();
        for name in self.capabilities.keys() {
            poset.add_element(PosetElement::new(name));
        }
        for (name, cap) in &self.capabilities {
            for dep in &cap.dependencies {
                if self.capabilities.contains_key(dep) {
                    poset.add_relation(&PosetElement::new(dep), &PosetElement::new(name));
                }
            }
        }
        poset
    }

    /// The trivial field (no capabilities).
    pub fn trivial() -> Self {
        Self::new("trivial")
    }

    /// Check if two fields are equal (same capabilities).
    pub fn equals(&self, other: &CapabilityField) -> bool {
        self.capabilities.keys().collect::<BTreeSet<_>>()
            == other.capabilities.keys().collect::<BTreeSet<_>>()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_field() {
        let mut f = CapabilityField::new("base");
        f.add_capability(Capability::new("perceive", 1));
        f.add_capability(Capability::new("act", 1));
        assert_eq!(f.degree(), 2);
        assert!(f.contains("perceive"));
        assert!(!f.contains("think"));
    }

    #[test]
    fn test_combinations() {
        let mut f = CapabilityField::new("base");
        f.add_capability(Capability::new("read", 1));
        f.add_capability(Capability::new("write", 1));
        f.add_capability(Capability::new("literacy", 2));
        f.add_combination("read", "write", "literacy");
        assert_eq!(f.combine("read", "write"), Some("literacy".to_string()));
        assert_eq!(f.combine("write", "read"), Some("literacy".to_string())); // commutative
    }

    #[test]
    fn test_compositions() {
        let mut f = CapabilityField::new("base");
        f.add_capability(Capability::new("plan", 2));
        f.add_capability(Capability::new("execute", 2));
        f.add_capability(Capability::new("agent", 4));
        f.add_composition("plan", "execute", "agent");
        assert_eq!(f.compose("plan", "execute"), Some("agent".to_string()));
    }

    #[test]
    fn test_subfield() {
        let mut base = CapabilityField::new("base");
        base.add_capability(Capability::new("a", 1));
        base.add_capability(Capability::new("b", 1));
        let mut sub = CapabilityField::new("sub");
        sub.add_capability(Capability::new("a", 1));
        assert!(sub.is_subfield_of(&base));
        assert!(!base.is_subfield_of(&sub));
    }

    #[test]
    fn test_intersection() {
        let mut f1 = CapabilityField::new("f1");
        f1.add_capability(Capability::new("a", 1));
        f1.add_capability(Capability::new("b", 1));
        let mut f2 = CapabilityField::new("f2");
        f2.add_capability(Capability::new("b", 1));
        f2.add_capability(Capability::new("c", 1));
        let inter = f1.intersection(&f2);
        assert!(inter.contains("b"));
        assert!(!inter.contains("a"));
        assert!(!inter.contains("c"));
    }

    #[test]
    fn test_compositum() {
        let mut f1 = CapabilityField::new("f1");
        f1.add_capability(Capability::new("a", 1));
        let mut f2 = CapabilityField::new("f2");
        f2.add_capability(Capability::new("b", 1));
        let comp = f1.compositum(&f2);
        assert!(comp.contains("a"));
        assert!(comp.contains("b"));
        assert_eq!(comp.degree(), 2);
    }

    #[test]
    fn test_to_poset() {
        let mut f = CapabilityField::new("base");
        f.add_capability(Capability::new("base_action", 1));
        f.add_capability(Capability::new("complex_action", 2).with_dependency("base_action"));
        let poset = f.to_poset();
        assert!(poset.lt(&PosetElement::new("base_action"), &PosetElement::new("complex_action")));
    }

    #[test]
    fn test_trivial_field() {
        let f = CapabilityField::trivial();
        assert_eq!(f.degree(), 0);
    }

    #[test]
    fn test_equals() {
        let mut f1 = CapabilityField::new("f1");
        f1.add_capability(Capability::new("a", 1));
        let mut f2 = CapabilityField::new("f2");
        f2.add_capability(Capability::new("a", 1));
        assert!(f1.equals(&f2));
    }
}
