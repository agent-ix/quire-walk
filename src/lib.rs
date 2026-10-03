//! Shared no_std walker toolkit: an explicit heap stack of typed frames and a children-first arena.

#![warn(missing_docs)]

/// Placeholder entry point.
pub fn hello() -> &'static str {
    "hello from quire_walk"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hello_returns_greeting() {
        assert!(hello().contains("quire_walk"));
    }
}
