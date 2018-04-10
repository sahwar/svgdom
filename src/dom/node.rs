// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use std::fmt;

use error::Result;
use {
    Attribute,
    AttributeId,
    AttributeQName,
    AttributeQNameRef,
    Attributes,
    AttributeValue,
    ElementId,
    Error,
    NodeType,
    QName,
    QNameRef,
    TagName,
    TagNameRef,
};
use super::{
    tree,
    NodeData,
};

impl<'a, N, V> From<(N, V)> for Attribute
    where AttributeQNameRef<'a>: From<N>, AttributeValue: From<V>
{
    fn from(v: (N, V)) -> Self {
        Attribute::new(v.0, v.1)
    }
}

impl<'a, N> From<(N, Node)> for Attribute
    where AttributeQNameRef<'a>: From<N>, N: Clone
{
    fn from(v: (N, Node)) -> Self {
        let n = AttributeQNameRef::from(v.0.clone());

        if n.has_id("xlink", AttributeId::Href) {
            Attribute::new(v.0, AttributeValue::Link(v.1))
        } else {
            Attribute::new(v.0, AttributeValue::FuncLink(v.1))
        }
    }
}


/// Representation of the SVG node.
///
/// This is the main block of the library.
///
/// It's designed as classical DOM node. It has links to a parent node, first child, last child,
/// previous sibling and next sibling. So DOM nodes manipulations are very fast.
///
/// Node consists of:
///
/// - The [`NodeType`], which indicates it's type. It can't be changed.
/// - Optional [`TagName`], used only by element nodes.
/// - Unique ID of the `Element` node. Can be set to nodes with other types,
///   but without any affect.
/// - [`Attributes`] - list of [`Attribute`]s.
/// - List of linked nodes. [Details.](#method.set_attribute_checked)
/// - Text data, which is used by non-element nodes. Empty by default.
///
/// [`Attribute`]: struct.Attribute.html
/// [`Attributes`]: struct.Attributes.html
/// [`NodeType`]: enum.NodeType.html
/// [`TagName`]: type.TagName.html
pub type Node = tree::Node<NodeData>;

impl Node {
    /// Returns `true` if the node has a parent node.
    ///
    /// This method ignores root node.
    ///
    /// # Panics
    ///
    /// Panics if the node is currently mutably borrowed.
    ///
    /// # Examples
    /// ```
    /// use svgdom::Document;
    ///
    /// let doc = Document::from_str(
    /// "<svg>
    ///     <rect/>
    /// </svg>").unwrap();
    ///
    /// let svg = doc.root().first_child().unwrap();
    /// let rect = svg.first_child().unwrap();
    /// assert_eq!(svg.has_parent(), false);
    /// assert_eq!(rect.has_parent(), true);
    /// ```
    pub fn has_parent(&self) -> bool {
        match self.parent() {
            Some(node) => node.node_type() != NodeType::Root,
            None => false,
        }
    }

    /// Returns node's type.
    ///
    /// You can't change the type of the node. Only create a new one.
    ///
    /// # Panics
    ///
    /// Panics if the node is currently mutably borrowed.
    pub fn node_type(&self) -> NodeType {
        self.borrow().node_type
    }

    /// Returns a text data of the node.
    ///
    /// Nodes with `Element` type can't contain text data.
    ///
    /// # Panics
    ///
    /// Panics if the node is currently mutably borrowed.
    pub fn text(&self) -> &str {
        self.borrow().text.as_str()
    }

    /// Returns a mutable text data of the node.
    ///
    /// Nodes with `Element` type can't contain text data.
    ///
    /// # Panics
    ///
    /// Panics if the node is currently mutably borrowed.
    pub fn text_mut(&mut self) -> &mut String {
        &mut self.borrow_mut().text
    }

    /// Sets a text data to the node.
    ///
    /// # Panics
    ///
    /// Panics if the node is currently mutably borrowed.
    pub fn set_text(&mut self, text: &str) {
        debug_assert_ne!(self.node_type(), NodeType::Element);
        self.borrow_mut().text = text.to_owned();
    }

    /// Returns an ID of the element node.
    ///
    /// # Panics
    ///
    /// Panics if the node is currently mutably borrowed.
    pub fn id(&self) -> &str {
        self.borrow().id.as_str()
    }

    /// Returns `true` if node has a not empty ID.
    ///
    /// # Panics
    ///
    /// Panics if the node is currently mutably borrowed.
    pub fn has_id(&self) -> bool {
        !self.id().is_empty()
    }

    /// Sets an ID of the element.
    ///
    /// Only element nodes can contain an ID.
    ///
    /// # Panics
    ///
    /// Panics if the node is currently borrowed.
    pub fn set_id<S: Into<String>>(&mut self, id: S) {
        // TODO: check that it's unique.
        debug_assert_eq!(self.node_type(), NodeType::Element);
        self.borrow_mut().id = id.into().to_owned();
    }

    /// Returns `true` if node has an `Element` type and an SVG tag name.
    ///
    /// # Panics
    ///
    /// Panics if the node is currently mutably borrowed.
    pub fn is_svg_element(&self) -> bool {
        if self.node_type() != NodeType::Element {
            return false;
        }

        match self.borrow().tag_name {
            QName::Id(_, _) => true,
            QName::Name(_, _) => false,
        }
    }

    /// Returns a tag name of the element node.
    ///
    /// # Panics
    ///
    /// Panics if the node is currently mutably borrowed.
    pub fn tag_name(&self) -> &TagName {
        &self.borrow().tag_name
    }

    /// Returns a tag name id of the SVG element node.
    ///
    /// # Panics
    ///
    /// Panics if the node is currently mutably borrowed.
    pub fn tag_id(&self) -> Option<ElementId> {
        match self.borrow().tag_name {
            QName::Id(_, ref id) => Some(*id),
            QName::Name(_, _) => None,
        }
    }

    /// Returns `true` if node has the same tag name as supplied.
    ///
    /// # Panics
    ///
    /// Panics if the node is currently mutably borrowed.
    pub fn is_tag_name<'a, T>(&self, tag_name: T) -> bool
        where TagNameRef<'a>: From<T>
    {
        self.borrow().tag_name.as_ref() == TagNameRef::from(tag_name)
    }

    /// Sets a tag name of the element node.
    ///
    /// Only element nodes can contain tag name.
    ///
    /// # Errors
    ///
    /// The string tag name must be non-empty.
    ///
    /// # Panics
    ///
    /// - Panics if the node is currently borrowed.
    /// - Panics if a string tag name is empty.
    pub fn set_tag_name<'a, T>(&mut self, tag_name: T)
        where TagNameRef<'a>: From<T>
    {
        debug_assert_eq!(self.node_type(), NodeType::Element);

        let tn = TagNameRef::from(tag_name);
        if let QNameRef::Name(_, name) = tn {
            if name.is_empty() {
                panic!("supplied tag name is empty");
            }
        }

        self.borrow_mut().tag_name = TagName::from(tn);
    }

    /// Returns a reference to the `Attributes` of the current node.
    ///
    /// # Panics
    ///
    /// Panics if the node is currently mutably borrowed.
    pub fn attributes(&self) -> &Attributes {
        &self.borrow().attributes
    }

    /// Returns a mutable reference to the `Attributes` of the current node.
    ///
    /// # Panics
    ///
    /// Panics if the node is currently borrowed.
    pub fn attributes_mut(&mut self) -> &mut Attributes {
        &mut self.borrow_mut().attributes
    }

    /// Returns `true` if the node has an attribute with such `id`.
    ///
    /// # Panics
    ///
    /// Panics if the node is currently mutably borrowed.
    #[inline]
    pub fn has_attribute<'a, N>(&self, name: N) -> bool
        where AttributeQNameRef<'a>: From<N>
    {
        self.borrow().attributes.contains(name)
    }

    /// Returns `true` if the node has an attribute with such `id` and this attribute is visible.
    ///
    /// # Panics
    ///
    /// Panics if the node is currently mutably borrowed.
    pub fn has_visible_attribute(&self, id: AttributeId) -> bool {
        if let Some(attr) = self.attributes().get(id) { attr.visible } else { false }
    }

    // TODO: remove
    /// Returns `true` if the node has any of provided attributes.
    ///
    /// # Panics
    ///
    /// Panics if the node is currently mutably borrowed.
    pub fn has_attributes(&self, ids: &[AttributeId]) -> bool {
        let attrs = self.attributes();
        for id in ids {
            if attrs.contains(*id) {
                return true;
            }
        }

        false
    }

    /// Inserts a new attribute into attributes list.
    ///
    /// Unwrapped version of the [`set_attribute_checked`] method.
    ///
    /// # Panics
    ///
    /// Will panic on any error produced by the [`set_attribute_checked`] method.
    ///
    /// [`set_attribute_checked`]: #method.set_attribute_checked
    pub fn set_attribute<T>(&mut self, v: T)
        where T: Into<Attribute>
    {
        self.set_attribute_checked(v).unwrap();
    }

    /// Inserts a new attribute into attributes list.
    ///
    /// You can set attribute using one of the possible combinations:
    ///
    /// - ([`AttributeId`]/`&str`, [`AttributeValue`])
    /// - ([`AttributeId`], [`Node`])
    /// - [`Attribute`]
    ///
    /// [`AttributeId`]: enum.AttributeId.html
    /// [`Attribute`]: struct.Attribute.html
    /// [`Node`]: struct.Node.html
    /// [`AttributeValue`]: enum.AttributeValue.html
    ///
    /// This method will overwrite an existing attribute with the same name.
    ///
    /// # Errors
    ///
    /// - [`ElementMustHaveAnId`]
    /// - [`ElementCrosslink`]
    ///
    /// # Panics
    ///
    /// Panics if the node is currently borrowed.
    ///
    /// # Examples
    ///
    /// Ways to specify attributes:
    ///
    /// ```
    /// use svgdom::{
    ///     Document,
    ///     Attribute,
    ///     AttributeId as AId,
    ///     ElementId as EId,
    /// };
    ///
    /// // Create a simple document.
    /// let mut doc = Document::new();
    /// let mut svg = doc.create_element(EId::Svg);
    /// let mut rect = doc.create_element(EId::Rect);
    ///
    /// doc.root().append(svg.clone());
    /// svg.append(rect.clone());
    ///
    /// // In order to set element as an attribute value, we must set id first.
    /// rect.set_id("rect1");
    ///
    /// // Using predefined attribute name.
    /// svg.set_attribute((AId::X, 1.0));
    /// svg.set_attribute((AId::X, "random text"));
    /// // Using custom attribute name.
    /// svg.set_attribute(("non-svg-attr", 1.0));
    /// // Using existing attribute object.
    /// svg.set_attribute(Attribute::new(AId::X, 1.0));
    /// svg.set_attribute(Attribute::new("non-svg-attr", 1.0));
    /// // Using an existing node as an attribute value.
    /// svg.set_attribute((("xlink", AId::Href), rect));
    /// ```
    ///
    /// Linked attributes:
    ///
    /// ```
    /// use svgdom::{
    ///     Document,
    ///     AttributeId as AId,
    ///     ElementId as EId,
    ///     ValueId,
    /// };
    ///
    /// // Create a simple document.
    /// let mut doc = Document::new();
    /// let mut gradient = doc.create_element(EId::LinearGradient);
    /// let mut rect = doc.create_element(EId::Rect);
    ///
    /// doc.root().append(gradient.clone());
    /// doc.root().append(rect.clone());
    ///
    /// gradient.set_id("lg1");
    /// rect.set_id("rect1");
    ///
    /// // Set a `fill` attribute value to the `none`.
    /// // For now everything like in any other XML DOM library.
    /// rect.set_attribute((AId::Fill, ValueId::None));
    ///
    /// // Now we want to fill our rect with a gradient.
    /// // To do this we need to set a link attribute:
    /// rect.set_attribute((AId::Fill, gradient.clone()));
    ///
    /// // Now our fill attribute has a link to the `gradient` node.
    /// // Not as text, aka `url(#lg1)`, but as actual reference.
    ///
    /// // This adds support for fast checking that the element is used. Which is very useful.
    ///
    /// // `gradient` is now used, since we link it.
    /// assert_eq!(gradient.is_used(), true);
    /// // Also, we can check how many elements are uses this `gradient`.
    /// assert_eq!(gradient.uses_count(), 1);
    /// // And even get this elements.
    /// assert_eq!(gradient.linked_nodes()[0], rect);
    ///
    /// // And now, if we remove our `rect` - `gradient` will became unused again.
    /// doc.remove_node(rect);
    /// assert_eq!(gradient.is_used(), false);
    /// ```
    ///
    /// [`ElementMustHaveAnId`]: enum.Error.html
    /// [`ElementCrosslink`]: enum.Error.html
    pub fn set_attribute_checked<T>(&mut self, v: T) -> Result<()>
        where T: Into<Attribute>
    {
        self.set_attribute_checked_impl(v.into())
    }

    fn set_attribute_checked_impl(&mut self, attr: Attribute) -> Result<()> {
        // TODO: to error in _checked mode
        debug_assert_eq!(self.node_type(), NodeType::Element);

        match attr.value {
              AttributeValue::Link(ref iri)
            | AttributeValue::FuncLink(ref iri) => {
                self.set_link_attribute(attr.name, iri.clone())?;
                return Ok(());
            }
            _ => {}
        }

        self.set_simple_attribute(attr);

        Ok(())
    }

    fn set_simple_attribute(&mut self, attr: Attribute) {
        debug_assert!(!attr.is_link() && !attr.is_func_link());

        // we must remove existing attribute to prevent dangling links
        self.remove_attribute(attr.name.as_ref());

        let attrs = self.attributes_mut();
        attrs.insert(attr);
    }

    fn set_link_attribute(&mut self, name: AttributeQName, mut node: Node) -> Result<()> {
        if node.id().is_empty() {
            return Err(Error::ElementMustHaveAnId);
        }

        // check for recursion
        if *self.id() == *node.id() {
            return Err(Error::ElementCrosslink);
        }

        // check for recursion 2
        if self.linked_nodes().iter().any(|n| *n == node) {
            return Err(Error::ElementCrosslink);
        }

        // we must remove existing attribute to prevent dangling links
        self.remove_attribute(name.as_ref());

        {
            let a = if name.has_id("xlink", AttributeId::Href) {
                Attribute::new(name.as_ref(), AttributeValue::Link(node.clone()))
            } else {
                Attribute::new(name.as_ref(), AttributeValue::FuncLink(node.clone()))
            };

            let attributes = self.attributes_mut();
            attributes.insert_impl(a);
        }

        node.borrow_mut().linked_nodes.push(self.clone());

        Ok(())
    }

    /// Inserts a new attribute into attributes list if it doesn't contain one.
    ///
    /// `value` will be cloned if needed.
    ///
    /// Shorthand for:
    ///
    /// ```ignore
    /// if !node.has_attribute(...) {
    ///     node.set_attribute(...);
    /// }
    /// ```
    ///
    /// # Panics
    ///
    /// Will panic on any error produced by the [`set_attribute_checked`] method.
    ///
    /// [`set_attribute_checked`]: #method.set_attribute_checked
    pub fn set_attribute_if_none<'a, N, T>(&mut self, name: N, value: &T)
        where AttributeQNameRef<'a>: From<N>, N: Copy, AttributeValue: From<T>, T: Clone
    {
        if !self.has_attribute(name) {
            self.set_attribute((name, value.clone()));
        }
    }

    /// Removes an attribute from the node.
    ///
    /// It will also unlink it, if it was an referenced attribute.
    ///
    /// # Panics
    ///
    /// Panics if the node is currently borrowed.
    pub fn remove_attribute<'a, N>(&mut self, name: N)
        where AttributeQNameRef<'a>: From<N>, N: Copy
    {
        if !self.has_attribute(name) {
            return;
        }

        // we must unlink referenced attributes
        if let Some(value) = self.attributes().get_value(name) {
            match *value {
                AttributeValue::Link(ref node) | AttributeValue::FuncLink(ref node) => {
                    let mut node = node.clone();

                    // this code can't panic, because we know that such node exist
                    let index = node.borrow().linked_nodes.iter().position(|n| n == self).unwrap();
                    node.borrow_mut().linked_nodes.remove(index);
                }
                _ => {}
            }
        }

        self.attributes_mut().remove_impl(name);
    }

    /// Returns an iterator over linked nodes.
    ///
    /// See [Node::set_attribute()](#method.set_attribute) for details.
    ///
    /// # Panics
    ///
    /// Panics if the node is currently mutably borrowed.
    pub fn linked_nodes(&self) -> &[Node] {
        &self.borrow().linked_nodes
    }

    pub fn linked_nodes_mut(&mut self) -> &mut [Node] {
        &mut self.borrow_mut().linked_nodes
    }

    /// Returns `true` if the current node is linked to any of the DOM nodes.
    ///
    /// See [Node::set_attribute()](#method.set_attribute) for details.
    ///
    /// # Panics
    ///
    /// Panics if the node is currently mutably borrowed.
    pub fn is_used(&self) -> bool {
        !self.borrow().linked_nodes.is_empty()
    }

    /// Returns a number of nodes, which is linked to this node.
    ///
    /// See [Node::set_attribute()](#method.set_attribute) for details.
    ///
    /// # Panics
    ///
    /// Panics if the node is currently mutably borrowed.
    pub fn uses_count(&self) -> usize {
        self.borrow().linked_nodes.len()
    }
}

impl fmt::Debug for Node {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.node_type() {
            NodeType::Root => write!(f, "RootNode"),
            NodeType::Element => write!(f, "ElementNode({:?} id={:?})", self.tag_name(), self.id()),
            NodeType::Declaration => write!(f, "DeclarationNode({:?})", self.text()),
            NodeType::Comment => write!(f, "CommentNode({:?})", self.text()),
            NodeType::Cdata => write!(f, "CdataNode({:?})", self.text()),
            NodeType::Text => write!(f, "TextNode({:?})", self.text()),
        }
    }
}
