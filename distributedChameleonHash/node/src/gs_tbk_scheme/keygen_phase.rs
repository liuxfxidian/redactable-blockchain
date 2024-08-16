use std::fs::File;
use std::io::{BufRead, BufReader, Write};
use curv::arithmetic::traits::*;
use curv::elliptic::curves::{Secp256k1, Point, Scalar};
pub type FE = Scalar<Secp256k1>;
use curv::BigInt;
use log::info;
use sha2::{Digest, Sha256};
use crate::Error::{self};
use message::node::keygen_msg::{NodeKeyGenPhaseOneBroadcastMsg,NodeToProxyKeyGenPhaseTwoP2PMsg};
use message::proxy::keygen_msg::ProxyToNodeGetSTotal;
use crate::node::Node;


static HEX_TABLE :[char;16] = ['0','1','2','3','4','5','6','7','8','9',
'A','B','C','D','E','F'];

impl Node { 
    pub fn keygen_phase_one(&mut self) -> NodeKeyGenPhaseOneBroadcastMsg
    {
        info!("Adapt is starting!");
        let key_path = "/home/arlo/CH/keypair/keypair".to_string() + &self.id.unwrap().to_string() + ".txt";
        let key_file = File::open(key_path).unwrap();
        let key_reader = BufReader::new(key_file);
        let mut key_lines = key_reader.lines().map(|l| l.unwrap());

        let random_path = "/home/arlo/CH/keypair/keypair".to_string() + &self.id.unwrap().to_string() + ".txt";
        let random_file = File::open(random_path).unwrap();
        let random_reader = BufReader::new(random_file);
        let mut random_lines = random_reader.lines().map(|l| l.unwrap());

        let ch_path = "src/node/ch.txt";
        let ch_file = File::open(ch_path).unwrap();
        let ch_reader = BufReader::new(ch_file);
        let mut ch_lines = ch_reader.lines().map(|l| l.unwrap());

        self.pk = serde_json::from_str(&key_lines.next().unwrap()).unwrap();
        let K:Point<Secp256k1> = serde_json::from_str(&random_lines.next().unwrap()).unwrap();

        let sk = FE::from_bigint(&BigInt::from_hex(&key_lines.next().unwrap()).unwrap());
        info!("sk is {:?}", sk);
        let k = FE::from_bigint(&BigInt::from_hex(&random_lines.next().unwrap()).unwrap());
        
        let H:Point<Secp256k1> = serde_json::from_str(&ch_lines.next().unwrap()).unwrap();
        let R:Point<Secp256k1> = serde_json::from_str(&ch_lines.next().unwrap()).unwrap();
        let s = FE::from_bigint(&BigInt::from_hex(&ch_lines.next().unwrap()).unwrap());
        info!("H is {:?}", H);
        info!("R is {:?}", R);
        info!("s is {:?}", s);

        self.r_new = H + K;
        let message = "123".to_string() + &Self::to_hex(self.r_new.to_bytes(true).as_ref());
        let hash_str = Self::sha256(&message.as_str());
        let hash_new = FE::from_bigint(&BigInt::from_hex(&hash_str).unwrap());
        let s_new = k - hash_new.clone() * sk;
        self.h_new = hash_new;

        NodeKeyGenPhaseOneBroadcastMsg
        {
            sender:self.id.unwrap(),
            role:self.role.clone(),
            s_new:s_new.to_bigint(),
            h_new:self.h_new.clone().to_bigint()
        }
    }

    pub fn keygen_phase_two(&mut self, msg:ProxyToNodeGetSTotal)
    {
            let s_total = FE::from_bigint(&msg.s_total.clone());
            let G = Point::generator();
            let hash_str = Self::sha256(&("123".to_string() + &Self::to_hex(self.r_new.to_bytes(true).as_ref())));
            let hash = FE::from_bigint(&BigInt::from_hex(&hash_str).unwrap());
            let H_ = self.r_new.clone() - (hash * self.pk.clone() + s_total.clone() * G);
            info!("H_new is {:?}", H_);
    
            let ch_str = serde_json::to_string(&H_).unwrap() + "\n" + &serde_json::to_string(&self.r_new).unwrap() + "\n" + &msg.s_total.to_hex();
    
            let path = "src/node/ch_new.txt";
            let file = File::create(path);
            match file.unwrap().write_all(ch_str.as_bytes()){
                Ok(_) => println!("ch write in ch_new.txt"),
                Err(err) => eprintln!("write ch_new error: {}", err),
            }

            info!("CH adapt is finished");
             
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

    pub fn sha256(input: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(input.as_bytes());
        let result = hasher.finalize();
        // 将哈希值转换为16进制字符串
        result.to_vec().iter()
        .map(|b| format!("{:02x}", b))
        .collect::<String>()
    }

    pub fn map_share_to_new_params(index: BigInt, s: &[BigInt]) -> FE {
        let s_len = s.len();
        // add one to indices to get points
        let points: Vec<FE> = s.iter().map(|i| Scalar::from(i)).collect();
    
        let xi: FE = Scalar::from(&index);
        let num: FE = Scalar::from(&BigInt::from(1));
        let denum: FE = Scalar::from(&BigInt::from(1));
        let num = (0..s_len).fold(
            num,
            |acc, i| {
                if s[i] != index {
                    acc * &points[i]
                } else {
                    acc
                }
            },
        );
        let denum = (0..s_len).fold(denum, |acc, i| {
            if s[i] != index {
                let xj_sub_xi = &points[i] - &xi;
                acc * xj_sub_xi
            } else {
                acc
            }
        });
        let denum = denum.invert().unwrap();
        num * denum
    }
}

