use sha2::{Digest, Sha256};

static HEX_TABLE :[char;16] = ['0','1','2','3','4','5','6','7','8','9',
'A','B','C','D','E','F'];

pub fn sha256(input: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(input.as_bytes());
    let result = hasher.finalize();
    // 将哈希值转换为16进制字符串
    result.to_vec().iter()
    .map(|b| format!("{:02x}", b))
    .collect::<String>()
}

pub fn to_hex(data : impl AsRef<[u8]>) -> String {
    let data = data.as_ref();
    let len = data.len();
    let mut res = String::with_capacity(len * 2);

    for i in 0..len {
    res.push(HEX_TABLE[usize::from(data[i] >> 4)] );
    res.push(HEX_TABLE[usize::from(data[i] & 0x0F)]);
    }
    res
}
