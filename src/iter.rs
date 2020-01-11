use std::slice;

use crate::{Child, Node, NodeOrToken, Token};

pub struct NodeChildren<'a> {
    pub(crate) slice: slice::Iter<'a, Child>,
}

impl<'a> Iterator for NodeChildren<'a> {
    type Item = &'a Node;
    fn next(&mut self) -> Option<&'a Node> {
        self.slice.by_ref().map(Child::as_ref).find_map(|child| match child {
            NodeOrToken::Node(it) => Some(it),
            NodeOrToken::Token(_) => None,
        })
    }
}

pub struct TokenChildren<'a> {
    pub(crate) slice: slice::Iter<'a, Child>,
}

impl<'a> Iterator for TokenChildren<'a> {
    type Item = &'a Token;
    fn next(&mut self) -> Option<&'a Token> {
        self.slice.by_ref().map(Child::as_ref).find_map(|child| match child {
            NodeOrToken::Node(_) => None,
            NodeOrToken::Token(it) => Some(it),
        })
    }
}

pub struct AllChildren<'a> {
    pub(crate) slice: slice::Iter<'a, Child>,
}

impl<'a> Iterator for AllChildren<'a> {
    type Item = NodeOrToken<&'a Node, &'a Token>;
    fn next(&mut self) -> Option<Self::Item> {
        self.slice.next().map(Child::as_ref)
    }
}

impl<'a> ExactSizeIterator for AllChildren<'a> {
    fn len(&self) -> usize {
        self.slice.len()
    }
}
