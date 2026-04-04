mod proposal;
mod sponsor;

pub use proposal::*;
pub use sponsor::*;

use serde::Deserialize;

/// Deserialize a `Vec<T>` that may be `null` in JSON (common with Sanity CMS).
/// Maps both missing fields and explicit `null` to an empty `Vec`.
pub fn null_to_vec<'de, D, T>(deserializer: D) -> Result<Vec<T>, D::Error>
where
    D: serde::Deserializer<'de>,
    T: Deserialize<'de>,
{
    Option::<Vec<T>>::deserialize(deserializer).map(Option::unwrap_or_default)
}
