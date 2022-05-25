//! Runtime representation of a module.
//!
//! For now it's just a GC wrapper around [`compiler::Module`].

use std::{
    fmt::{self, Debug, Formatter},
    ptr::addr_of_mut,
};

use common::{Get, Index};
use compiler::{Constant, ModuleDebug};

use crate::{
    classes::Prototype,
    memory::{Class, ClassId, Gc, InitFrom, Object, Trace},
    primitives::PrimitiveOperations,
    VirtualMachine,
};

#[derive(PartialEq)]
#[repr(C, align(8))]
pub struct Module {
    base: Object,

    constants: Vec<Constant>,
    prototypes: Vec<Gc<Prototype>>,

    debug_info: Option<ModuleDebug>,
}

impl Module {
    /// # Safety
    ///
    /// This (unsafely) mutates the [`Module`] object.
    pub(crate) unsafe fn destructively_set_up_from_compiler_module(
        gc: Gc<Module>,
        module: compiler::Module,
        vm: &mut VirtualMachine,
    ) {
        let live_module = gc.deref_mut();

        debug_assert!(
            live_module.prototypes.is_empty(),
            "modules should only be set up once"
        );

        debug_assert!(
            live_module.constants.is_empty(),
            "modules should only be set up once"
        );

        live_module.constants = module.constants().to_owned();
        live_module.debug_info = module.debug_info().map(ToOwned::to_owned);

        for function in module.functions() {
            live_module
                .prototypes
                .push(vm.make_from((gc, function.to_owned())))
        }
    }

    pub fn main(&self) -> Gc<Prototype> {
        debug_assert!(
            !self.prototypes.is_empty(),
            "module needs top-level code"
        );

        self.prototypes[0]
    }

    pub fn debug_info(&self) -> Option<&ModuleDebug> {
        self.debug_info.as_ref()
    }
}

impl Get<compiler::Function, Gc<Prototype>> for Module {
    fn get(&self, index: Index<compiler::Function>) -> Option<&Gc<Prototype>> {
        self.prototypes.get(index.as_usize())
    }
}

impl Get<Constant> for Module {
    fn get(&self, index: Index<Constant>) -> Option<&Constant> {
        self.constants.get(index.as_usize())
    }
}

impl Debug for Module {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "<module>")
    }
}

impl PartialOrd for Module {
    fn partial_cmp(&self, _: &Self) -> Option<std::cmp::Ordering> {
        None
    }
}

impl Trace for Module {
    fn enqueue_gc_references(&self, _worklist: &mut crate::memory::WorkList) {
        // No gc values, yet.
    }
}

impl Class for Module {
    const ID: ClassId = ClassId::Module;
}

impl PrimitiveOperations for Module {
    fn type_name(&self) -> &'static str {
        "module"
    }
}

impl InitFrom<()> for Module {
    fn extra_size(_arg: &()) -> usize {
        0 // none
    }

    unsafe fn init(ptr: *mut Self, _args: ()) {
        addr_of_mut!((*ptr).debug_info).write(None);
        addr_of_mut!((*ptr).constants).write(Vec::new());
        addr_of_mut!((*ptr).prototypes).write(Vec::new());
    }
}
