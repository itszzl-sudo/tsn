#[allow(dead_code)]
use std::alloc::{alloc, realloc, Layout};
use std::ptr::NonNull;

// NaN-boxing relies on 48-bit virtual address space (x86-64, ARM64 with 48-bit VA).
// ARM64 with 52-bit PA or PAC pointer auth would break the PTR_MASK approach.
#[cfg(not(any(target_arch = "x86_64", target_arch = "aarch64")))]
compile_error!("ts-native NaN-boxing currently only supports x86_64 and aarch64");

// On x86_64/aarch64, only the lower 48 bits of virtual addresses are used,
// so NaN-boxing tag scheme works correctly despite usize being 64 bits.

const POINTER_TAG: u64 = 0x7FFD_0000_0000_0000;
const STRING_TAG: u64 = 0x7FFC_0000_0000_0000;
const ARRAY_TAG: u64 = 0x7FFB_0000_0000_0000;
const OBJECT_TAG: u64 = 0x7FFA_0000_0000_0000;

pub struct JsString {
    pub len: u32,
    pub hash: u32,
    pub data: [u8; 0],
}

impl JsString {
    pub fn new(s: &str) -> NonNull<Self> {
        let len = s.len() as u32;
        let hash = Self::compute_hash(s.as_bytes());
        
        let layout = Layout::from_size_align(
            std::mem::size_of::<JsString>() + len as usize,
            8,
        ).unwrap();
        
        unsafe {
            let ptr = alloc(layout) as *mut JsString;
            (*ptr).len = len;
            (*ptr).hash = hash;
            std::ptr::copy_nonoverlapping(
                s.as_ptr(),
                (*ptr).data.as_mut_ptr(),
                len as usize,
            );
            NonNull::new_unchecked(ptr)
        }
    }
    
    pub fn as_str(&self) -> &str {
        unsafe {
            std::str::from_utf8_unchecked(std::slice::from_raw_parts(
                self.data.as_ptr(),
                self.len as usize,
            ))
        }
    }
    
    fn compute_hash(data: &[u8]) -> u32 {
        let mut hash: u32 = 0;
        for &byte in data {
            hash = hash.wrapping_mul(31).wrapping_add(byte as u32);
        }
        hash
    }
    
    pub fn concat(a: &str, b: &str) -> NonNull<Self> {
        let mut result = String::with_capacity(a.len() + b.len());
        result.push_str(a);
        result.push_str(b);
        Self::new(&result)
    }
}

pub struct JsArray {
    pub len: u32,
    pub capacity: u32,
    pub data: [u64; 0],
}

impl JsArray {
    pub fn new(capacity: u32) -> NonNull<Self> {
        let cap = if capacity == 0 { 8 } else { capacity };
        let layout = Layout::from_size_align(
            std::mem::size_of::<JsArray>() + cap as usize * 8,
            8,
        ).unwrap();
        
        unsafe {
            let ptr = alloc(layout) as *mut JsArray;
            (*ptr).len = 0;
            (*ptr).capacity = cap;
            NonNull::new_unchecked(ptr)
        }
    }
    
    /// Push a value. Returns the new pointer if reallocation occurred.
    /// The caller MUST use the returned NonNull if it's different from the original.
    pub unsafe fn push(ptr: &mut NonNull<JsArray>, value: u64) {
        let arr = ptr.as_mut();
        if arr.len >= arr.capacity {
            *ptr = Self::grow_raw(*ptr);
        }
        let arr = ptr.as_mut();
        *arr.data.as_mut_ptr().add(arr.len as usize) = value;
        arr.len += 1;
    }
    
    /// Reallocate the array to double capacity. Returns the new NonNull.
    /// The old pointer is invalidated by realloc.
    unsafe fn grow_raw(old: NonNull<JsArray>) -> NonNull<JsArray> {
        let arr = old.as_ref();
        let new_cap = arr.capacity * 2;
        let old_layout = Layout::from_size_align(
            std::mem::size_of::<JsArray>() + arr.capacity as usize * 8,
            8,
        ).unwrap();
        
        let new_ptr = realloc(
            old.as_ptr() as *mut u8,
            old_layout,
            std::mem::size_of::<JsArray>() + new_cap as usize * 8,
        ) as *mut JsArray;
        debug_assert!(!new_ptr.is_null(), "JsArray realloc failed");
        (*new_ptr).capacity = new_cap;
        NonNull::new_unchecked(new_ptr)
    }
    
    pub fn get(&self, index: u32) -> u64 {
        if index >= self.len {
            return super::codegen::UNDEFINED;
        }
        unsafe { *self.data.as_ptr().add(index as usize) }
    }
    
    /// Set a value at index. Returns the new pointer if reallocation occurred.
    pub unsafe fn set(ptr: &mut NonNull<JsArray>, index: u32, value: u64) {
        let arr = ptr.as_ref();
        if index >= arr.len {
            // Extend array to include index
            while ptr.as_ref().len <= index {
                if ptr.as_ref().len >= ptr.as_ref().capacity {
                    *ptr = Self::grow_raw(*ptr);
                }
                let arr = ptr.as_mut();
                *arr.data.as_mut_ptr().add(arr.len as usize) = super::codegen::UNDEFINED;
                arr.len += 1;
            }
        }
        let arr = ptr.as_mut();
        *arr.data.as_mut_ptr().add(index as usize) = value;
    }
}

pub struct JsObject {
    pub size: u32,
    pub capacity: u32,
    pub entries: [ObjectEntry; 0],
}

#[repr(C)]
pub struct ObjectEntry {
    pub key: u64,
    pub value: u64,
}

impl JsObject {
    pub fn new() -> NonNull<Self> {
        let layout = Layout::from_size_align(
            std::mem::size_of::<JsObject>() + 8 * std::mem::size_of::<ObjectEntry>(),
            8,
        ).unwrap();
        
        unsafe {
            let ptr = alloc(layout) as *mut JsObject;
            (*ptr).size = 0;
            (*ptr).capacity = 8;
            NonNull::new_unchecked(ptr)
        }
    }
    
    /// Set a key-value pair. Returns the new pointer if reallocation occurred.
    /// The caller MUST use the returned NonNull if it's different from the original.
    pub unsafe fn set(ptr: &mut NonNull<JsObject>, key: u64, value: u64) {
        let obj = ptr.as_mut();
        for i in 0..obj.size {
            let entry = &mut *obj.entries.as_mut_ptr().add(i as usize);
            if entry.key == key {
                entry.value = value;
                return;
            }
        }
        
        if ptr.as_ref().size >= ptr.as_ref().capacity {
            *ptr = Self::grow_raw(*ptr);
        }
        
        let obj = ptr.as_mut();
        let entry = &mut *obj.entries.as_mut_ptr().add(obj.size as usize);
        entry.key = key;
        entry.value = value;
        obj.size += 1;
    }
    
    /// Reallocate the object to double capacity. Returns the new NonNull.
    unsafe fn grow_raw(old: NonNull<JsObject>) -> NonNull<JsObject> {
        let obj = old.as_ref();
        let new_cap = obj.capacity * 2;
        let old_layout = Layout::from_size_align(
            std::mem::size_of::<JsObject>() + obj.capacity as usize * std::mem::size_of::<ObjectEntry>(),
            8,
        ).unwrap();
        
        let new_ptr = realloc(
            old.as_ptr() as *mut u8,
            old_layout,
            std::mem::size_of::<JsObject>() + new_cap as usize * std::mem::size_of::<ObjectEntry>(),
        ) as *mut JsObject;
        debug_assert!(!new_ptr.is_null(), "JsObject realloc failed");
        (*new_ptr).capacity = new_cap;
        NonNull::new_unchecked(new_ptr)
    }
    
    pub fn get(&self, key: u64) -> u64 {
        for i in 0..self.size {
            unsafe {
                let entry = &*self.entries.as_ptr().add(i as usize);
                if entry.key == key {
                    return entry.value;
                }
            }
        }
        super::codegen::UNDEFINED
    }
}

pub fn nanbox_pointer(ptr: NonNull<()>) -> u64 {
    POINTER_TAG | (ptr.as_ptr() as u64 & 0x0000_FFFF_FFFF_FFFF)
}

pub fn nanbox_string(ptr: NonNull<JsString>) -> u64 {
    STRING_TAG | (ptr.as_ptr() as u64 & 0x0000_FFFF_FFFF_FFFF)
}

pub fn nanbox_array(ptr: NonNull<JsArray>) -> u64 {
    ARRAY_TAG | (ptr.as_ptr() as u64 & 0x0000_FFFF_FFFF_FFFF)
}

pub fn nanbox_object(ptr: NonNull<JsObject>) -> u64 {
    OBJECT_TAG | (ptr.as_ptr() as u64 & 0x0000_FFFF_FFFF_FFFF)
}

pub fn get_pointer(val: u64) -> *mut () {
    (val & 0x0000_FFFF_FFFF_FFFF) as *mut ()
}

pub fn is_string(val: u64) -> bool {
    (val >> 48) == (STRING_TAG >> 48)
}

pub fn is_array(val: u64) -> bool {
    (val >> 48) == (ARRAY_TAG >> 48)
}

pub fn is_object(val: u64) -> bool {
    (val >> 48) == (OBJECT_TAG >> 48)
}
