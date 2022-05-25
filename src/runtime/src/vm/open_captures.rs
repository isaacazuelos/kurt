use common::Index;

use crate::{classes::CaptureCell, memory::Gc, vm::value_stack::ValueStack};

#[derive(Default)]
pub(crate) struct OpenCaptures {
    cells: Vec<Gc<CaptureCell>>,
}

impl OpenCaptures {
    pub(crate) fn push(&mut self, cell: Gc<CaptureCell>) {
        match cell.contents() {
            crate::classes::CaptureCellContents::Inline(_) => {
                panic!("attempted to add a closed capture to the open list")
            }
            crate::classes::CaptureCellContents::Stack(i) => {
                debug_assert!(
                    if let Some(last) = self.last_index() {
                        last < i
                    } else {
                        true
                    },
                    "open captures list must remain sorted"
                );

                self.cells.push(cell)
            }
        }
    }

    pub(crate) fn last_index(&self) -> Option<Index<ValueStack>> {
        let cell = self.cells.last()?;
        let index = cell
            .stack_index()
            .expect("all cells in the open list should be open");

        Some(index)
    }

    /// Pop an open cell if it's stack index is above the given `top` index.
    pub(crate) fn pop_up_to(
        &mut self,
        top: Index<ValueStack>,
    ) -> Option<Gc<CaptureCell>> {
        if self.last_index()? >= top {
            self.cells.pop()
        } else {
            None
        }
    }

    /// Iterator over all the open capture cells, from most recent to least recent.
    #[cfg(feature = "trace")]
    pub(crate) fn iter(&self) -> impl Iterator<Item = &Gc<CaptureCell>> {
        self.cells.iter().rev()
    }
}
