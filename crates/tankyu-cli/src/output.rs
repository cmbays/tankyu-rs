use std::io::IsTerminal;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputMode {
    Rich,
    Plain,
    Json,
}

impl OutputMode {
    pub fn detect(json: bool) -> Self {
        if json {
            return Self::Json;
        }
        if std::env::var("NO_COLOR").is_ok() {
            return Self::Plain;
        }
        if std::io::stdout().is_terminal() {
            Self::Rich
        } else {
            Self::Plain
        }
    }

    pub fn is_json(self) -> bool {
        self == Self::Json
    }
}
