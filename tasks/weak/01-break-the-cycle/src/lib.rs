//! A tree whose nodes point at their parent, without leaking.

use std::cell::RefCell;
use std::rc::Rc;

/// A node in a two-level tree.
pub struct Node {
    /// The node's value.
    pub value: u32,
    /// Children this node owns.
    pub children: RefCell<Vec<Rc<Node>>>,
    // TODO: add a link back to the parent that does NOT keep it alive.
}

/// Build a parent with one child, and link the child back to its parent.
///
/// The parent must be dropped when the caller drops its last handle -- no cycle.
///
/// # Panics
/// The stub panics until you implement it.
#[must_use]
pub fn build() -> Rc<Node> {
    todo!("build a parent and child linked in both directions, without a cycle")
}

/// Read a child's parent value, if the link is still live.
///
/// # Panics
/// The stub panics until you implement it.
#[must_use]
pub fn parent_value_of(_child: &Rc<Node>) -> Option<u32> {
    todo!("follow the upward link")
}

/// The first child of a node.
///
/// # Panics
/// Panics when the node has no children.
#[must_use]
pub fn first_child(node: &Rc<Node>) -> Rc<Node> {
    Rc::clone(&node.children.borrow()[0])
}
