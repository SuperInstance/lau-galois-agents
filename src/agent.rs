//! High-level agent API - applying Galois theory to agent capability spaces.
//!
//! This module provides the main entry points for analyzing agent capabilities
//! using Galois-theoretic tools.

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use crate::field::{Capability, CapabilityField};
use crate::extension::CapabilityExtension;
use crate::galois_group::GaloisGroup;
use crate::fundamental_theorem::FundamentalTheorem;
use crate::normal::NormalityCheck;
use crate::solvable::SolvabilityResult;
use crate::splitting::{SplittingField, CapabilityPolynomial};
use crate::fixed::FixedField;
use crate::constructible::{ConstructibilityReport, Constructibility};

/// An agent with a capability space amenable to Galois analysis.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GaloisAgent {
    pub name: String,
    pub base_capabilities: CapabilityField,
    pub extensions: Vec<CapabilityExtension>,
}

impl GaloisAgent {
    /// Create a new agent with base capabilities.
    pub fn new(name: &str, base_capabilities: Vec<(&str, u32)>) -> Self {
        let mut field = CapabilityField::new(&format!("{}_base", name));
        for (cap_name, power) in &base_capabilities {
            field.add_capability(Capability::new(cap_name, *power));
        }
        Self {
            name: name.to_string(),
            base_capabilities: field,
            extensions: Vec::new(),
        }
    }

    /// Add an extension (new capabilities).
    pub fn extend(&mut self, name: &str, new_caps: Vec<(&str, u32)>) {
        let caps: Vec<Capability> = new_caps.iter()
            .map(|(n, p)| Capability::new(n, *p))
            .collect();
        let ext = CapabilityExtension::new(name, self.base_capabilities.clone(), caps);
        self.extensions.push(ext);
    }

    /// Analyze a specific extension.
    pub fn analyze_extension(&self, idx: usize) -> Option<GaloisAnalysis> {
        let ext = self.extensions.get(idx)?;
        Some(GaloisAnalysis::analyze(ext))
    }

    /// Analyze all extensions.
    pub fn analyze_all(&self) -> Vec<GaloisAnalysis> {
        self.extensions.iter().map(|ext| GaloisAnalysis::analyze(ext)).collect()
    }

    /// Check if a capability combination is constructible.
    pub fn is_constructible(&self, capabilities: &[&str]) -> bool {
        let caps: Vec<Capability> = capabilities.iter()
            .map(|&n| Capability::new(n, 1))
            .collect();
        let ext = CapabilityExtension::new("check", self.base_capabilities.clone(), caps);
        let report = ConstructibilityReport::analyze(&ext);
        capabilities.iter().all(|&cap| report.is_constructible(cap))
    }

    /// Find the minimal extension containing all specified capabilities.
    pub fn minimal_extension(&self, capabilities: &[&str]) -> CapabilityExtension {
        let caps: Vec<Capability> = capabilities.iter()
            .map(|&n| Capability::new(n, 1))
            .collect();
        CapabilityExtension::new("minimal", self.base_capabilities.clone(), caps)
    }
}

/// Complete Galois analysis of a capability extension.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GaloisAnalysis {
    pub extension_name: String,
    pub galois_group_order: usize,
    pub galois_group_abelian: bool,
    pub degree: usize,
    pub is_normal: bool,
    pub is_separable: bool,
    pub is_galois: bool,
    pub solvability: SolvabilityResult,
    pub constructibility: ConstructibilityReport,
    pub correspondences_count: usize,
    pub intermediate_fields_count: usize,
}

impl GaloisAnalysis {
    /// Perform a full Galois analysis of an extension.
    pub fn analyze(extension: &CapabilityExtension) -> Self {
        let gal = GaloisGroup::compute(extension);
        let ft = FundamentalTheorem::compute(extension);
        let normality = extension.is_normal();

        GaloisAnalysis {
            extension_name: extension.name.clone(),
            galois_group_order: gal.order(),
            galois_group_abelian: gal.group.is_abelian(),
            degree: extension.adjoined.len(),
            is_normal: normality.is_normal,
            is_separable: extension.is_separable(),
            is_galois: normality.is_normal && extension.is_separable(),
            solvability: extension.solvability(),
            constructibility: ConstructibilityReport::analyze(extension),
            correspondences_count: ft.correspondences.len(),
            intermediate_fields_count: extension.intermediate_fields().len(),
        }
    }

    /// A human-readable summary.
    pub fn summary(&self) -> String {
        format!(
            "Extension '{}': degree={}, Galois group order={} ({}), normal={}, Galois={}, solvable={}",
            self.extension_name,
            self.degree,
            self.galois_group_order,
            if self.galois_group_abelian { "abelian" } else { "non-abelian" },
            self.is_normal,
            self.is_galois,
            self.solvability.is_solvable,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_agent() {
        let agent = GaloisAgent::new("test_agent", vec![
            ("perceive", 1),
            ("act", 1),
        ]);
        assert_eq!(agent.name, "test_agent");
        assert_eq!(agent.base_capabilities.degree(), 2);
    }

    #[test]
    fn test_extend_agent() {
        let mut agent = GaloisAgent::new("agent", vec![("base", 1)]);
        agent.extend("step1", vec![("a", 1)]);
        assert_eq!(agent.extensions.len(), 1);
    }

    #[test]
    fn test_analyze_extension() {
        let mut agent = GaloisAgent::new("agent", vec![("base", 1)]);
        agent.extend("step1", vec![("a", 1), ("b", 1)]);
        let analysis = agent.analyze_extension(0).unwrap();
        assert!(analysis.galois_group_order >= 1);
        assert!(analysis.is_separable);
    }

    #[test]
    fn test_analyze_all() {
        let mut agent = GaloisAgent::new("agent", vec![]);
        agent.extend("ext1", vec![("a", 1)]);
        agent.extend("ext2", vec![("b", 2)]);
        let analyses = agent.analyze_all();
        assert_eq!(analyses.len(), 2);
    }

    #[test]
    fn test_is_constructible() {
        let agent = GaloisAgent::new("agent", vec![("base", 1)]);
        assert!(agent.is_constructible(&["a"]));
    }

    #[test]
    fn test_minimal_extension() {
        let agent = GaloisAgent::new("agent", vec![("base", 1)]);
        let ext = agent.minimal_extension(&["a", "b"]);
        assert_eq!(ext.adjoined.len(), 2);
    }

    #[test]
    fn test_analysis_summary() {
        let mut agent = GaloisAgent::new("agent", vec![]);
        agent.extend("test", vec![("x", 1), ("y", 1)]);
        let analysis = agent.analyze_extension(0).unwrap();
        let summary = analysis.summary();
        assert!(summary.contains("test"));
        assert!(summary.contains("degree=2"));
    }

    #[test]
    fn test_galois_analysis_full() {
        let mut agent = GaloisAgent::new("agent", vec![("base", 0)]);
        agent.extend("capabilities", vec![
            ("plan", 2),
            ("execute", 2),
            ("reason", 3),
        ]);
        let analysis = agent.analyze_extension(0).unwrap();
        assert!(analysis.solvability.is_solvable);
        assert!(analysis.constructibility.constructible_count > 0);
    }

    #[test]
    fn test_correspondences_count() {
        let mut agent = GaloisAgent::new("agent", vec![]);
        agent.extend("test", vec![("a", 1), ("b", 1)]);
        let analysis = agent.analyze_extension(0).unwrap();
        assert!(analysis.correspondences_count >= 1);
    }

    #[test]
    fn test_intermediate_fields() {
        let mut agent = GaloisAgent::new("agent", vec![]);
        agent.extend("test", vec![("a", 1), ("b", 1), ("c", 1)]);
        let analysis = agent.analyze_extension(0).unwrap();
        assert_eq!(analysis.intermediate_fields_count, 7); // 2^3 - 1
    }

    #[test]
    fn test_normal_extension_analysis() {
        let mut agent = GaloisAgent::new("agent", vec![]);
        agent.extend("test", vec![("a", 1), ("b", 1)]);
        let analysis = agent.analyze_extension(0).unwrap();
        assert!(analysis.is_normal);
    }
}
