use super::Result;
use serde_json::Value;

/// Parses a string-encoded JSON blob (as stored in `Items`, `BaseData`, ...).
pub fn parse_inner(s: &str) -> Result<Value> {
    Ok(serde_json::from_str(s)?)
}

/// Serializes a value back to a compact string, matching the game's inner items.
pub fn dump_inner(v: &Value) -> Result<String> {
    Ok(serde_json::to_string(v)?)
}

/// Applies `f` to each item of a `["{...}", ...]` array of escaped-JSON strings.
/// `f` mutates the decoded value and returns true if it changed.
/// Returns the number of items changed.
pub fn edit_string_items<F>(items: &mut Value, mut f: F) -> Result<usize>
where
    F: FnMut(&mut Value) -> bool,
{
    let mut changed = 0;
    if let Some(arr) = items.as_array_mut() {
        for slot in arr.iter_mut() {
            let Some(text) = slot.as_str() else { continue };
            let mut inner = match parse_inner(text) {
                Ok(v) => v,
                Err(_) => continue,
            };
            if f(&mut inner) {
                *slot = Value::String(dump_inner(&inner)?);
                changed += 1;
            }
        }
    }
    Ok(changed)
}
