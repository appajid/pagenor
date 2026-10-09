//! Persisted, provider-independent AI review data. Credentials never belong here.
use crate::StoryId;
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, sync::Arc};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AiDocument {
    pub version: u32,
    pub next_id: u64,
    pub messages: Vec<Message>,
    pub snapshots: BTreeMap<u64, Snapshot>,
    pub suggestions: Vec<Suggestion>,
    pub task_status: String,
}
impl Default for AiDocument {
    fn default() -> Self {
        Self { version: 1, next_id: 1, messages: vec![], snapshots: BTreeMap::new(), suggestions: vec![], task_status: String::new() }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Message {
    pub role: String,
    pub text: String,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Snapshot {
    pub story: StoryId,
    pub text: Arc<String>,
    pub start: usize,
    pub end: usize,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Suggestion {
    pub id: u64,
    pub snapshot: u64,
    pub story: StoryId,
    pub start: usize,
    pub end: usize,
    pub original: String,
    pub replacement: String,
    pub reason: String,
    pub status: Status,
    /// Immutable provenance for exported reports (live positions may be rebased).
    pub source_start: usize,
    pub source_end: usize,
    pub context: String,
    pub page: Option<usize>,
    pub page_label: Option<String>,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Status {
    Pending,
    Accepted,
    Rejected,
    Stale,
}
impl Status {
    pub fn label(self) -> &'static str {
        match self {
            Self::Pending => "Pending",
            Self::Accepted => "Accepted",
            Self::Rejected => "Rejected",
            Self::Stale => "Stale",
        }
    }
}

// Shared bounds also apply to imported/persisted review data.
pub const MAX_STORY_BYTES: usize = 4 << 20;
pub const MAX_RECORD_BYTES: usize = 16 << 20;
pub const MAX_CHAT_BYTES: usize = 4 << 20;
pub const MAX_SNAPSHOTS: usize = 256;
pub const MAX_SUGGESTIONS: usize = 2048;
impl AiDocument {
    pub fn alloc(&mut self) -> crate::Result<u64> {
        let id = self.next_id;
        if id == 0 || id >= crate::MAX_NEXT_ID {
            return Err(crate::DocError::Invalid("AI record IDs exhausted".into()));
        }
        self.next_id = id.checked_add(1).ok_or_else(|| crate::DocError::Invalid("AI record IDs exhausted".into()))?;
        Ok(id)
    }
    pub fn byte_size(&self) -> usize {
        let snapshots = self.snapshots.values().fold(0usize, |n, s| n.saturating_add(s.text.len()).saturating_add(64));
        let messages = self.messages.iter().fold(0usize, |n, m| n.saturating_add(m.text.len()).saturating_add(m.role.len()).saturating_add(32));
        self.suggestions.iter().fold(snapshots.saturating_add(messages).saturating_add(self.task_status.len()), |n, s| {
            n.saturating_add(s.original.len())
                .saturating_add(s.replacement.len())
                .saturating_add(s.reason.len())
                .saturating_add(s.context.len())
                .saturating_add(s.page_label.as_ref().map_or(0, String::len))
                .saturating_add(128)
        })
    }
    pub fn check(&self) -> crate::Result<()> {
        let invalid = |why: &str| crate::DocError::Invalid(format!("invalid AI records: {why}"));
        if self.version != 1 || self.next_id == 0 || self.next_id > crate::MAX_NEXT_ID {
            return Err(invalid("version or ID counter"));
        }
        if self.snapshots.len() > MAX_SNAPSHOTS
            || self.suggestions.len() > MAX_SUGGESTIONS
            || self.messages.len() > 256
            || self.task_status.len() > 4096
            || self.byte_size() > MAX_RECORD_BYTES
        {
            return Err(invalid("record size/count limit"));
        }
        let mut chat_bytes = 0usize;
        for m in &self.messages {
            chat_bytes = chat_bytes.saturating_add(m.text.len());
            if !matches!(m.role.as_str(), "user" | "assistant") || m.text.len() > 1_000_000 || chat_bytes > MAX_CHAT_BYTES {
                return Err(invalid("chat size/role"));
            }
        }
        let mut ids = std::collections::HashSet::new();
        for (id, sp) in &self.snapshots {
            if *id == 0 || *id >= self.next_id || !ids.insert(*id) || sp.text.len() > MAX_STORY_BYTES || sp.text.get(sp.start..sp.end).is_none() {
                return Err(invalid("snapshot ID/range/size"));
            }
        }
        for s in &self.suggestions {
            let sp = self.snapshots.get(&s.snapshot).ok_or_else(|| invalid("missing suggestion snapshot"))?;
            if s.id == 0
                || s.id >= self.next_id
                || !ids.insert(s.id)
                || s.story != sp.story
                || s.start < sp.start
                || s.end > sp.end
                || sp.text.get(s.start..s.end).is_none()
                || s.source_start > s.source_end
                || s.source_end > MAX_STORY_BYTES
                || s.original.len() > MAX_STORY_BYTES
                || s.replacement.len() > 200_000
                || s.reason.len() > 10_000
                || s.context.len() > 64_000
                || s.page_label.as_ref().is_some_and(|l| l.len() > 4096)
            {
                return Err(invalid("suggestion ID/range/size"));
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn exhausted_ids_and_invalid_unicode_records_are_rejected() {
        let mut a = AiDocument { next_id: crate::MAX_NEXT_ID, ..Default::default() };
        assert!(a.alloc().is_err());
        assert_eq!(a.next_id, crate::MAX_NEXT_ID);
        a = AiDocument { next_id: 2, ..Default::default() };
        a.snapshots.insert(1, Snapshot { story: StoryId(1), text: Arc::new("😀".into()), start: 1, end: 4 });
        assert!(a.check().is_err());
        a.snapshots.get_mut(&1).unwrap().start = 0;
        assert!(a.check().is_ok());
        a.messages.push(Message { role: "system".into(), text: "override".into() });
        assert!(a.check().is_err());
    }
}
