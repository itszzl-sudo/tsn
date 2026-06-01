use std::alloc::{alloc, realloc, Layout};
use std::ptr::NonNull;

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
    
    pub fn push(&mut self, value: u64) {
        if self.len >= self.capacity {
            self.grow();
        }
        unsafe {
            *self.data.as_mut_ptr().add(self.len as usize) = value;
        }
        self.len += 1;
    }
    
    fn grow(&mut self) {
        let new_cap = self.capacity * 2;
        let old_layout = Layout::from_size_align(
            std::mem::size_of::<JsArray>() + self.capacity as usize * 8,
            8,
        ).unwrap();
        let new_layout = Layout::from_size_align(
            std::mem::size_of::<JsArray>() + new_cap as usize * 8,
            8,
        ).unwrap();
        
        unsafe {
            let ptr = realloc(
                self as *mut Self as *mut u8,
                old_layout,
                new_layout.size(),
            ) as *mut JsArray;
            (*ptr).capacity = new_cap;
        }
    }
    
    pub fn get(&self, index: u32) -> u64 {
        if index >= self.len {
            return super::codegen::UNDEFINED;
        }
        unsafe { *self.data.as_ptr().add(index as usize) }
    }
    
    pub fn set(&mut self, index: u32, value: u64) {
        if index < self.len {
            unsafe {
                *self.data.as_mut_ptr().add(index as usize) = value;
            }
        }
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
    
    pub fn set(&mut self, key: u64, value: u64) {
        for i in 0..self.size {
            unsafe {
                let entry = &mut *self.entries.as_mut_ptr().add(i as usize);
                if entry.key == key {
                    entry.value = value;
                    return;
                }
            }
        }
        
        if self.size >= self.capacity {
            self.grow();
        }
        
        unsafe {
            let entry = &mut *self.entries.as_mut_ptr().add(self.size as usize);
            entry.key = key;
            entry.value = value;
        }
        self.size += 1;
    }
    
    fn grow(&mut self) {
        let new_cap = self.capacity * 2;
        let old_layout = Layout::from_size_align(
            std::mem::size_of::<JsObject>() + self.capacity as usize * std::mem::size_of::<ObjectEntry>(),
            8,
        ).unwrap();
        let new_layout = Layout::from_size_align(
            std::mem::size_of::<JsObject>() + new_cap as usize * std::mem::size_of::<ObjectEntry>(),
            8,
        ).unwrap();
        
        unsafe {
            let ptr = realloc(
                self as *mut Self as *mut u8,
                old_layout,
                new_layout.size(),
            ) as *mut JsObject;
            (*ptr).capacity = new_cap;
        }
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
