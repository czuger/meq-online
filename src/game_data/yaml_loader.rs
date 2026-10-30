//! Reads the Ruby-flavoured YAML files of `data/`.
//!
//! The files were written by Ruby's `YAML.dump`: keys and many values are Ruby symbols
//! (`:name: :Quick Shot`) and some entries carry `!ruby/object` tags. Every document is
//! normalised into plain YAML before being deserialized into typed structs.

use std::path::{Path, PathBuf};

use serde::de::DeserializeOwned;
use serde_norway::{Mapping, Value};

use crate::game_data::code_map::CodeMap;
use crate::game_data::game_data_error::GameDataError;

/// Loads `<dir>/<name>.yaml` and deserializes the whole normalised document into `T`.
///
/// # Errors
/// Returns [`GameDataError::Io`] when the file can't be read and
/// [`GameDataError::Yaml`] when it doesn't match `T`.
pub fn load<T: DeserializeOwned>(dir: &Path, name: &str) -> Result<T, GameDataError> {
    let (path, value) = read_normalized(dir, name)?;
    serde_norway::from_value(value).map_err(|source| GameDataError::Yaml { path, source })
}

/// Loads a top-level mapping as a [`CodeMap`], keeping the file's entry order.
///
/// # Errors
/// Same as [`load`]; also fails when the document isn't a mapping.
pub fn load_map<K, V>(dir: &Path, name: &str) -> Result<CodeMap<K, V>, GameDataError>
where
    K: DeserializeOwned + Ord + Clone,
    V: DeserializeOwned,
{
    let (path, value) = read_normalized(dir, name)?;
    let Value::Mapping(mapping) = value else {
        return Err(GameDataError::NotAMapping { path });
    };
    mapping_to_code_map(mapping).map_err(|source| GameDataError::Yaml { path, source })
}

/// Converts an already normalised mapping into a [`CodeMap`].
///
/// # Errors
/// Fails when a key or value doesn't match `K` or `V`.
pub fn mapping_to_code_map<K, V>(mapping: Mapping) -> Result<CodeMap<K, V>, serde_norway::Error>
where
    K: DeserializeOwned + Ord + Clone,
    V: DeserializeOwned,
{
    let entries = mapping
        .into_iter()
        .map(|(k, v)| Ok((serde_norway::from_value(k)?, serde_norway::from_value(v)?)))
        .collect::<Result<Vec<_>, serde_norway::Error>>()?;
    Ok(CodeMap::new(entries))
}

fn read_normalized(dir: &Path, name: &str) -> Result<(PathBuf, Value), GameDataError> {
    let path = dir.join(format!("{name}.yaml"));
    let text = std::fs::read_to_string(&path).map_err(|source| GameDataError::Io {
        path: path.clone(),
        source,
    })?;
    tracing::debug!(path = %path.display(), bytes = text.len(), "reading game data");
    let value: Value = serde_norway::from_str(&text).map_err(|source| GameDataError::Yaml {
        path: path.clone(),
        source,
    })?;
    Ok((path, normalize(value)))
}

/// Strips Ruby symbol colons from strings and drops `!ruby/...` tagged values.
fn normalize(value: Value) -> Value {
    match value {
        Value::String(s) => Value::String(strip_symbol(s)),
        Value::Sequence(seq) => Value::Sequence(seq.into_iter().map(normalize).collect()),
        Value::Mapping(mapping) => Value::Mapping(
            mapping
                .into_iter()
                .map(|(k, v)| (normalize(k), normalize(v)))
                .collect(),
        ),
        // Only Ruby timestamps are tagged; nothing in the game reads them.
        Value::Tagged(_) => Value::Null,
        other => other,
    }
}

fn strip_symbol(s: String) -> String {
    match s.strip_prefix(':') {
        Some(stripped) => stripped.to_owned(),
        None => s,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strips_symbols_in_keys_and_values() {
        let value: Value = serde_norway::from_str(":name: :Quick Shot\n:n: 3\n").unwrap();
        let normalized = normalize(value);
        let expected: Value = serde_norway::from_str("name: Quick Shot\nn: 3\n").unwrap();
        assert_eq!(normalized, expected);
    }

    #[test]
    fn drops_ruby_tags() {
        let value: Value =
            serde_norway::from_str(":t: !ruby/object:DateTime 2019-02-05 17:34:41\n").unwrap();
        let expected: Value = serde_norway::from_str("t: null\n").unwrap();
        assert_eq!(normalize(value), expected);
    }
}
