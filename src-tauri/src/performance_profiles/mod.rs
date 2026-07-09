use crate::commands::PerformanceProfileId;

#[derive(Debug, Clone, Copy)]
pub struct PerformanceProfile {
    pub id: PerformanceProfileId,
    pub label: &'static str,
    pub recommended_memory_mb: u32,
    pub includes_shaders: bool,
}

pub fn resolve_profile(id: &PerformanceProfileId) -> PerformanceProfile {
    match id {
        PerformanceProfileId::LowEnd => PerformanceProfile {
            id: *id,
            label: "Low-end",
            recommended_memory_mb: 3072,
            includes_shaders: false,
        },
        PerformanceProfileId::Balanced => PerformanceProfile {
            id: *id,
            label: "Balanced",
            recommended_memory_mb: 4096,
            includes_shaders: false,
        },
        PerformanceProfileId::Shaders => PerformanceProfile {
            id: *id,
            label: "Shaders",
            recommended_memory_mb: 6144,
            includes_shaders: true,
        },
    }
}
