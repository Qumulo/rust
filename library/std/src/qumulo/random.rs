use crate::qumulo::bindings::rust_sys_random_bytes;

pub fn fill_bytes(bytes: &mut [u8]) {
    unsafe { rust_sys_random_bytes(bytes.as_mut_ptr(), bytes.len() as u64) }
}

pub fn hashmap_random_keys() -> (u64, u64) {
    let mut bytes = [0; 16];
    fill_bytes(&mut bytes);
    let k1 = u64::from_ne_bytes(bytes[..8].try_into().unwrap());
    let k2 = u64::from_ne_bytes(bytes[8..].try_into().unwrap());
    (k1, k2)
}
