use compiler::Index;

use crate::{classes::CaptureCell, memory::Gc, vm::value_stack::ValueStack};

#[derive(Default)]
pub(crate) struct OpenCaptures {
    pub(crate) open: Vec<(Index<ValueStack>, Gc<CaptureCell>)>,
}

impl OpenCaptures {
    // /// Closes all open upvalues which occur in the open list with a stack index
    // /// above `top`.
    // pub(crate) fn close_above(
    //     &mut self,
    //     top: Index<ValueStack>,
    //     stack: &mut ValueStack,
    // ) {
    //     while let Some((index, cell)) = self.open.last().cloned() {
    //         if index > top {
    //             let value = stack
    //                 .get(index)
    //                 .expect("open capture cell past end of stack");

    //             cell.close(value);
    //             self.open.pop();
    //         }
    //     }
    // }

    /// Iterator over all the open capture cells, from most recent to least recent.
    pub(crate) fn iter(&self) -> impl Iterator<Item = &Gc<CaptureCell>> {
        self.open.iter().rev().map(|(_, cell)| cell)
    }
}
