/// # Safety
/// This function is unsafe and should be used with caution
pub unsafe fn as_mut_ptr<T>(reference: &T) -> *mut T {
    reference as *const T as *mut T
}
