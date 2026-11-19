pub(crate) mod change;
pub(crate) mod chunk;

pub use change::{Change, ChangesIter};
pub use chunk::{ChunksIter, DiffChunk};
