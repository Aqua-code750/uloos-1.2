// ==========================================
// UloOS Heap Allocator for DOOM Engine
// ==========================================
// Provides malloc/free via a linked-list allocator so the DOOM engine
// (doomgeneric) can dynamically allocate memory at runtime.

use linked_list_allocator::LockedHeap;

// 2MB heap — plenty for DOOM shareware, extremely safe for BSS limits
pub const HEAP_SIZE: usize = 2 * 1024 * 1024; // 2MB

static mut HEAP_MEM: [u8; HEAP_SIZE] = [0u8; HEAP_SIZE];

#[global_allocator]
static ALLOCATOR: LockedHeap = LockedHeap::empty();

/// Initialize the heap allocator. Must be called once at boot before any
/// allocations are made (i.e., before launching DOOM).
pub fn init_heap() {
    unsafe {
        ALLOCATOR.lock().init(HEAP_MEM.as_mut_ptr(), HEAP_SIZE);
    }
}

// ==========================================
// C-compatible malloc/free/realloc/calloc
// ==========================================
// These are exported with C linkage so doomgeneric's compiled C code
// can call them. They delegate to Rust's global allocator.

use core::alloc::Layout;

#[no_mangle]
pub unsafe extern "C" fn malloc(size: usize) -> *mut u8 {
    if size == 0 {
        return core::ptr::null_mut();
    }
    let layout = Layout::from_size_align(size, 8).unwrap();
    let ptr = alloc::alloc::alloc(layout);
    ptr
}

#[no_mangle]
pub unsafe extern "C" fn free(ptr: *mut u8) {
    if ptr.is_null() {
        return;
    }
    // We don't know the size, so we use a minimal layout.
    // The linked list allocator tracks block sizes internally.
    let layout = Layout::from_size_align(1, 8).unwrap();
    alloc::alloc::dealloc(ptr, layout);
}

#[no_mangle]
pub unsafe extern "C" fn calloc(count: usize, size: usize) -> *mut u8 {
    let total = count * size;
    let ptr = malloc(total);
    if !ptr.is_null() {
        core::ptr::write_bytes(ptr, 0, total);
    }
    ptr
}

#[no_mangle]
pub unsafe extern "C" fn realloc(ptr: *mut u8, new_size: usize) -> *mut u8 {
    if ptr.is_null() {
        return malloc(new_size);
    }
    let new_ptr = malloc(new_size);
    if !new_ptr.is_null() {
        // Copy old data — we copy new_size bytes (safe minimum estimate)
        core::ptr::copy_nonoverlapping(ptr, new_ptr, new_size);
        free(ptr);
    }
    new_ptr
}

// Allocation error handler (required by #[global_allocator])
#[alloc_error_handler]
fn alloc_error_handler(layout: Layout) -> ! {
    panic!("DOOM allocation error: {:?}", layout);
}
