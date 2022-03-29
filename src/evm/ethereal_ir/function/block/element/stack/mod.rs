//!
//! The Ethereal IR block element stack.
//!

pub mod element;

use self::element::Element;

///
/// The Ethereal IR block element stack.
///
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Stack {
    /// The stack elements.
    pub elements: Vec<Element>,
}

impl Stack {
    /// The default stack size.
    pub const DEFAULT_STACK_SIZE: usize = 16;

    ///
    /// A shortcut constructor.
    ///
    pub fn new() -> Self {
        Self {
            elements: Vec::with_capacity(Self::DEFAULT_STACK_SIZE),
        }
    }

    ///
    /// Pushes a stack element.
    ///
    pub fn push(&mut self, element: Element) {
        self.elements.push(element);
    }

    ///
    /// Extends the stack with some elements.
    ///
    pub fn extend(&mut self, elements: Vec<Element>) {
        self.elements.extend(elements);
    }

    ///
    /// Pops a stack element.
    ///
    pub fn pop(&mut self) -> Option<Element> {
        self.elements.pop()
    }

    ///
    /// Pops the tag from the top.
    ///
    pub fn pop_tag(&mut self) -> anyhow::Result<usize> {
        match self.elements.pop() {
            Some(Element::Tag(tag)) => Ok(tag),
            element => anyhow::bail!("Expected tag, found {:?}", element),
        }
    }

    ///
    /// Swaps two stack elements.
    ///
    pub fn swap(&mut self, index: usize) {
        let length = self.elements.len();
        self.elements.swap(length - 1, length - 1 - index);
    }

    ///
    /// Duplicates a stack element.
    ///
    pub fn dup(&mut self, index: usize) {
        let dupped = self.elements[self.elements.len() - index].to_owned();
        self.elements.push(dupped);
    }
}

impl std::fmt::Display for Stack {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.elements)
    }
}
