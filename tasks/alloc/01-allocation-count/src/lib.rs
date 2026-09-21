//! Join the names of every even-scored entry, uppercased.

/// An entry with a name and a score.
pub struct Entry {
    /// The entry's name.
    pub name: String,
    /// The entry's score.
    pub score: u32,
}

/// Uppercase the names of every even-scored entry and join them with `", "`.
///
/// The current implementation allocates far more than it needs to. Same output, fewer
/// allocations -- that is the exercise. Do not change what it returns.
#[must_use]
pub fn even_names(entries: &[Entry]) -> String {
    let filtered: Vec<&Entry> = entries.iter().filter(|e| e.score % 2 == 0).collect();
    let names: Vec<String> = filtered.iter().map(|e| e.name.clone()).collect();
    let upper: Vec<String> = names.iter().map(|n| n.to_uppercase()).collect();
    let parts: Vec<&str> = upper.iter().map(String::as_str).collect();
    parts.join(", ")
}
