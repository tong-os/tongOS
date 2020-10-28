// kmem.rs
// Sub-page level: malloc-like allocation system
// Stephen Marz
// tongOS team

use crate::page;

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
        self.flags_size & KernelPageDescriptorFlags::Taken as usize
            == KernelPageDescriptorFlags::Taken as usize
    }

    pub fn set_taken(&mut self) {
        self.flags_size |= KernelPageDescriptorFlags::Taken as usize;
    }

    pub fn set_free(&mut self) {
        self.flags_size &= !(KernelPageDescriptorFlags::Taken as usize);
    }

    pub fn set_size(&mut self, size: usize) {
        let is_taken = self.is_taken();

        self.flags_size = size & !(KernelPageDescriptorFlags::Taken as usize);

        if is_taken {
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

        (*KMEM_HEAD).set_size(KMEM_ALLOC * page::PAGE_SIZE);
        (*KMEM_HEAD).set_free();
    }
}

/// Allocate sub-page level allocation based on bytes
pub fn kmalloc(size: usize) -> *mut u8 {
    unsafe {
        let size = page::align_address(size, 3) + core::mem::size_of::<KernelPageDescriptor>();
        let mut head = KMEM_HEAD;
        // .add() uses pointer arithmetic, so we type-cast into a u8
        // so that we multiply by an absolute size (KMEM_ALLOC *
        // PAGE_SIZE).
        let tail =
            (KMEM_HEAD as *mut u8).add(KMEM_ALLOC * page::PAGE_SIZE) as *mut KernelPageDescriptor;

        while head < tail {
            if !(*head).is_taken() && size <= (*head).get_size() {
                let chunk_size = (*head).get_size();
                let rem = chunk_size - size;
                (*head).set_taken();
                if rem > core::mem::size_of::<KernelPageDescriptor>() {
                    let next = (head as *mut u8).add(size) as *mut KernelPageDescriptor;
                    // There is space remaining here.
                    (*next).set_free();
                    (*next).set_size(rem);
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
    // If we get here, we didn't find any free chunks--i.e. there isn't
    // enough memory for this. TODO: Add on-demand page allocation.
    core::ptr::null_mut()
}

// Allocate sub-page level allocation based on bytes and zero the memory
pub fn kzmalloc(sz: usize) -> *mut u8 {
    let size = page::align_address(sz, 3);
    let ret = kmalloc(size);

    if !ret.is_null() {
        for i in 0..size {
            unsafe {
                (*ret.add(i)) = 0;
            }
        }
    }
    ret
}

pub fn kfree(ptr: *mut u8) {
    unsafe {
        if !ptr.is_null() {
            let descriptor = (ptr as *mut KernelPageDescriptor).offset(-1);
            if (*descriptor).is_taken() {
                (*descriptor).set_free()
            }

            coalesce();
        }
    }
}

/// Merge smaller chunks into a bigger chunk
fn coalesce() {
    unsafe {
        let mut head = KMEM_HEAD;
        let tail =
            (KMEM_HEAD as *mut u8).add(KMEM_ALLOC * page::PAGE_SIZE) as *mut KernelPageDescriptor;

        while head < tail {
            let next = (head as *mut u8).add((*head).get_size()) as *mut KernelPageDescriptor;
            if (*head).get_size() == 0 {
                // If this happens, then we have a bad heap
                // (double free or something). However, that
                // will cause an infinite loop since the next
                // pointer will never move beyond the current
                // location.
                break;
            } else if next >= tail {
                // We calculated the next by using the size
                // given as get_size(), however this could push
                // us past the tail. In that case, the size is
                // wrong, hence we break and stop doing what we
                // need to do.
                break;
            } else if !(*head).is_taken() && !(*next).is_taken() {
                // This means we have adjacent blocks needing to
                // be freed. So, we combine them into one
                // allocation.
                (*head).set_size((*head).get_size() + (*next).get_size());
            }
            // If we get here, we might've moved. Recalculate new
            // head.
            head = (head as *mut u8).add((*head).get_size()) as *mut KernelPageDescriptor;
        }
    }
}

pub fn print_table() {
    unsafe {
        let mut head = KMEM_HEAD;
        let tail =
            (KMEM_HEAD as *mut u8).add(KMEM_ALLOC * page::PAGE_SIZE) as *mut KernelPageDescriptor;

        while head < tail {
            println!(
                "{:p}: Length = {:<10} Taken = {}",
                head,
                (*head).get_size(),
                (*head).is_taken()
            );
            head = (head as *mut u8).add((*head).get_size()) as *mut KernelPageDescriptor;
        }
    }
}

use core::alloc::{GlobalAlloc, Layout};

struct OsGlobalAlloc;

unsafe impl GlobalAlloc for OsGlobalAlloc {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        // We align to the next page size so that when
        // we divide by PAGE_SIZE, we get exactly the number
        // of pages necessary.
        kzmalloc(layout.size())
    }

    unsafe fn dealloc(&self, ptr: *mut u8, _layout: Layout) {
        // We ignore layout since our allocator uses ptr_start -> last
        // to determine the span of an allocation.
        kfree(ptr);
    }
}

/// Technically, we don't need the {} at the end, but it
/// reveals that we're creating a new structure and not just
/// copying a value.
#[global_allocator]
static GA: OsGlobalAlloc = OsGlobalAlloc {};

#[alloc_error_handler]
/// If for some reason alloc() in the global allocator gets null_mut(),
/// then we come here. This is a divergent function, so we call panic to
/// let the tester know what's going on.
pub fn alloc_error(l: Layout) -> ! {
    panic!(
        "Allocator failed to allocate {} bytes with {}-byte alignment.",
        l.size(),
        l.align()
    );
}
