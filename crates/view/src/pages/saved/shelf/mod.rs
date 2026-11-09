//! Which shelf of saved statuses a reader is on.
//!
//! Favourites and bookmarks are two endpoints with the same shape, so the screen is one page
//! with a tab each. The shelf is an address segment, not a request: the page only needs to
//! know which tab is lit, where the other one leads, and what a shelf says when it holds
//! nothing.

/// One shelf of saved statuses.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Shelf {
    /// Statuses the account starred.
    Favourites,
    /// Statuses the account saved for later.
    Bookmarks,
}

impl Shelf {
    /// Both shelves, in tab order.
    pub const ALL: [Self; 2] = [Self::Favourites, Self::Bookmarks];

    /// The shelf a `/saved/{slug}` address names.
    ///
    /// Anything else — a bare `/saved`, a stale bookmark to a shelf that no longer exists —
    /// is the first shelf, which is where the address with no segment already leads.
    #[must_use]
    pub fn of(slug: &str) -> Self {
        match slug {
            "bookmarks" => Self::Bookmarks,
            _ => Self::Favourites,
        }
    }

    /// The address segment this shelf lives at.
    #[must_use]
    pub fn slug(self) -> &'static str {
        match self {
            Self::Favourites => "favourites",
            Self::Bookmarks => "bookmarks",
        }
    }

    /// The tab label.
    #[must_use]
    pub fn label(self) -> &'static str {
        match self {
            Self::Favourites => "Favourites",
            Self::Bookmarks => "Bookmarks",
        }
    }

    /// What the shelf says when it holds nothing.
    ///
    /// The words name the shelf rather than the screen: an empty favourites tab must not
    /// claim the reader has no bookmarks.
    #[must_use]
    pub fn empty(self) -> &'static str {
        match self {
            Self::Favourites => "Nothing is favourited yet.",
            Self::Bookmarks => "Nothing is bookmarked yet.",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Shelf;

    #[test]
    fn every_slug_leads_back_to_its_shelf() {
        for shelf in Shelf::ALL {
            assert_eq!(Shelf::of(shelf.slug()), shelf);
        }
    }

    #[test]
    fn a_slug_that_names_no_shelf_is_the_first_one() {
        assert_eq!(Shelf::of(""), Shelf::Favourites);
        assert_eq!(Shelf::of("favourites"), Shelf::Favourites);
    }
}
