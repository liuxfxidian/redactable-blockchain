use std::{fs::File, io::{BufRead, BufReader, Write}};

use curv::{arithmetic::Converter, elliptic::curves::{Point, Scalar, Secp256k1}, BigInt};
use sha2::{Digest, Sha256};

pub type CU = Secp256k1;
pub type FE = Scalar<Secp256k1>;
pub type GE = Point<Secp256k1>;

static HEX_TABLE :[char;16] = ['0','1','2','3','4','5','6','7','8','9',
                                        'A','B','C','D','E','F'];

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

pub fn sha256(input: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(input.as_bytes());
    let result = hasher.finalize();
    // 将哈希值转换为16进制字符串
    result.to_vec().iter()
        .map(|b| format!("{:02x}", b))
        .collect::<String>()
}

pub fn generate_hash(pk:GE, R:GE, s:FE, message:String) -> GE {
    let G = Point::generator();
    let hash_str = sha256(&(message + &to_hex(R.to_bytes(true).as_ref())));
    let hash = FE::from_bigint(&BigInt::from_hex(&hash_str).unwrap());
    let H = R - (hash * pk + s * G);
    return H;
}

pub fn adapt(sk:FE, new_message:String, hash:GE) -> (GE, FE){
    let G = Point::generator();
    let k = FE::random();
    let R =  hash + k.clone() * G;
    let message = new_message + &to_hex(R.to_bytes(true).as_ref());
    let hash_str = sha256(&message.as_str());
    let hash_new = FE::from_bigint(&BigInt::from_hex(&hash_str).unwrap());
    let s = k - hash_new * sk;
    return (R, s)
}

pub fn verify(pk:GE, R:GE, s:FE, message:String, H:GE) -> bool{
    let G = Point::generator();
    let hash_str = sha256(&(message + &to_hex(R.to_bytes(true).as_ref())));
    let hash = FE::from_bigint(&BigInt::from_hex(&hash_str).unwrap());
    let H_ = R - (hash * pk + s * G);
    return H == H_;
}

#[test]
fn get_hash(){
    let path = "src/node/node1/keypair.txt";
    let file = File::open(path).unwrap();
    let reader = BufReader::new(file);
    let mut lines = reader.lines().map(|l| l.unwrap());
    let pk:Point<Secp256k1> = serde_json::from_str(&lines.next().unwrap()).unwrap();
    let G = Point::generator();
    let r = FE::random();
    let R = r * G;
    let s = FE::random();
    let H = generate_hash(pk, R.clone(), s.clone(), "12345".to_string());
    println!("H is {:?}", H );
    let ch_str = serde_json::to_string(&H).unwrap() + "\n" + &serde_json::to_string(&R).unwrap() + "\n" + &s.to_bigint().to_hex();

    let path = "src/node/ch.txt";
    let file = File::create(path);
    match file.unwrap().write_all(ch_str.as_bytes()){
        Ok(_) => println!("ch write in ch.txt"),
        Err(err) => eprintln!("write ch error: {}", err),
    }
    // generate_hash(pk, R, s, message)
}

#[test]
fn test(){
    let G = Point::generator();
    let r = FE::random();
    let R = r * G;
    let s = FE::random();
    let sk = FE::random();
    let pk = GE::generator() * sk.clone();

    let hash = generate_hash(pk.clone(), R.clone(), s.clone(), "123".to_string());

    let (R_new, s_new) = adapt(sk.clone(), "1234".to_string(), hash.clone());
    
    let ret = verify(pk.clone(), R, s, "123".to_string(), hash.clone());
    println!("ret: {}", ret);

    let ret_ = verify(pk.clone(), R_new, s_new, "1234".to_string(), hash);
    println!("ret_: {}", ret_);
    
}
