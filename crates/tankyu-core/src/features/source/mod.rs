pub mod source_manager;
pub use source_manager::SourceManager;

pub mod url_detect;
pub use url_detect::{detect_source_type, name_from_url};
