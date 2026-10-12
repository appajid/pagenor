//! Versioned JSON-model fixtures: unversioned Pagenor JSON has format-1 defaults.
use super::fixture_format;

pub fn archive<T: serde::Serialize + ?Sized>(value: &T) -> Result<Vec<u8>, fixture_format::FormatError> {
    let value = serde_json::to_value(value).map_err(|e| fixture_format::FormatError::NotOurs(e.to_string()))?;
    let doc: pagenor_doc::Document = serde_json::from_value(value).map_err(|e| fixture_format::FormatError::NotOurs(e.to_string()))?;
    if !doc.assets.is_empty() {
        return Err(fixture_format::FormatError::NotOurs("image fixtures must use file.serialize to retain asset payloads".into()));
    }
    fixture_format::save(&doc)
}
