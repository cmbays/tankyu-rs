use crate::domain::types::SourceType;

#[must_use]
pub fn detect_source_type(url: &str) -> SourceType {
    // Order matters: more-specific GitHub paths before general ones
    if url.contains("github.com") {
        if url.contains("/issues") {
            return SourceType::GithubIssues;
        }
        if url.contains("/releases") {
            return SourceType::GithubReleases;
        }
        // Two path segments → repo; one → user
        let path = url
            .split_once("github.com/")
            .map_or("", |x| x.1)
            .trim_end_matches('/');
        if path.contains('/') {
            return SourceType::GithubRepo;
        }
        return SourceType::GithubUser;
    }
    if url.contains("x.com/") || url.contains("twitter.com/") {
        return SourceType::XAccount;
    }
    if url.contains("medium.com")
        || url.contains("substack.com")
        || url.contains("dev.to")
        || url.contains(".blog")
        || url.contains("blog.")
    {
        return SourceType::Blog;
    }
    if url.contains("/feed")
        || url.contains("/rss")
        || url.contains("/atom")
        || std::path::Path::new(url)
            .extension()
            .is_some_and(|ext| ext.eq_ignore_ascii_case("xml"))
    {
        return SourceType::RssFeed;
    }
    if url.starts_with("file:") {
        return SourceType::AgentReport;
    }
    SourceType::WebPage
}

#[must_use]
pub fn name_from_url(url: &str) -> String {
    let without_scheme = url.split_once("://").map_or(url, |x| x.1);
    let without_query = without_scheme.split('?').next().unwrap_or(without_scheme);
    let parts: Vec<&str> = without_query.split('/').filter(|s| !s.is_empty()).collect();
    // parts[0] = hostname; parts[1..] = path segments
    parts.get(1).map_or_else(
        || parts.first().copied().unwrap_or(url).to_string(),
        |seg1| {
            parts
                .get(2)
                .map_or_else(|| (*seg1).to_string(), |seg2| format!("{seg1}/{seg2}"))
        },
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::types::SourceType;

    // ── detect_source_type ─────────────────────────────────────────────────

    #[test]
    fn github_issues_url() {
        assert_eq!(
            detect_source_type("https://github.com/rust-lang/rust/issues"),
            SourceType::GithubIssues
        );
    }

    #[test]
    fn github_releases_url() {
        assert_eq!(
            detect_source_type("https://github.com/rust-lang/rust/releases"),
            SourceType::GithubReleases
        );
    }

    #[test]
    fn github_repo_url() {
        assert_eq!(
            detect_source_type("https://github.com/rust-lang/rust"),
            SourceType::GithubRepo
        );
    }

    /// Mutation killer: issues must be checked BEFORE repo to avoid false GithubRepo match
    #[test]
    fn github_issues_not_matched_as_repo() {
        assert_ne!(
            detect_source_type("https://github.com/rust-lang/rust/issues"),
            SourceType::GithubRepo
        );
    }

    #[test]
    fn github_user_url() {
        assert_eq!(
            detect_source_type("https://github.com/torvalds"),
            SourceType::GithubUser
        );
    }

    #[test]
    fn x_account_url() {
        assert_eq!(
            detect_source_type("https://x.com/karpathy"),
            SourceType::XAccount
        );
    }

    #[test]
    fn twitter_account_url() {
        assert_eq!(
            detect_source_type("https://twitter.com/karpathy"),
            SourceType::XAccount
        );
    }

    #[test]
    fn blog_medium_url() {
        assert_eq!(
            detect_source_type("https://medium.com/some-article"),
            SourceType::Blog
        );
    }

    #[test]
    fn blog_substack_url() {
        assert_eq!(
            detect_source_type("https://example.substack.com/post"),
            SourceType::Blog
        );
    }

    #[test]
    fn rss_feed_url() {
        assert_eq!(
            detect_source_type("https://example.com/feed.xml"),
            SourceType::RssFeed
        );
    }

    #[test]
    fn rss_atom_path() {
        assert_eq!(
            detect_source_type("https://example.com/atom"),
            SourceType::RssFeed
        );
    }

    #[test]
    fn agent_report_file_scheme() {
        assert_eq!(
            detect_source_type("file:///reports/spike.md"),
            SourceType::AgentReport
        );
    }

    #[test]
    fn web_page_fallback() {
        assert_eq!(
            detect_source_type("https://example.com/some/page"),
            SourceType::WebPage
        );
    }

    // ── name_from_url ──────────────────────────────────────────────────────

    #[test]
    fn github_repo_name() {
        assert_eq!(
            name_from_url("https://github.com/rust-lang/rust"),
            "rust-lang/rust"
        );
    }

    #[test]
    fn single_path_segment() {
        assert_eq!(name_from_url("https://github.com/torvalds"), "torvalds");
    }

    #[test]
    fn hostname_fallback() {
        assert_eq!(name_from_url("https://example.com"), "example.com");
    }

    #[test]
    fn x_account_name() {
        assert_eq!(name_from_url("https://x.com/karpathy"), "karpathy");
    }
}
