/// Derive a URL-safe slug from a human-readable name.
///
/// Converts to lowercase, replaces non-alphanumeric characters with hyphens,
/// collapses consecutive hyphens, and trims leading/trailing hyphens.
#[must_use]
pub fn slug_from_name(name: &str) -> String {
    let mut slug = String::with_capacity(name.len());
    let mut last_was_hyphen = true; // prevents leading hyphen

    for ch in name.chars() {
        if ch.is_ascii_alphanumeric() {
            slug.push(ch.to_ascii_lowercase());
            last_was_hyphen = false;
        } else if !last_was_hyphen {
            slug.push('-');
            last_was_hyphen = true;
        }
    }

    // Trim trailing hyphen
    if slug.ends_with('-') {
        slug.pop();
    }

    slug
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simple_name() {
        assert_eq!(slug_from_name("rust"), "rust");
    }

    #[test]
    fn hyphenated_name() {
        assert_eq!(slug_from_name("rust-async"), "rust-async");
    }

    #[test]
    fn spaces_become_hyphens() {
        assert_eq!(slug_from_name("Rust Async"), "rust-async");
    }

    #[test]
    fn special_chars_become_hyphens() {
        assert_eq!(slug_from_name("foo/bar_baz"), "foo-bar-baz");
    }

    #[test]
    fn consecutive_specials_collapse() {
        assert_eq!(slug_from_name("foo---bar"), "foo-bar");
    }

    #[test]
    fn leading_trailing_specials_trimmed() {
        assert_eq!(slug_from_name("--foo--"), "foo");
    }

    #[test]
    fn url_path_slugifies() {
        assert_eq!(
            slug_from_name("https://github.com/tokio-rs/tokio"),
            "https-github-com-tokio-rs-tokio"
        );
    }

    #[test]
    fn empty_string() {
        assert_eq!(slug_from_name(""), "");
    }
}
