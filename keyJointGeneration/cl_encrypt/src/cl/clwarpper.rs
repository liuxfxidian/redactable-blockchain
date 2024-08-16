extern crate libc;

use libc::c_char;
use std::ffi::{CString, CStr};
use encoding::{DecoderTrap, Encoding};
use encoding::all::GBK;

static HEX_TABLE :[char;16] = ['0','1','2','3','4','5','6','7','8','9',
                                        'A','B','C','D','E','F'];

#[link(name = "encrypt")]
extern "C" {
    pub fn public_key_gen_cpp(sk_str: *const c_char) -> *const c_char;
    pub fn encrypt_cpp(pk_str: *const c_char, message: *const c_char, random: *const c_char) -> *const c_char;
    pub fn decrypt_cpp(sk_str: *const c_char, cipher_str: *const c_char) -> *const c_char;
    pub fn add_ciphertexts_cpp(cipher_str_first: *const c_char, cipher_str_second: *const c_char) -> *const c_char;
    pub fn scal_ciphertexts_cpp(cipher_str: *const c_char, m_str: *const c_char) -> *const c_char;    
    pub fn batch_cl_pc_proof_cpp(PC_str: *const c_char, enc_s_str: *const c_char, enc_r_str: *const c_char, H_str: *const c_char, secret_r_str: *const c_char, random_r_str: *const c_char, st_str: *const c_char, rt_str: *const c_char, t_total_str: *const c_char, t_vec_str: *const c_char, pk_vec_str: *const c_char) -> *const c_char;
    pub fn batch_cl_pc_verify_cpp(proof_str: *const c_char, PC_str: *const c_char, enc_s_str: *const c_char, enc_r_str: *const c_char, H_str: *const c_char,  t_total_str: *const c_char, t_vec_str: *const c_char, pk_vec_str: *const c_char) -> *const c_char;
}

pub fn c_char_decode(input: *const i8) -> String {
    unsafe{
        let mut msgstr: String = "".to_string();
        if input != (0 as *mut c_char) {
            let errcstr = CStr::from_ptr(input);
            let errcstr_tostr = errcstr.to_str();
            //这里要处理下编码，rust默认是UTF-8,如果不ok，那就是其他字符集
            if errcstr_tostr.is_ok() {
                msgstr = errcstr_tostr.unwrap().to_string();
            } else {
                //强行尝试对CStr对象进行GBK解码,采用replace策略
                //todo: 如果在使用其他编码的平台上依旧有可能失败，得到空消息，但不会抛异常了
                let alter_msg = GBK.decode(errcstr.to_bytes(), DecoderTrap::Replace);
                // let alter_msg = encoding::all::UTF_8.decode(errcstr.to_bytes(),DecoderTrap::Replace);
                if alter_msg.is_ok() {
                    msgstr = alter_msg.unwrap();
                }
            }
        }
        return msgstr;
    }
    
}

pub fn public_key_gen(sk: String) -> String {
    let sk_str = CString::new(sk).unwrap();
    unsafe{
        return c_char_decode(public_key_gen_cpp(sk_str.as_ptr()));
    }
}

pub fn encrypt(pk: String, message: String, random: String) -> String{
    let pk_str = CString::new(pk).unwrap();
    let m_str = CString::new(message).unwrap();
    let r_str = CString::new(random).unwrap();
    unsafe{
        return c_char_decode(encrypt_cpp(pk_str.as_ptr(), m_str.as_ptr(), r_str.as_ptr()));
    }
}

pub fn decrypt(sk: String, cipher: String) -> String{
    let sk_str = CString::new(sk).unwrap();
    let c_str = CString::new(cipher).unwrap();
    unsafe{
        return c_char_decode(decrypt_cpp(sk_str.as_ptr(), c_str.as_ptr()));
    }
}

pub fn add_ciphertexts(cipher_first: String, cipher_second: String) -> String{
    let c_first_str = CString::new(cipher_first).unwrap();
    let c_second_str = CString::new(cipher_second).unwrap();
    unsafe{
        return c_char_decode(add_ciphertexts_cpp(c_first_str.as_ptr(), c_second_str.as_ptr()));
    }
}

pub fn scal_ciphertexts(cipher: String, message: String) -> String{
    let c_str = CString::new(cipher).unwrap();
    let m_str = CString::new(message).unwrap();
    unsafe{
        return c_char_decode(scal_ciphertexts_cpp(c_str.as_ptr(), m_str.as_ptr()));
    }
}

pub fn batch_cl_pc_proof(pc: String, enc_s: String, enc_r: String, h: String, secret_r: String, random_r: String, st: String, rt: String, t_total: String, t_vec: String, pk_vec: String) -> String {
    let pc_str = CString::new(pc).unwrap();
    let enc_s_str =  CString::new(enc_s).unwrap();
    let enc_r_str =  CString::new(enc_r).unwrap();
    let h_str =  CString::new(h).unwrap();
    let secret_r_str =  CString::new(secret_r).unwrap();
    let random_r_str =  CString::new(random_r).unwrap();
    let st_str = CString::new(st).unwrap();
    let rt_str = CString::new(rt).unwrap();
    let t_total_str = CString::new(t_total).unwrap();
    let t_vec_str = CString::new(t_vec).unwrap();
    let pk_vec_str = CString::new(pk_vec).unwrap();
    unsafe{
        return c_char_decode(batch_cl_pc_proof_cpp(pc_str.as_ptr(), enc_s_str.as_ptr(), enc_r_str.as_ptr(), h_str.as_ptr(), secret_r_str.as_ptr(), random_r_str.as_ptr(), st_str.as_ptr(), rt_str.as_ptr(), t_total_str.as_ptr(), t_vec_str.as_ptr(), pk_vec_str.as_ptr()));
    }
}

pub fn batch_cl_pc_verify(proof: String, pc: String, enc_s: String, enc_r: String, h: String, t_total: String, t_vec: String, pk_vec: String) -> String {
    let proof_str = CString::new(proof).unwrap();
    let pc_str = CString::new(pc).unwrap();
    let enc_s_str =  CString::new(enc_s).unwrap();
    let enc_r_str =  CString::new(enc_r).unwrap();
    let h_str =  CString::new(h).unwrap();
    let t_total_str = CString::new(t_total).unwrap();
    let t_vec_str = CString::new(t_vec).unwrap();
    let pk_vec_str = CString::new(pk_vec).unwrap();
    unsafe{
        return c_char_decode(batch_cl_pc_verify_cpp(proof_str.as_ptr(), pc_str.as_ptr(), enc_s_str.as_ptr(), enc_r_str.as_ptr(), h_str.as_ptr(), t_total_str.as_ptr(), t_vec_str.as_ptr(), pk_vec_str.as_ptr()));
    }
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
