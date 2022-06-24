//! Runtime representation of a module.
//!
//! For now it's just a GC wrapper around [`compiler::Module`].

use std::{
    cell::RefCell,
    fmt::{self, Debug, Formatter},
    ptr::addr_of_mut,
};

use common::{Get, Index};
use compiler::{Constant, Export, Import};
use diagnostic::InputId;

use crate::{
    classes::Prototype,
    memory::{Class, ClassId, Gc, GcAny, InitFrom, Object, Trace},
    primitives::PrimitiveOperations,
    Value, VirtualMachine,
};

#[derive(PartialEq)]
#[repr(C, align(8))]
pub struct Module {
    base: Object,

    id: Option<InputId>,
    name: Value,
    constants: Vec<Value>,
    prototypes: Vec<Gc<Prototype>>,
    exports: RefCell<Vec<Value>>,
    imports: RefCell<Vec<Gc<Module>>>,
}

impl Module {
    /// # Safety
    ///
    /// This (unsafely) mutates the [`Module`] object. The [`Module`] must be
    /// rooted when this is called.
    ///
    /// This sucks. I'm _really_ hoping that eventual cleaning up of the whole
    /// GC makes this a normal part of initialization.
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

        for constant in module.constants() {
            let value = vm.inflate(constant);
            live_module.constants.push(value);
        }

        for function in module.functions() {
            live_module
                .prototypes
                .push(vm.make_from((gc, function.to_owned())))
        }

        // We create and set the right number of exports to `()` so that we
        // don't need to worry about bounds checks while the module's `main` is
        // being run. The compiler would have ensured that no value is loaded
        // before it's bound with a `pub let` so this is fine.
        //
        // TODO: What does this mean if another module tries to load one of
        //       these before it's assigned? We should ensure a module's `main`
        //       is finished running before some other module bind it to their
        //       imports.
        live_module
            .exports
            .replace(vec![Value::UNIT; module.export_count()]);

        for dependency in module.imports() {
            if let Some(live_dep) = vm.get_module_by_name(dependency.name()) {
                live_module.imports.borrow_mut().push(live_dep)
            } else {
                panic!(
                    "no module named `{}` found in scope",
                    dependency.name()
                );
            }
        }
    }

    pub(crate) unsafe fn steal_exports(new: Gc<Module>, old: Gc<Module>) {
        assert!(old.exports.borrow().len() <= new.exports.borrow().len());

        for i in 0..old.exports.borrow().len() {
            let index = Index::new(i as _);
            let export = old.export(index);
            new.set_export(index, export);
        }
    }

    pub fn name(&self) -> Value {
        self.name
    }

    pub fn main(&self) -> Gc<Prototype> {
        debug_assert!(
            !self.prototypes.is_empty(),
            "module needs top-level code"
        );

        self.prototypes[0]
    }

    pub fn id(&self) -> Option<InputId> {
        self.id
    }

    pub fn constant(&self, index: Index<Constant>) -> Option<Value> {
        self.constants.get(index.as_usize()).copied()
    }

    pub fn export(&self, index: Index<Export>) -> Value {
        self.exports.borrow()[index.as_usize()]
    }

    pub fn set_export(&self, index: Index<Export>, value: Value) {
        self.exports.borrow_mut()[index.as_usize()] = value;
    }

    pub fn import(&self, index: Index<Import>) -> Gc<Module> {
        self.imports.borrow()[index.as_usize()]
    }
}

impl Get<compiler::Function, Gc<Prototype>> for Module {
    fn get(&self, index: Index<compiler::Function>) -> Option<&Gc<Prototype>> {
        self.prototypes.get(index.as_usize())
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
    fn enqueue_gc_references(&self, worklist: &mut crate::memory::WorkList) {
        for p in self.prototypes.iter() {
            worklist.enqueue(GcAny::from(*p));
        }

        for v in self.constants.iter() {
            v.enqueue_gc_references(worklist);
        }
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
        addr_of_mut!((*ptr).id).write(None);
        addr_of_mut!((*ptr).name).write(Value::UNIT);
        addr_of_mut!((*ptr).constants).write(Vec::new());
        addr_of_mut!((*ptr).prototypes).write(Vec::new());
        addr_of_mut!((*ptr).exports).write(RefCell::new(Vec::new()));
        addr_of_mut!((*ptr).imports).write(RefCell::new(Vec::new()));
    }
}
