use rusqlite::Row;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::convert::TryInto;
use std::fmt;
use uuid::Uuid;

/// 記憶のカテゴリ
#[derive(
    Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash, JsonSchema, clap::ValueEnum,
)]
#[serde(rename_all = "kebab-case")]
pub enum MemoryType {
    Tech,        // プロジェクトに依存しない技術
    ProjectTech, // プロジェクト固有の技術
    Domain,      // ドメイン知識
}

impl fmt::Display for MemoryType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MemoryType::Tech => write!(f, "tech"),
            MemoryType::ProjectTech => write!(f, "project-tech"),
            MemoryType::Domain => write!(f, "domain"),
        }
    }
}

impl MemoryType {
    /// 文字列からMemoryTypeを作成
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "tech" => Some(MemoryType::Tech),
            "project-tech" => Some(MemoryType::ProjectTech),
            "domain" => Some(MemoryType::Domain),
            _ => None,
        }
    }
}

/// メモリエントリ
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct Memory {
    pub id: String,
    #[serde(rename = "type")]
    pub memory_type: MemoryType,
    pub topic: String,
    #[serde(default)]
    pub tags: Vec<String>,
    pub content: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub examples: Vec<String>,
    pub reference_count: u32,
    pub confidence: f32,
    pub created_at: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_accessed: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
    #[serde(skip)]
    pub deleted: bool,
}

impl Memory {
    /// 新しいメモリを作成
    pub fn new(memory_type: MemoryType, topic: String, content: String) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            memory_type,
            topic,
            tags: Vec::new(),
            content,
            examples: Vec::new(),
            reference_count: 0,
            confidence: 1.0,
            created_at: chrono::Utc::now().timestamp(),
            last_accessed: None,
            source: None,
            deleted: false,
        }
    }

    /// タグ付きで新しいメモリを作成（テスト専用）
    #[cfg(test)]
    pub fn with_tags(
        memory_type: MemoryType,
        topic: String,
        content: String,
        tags: Vec<String>,
    ) -> Self {
        let mut memory = Self::new(memory_type, topic, content);
        memory.tags = tags;
        memory
    }

    /// SQLite行からMemoryを構築
    pub fn from_row(row: &Row) -> rusqlite::Result<Self> {
        let memory_type_str: String = row.get(1)?;
        let memory_type =
            MemoryType::from_str(&memory_type_str).ok_or_else(|| rusqlite::Error::InvalidQuery)?;

        let tags_str: String = row.get(3)?;
        let tags: Vec<String> = if tags_str.is_empty() {
            Vec::new()
        } else {
            tags_str.split(',').map(|s| s.to_string()).collect()
        };

        let examples_json: String = row.get(5)?;
        let examples: Vec<String> = if examples_json.is_empty() {
            Vec::new()
        } else {
            serde_json::from_str(&examples_json).unwrap_or_default()
        };

        Ok(Memory {
            id: row.get(0)?,
            memory_type,
            topic: row.get(2)?,
            tags,
            content: row.get(4)?,
            examples,
            reference_count: {
                let ref_count_i64: i64 = row.get(6)?;
                if ref_count_i64 < 0 {
                    0
                } else {
                    ref_count_i64.try_into().unwrap_or(u32::MAX)
                }
            },
            confidence: row.get(7)?,
            created_at: row.get(8)?,
            last_accessed: row.get(9)?,
            source: row.get(10)?,
            deleted: row.get::<_, i32>(11)? != 0,
        })
    }
}

/// MCP remember ツールのパラメータ
#[derive(Debug, Deserialize)]
pub struct RememberParams {
    #[serde(rename = "type")]
    pub memory_type: MemoryType,
    pub topic: String,
    pub content: String,
    #[serde(default)]
    pub tags: Option<Vec<String>>,
    #[serde(default)]
    pub examples: Option<Vec<String>>,
    #[serde(default)]
    pub source: Option<String>,
}

/// MCP remember ツールのレスポンス
#[derive(Debug, Serialize, Deserialize)]
pub struct RememberResponse {
    pub memory_id: String,
    pub action: String, // "created" or "updated"
    #[serde(skip_serializing_if = "Option::is_none")]
    pub similar_count: Option<usize>,
}

/// MCP recall ツールのパラメータ
#[derive(Debug, Deserialize)]
pub struct RecallParams {
    pub query: String,
    #[serde(rename = "type", default)]
    pub memory_type: Option<MemoryType>,
    #[serde(default)]
    pub tags: Option<Vec<String>>,
    #[serde(default)]
    pub limit: Option<usize>,
}

/// MCP recall ツールのレスポンス
#[derive(Debug, Serialize, Deserialize)]
pub struct RecallResponse {
    pub memories: Vec<Memory>,
    pub total_count: usize,
}

// =============================================================================
// rmcp 0.5.0 Compatible Types
// =============================================================================

/// rmcp remember ツールのパラメータ
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct RmcpRememberParams {
    pub r#type: String,
    pub topic: String,
    pub content: String,
    pub tags: Option<Vec<String>>,
    pub examples: Option<Vec<String>>,
    pub source: Option<String>,
}

/// rmcp remember ツールのレスポンス
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct RmcpRememberResponse {
    pub memory_id: String,
    pub action: String,
    pub similar_count: Option<usize>,
}

/// rmcp recall ツールのパラメータ
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct RmcpRecallParams {
    pub query: String,
    pub r#type: Option<String>,
    pub tags: Option<Vec<String>>,
    pub limit: Option<u32>,
}

/// rmcp recall ツールのレスポンス
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct RmcpRecallResponse {
    pub memories: Vec<Memory>,
    pub total_count: usize,
}

/// rmcp delete ツールのパラメータ
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct RmcpDeleteParams {
    pub memory_id: String,
}

/// rmcp delete ツールのレスポンス
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct RmcpDeleteResponse {
    pub deleted: bool,
    pub memory_id: String,
}

// Conversion implementations
impl From<RmcpRememberParams> for RememberParams {
    fn from(params: RmcpRememberParams) -> Self {
        Self {
            memory_type: MemoryType::from_str(&params.r#type).unwrap_or(MemoryType::Tech),
            topic: params.topic,
            content: params.content,
            tags: params.tags,
            examples: params.examples,
            source: params.source,
        }
    }
}

impl From<RememberResponse> for RmcpRememberResponse {
    fn from(response: RememberResponse) -> Self {
        Self {
            memory_id: response.memory_id,
            action: response.action,
            similar_count: response.similar_count,
        }
    }
}

impl From<RmcpRecallParams> for RecallParams {
    fn from(params: RmcpRecallParams) -> Self {
        Self {
            query: params.query,
            memory_type: params.r#type.and_then(|t| MemoryType::from_str(&t)),
            tags: params.tags,
            limit: params.limit.map(|l| l as usize),
        }
    }
}

impl From<RecallResponse> for RmcpRecallResponse {
    fn from(response: RecallResponse) -> Self {
        Self {
            memories: response.memories,
            total_count: response.total_count,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memory_type_conversion() {
        assert_eq!(MemoryType::from_str("tech"), Some(MemoryType::Tech));
        assert_eq!(
            MemoryType::from_str("project-tech"),
            Some(MemoryType::ProjectTech)
        );
        assert_eq!(MemoryType::from_str("domain"), Some(MemoryType::Domain));
        assert_eq!(MemoryType::from_str("invalid"), None);

        assert_eq!(MemoryType::Tech.to_string(), "tech");
        assert_eq!(MemoryType::ProjectTech.to_string(), "project-tech");
        assert_eq!(MemoryType::Domain.to_string(), "domain");
    }

    #[test]
    fn test_memory_creation() {
        let memory = Memory::new(
            MemoryType::Tech,
            "Test Topic".to_string(),
            "Test Content".to_string(),
        );

        assert!(!memory.id.is_empty());
        assert_eq!(memory.memory_type, MemoryType::Tech);
        assert_eq!(memory.topic, "Test Topic");
        assert_eq!(memory.content, "Test Content");
        assert_eq!(memory.reference_count, 0);
        assert_eq!(memory.confidence, 1.0);
        assert!(!memory.deleted);
    }

    #[test]
    fn test_memory_with_tags() {
        let memory = Memory::with_tags(
            MemoryType::Tech,
            "Test Topic".to_string(),
            "Test Content".to_string(),
            vec!["rust".to_string(), "async".to_string()],
        );

        assert_eq!(memory.tags.len(), 2);
        assert_eq!(memory.tags[0], "rust");
        assert_eq!(memory.tags[1], "async");
    }
}
