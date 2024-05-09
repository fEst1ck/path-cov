use core::slice;
use std::ffi::{c_char, c_int, CString};

use crate::{
   extern_cfg::{process_top_level, BlockID, FunID, TopLevel},
   hash::hash_path,
   path_reduction::PathReducer,
};

#[no_mangle]
pub unsafe extern "C" fn get_path_reducer(
   top_level: *const TopLevel,
   k: c_int,
) -> *const PathReducer<BlockID, BlockID> {
   let cfgs = process_top_level(top_level);
   let reducer = PathReducer::from_cfgs(cfgs, k as usize);
   Box::into_raw(Box::new(reducer)).cast_const()
}

#[no_mangle]
pub unsafe extern "C" fn free_path_reducer(ptr: *mut PathReducer<BlockID, FunID>) {
    if !ptr.is_null() {
      let _ = Box::from_raw(ptr);
    }
}

#[no_mangle]
pub extern "C" fn free_boxed_array(ptr: *mut i32, len: usize) {
    unsafe {
        // Reconstruct the Box from the raw pointer
        let _boxed_slice = Box::from_raw(std::slice::from_raw_parts_mut(ptr, len));
        // Memory is freed when _boxed_slice goes out of scope
    }
}

#[no_mangle]
pub unsafe extern "C" fn reduce_path(
   reducer: *const PathReducer<BlockID, BlockID>,
   path: *const BlockID,
   path_size: c_int,
   entry_fun_id: FunID,
) -> *const c_char {
   let reducer = reducer.as_ref().expect("bad pointer");
   let path = slice::from_raw_parts(path, path_size as usize);
   let reduced_path = reducer.reduce(path, entry_fun_id);
   let hash = hash_path(&reduced_path);
   let c_string = CString::new(hash).unwrap();
   c_string.as_ptr()
}

#[no_mangle]
pub unsafe extern "C" fn reduce_path1(
   reducer: *const PathReducer<BlockID, BlockID>,
   path: *const BlockID,
   path_size: c_int,
   entry_fun_id: FunID,
   out_len: *mut c_int,
) -> *mut BlockID {
   let reducer = reducer.as_ref().expect("bad pointer");
   let path = slice::from_raw_parts(path, path_size as usize);
   let reduced_path = reducer.reduce(path, entry_fun_id);
   *out_len = reduced_path.len() as c_int;
   Box::into_raw(reduced_path.into_boxed_slice()) as *mut i32
}
