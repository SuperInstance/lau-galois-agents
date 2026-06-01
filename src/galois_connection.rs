//! Galois connections between posets - adjoint functors in the category of posets.

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use crate::poset::{Poset, PosetElement};

/// A Galois connection between two posets (P, Q) consists of two monotone maps:
/// f: P -> Q (left adjoint, "lower adjoint") and g: Q -> P (right adjoint, "upper adjoint")
/// such that f(p) <= q iff p <= g(q) for all p in P, q in Q.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GaloisConnection {
    /// Name/label for this connection.
    pub name: String,
    /// The left poset P.
    pub poset_p: Poset,
    /// The right poset Q.
    pub poset_q: Poset,
    /// Left adjoint f: P -> Q. Maps element labels.
    left_adjoint: BTreeMap<String, String>,
    /// Right adjoint g: Q -> P. Maps element labels.
    right_adjoint: BTreeMap<String, String>,
}

impl GaloisConnection {
    /// Create a new Galois connection with explicit adjoint maps.
    pub fn new(
        name: &str,
        poset_p: Poset,
        poset_q: Poset,
        left_adjoint: BTreeMap<String, String>,
        right_adjoint: BTreeMap<String, String>,
    ) -> Self {
        Self {
            name: name.to_string(),
            poset_p,
            poset_q,
            left_adjoint,
            right_adjoint,
        }
    }

    /// Apply the left adjoint f: P -> Q.
    pub fn apply_left(&self, p: &PosetElement) -> Option<PosetElement> {
        self.left_adjoint.get(&p.0).map(|s| PosetElement::new(s))
    }

    /// Apply the right adjoint g: Q -> P.
    pub fn apply_right(&self, q: &PosetElement) -> Option<PosetElement> {
        self.right_adjoint.get(&q.0).map(|s| PosetElement::new(s))
    }

    /// Verify the Galois connection condition: f(p) <= q iff p <= g(q) for all pairs.
    pub fn verify(&self) -> bool {
        for p in self.poset_p.elements() {
            for q in self.poset_q.elements() {
                let fp = match self.apply_left(p) {
                    Some(fp) => fp,
                    None => return false,
                };
                let gq = match self.apply_right(q) {
                    Some(gq) => gq,
                    None => return false,
                };
                let fp_leq_q = self.poset_q.leq(&fp, q);
                let p_leq_gq = self.poset_p.leq(p, &gq);
                if fp_leq_q != p_leq_gq {
                    return false;
                }
            }
        }
        true
    }

    /// Compute the closure operator: c = g ∘ f. Must be monotone, extensive, idempotent.
    pub fn closure(&self, p: &PosetElement) -> Option<PosetElement> {
        self.apply_left(p).and_then(|fp| self.apply_right(&fp))
    }

    /// Compute the kernel operator: k = f ∘ g. Must be monotone, contracting, idempotent.
    pub fn kernel(&self, q: &PosetElement) -> Option<PosetElement> {
        self.apply_right(q).and_then(|gq| self.apply_left(&gq))
    }

    /// Check if the closure operator is idempotent: c(c(p)) = c(p).
    pub fn is_closure_idempotent(&self) -> bool {
        for p in self.poset_p.elements() {
            if let Some(cp) = self.closure(p) {
                if let Some(ccp) = self.closure(&cp) {
                    if cp != ccp {
                        return false;
                    }
                }
            }
        }
        true
    }

    /// Check if the closure operator is extensive: p <= c(p).
    pub fn is_closure_extensive(&self) -> bool {
        for p in self.poset_p.elements() {
            if let Some(cp) = self.closure(p) {
                if !self.poset_p.leq(p, &cp) {
                    return false;
                }
            }
        }
        true
    }

    /// Get the set of closed elements (fixed points of the closure operator).
    pub fn closed_elements(&self) -> Vec<PosetElement> {
        self.poset_p.elements()
            .iter()
            .filter(|p| self.closure(p) == Some((*p).clone()))
            .cloned()
            .collect()
    }

    /// Compose two Galois connections (if compatible).
    pub fn compose(&self, other: &GaloisConnection) -> Result<GaloisConnection, String> {
        // Check compatibility: self's Q must equal other's P
        if self.poset_q.elements() != other.poset_p.elements() {
            return Err("Incompatible posets for composition".to_string());
        }
        // Compose left adjoints: other.left ∘ self.left
        let mut composed_left = BTreeMap::new();
        for p in self.poset_p.elements() {
            if let Some(mid) = self.apply_left(p) {
                if let Some(final_e) = other.apply_left(&mid) {
                    composed_left.insert(p.0.clone(), final_e.0.clone());
                }
            }
        }
        // Compose right adjoints: self.right ∘ other.right
        let mut composed_right = BTreeMap::new();
        for q in other.poset_q.elements() {
            if let Some(mid) = other.apply_right(q) {
                if let Some(final_e) = self.apply_right(&mid) {
                    composed_right.insert(q.0.clone(), final_e.0.clone());
                }
            }
        }
        Ok(GaloisConnection::new(
            &format!("{} ∘ {}", other.name, self.name),
            self.poset_p.clone(),
            other.poset_q.clone(),
            composed_left,
            composed_right,
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeSet;

    fn make_chain_poset(labels: &[&str]) -> Poset {
        let mut elems = BTreeSet::new();
        for l in labels {
            elems.insert(PosetElement::new(l));
        }
        let mut rels = Vec::new();
        for i in 0..labels.len() - 1 {
            rels.push((PosetElement::new(labels[i]), PosetElement::new(labels[i + 1])));
        }
        Poset::from_relations(elems, &rels)
    }

    #[test]
    fn test_identity_galois_connection() {
        let p = make_chain_poset(&["a", "b", "c"]);
        let mut left = BTreeMap::new();
        let mut right = BTreeMap::new();
        for e in p.elements() {
            left.insert(e.0.clone(), e.0.clone());
            right.insert(e.0.clone(), e.0.clone());
        }
        let gc = GaloisConnection::new("identity", p.clone(), p.clone(), left, right);
        assert!(gc.verify());
    }

    #[test]
    fn test_closure_idempotent() {
        let p = make_chain_poset(&["bot", "mid", "top"]);
        let mut left = BTreeMap::new();
        let mut right = BTreeMap::new();
        left.insert("bot".into(), "mid".into());
        left.insert("mid".into(), "mid".into());
        left.insert("top".into(), "top".into());
        right.insert("mid".into(), "mid".into());
        right.insert("top".into(), "top".into());
        let gc = GaloisConnection::new("test", p.clone(), p, left, right);
        assert!(gc.is_closure_idempotent());
    }

    #[test]
    fn test_closed_elements() {
        let p = make_chain_poset(&["bot", "mid", "top"]);
        let mut left = BTreeMap::new();
        let mut right = BTreeMap::new();
        left.insert("bot".into(), "mid".into());
        left.insert("mid".into(), "mid".into());
        left.insert("top".into(), "top".into());
        right.insert("mid".into(), "mid".into());
        right.insert("top".into(), "top".into());
        let gc = GaloisConnection::new("test", p.clone(), p, left, right);
        let closed = gc.closed_elements();
        assert!(closed.contains(&PosetElement::new("mid")));
        assert!(closed.contains(&PosetElement::new("top")));
        assert!(!closed.contains(&PosetElement::new("bot")));
    }

    #[test]
    fn test_closure_extensive() {
        let p = make_chain_poset(&["bot", "mid", "top"]);
        let mut left = BTreeMap::new();
        let mut right = BTreeMap::new();
        left.insert("bot".into(), "mid".into());
        left.insert("mid".into(), "mid".into());
        left.insert("top".into(), "top".into());
        right.insert("mid".into(), "mid".into());
        right.insert("top".into(), "top".into());
        let gc = GaloisConnection::new("test", p.clone(), p, left, right);
        assert!(gc.is_closure_extensive());
    }

    #[test]
    fn test_galois_connection_verification() {
        // Simple case: two identical chain posets with identity maps
        let p = make_chain_poset(&["0", "1"]);
        let mut left = BTreeMap::new();
        let mut right = BTreeMap::new();
        left.insert("0".into(), "0".into());
        left.insert("1".into(), "1".into());
        right.insert("0".into(), "0".into());
        right.insert("1".into(), "1".into());
        let gc = GaloisConnection::new("id", p.clone(), p, left, right);
        assert!(gc.verify());
    }

    #[test]
    fn test_compose_galois_connections() {
        let p = make_chain_poset(&["a", "b", "c"]);
        let mut left = BTreeMap::new();
        let mut right = BTreeMap::new();
        for e in p.elements() {
            left.insert(e.0.clone(), e.0.clone());
            right.insert(e.0.clone(), e.0.clone());
        }
        let gc = GaloisConnection::new("id", p.clone(), p.clone(), left, right);
        let composed = gc.compose(&gc).unwrap();
        assert!(composed.verify());
    }

    #[test]
    fn test_kernel_operator() {
        let p = make_chain_poset(&["bot", "top"]);
        let mut left = BTreeMap::new();
        let mut right = BTreeMap::new();
        left.insert("bot".into(), "bot".into());
        left.insert("top".into(), "top".into());
        right.insert("bot".into(), "bot".into());
        right.insert("top".into(), "top".into());
        let gc = GaloisConnection::new("test", p.clone(), p, left, right);
        // f∘g should be identity
        assert_eq!(gc.kernel(&PosetElement::new("bot")), Some(PosetElement::new("bot")));
        assert_eq!(gc.kernel(&PosetElement::new("top")), Some(PosetElement::new("top")));
    }
}
