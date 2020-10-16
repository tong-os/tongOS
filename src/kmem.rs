// kmem.rs
// Sub-page level: malloc-like allocation system
// Stephen Marz
// tongOS team

use crate::page;
use core::{mem::size_of, ptr::null_mut};

static mut KMEM_ALLOC: usize = 0;
static mut KMEM_HEAD: *mut KernelPageDescriptor = core::ptr::null_mut();

#[repr(usize)]
pub enum KernelPageDescriptorFlags {
    Taken = 1 << 63,
}

pub struct KernelPageDescriptor {
    pub flags_size: usize,
}

impl KernelPageDescriptor {
    pub fn new(size: usize) -> Self {
        KernelPageDescriptor {
            flags_size: size & !(KernelPageDescriptorFlags::Taken as usize),
        }
    }

    pub fn is_taken(&self) -> bool {
        self.flags_size & KernelPageDescriptorFlags::Taken as usize != 0
    }

    pub fn clear(&mut self) {
        self.flags_size = 0;
    }

    pub fn set_flag(&mut self, flag: KernelPageDescriptorFlags) {
        self.flags_size |= flag as usize;
    }

    pub fn set_size(&mut self, sz: usize) {
        self.flags_size = sz & !(KernelPageDescriptorFlags::Taken as usize);
        if self.is_taken() {
            self.flags_size |= KernelPageDescriptorFlags::Taken as usize
        }
    }
    pub fn get_size(&self) -> usize {
        self.flags_size & !(KernelPageDescriptorFlags::Taken as usize)
    }
}

/// Initialize kernel's memory
/// This is not to be used to allocate memory
/// for user processes. If that's the case, use
// alloc/dealloc from the page crate.
pub fn init() {
    unsafe {
        // Allocate kernel pages (KMEM_ALLOC)
        KMEM_ALLOC = 2048;
        let k_alloc = page::zalloc(KMEM_ALLOC);
        // Check if allcation is right
        assert!(!k_alloc.is_null());

        KMEM_HEAD = k_alloc as *mut KernelPageDescriptor;

        KMEM_HEAD.write_volatile(KernelPageDescriptor::new(KMEM_ALLOC * page::PAGE_SIZE))
    }
}

// Allocate sub-page level allocation based on bytes
pub fn kmalloc(size: usize) -> *mut u8 {
    unsafe {
        let size = page::align_address(size, 3) * core::mem::size_of::<KernelPageDescriptor>();

        let mut head = KMEM_HEAD;

        let tail =
            (KMEM_HEAD as *mut u8).add(KMEM_ALLOC * page::PAGE_SIZE) as *mut KernelPageDescriptor;

        while head < tail {
            // Check if is free and new size <= current size
            if !(*head).is_taken() && size <= (*head).get_size() {
                let chunk_size = (*head).get_size();
                let remaining_space = chunk_size - size;
                // Set taken bit
                (*head).set_flag(KernelPageDescriptorFlags::Taken);
                // Alloc whole size if possible.
                // Otherwise, only remaining space available.
                if remaining_space > size_of::<KernelPageDescriptor>() {
                    let next = (head as *mut u8).add(size) as *mut KernelPageDescriptor;
                    // There is space remaining here.
                    (*next).clear();
                    (*next).set_size(remaining_space);
                    (*head).set_size(size);
                } else {
                    // If we get here, take the entire chunk
                    (*head).set_size(chunk_size);
                }
                return head.add(1) as *mut u8;
            } else {
                // If we get here, what we saw wasn't a free
                // chunk, move on to the next.
                head = (head as *mut u8).add((*head).get_size()) as *mut KernelPageDescriptor;
            }
        }
    }

    core::ptr::null_mut()
}

// Allocate sub-page level allocation based on bytes and zero the memory
pub fn kzmalloc(sz: usize) -> *mut u8 {
    let size = page::align_address(sz, 3);
    let ret = kmalloc(size) as *mut u64;

    if !ret.is_null() {
        for i in 0..size {
            unsafe {
                (*ret.add(i)) = 0;
            }
        }
    }
    ret as *mut u8
}

pub fn print_table() {
	unsafe {
		let mut head = KMEM_HEAD;
		let tail = (KMEM_HEAD as *mut u8).add(KMEM_ALLOC * page::PAGE_SIZE)
		           as *mut KernelPageDescriptor;
		while head < tail {
			println!(
			         "{:p}: Length = {:<10} Taken = {}",
			         head,
			         (*head).get_size(),
			         (*head).is_taken()
			);
			head = (head as *mut u8).add((*head).get_size())
			       as *mut KernelPageDescriptor;
		}
	}
}