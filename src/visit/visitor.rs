//  VISITOR.rs
//    by Lut99
//
//  Description:
//!   Implements the generic visitor implementation.
//

use std::any::TypeId;
use std::collections::HashMap;

use super::{Visit, VisitMut};
use crate::tree::Node;


/***** LIBRARY *****/
/// A generic visitor that allows you to program it to trigger and process certain nodes in a tree.
///
/// This version does read-only iteration. See [`VisitorMut`] for one that does mutable access.
pub struct Visitor<'f> {
    ref_callbacks: HashMap<TypeId, Box<dyn 'f + for<'s, 'a> FnMut(&'s Self, &'a dyn Node)>>,
    mut_callbacks: HashMap<TypeId, Box<dyn 'f + for<'s, 'a> FnMut(&'s Self, &'a mut dyn Node)>>,
}

// Constructors
impl<'f> Default for Visitor<'f> {
    #[inline(always)]
    fn default() -> Self { Self::new() }
}
impl<'f> Visitor<'f> {
    /// Constructor for the Visitor that initializes it as empty.
    ///
    /// If you were to traverse using it now, it would visit all nodes but nothing would happen.
    ///
    /// You can program it by calling [`Visitor::add_visit()`].
    ///
    /// # Returns
    /// A new, empty Visitor.
    #[inline(always)]
    pub fn new() -> Self { Self { ref_callbacks: HashMap::new(), mut_callbacks: HashMap::new() } }
}

// Collection
impl<'f> Visitor<'f> {
    /// Adds a new read-only callback to the visitor.
    ///
    /// # Generics
    /// - `T`: The node type to add the callback for.
    ///
    /// # Arguments
    /// - `cb`: Some [`FnMut`]-function that gets called whenever a node of the given `T`ype is
    ///   visited. **Note that this function should also handle the next traversal step!** I.e.,
    ///   once you have visited your node, the callback function is responsible for calling the
    ///   Visitor again to further drive the traversal.
    ///
    /// # Returns
    /// Self for chaining.
    pub fn add_visit<T: 'static>(&mut self, mut cb: impl 'f + FnMut(&Self, &T)) -> &mut Self {
        let type_id = TypeId::of::<T>();
        self.ref_callbacks.insert(
            type_id,
            Box::new(move |visitor, node| {
                // SAFETY: We can unwrap this because the callback is only called if we found a
                // type ID match.
                let node = node.as_any().downcast_ref::<T>().unwrap();
                cb(visitor, node)
            }),
        );
        self
    }
}

// Visitor
impl<'f> Visitor<'f> {
    /// Visits a given [`Node`] and all of its children, read-only.
    ///
    /// # Arguments
    /// - `node`: The node to visit.
    pub fn visit<T: Visit>(&mut self, node: &T) {
        // See if there's a callback for the current node
        let type_id = node.as_any().type_id();
        if let Some(cb) = self.ref_callbacks.get(&type_id) {
            return cb(self, node);
        }

        // Else, defer to the node's standard traversal function
        node.visit(self)
    }

    /// Visits a given [`Node`] and all of its children, mutably.
    ///
    /// # Arguments
    /// - `node`: The node to visit.
    pub fn visit_mut<T: VisitMut>(&mut self, node: &mut T) {
        // See if there's a callback for the current node
        let type_id = node.as_any().type_id();
        if let Some(cb) = self.mut_callbacks.get(&type_id) {
            return cb(self, node);
        }

        // Else, defer to the node's standard traversal function
        node.visit_mut(self)
    }
}
