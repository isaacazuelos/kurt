use common::Index;

use crate::internal::local::Local;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Capture {
    Local(Index<Local>),
    Recapture(Index<Capture>),
}
