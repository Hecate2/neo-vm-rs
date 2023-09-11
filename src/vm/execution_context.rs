use std::any::{Any, TypeId};
use std::collections::HashMap;
use crate::evaluation_stack::EvaluationStack;
use crate::exception_handling_context::ExceptionHandlingContext;
use crate::reference_counter::ReferenceCounter;
use crate::slot::Slot;
use crate::stack_item::{StackItem, StackItemTrait};
use crate::vm::script::Script;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Default)]
pub struct ExecutionContext<'a> {
    pub shared_states: SharedStates<'a>,
    pub instruction_pointer: usize,

    /// The number of return values when this context returns.
    pub rv_count: i32,

    /// The local variables of this context.
    pub local_variables: Option<Slot<'a>>,

    /// The arguments passed to this context.
    pub arguments: Option<Slot<'a>>,

    /// The try stack to handle exceptions.
    pub try_stack: Option<Vec<ExceptionHandlingContext>>,
}

struct SharedStates<'a> {
    pub(crate) script: Script,
    pub(crate) evaluation_stack: EvaluationStack<'a>,
    pub(crate) static_fields: Option<Slot<'a>>,
    states: HashMap<TypeId, Box<dyn Any>>,
}

impl ExecutionContext {
    pub fn new(script: Script, reference_counter: &ReferenceCounter) -> Self {
        let shared_states = SharedStates {
            script,
            evaluation_stack: EvaluationStack::new(reference_counter),
            static_fields: None,
            states: HashMap::new(),
        };
        Self {
            shared_states,
            instruction_pointer: 0,
            rv_count: 0,
            local_variables: None,
            arguments: None,
            try_stack: None,
        }
    }

    // Other fields and methods

    pub fn get_state<T: 'static>(&mut self) -> &mut T
        where
            T: Default + Any,
    {
        self.shared_states
            .states
            .entry(TypeId::of::<T>())
            .or_insert_with(|| Box::new(Default::default()))
            .downcast_mut::<T>()
            .unwrap()
    }

    pub fn peek(&self, index: usize) -> &StackItem {
        let idx = self.items.len() - index - 1;
        &self.items[idx]
    }

    pub fn push(&mut self, item: StackItem) {
        self.items.push(item);
        self.reference_counter.add_stack_reference(&item);
    }

    pub fn pop(&mut self) -> StackItem {
        let item = self.items.pop().expect("stack empty");
        self.reference_counter.remove_stack_reference(&item);
        item
    }

    pub fn remove(&mut self, index: usize) -> StackItem {
        let idx = self.items.len() - index - 1;
        let item = self.items.remove(idx).expect("index out of bounds");
        self.reference_counter.remove_stack_reference(&item);
        item.try_into().unwrap()
    }

    pub fn move_next(&mut self) {
        self.instruction_pointer += 1;

        if self.instruction_pointer >= self.script.len() {
            self.instruction_pointer = 0;
        }
    }
}