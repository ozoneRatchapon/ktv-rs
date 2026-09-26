use serde::{Deserialize, Serialize};

/// Song ids the singer starred, and the ones they sang lately (newest first).
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct Picks {
    #[serde(default)]
    pub favourites: Vec<String>,
    #[serde(default)]
    pub recent: Vec<String>,
}

/// Which songs the songbook lists.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Shelf {
    All,
    Favourites,
    /// Newest first.
    Recent,
    Category(&'static str),
}

impl Shelf {
    pub fn label(&self) -> &'static str {
        match self {
            Self::All => "All",
            Self::Favourites => "★ Favourites",
            Self::Recent => "Recently sung",
            Self::Category(name) => name,
        }
    }
}
