use core::slice;
use std::ffi::{c_char, c_int, CString};

use crate::{extern_cfg::{process_top_level, BlockID, FunID, TopLevel}, hash::hash_path, path_reduction::PathReducer};

#[no_mangle]
pub unsafe extern "C" fn get_path_reducer(top_level: *const TopLevel, k: c_int) -> *const PathReducer<BlockID, BlockID> {
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
pub unsafe extern "C" fn reduce_path(reducer: *const PathReducer<BlockID, BlockID>, path: *const BlockID, path_size: c_int, entry_fun_id: FunID) -> *const c_char {
   let reducer = reducer.as_ref().expect("bad pointer");
   let path = slice::from_raw_parts(path, path_size as usize);
   let reduced_path = reducer.reduce(path, entry_fun_id);
   let hash = hash_path(&reduced_path);
   let c_string = CString::new(hash).unwrap();
   c_string.as_ptr()
}
