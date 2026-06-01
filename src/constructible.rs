//! Constructibility analysis - determine which agent capabilities are constructible.
//!
//! Inspired by the classical result: a geometric construction is possible with
//! ruler and compass iff the associated Galois group is a 2-group.
//! For agents: a capability is constructible iff its Galois group is solvable.

use serde::{Deserialize, Serialize};
use crate::field::{Capability, CapabilityField};
use crate::extension::CapabilityExtension;
use crate::galois_group::{GaloisGroup, PermutationGroup};
use crate::solvable::SolvabilityResult;

/// The constructibility level of a capability.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Constructibility {
    /// The capability is trivially available (in the base field).
    Trivial,
    /// The capability can be constructed step-by-step.
    Constructible { steps: usize },
    /// The capability requires solving an unsolvable extension.
    Insolvable,
    /// Unknown constructibility status.
    Unknown,
}

impl std::fmt::Display for Constructibility {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Constructibility::Trivial => write!(f, "Trivial (already available)"),
            Constructibility::Constructible { steps } => write!(f, "Constructible in {} steps", steps),
            Constructibility::Insolvable => write!(f, "INSOLVABLE - fundamentally impossible"),
            Constructibility::Unknown => write!(f, "Unknown"),
        }
    }
}

/// Analysis result for a single capability.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilityAnalysis {
    pub capability_name: String,
    pub constructibility: Constructibility,
    pub galois_group_order: usize,
    pub galois_group_abelian: bool,
    pub dependencies: Vec<String>,
    pub power: u32,
}

/// Full constructibility analysis for an agent.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConstructibilityReport {
    pub agent_name: String,
    pub base_capabilities: Vec<String>,
    pub analyses: Vec<CapabilityAnalysis>,
    pub constructible_count: usize,
    pub insolvable_count: usize,
    pub total_capabilities: usize,
}

impl ConstructibilityReport {
    /// Analyze constructibility for all capabilities in an extension.
    pub fn analyze(extension: &CapabilityExtension) -> Self {
        let mut analyses = Vec::new();
        let mut constructible_count = 0;
        let mut insolvable_count = 0;

        for cap_name in &extension.adjoined {
            let cap = extension.extended.get(cap_name);
            let power = cap.map(|c| c.power).unwrap_or(1);
            let deps: Vec<String> = cap.map(|c| c.dependencies.iter().cloned().collect()).unwrap_or_default();

            // Check if this capability is in the base field (trivial)
            if extension.base.contains(cap_name) {
                analyses.push(CapabilityAnalysis {
                    capability_name: cap_name.clone(),
                    constructibility: Constructibility::Trivial,
                    galois_group_order: 1,
                    galois_group_abelian: true,
                    dependencies: deps,
                    power,
                });
                constructible_count += 1;
                continue;
            }

            // Create a simple extension for this capability
            let simple_ext = CapabilityExtension::new(
                &format!("for_{}", cap_name),
                extension.base.clone(),
                vec![Capability::new(cap_name, power)],
            );

            let gal = GaloisGroup::compute(&simple_ext);
            let group_order = gal.order();
            let is_abelian = gal.group.is_abelian();

            // Determine constructibility
            let solv = simple_ext.solvability();
            let constructibility = if solv.is_solvable {
                let steps = simple_ext.adjoined.len();
                constructible_count += 1;
                Constructibility::Constructible { steps }
            } else {
                insolvable_count += 1;
                Constructibility::Insolvable
            };

            analyses.push(CapabilityAnalysis {
                capability_name: cap_name.clone(),
                constructibility,
                galois_group_order: group_order,
                galois_group_abelian: is_abelian,
                dependencies: deps,
                power,
            });
        }

        ConstructibilityReport {
            agent_name: extension.name.clone(),
            base_capabilities: extension.base.capability_names().into_iter().map(String::from).collect(),
            analyses,
            constructible_count,
            insolvable_count,
            total_capabilities: extension.adjoined.len(),
        }
    }

    /// Check if a specific capability is constructible.
    pub fn is_constructible(&self, cap_name: &str) -> bool {
        self.analyses.iter()
            .find(|a| a.capability_name == cap_name)
            .map(|a| matches!(a.constructibility, Constructibility::Trivial | Constructibility::Constructible { .. }))
            .unwrap_or(false)
    }

    /// Get all insolvable capabilities.
    pub fn insolvable_capabilities(&self) -> Vec<&str> {
        self.analyses.iter()
            .filter(|a| matches!(a.constructibility, Constructibility::Insolvable))
            .map(|a| a.capability_name.as_str())
            .collect()
    }

    /// Summary string.
    pub fn summary(&self) -> String {
        format!(
            "Agent '{}': {}/{} constructible, {} insolvable",
            self.agent_name,
            self.constructible_count,
            self.total_capabilities,
            self.insolvable_count
        )
    }
}

/// Check if a degree-n extension is constructible (ruler-and-compass analogue).
/// A number is constructible iff it lies in an extension whose Galois group is a 2-group.
pub fn is_constructible_degree(degree: u32) -> bool {
    // Must be a power of 2 for ruler-and-compass constructibility
    degree > 0 && (degree & (degree - 1)) == 0
}

/// Classical result: which regular polygons are constructible?
/// A regular n-gon is constructible iff n = 2^k * p1 * p2 * ... where pi are distinct Fermat primes.
pub fn is_constructible_polygon(n: u32) -> bool {
    if n < 3 { return false; }
    let fermat_primes = [3, 5, 17, 257, 65537];
    let mut m = n;
    // Remove all factors of 2
    while m % 2 == 0 { m /= 2; }
    // Check remaining factors are distinct Fermat primes
    for &fp in &fermat_primes {
        if m % fp == 0 {
            m /= fp;
            if m % fp == 0 { return false; } // not distinct
        }
    }
    m == 1
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_agent_extension() -> CapabilityExtension {
        let mut base = CapabilityField::new("agent_base");
        base.add_capability(Capability::new("perceive", 1));
        CapabilityExtension::new("agent", base, vec![
            Capability::new("plan", 2),
            Capability::new("act", 1),
            Capability::new("reason", 3),
        ])
    }

    #[test]
    fn test_analyze_agent() {
        let ext = make_agent_extension();
        let report = ConstructibilityReport::analyze(&ext);
        assert_eq!(report.total_capabilities, 3);
        assert_eq!(report.constructible_count, 3); // all solvable
    }

    #[test]
    fn test_constructibility_display() {
        let c = Constructibility::Trivial;
        assert!(c.to_string().contains("Trivial"));
        let c = Constructibility::Constructible { steps: 3 };
        assert!(c.to_string().contains("3 steps"));
        let c = Constructibility::Insolvable;
        assert!(c.to_string().contains("INSOLVABLE"));
    }

    #[test]
    fn test_is_constructible() {
        let ext = make_agent_extension();
        let report = ConstructibilityReport::analyze(&ext);
        assert!(report.is_constructible("plan"));
        assert!(report.is_constructible("act"));
    }

    #[test]
    fn test_insolvable_capabilities() {
        let ext = make_agent_extension();
        let report = ConstructibilityReport::analyze(&ext);
        assert!(report.insolvable_capabilities().is_empty()); // all constructible
    }

    #[test]
    fn test_summary() {
        let ext = make_agent_extension();
        let report = ConstructibilityReport::analyze(&ext);
        let summary = report.summary();
        assert!(summary.contains("agent"));
        assert!(summary.contains("constructible"));
    }

    #[test]
    fn test_is_constructible_degree() {
        assert!(is_constructible_degree(1));
        assert!(is_constructible_degree(2));
        assert!(is_constructible_degree(4));
        assert!(!is_constructible_degree(3));
        assert!(!is_constructible_degree(5));
        assert!(is_constructible_degree(8));
    }

    #[test]
    fn test_constructible_polygon() {
        assert!(is_constructible_polygon(3));  // triangle
        assert!(is_constructible_polygon(4));  // square
        assert!(is_constructible_polygon(5));  // pentagon (Fermat prime)
        assert!(is_constructible_polygon(6));  // hexagon
        assert!(is_constructible_polygon(17)); // heptadecagon (Fermat prime)
        assert!(!is_constructible_polygon(7)); // heptagon
        assert!(!is_constructible_polygon(9)); // nonagon (3^2, not distinct)
        assert!(is_constructible_polygon(15)); // 3*5
    }

    #[test]
    fn test_trivial_capability() {
        let mut base = CapabilityField::new("base");
        base.add_capability(Capability::new("base_cap", 0));
        let ext = CapabilityExtension::new("agent", base.clone(), vec![
            Capability::new("base_cap", 0), // already in base
        ]);
        let report = ConstructibilityReport::analyze(&ext);
        let base_analysis = report.analyses.iter().find(|a| a.capability_name == "base_cap");
        assert!(base_analysis.is_some());
        assert_eq!(base_analysis.unwrap().constructibility, Constructibility::Trivial);
    }

    #[test]
    fn test_galois_group_in_analysis() {
        let ext = make_agent_extension();
        let report = ConstructibilityReport::analyze(&ext);
        for analysis in &report.analyses {
            assert!(analysis.galois_group_order >= 1);
        }
    }
}
