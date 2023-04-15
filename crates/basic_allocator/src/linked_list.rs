//! Provide the intrusive LinkedList for basic allocator
#![allow(dead_code)]

use core::{fmt, ptr};

/// An intrusive linked list
///
/// A clean room implementation of the one used in CS140e 2018 Winter
///
/// Thanks Sergio Benitez for his excellent work,
/// See [CS140e](https://cs140e.sergio.bz/) for more information

pub struct MemBlockNode {
    pub size: usize,
    pub nxt: *mut MemBlockNode,
}


#[derive(Copy, Clone)]
pub struct LinkedList {
    pub head: *mut MemBlockNode,
}

unsafe impl Send for LinkedList {}

impl LinkedList {
    /// Create a new LinkedList
    pub const fn new() -> LinkedList {
        LinkedList {
            head: ptr::null_mut(),
        }
    }

    /// Return `true` if the list is empty
    pub fn is_empty(&self) -> bool {
        self.head.is_null()
    }

    /// Push `item` to the front of the list
    pub unsafe fn push(&mut self, item: *mut MemBlockNode, size: usize) {
        (*item).size = size;
        (*item).nxt = self.head;
        self.head = item;
    }

    /// Try to remove the first item in the list
    pub fn pop(&mut self) -> Option<*mut MemBlockNode> {
        match self.is_empty() {
            true => None,
            false => {
                // Advance head pointer
                let item = self.head;
                self.head = unsafe { (*item).nxt as *mut MemBlockNode };
                Some(item)
            }
        }
    }

    /// Return an iterator over the items in the list
    pub fn iter(&self) -> Iter {
        Iter {
            curr: self.head,
            list: self,
        }
    }

    /// Return an mutable iterator over the items in the list
    pub fn iter_mut(&mut self) -> IterMut {
        IterMut {
            prev: ptr::null_mut(),//&mut self.head as *mut *mut MemBlockNode as *mut MemBlockNode,
            curr: self.head,
            list: self,
        }
    }
}

impl fmt::Debug for LinkedList {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_list().entries(self.iter()).finish()
    }
}

/// An iterator over the linked list
pub struct Iter<'a> {
    curr: *mut MemBlockNode,
    list: &'a LinkedList,
}

impl<'a> Iterator for Iter<'a> {
    type Item = *mut MemBlockNode;

    fn next(&mut self) -> Option<Self::Item> {
        if self.curr.is_null() {
            None
        } else {
            let item = self.curr;
            let next = unsafe {(*item).nxt};
            self.curr = next;
            Some(item)
        }
    }
}

/// Represent a mutable node in `LinkedList`
pub struct ListNode {
    prev: *mut MemBlockNode,
    curr: *mut MemBlockNode,
}

impl ListNode {
    /// Remove the node from the list
    pub fn pop(self, list: &mut LinkedList) -> *mut MemBlockNode {
        // Skip the current one
        unsafe {
            //let tmp: usize = 0xffffffc080231240;
            //log::debug!("{:#x}qwq{:#x}",*(tmp as *const usize),*((tmp + 8) as *const usize));
            //log::debug!("{:#x}:::{:#x}",list.head as usize, &list.head as *const *mut MemBlockNode as usize);
            //log::debug!("{:#x}qwq{:#x}",*(tmp as *const usize),*((tmp + 8) as *const usize));
            if list.head == self.curr {
                list.head = (*(self.curr)).nxt;
            } else {
                (*(self.prev)).nxt = (*(self.curr)).nxt;
            }
            //log::debug!("{:#x}qwq{:#x}",*(tmp as *const usize),*((tmp + 8) as *const usize));
            //log::debug!("{:#x}:::{:#x}",list.head as usize, &list.head as *const *mut MemBlockNode as usize);
        }
        self.curr
    }

    /// Returns the pointed address
    pub fn value(&self) -> *mut usize {
        self.curr as *mut usize
    }

    /// return current size of MemBlockNode
    pub unsafe fn size(&self) -> usize {
        (*self.curr).size
    }

    /// change size
    pub unsafe fn change_size(&mut self, size: usize) {
        (*self.curr).size = size;
    }
}

/// A mutable iterator over the linked list
pub struct IterMut<'a> {
    list: &'a mut LinkedList,
    prev: *mut MemBlockNode,
    curr: *mut MemBlockNode,
}

impl<'a> Iterator for IterMut<'a> {
    type Item = ListNode;

    fn next(&mut self) -> Option<Self::Item> {
        if self.curr.is_null() {
            None
        } else {
            let res = ListNode {
                prev: self.prev,
                curr: self.curr,
            };
            self.prev = self.curr;
            self.curr = unsafe {(*(self.curr)).nxt};
            Some(res)
        }
    }
}
