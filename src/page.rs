//  Sv39:Page-Based 39-bit Virtual-Memory System
//  Section 4.4 from ISA 1.12
//  Stephen Marz
//  tongOS team

use crate::assembly::{HEAP_SIZE, HEAP_START};

// Page size = 4096 bytes
const PAGE_ORDER: usize = 12;
pub const PAGE_SIZE: usize = 1 << PAGE_ORDER;

pub static mut NUMBER_OF_PAGES: usize = 0;
pub static mut PAGE_TABLE_START_ADDRESS: usize = 0;
pub static mut PAGE_DESCRIPTOR_PTR: *mut PageDescriptor = core::ptr::null_mut();

/// Align (set to a multiple of some power of two)
/// This takes an order which is the exponent to 2^order
/// Therefore, all alignments must be made as a power of two.
/// This function always rounds up.
pub const fn align_address(address: usize, order: usize) -> usize {
    // this will mask the bits (order - 1)..0 of the address
    let mask = (1usize << order) - 1;
    (address + mask) & !mask
}

#[repr(u8)]
pub enum PageDescriptorFlags {
    Empty = 0b0,
    Taken = 0b1,
    Last = 0b10,
}

pub struct PageDescriptor {
    pub flags: u8,
}

impl PageDescriptor {
    pub fn new() -> Self {
        PageDescriptor {
            flags: PageDescriptorFlags::Empty as u8,
        }
    }

    pub fn is_taken(&self) -> bool {
        self.flags & PageDescriptorFlags::Taken as u8 == 1
    }

    pub fn is_last(&self) -> bool {
        self.flags & PageDescriptorFlags::Last as u8 == 1
    }

    pub fn clear(&mut self) {
        self.flags = 0;
    }

    pub fn set_flag(&mut self, flag: PageDescriptorFlags) {
        self.flags |= flag as u8;
    }
}

// Represent (repr) our entry bits as
// unsigned 64-bit integers.
#[repr(usize)]
#[derive(Copy, Clone)]
pub enum PageTableEntryBits {
    None = 0,
    Valid = 1 << 0,
    Read = 1 << 1,
    Write = 1 << 2,
    Execute = 1 << 3,
    User = 1 << 4,
    Global = 1 << 5,
    Access = 1 << 6,
    Dirty = 1 << 7,

    // Convenience combinations
    ReadWrite = 1 << 1 | 1 << 2,
    ReadExecute = 1 << 1 | 1 << 3,
    ReadWriteExecute = 1 << 1 | 1 << 2 | 1 << 3,

    // User Convenience Combinations
    UserReadWrite = 1 << 1 | 1 << 2 | 1 << 4,
    UserReadExecute = 1 << 1 | 1 << 3 | 1 << 4,
    UserReadWriteExecute = 1 << 1 | 1 << 2 | 1 << 3 | 1 << 4,
}

struct Sv39PageTableEntry {
    pub entry: usize,
}

impl Sv39PageTableEntry {}

// 2^9 = 512 entries per table
struct Sv39PageTable {
    pub entries: [Sv39PageTableEntry; 512],
}

// Alloc 1 page strucutre per 4k bytes
pub fn init() {
    unsafe {
        NUMBER_OF_PAGES = HEAP_SIZE / PAGE_SIZE;

        PAGE_DESCRIPTOR_PTR = HEAP_START as *mut PageDescriptor;

        // Clear all pages
        for i in 0..NUMBER_OF_PAGES {
            PAGE_DESCRIPTOR_PTR
                .add(i)
                .write_volatile(PageDescriptor::new());
        }

        PAGE_TABLE_START_ADDRESS = align_address(
            HEAP_START + NUMBER_OF_PAGES * core::mem::size_of::<PageDescriptor>(),
            PAGE_ORDER,
        );
    }
}

/// Allocate a page or multiple pages
/// request_pages: the number of PAGE_SIZE pages to allocate
/// return
pub fn alloc(request_pages: usize) -> *mut u8 {
    // We have to find a contiguous allocation of pages
    assert!(request_pages > 0);
    unsafe {
        let page_descriptors = PAGE_DESCRIPTOR_PTR;
        for i in 0..NUMBER_OF_PAGES - request_pages {
            let mut found = false;
            // Check if is free
            if !(*page_descriptors.add(i)).is_taken() {
                found = true;
                for j in i..i + request_pages {
                    // Check if contigous allocation is possible
                    if (*page_descriptors.add(j)).is_taken() {
                        found = false;
                        break;
                    }
                }
            }
            if found {
                for k in i..i + request_pages - 1 {
                    (*page_descriptors.add(k)).set_flag(PageDescriptorFlags::Taken);
                }
                // The marker for the last page is
                // PageBits::Last This lets us know when we've
                // hit the end of this particular allocation.
                (*page_descriptors.add(i + request_pages - 1)).set_flag(PageDescriptorFlags::Taken);
                (*page_descriptors.add(i + request_pages - 1)).set_flag(PageDescriptorFlags::Last);

                return (PAGE_TABLE_START_ADDRESS + PAGE_SIZE * i) as *mut u8;
            }
        }
    }
    core::ptr::null_mut()
}

/// Allocate and zero a page or multiple pages
/// pages: the number of pages to allocate
/// Each page is PAGE_SIZE which is calculated as 1 << PAGE_ORDER
pub fn zalloc(pages: usize) -> *mut u8 {
    let ret = alloc(pages);
    if !ret.is_null() {
        let size = (PAGE_SIZE * pages) / 8;
        let big_ptr = ret as *mut u64;
        for i in 0..size {
            // We use big_ptr so that we can force an
            // sd (store doubleword).
            unsafe {
                (*big_ptr.add(i)) = 0;
            }
        }
    }
    ret
}

/// Deallocate a page by its pointer
/// The way we've structured this, it will automatically coalesce
/// contiguous pages.
pub fn dealloc(ptr: *mut u8) {
    // Make sure we don't try to free a null pointer.
    assert!(!ptr.is_null());
    unsafe {
        let address = HEAP_START + (ptr as usize - PAGE_TABLE_START_ADDRESS) / PAGE_SIZE;
        // Make sure that the address makes sense. The address we
        // calculate here is the page structure, not the HEAP address!
        assert!(address >= HEAP_START && address < PAGE_TABLE_START_ADDRESS);
        let mut p = address as *mut PageDescriptor;
        // println!("PTR in is {:p}, addr is 0x{:x}", ptr, addr);
        assert!((*p).is_taken(), "Freeing a non-taken page?");
        // Keep clearing pages until we hit the last page.
        while (*p).is_taken() && !(*p).is_last() {
            (*p).clear();
            p = p.add(1);
        }
        // If the following assertion fails, it is most likely
        // caused by a double-free.
        assert!(
            (*p).is_last() == true,
            "Possible double-free detected! (Not taken found \
                 before last)"
        );
        // If we get here, we've taken care of all previous pages and
        // we are on the last page.
        (*p).clear();
    }
}
