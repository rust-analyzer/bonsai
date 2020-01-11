use std::{cmp, hash};

use crate::{Child, Kind, Node, TextLen, Token};

impl Node {
    fn key(&self) -> (Kind, TextLen, &[Child]) {
        (self.kind(), self.text_len(), self.children())
    }
}

impl PartialEq for Node {
    fn eq(&self, other: &Node) -> bool {
        self.0 == other.0 || (self.key() == other.key())
    }
}

impl Eq for Node {}

impl PartialOrd for Node {
    fn partial_cmp(&self, other: &Node) -> Option<cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Node {
    fn cmp(&self, other: &Node) -> cmp::Ordering {
        self.key().cmp(&other.key())
    }
}

impl hash::Hash for Node {
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        hash::Hash::hash(&self.key(), state)
    }
}

impl Token {
    fn key(&self) -> (Kind, &str) {
        (self.kind(), self.text())
    }
}

impl PartialEq for Token {
    fn eq(&self, other: &Token) -> bool {
        self.0 == other.0 || (self.key() == other.key())
    }
}

impl Eq for Token {}

impl PartialOrd for Token {
    fn partial_cmp(&self, other: &Token) -> Option<cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Token {
    fn cmp(&self, other: &Token) -> cmp::Ordering {
        self.key().cmp(&other.key())
    }
}

impl hash::Hash for Token {
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        hash::Hash::hash(&self.key(), state)
    }
}

impl PartialEq for Child {
    fn eq(&self, other: &Child) -> bool {
        self.0 == other.0 || (self.as_ref() == other.as_ref())
    }
}

impl Eq for Child {}

impl PartialOrd for Child {
    fn partial_cmp(&self, other: &Child) -> Option<cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Child {
    fn cmp(&self, other: &Child) -> cmp::Ordering {
        self.as_ref().cmp(&other.as_ref())
    }
}

impl hash::Hash for Child {
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        hash::Hash::hash(&self.as_ref(), state)
    }
}
