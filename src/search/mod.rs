//! Song search tuned for Thai: tone marks and spacing ignored, romanised aliases, and queries typed on
//! the wrong keyboard layout (Thai Kedmanee vs US QWERTY) retried on the other one.

mod layout;
mod matcher;
mod normalize;

pub use layout::retype;
pub use matcher::{search, SearchHits};
pub use normalize::normalize;
