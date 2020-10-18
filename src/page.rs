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
    Taken = 1 << 0,
    Last = 1 << 1,
}

pub struct PageDescriptor {
    pub flags: u8,
}

impl PageDescriptor {
    pub fn new() -> Self {
        PageDescriptor { flags: 0 }
    }

    pub fn is_taken(&self) -> bool {
        self.flags & PageDescriptorFlags::Taken as u8 == PageDescriptorFlags::Taken as u8
    }

    pub fn is_last(&self) -> bool {
        self.flags & PageDescriptorFlags::Last as u8 == PageDescriptorFlags::Last as u8
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
pub enum PageTableEntryFlags {
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

pub struct Sv39PageTableEntry {
    pub entry: usize,
}

impl Sv39PageTableEntry {
    pub fn is_valid(&self) -> bool {
        self.entry & PageTableEntryFlags::Valid as usize == PageTableEntryFlags::Valid as usize
    }

    pub fn is_readable(&self) -> bool {
        self.entry & PageTableEntryFlags::Read as usize == PageTableEntryFlags::Read as usize
    }

    pub fn is_writable(&self) -> bool {
        self.entry & PageTableEntryFlags::Write as usize == PageTableEntryFlags::Write as usize
    }

    pub fn is_executable(&self) -> bool {
        self.entry & PageTableEntryFlags::Execute as usize == PageTableEntryFlags::Execute as usize
    }

    pub fn get_physical_address(&self) -> usize {
        (self.entry & !0x3ff) << 2
    }

    pub fn is_leaf(&self) -> bool {
        self.is_readable() || self.is_writable()
    }
}

// 2^9 = 512 entries per table
pub struct Sv39PageTable {
    pub entries: [Sv39PageTableEntry; 512],
}

impl Sv39PageTable {
    pub fn levels() -> usize {
        3
    }
    // Map a virtual address to a physical address using 4096-byte page
    // size.
    pub fn map(
        &mut self,
        virtual_address: usize,
        physical_address: usize,
        flags: usize,
        level: usize,
    ) {
        // Make sure that Read, Write, or Execute have been provided
        // otherwise, we'll leak memory and always create a page fault.
        assert!(flags & 0xe != 0);

        // Sv39 virtual address (9 bits each)
        let virtual_page_number = [
            (virtual_address >> 12) & 0x1ff,
            (virtual_address >> 21) & 0x1ff,
            (virtual_address >> 30) & 0x1ff,
        ];
        //  Sv39 physical address
        let physical_page_number = [
            (physical_address >> 12) & 0x1ff,
            (physical_address >> 21) & 0x1ff,
            (physical_address >> 30) & 0x1ff,
        ];

        let mut page_table_entry = &mut self.entries[virtual_page_number[2]];

        for i in (level..Sv39PageTable::levels() - 1).rev() {
            // If it's not valid, you can use it
            if !page_table_entry.is_valid() {
                let page = zalloc(1);
                // The page is stored in the entry shifted right by 2 places.
                page_table_entry.entry = (page as usize >> 2) | PageTableEntryFlags::Valid as usize;
            }

            let entry_as_table = page_table_entry.get_physical_address() as *mut Sv39PageTable;
            page_table_entry = unsafe { &mut (*entry_as_table).entries[virtual_page_number[i]] };
        }
        // VPN[0]
        // Create new page table entry.
        // Reserved | PPN[2] | PPN[1] | PPN[0] | RSW | FLAG_BITS
        // PPN[2] = [53:28], PPN[1] = [27:19], PPN[0] = [18:10]
        page_table_entry.entry = (physical_page_number[2] << 28)
            | (physical_page_number[1] << 19)
            | (physical_page_number[0] << 10)
            | flags
            | PageTableEntryFlags::Valid as usize
            | PageTableEntryFlags::Dirty as usize
            | PageTableEntryFlags::Access as usize;
    }

    /// Unmaps and frees all memory associated with a table.
    pub fn unmap(&mut self) {
        for entry in self.entries.iter_mut() {
            // Check if entry is valid and is a branch
            if entry.is_valid() && !entry.is_leaf() {
                let table = entry.get_physical_address() as *mut Sv39PageTable;

                unsafe {
                    (*table).unmap();
                }

                dealloc(table as *mut u8);
            }
        }
    }

    pub fn virtual_address_translation(&self, virtual_address: usize) -> Option<usize> {
        // Sv39 virtual address (9 bits each)
        let virtual_page_number = [
            (virtual_address >> 12) & 0x1ff,
            (virtual_address >> 21) & 0x1ff,
            (virtual_address >> 30) & 0x1ff,
        ];

        // a = satp.ppn * PAGE_SIZE, althou self points to a already
        // a + va.ppn[i] * PTESIZE
        let mut page_table_entry = &self.entries[virtual_page_number[2]];

        for i in (0..=(Sv39PageTable::levels() - 1)).rev() {
            // pte.v = 0 OR (pte.r = 0 AND pte.w = 1)
            if !page_table_entry.is_valid()
                || (!page_table_entry.is_readable() && page_table_entry.is_writable())
            {
                // Page fault
                return None;
            }

            // pte.r = 1 OR pte.x = 1
            if page_table_entry.is_leaf() {
                // Leaf found
                // Masks PPN[i]. Starts at #12, each one with 9 bits
                let offset_mask = (1 << (12 + i * 9)) - 1;
                // pa.pgoff = vaa.pgoff
                let vaddr_pgoff = virtual_address & offset_mask;

                // maybe do see how the ISA checks for supper pages

                // pa.ppn[]
                let addr = ((page_table_entry.entry << 2) as usize) & !offset_mask;

                return Some(addr | vaddr_pgoff);
            }

            let entry_as_table = page_table_entry.get_physical_address() as *const Sv39PageTable;

            page_table_entry = unsafe { &(*entry_as_table).entries[virtual_page_number[i - 1]] };
        }

        None
    }
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
pub fn dealloc(ptr: *mut u8) {
    // Make sure we don't try to free a null pointer.
    assert!(!ptr.is_null());
    unsafe {
        let address = HEAP_START + (ptr as usize - PAGE_TABLE_START_ADDRESS) / PAGE_SIZE;
        // Make sure that the address makes sense. The address we
        // calculate here is the page structure, not the HEAP address!
        assert!(address >= HEAP_START && address < PAGE_TABLE_START_ADDRESS);
        let mut p = address as *mut PageDescriptor;
        assert!((*p).is_taken(), "Freeing a non-taken page?");
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

/// Print all page allocations
/// This is mainly used for debugging.
pub fn print_page_allocations() {
    unsafe {
        let num_pages = (HEAP_SIZE - (PAGE_TABLE_START_ADDRESS - HEAP_START)) / PAGE_SIZE;
        let mut beg = HEAP_START as *const PageDescriptor;
        let end = beg.add(num_pages);
        let alloc_beg = PAGE_TABLE_START_ADDRESS;
        let alloc_end = PAGE_TABLE_START_ADDRESS + num_pages * PAGE_SIZE;
        println!();
        println!(
            "PAGE ALLOCATION TABLE\nMETA: {:p} -> {:p}\nPHYS: \
		          0x{:x} -> 0x{:x}",
            beg, end, alloc_beg, alloc_end
        );
        println!("~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~");
        let mut num = 0;
        while beg < end {
            if (*beg).is_taken() {
                let start = beg as usize;
                let memaddr = PAGE_TABLE_START_ADDRESS + (start - HEAP_START) * PAGE_SIZE;
                print!("0x{:x} => ", memaddr);
                loop {
                    num += 1;
                    if (*beg).is_last() {
                        let end = beg as usize;
                        let memaddr =
                            PAGE_TABLE_START_ADDRESS + (end - HEAP_START) * PAGE_SIZE + PAGE_SIZE
                                - 1;
                        print!("0x{:x}: {:>3} page(s)", memaddr, (end - start + 1));
                        println!(".");
                        break;
                    }
                    beg = beg.add(1);
                }
            }
            beg = beg.add(1);
        }
        println!("~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~");
        println!(
            "Allocated: {:>6} pages ({:>10} bytes).",
            num,
            num * PAGE_SIZE
        );
        println!(
            "Free     : {:>6} pages ({:>10} bytes).",
            num_pages - num,
            (num_pages - num) * PAGE_SIZE
        );
        println!();
    }
}

