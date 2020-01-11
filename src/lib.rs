mod iter;
mod eq;

use std::{alloc, mem, ptr, slice, sync::Arc};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct TextLen(pub u32);
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Kind(pub u16);

type Ptr = ptr::NonNull<u8>; // who needs types anyway
type NodeData = VarData<(Kind, TextLen), [Child]>;
type TokenData = VarData<Kind, [u8]>;

// (Private) constructors of these types are  `unsafe`.
#[repr(transparent)]
pub struct Node(Ptr);
#[repr(transparent)]
pub struct Token(Ptr);

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum NodeOrToken<N, T> {
    Node(N),
    Token(T),
}

#[repr(transparent)]
struct Child(Ptr);

pub use crate::iter::{AllChildren, NodeChildren, TokenChildren};

impl Clone for Node {
    fn clone(&self) -> Self {
        unsafe { Node(NodeData::clone(self.0)) }
    }
}

impl Drop for Node {
    fn drop(&mut self) {
        unsafe { NodeData::drop(self.0) }
    }
}

impl Clone for Token {
    fn clone(&self) -> Self {
        unsafe {
            let ptr = untag(self.0);
            let ptr = TokenData::clone(ptr);
            let ptr = tag(ptr);
            Token(ptr)
        }
    }
}

impl Drop for Token {
    fn drop(&mut self) {
        unsafe {
            let ptr = untag(self.0);
            TokenData::drop(ptr)
        }
    }
}

impl Drop for Child {
    fn drop(&mut self) {
        if has_tag(self.0) {
            drop(Token(self.0))
        } else {
            drop(Node(self.0))
        }
    }
}

impl Node {
    pub fn new<I>(kind: Kind, text_len: TextLen, children: I) -> Node
    where
        I: Iterator<Item = NodeOrToken<Node, Token>>,
        I: ExactSizeIterator,
    {
        let ptr = NodeData::new((kind, text_len), children.map(Child::new));
        Node(ptr)
    }

    pub fn kind(&self) -> Kind {
        unsafe { NodeData::header(self.0).1 .0 }
    }
    pub fn text_len(&self) -> TextLen {
        unsafe { NodeData::header(self.0).1 .1 }
    }
    pub fn node_children(&self) -> NodeChildren<'_> {
        NodeChildren { slice: self.children().iter() }
    }
    pub fn token_children(&self) -> TokenChildren<'_> {
        TokenChildren { slice: self.children().iter() }
    }
    pub fn all_children(&self) -> AllChildren<'_> {
        AllChildren { slice: self.children().iter() }
    }

    fn children(&self) -> &[Child] {
        unsafe { NodeData::slice(self.0) }
    }
}

impl Token {
    pub fn new(kind: Kind, text: &str) -> Token {
        let ptr = TokenData::new(kind, text.bytes());
        Token(tag(ptr))
    }

    pub fn kind(&self) -> Kind {
        unsafe {
            let ptr = untag(self.0);
            TokenData::header(ptr).1
        }
    }
    pub fn text_len(&self) -> TextLen {
        unsafe {
            let ptr = untag(self.0);
            let len = TokenData::header(ptr).0;
            TextLen(len)
        }
    }
    pub fn text(&self) -> &str {
        unsafe {
            let ptr = untag(self.0);
            let bytes = TokenData::slice(ptr);
            std::str::from_utf8_unchecked(bytes)
        }
    }
}

impl Child {
    fn new(node_or_token: NodeOrToken<Node, Token>) -> Child {
        let res = match &node_or_token {
            NodeOrToken::Node(it) => Child(it.0),
            NodeOrToken::Token(it) => Child(it.0),
        };
        mem::forget(node_or_token);
        res
    }

    fn as_ref(&self) -> NodeOrToken<&Node, &Token> {
        if has_tag(self.0) {
            NodeOrToken::Token(unsafe { std::mem::transmute::<&Ptr, &Token>(&self.0) })
        } else {
            NodeOrToken::Node(unsafe { std::mem::transmute::<&Ptr, &Node>(&self.0) })
        }
    }
}

#[repr(C, align(2))]
struct VarData<H, T: ?Sized> {
    header: (u32, H),
    slice: T,
}

impl<H, T> VarData<H, [T]> {
    fn layout(n: usize) -> alloc::Layout {
        unsafe {
            assert!(mem::size_of::<T>().saturating_mul(n) <= isize::max_value() as usize);
            let dangling_ptr: *const VarData<H, [T]> =
                { slice::from_raw_parts_mut(Ptr::dangling().as_ptr(), n) as *mut [u8] as *mut _ };
            alloc::Layout::for_value(&*dangling_ptr) // this is def unsound.
        }
    }

    fn new<I>(header: H, iter: I) -> Ptr
    where
        I: IntoIterator<Item = T>,
        I::IntoIter: ExactSizeIterator,
    {
        let mut iter = iter.into_iter();
        let len = iter.len();
        assert!(len <= (u32::max_value() as usize));
        let layout = Self::layout(len);
        unsafe {
            let ptr = alloc::alloc(layout);
            let ptr = Ptr::new(ptr).unwrap_or_else(|| alloc::handle_alloc_error(layout));
            ptr::write(ptr.as_ptr() as *mut (u32, H), (len as u32, header));
            let fat_ptr = Self::fat(ptr);
            for slot in (&mut *(fat_ptr as *mut VarData<H, [mem::MaybeUninit<T>]>)).slice.iter_mut()
            {
                ptr::write(slot.as_mut_ptr(), iter.next().unwrap())
            }

            let box_: Box<VarData<H, [T]>> = {
                assert_eq!(alloc::Layout::for_value(&*fat_ptr), layout);
                Box::from_raw(fat_ptr)
            };

            let arc: Arc<VarData<H, [T]>> = box_.into(); // sad copy :-(
            ptr::NonNull::new_unchecked(Arc::into_raw(arc) as *mut VarData<H, [T]>).cast()
        }
    }
    unsafe fn drop(ptr: Ptr) {
        let fat_ptr = Self::fat(ptr);
        drop(Arc::from_raw(fat_ptr))
    }
    unsafe fn clone(ptr: Ptr) -> Ptr {
        let fat_ptr = Self::fat(ptr);
        let arc = mem::ManuallyDrop::new(Arc::from_raw(fat_ptr));
        mem::forget(Arc::clone(&arc));
        ptr
    }

    unsafe fn header<'a>(ptr: Ptr) -> &'a (u32, H) {
        &(*(ptr.as_ptr() as *const VarData<H, [T; 0]>)).header
    }
    unsafe fn slice<'a>(ptr: Ptr) -> &'a [T] {
        let len = Self::header(ptr).0 as usize;
        if len == 0 {
            return &[];
        }
        let ptr = &((*(ptr.as_ptr() as *const VarData<H, [T; 1]>)).slice[0]) as *const T;
        slice::from_raw_parts(ptr, len)
    }
    unsafe fn fat(ptr: Ptr) -> *mut VarData<H, [T]> {
        let len = Self::header(ptr).0 as usize;
        slice::from_raw_parts_mut(ptr.as_ptr(), len) as *mut [u8] as *mut _
    }
}

fn tag(ptr: Ptr) -> Ptr {
    let ptr = ptr.as_ptr() as usize;
    debug_assert!(ptr & 1 == 0);
    unsafe { Ptr::new_unchecked((ptr | 1) as *mut u8) }
}

unsafe fn untag(ptr: Ptr) -> Ptr {
    let ptr = ptr.as_ptr() as usize;
    debug_assert!(ptr & 1 == 1 && ptr != 1);
    Ptr::new_unchecked((ptr & !1) as *mut u8)
}

fn has_tag(ptr: Ptr) -> bool {
    (ptr.as_ptr() as usize) & 1 == 1
}
