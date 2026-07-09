use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SetupManifest {
    pub schema_version: u16,
    pub manifest_version: String,
    pub id: String,
    pub display_name: String,
    pub server: ManifestServer,
    pub minecraft: ManifestMinecraft,
    pub install: ManifestInstall,
    #[serde(default)]
    pub profiles: Vec<ManifestPerformanceProfile>,
    #[serde(default)]
    pub resources: Vec<ManifestResource>,
    pub server_entry: Option<ManifestServerEntry>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ManifestServer {
    pub name: String,
    pub address: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ManifestMinecraft {
    pub version: String,
    pub loader: ManifestLoader,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ManifestLoader {
    pub kind: ManifestLoaderKind,
    pub version: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ManifestLoaderKind {
    None,
    Fabric,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ManifestInstall {
    pub game_directory_name: String,
    pub launcher_profile_name: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ManifestPerformanceProfile {
    pub id: String,
    pub label: String,
    pub recommended_memory_mb: u32,
    #[serde(default)]
    pub includes_shaders: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ManifestResource {
    pub id: String,
    pub name: String,
    pub resource_type: ManifestResourceType,
    pub target: ManifestResourceTarget,
    #[serde(default)]
    pub required: bool,
    pub source: ManifestResourceSource,
    #[serde(default)]
    pub hashes: ManifestResourceHashes,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ManifestResourceType {
    Mod,
    ResourcePack,
    ShaderPack,
    Config,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ManifestResourceTarget {
    Mods,
    Resourcepacks,
    Shaderpacks,
    Config,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ManifestResourceSource {
    Modrinth { project: String, version: String },
    Direct { url: String },
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ManifestResourceHashes {
    pub sha512: Option<String>,
    pub sha256: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ManifestServerEntry {
    pub name: String,
    pub address: String,
}
