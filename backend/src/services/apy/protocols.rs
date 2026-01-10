/// Configuration for a supported lending protocol
#[derive(Debug, Clone)]
pub struct ProtocolConfig {
    /// Name used by DeFiLlama API (e.g., "kamino-lend")
    pub defillama_name: &'static str,
    /// Internal name used in our system (e.g., "kamino")
    pub internal_name: &'static str,
    /// Optional pool filter - only include pools matching this pool_meta value
    pub pool_filter: Option<&'static str>,
}

impl ProtocolConfig {
    /// Check if a pool should be included based on pool_meta
    pub fn matches_pool(&self, pool_meta: Option<&str>) -> bool {
        match self.pool_filter {
            Some(required) => pool_meta == Some(required),
            None => true, // No filter means include all pools
        }
    }
}

/// All supported lending protocols
pub const SUPPORTED_PROTOCOLS: &[ProtocolConfig] = &[
    ProtocolConfig {
        defillama_name: "kamino-lend",
        internal_name: "kamino",
        pool_filter: None,
    },
    ProtocolConfig {
        defillama_name: "save",
        internal_name: "save",
        pool_filter: Some("Main Pool"),
    },
    ProtocolConfig {
        defillama_name: "marginfi-lend",
        internal_name: "marginfi",
        pool_filter: None,
    },
];

/// Find protocol config by DeFiLlama project name
pub fn find_by_defillama_name(name: &str) -> Option<&'static ProtocolConfig> {
    SUPPORTED_PROTOCOLS
        .iter()
        .find(|p| p.defillama_name == name)
}

/// Get list of all DeFiLlama project names we support
pub fn defillama_project_names() -> Vec<&'static str> {
    SUPPORTED_PROTOCOLS
        .iter()
        .map(|p| p.defillama_name)
        .collect()
}
