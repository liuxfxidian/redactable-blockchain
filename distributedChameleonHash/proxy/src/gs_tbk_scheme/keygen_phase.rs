use message::node::keygen_msg::NodeKeyGenPhaseOneBroadcastMsg;
use message::proxy::keygen_msg::ProxyToNodeGetSTotal;

use curv::elliptic::curves::{Secp256k1, Point, Scalar};
pub type FE = Scalar<Secp256k1>;
use curv::BigInt;
use log::info;
use sha2::{Digest, Sha256};
use std::fs::File;
use std::io::{BufRead, BufReader};
use crate::Error::{self};
use crate::proxy::Proxy;

static HEX_TABLE :[char;16] = ['0','1','2','3','4','5','6','7','8','9',
'A','B','C','D','E','F'];
impl Proxy
{   /// 聚合s,并通过陷门来编辑
    pub fn keygen_get_s(&mut self, msg_vec:&Vec<NodeKeyGenPhaseOneBroadcastMsg>)
    -> Result<ProxyToNodeGetSTotal, Error>
    {


        let key_path = "/home/arlo/CH/pk_share/pk_share".to_string() + "1" + ".txt";
        let key_file = File::open(key_path.clone()).unwrap();
        let key_reader = BufReader::new(key_file);
        let mut key_lines = key_reader.lines().map(|l| l.unwrap());
        let key_commitment_vec: Vec<Point<Secp256k1>> = serde_json::from_str(&key_lines.next().unwrap()).unwrap();
        
        let random_path = "/home/arlo/CH/pk_share/pk_share".to_string() + "1" + ".txt";
        let random_file = File::open(random_path).unwrap();
        let random_reader = BufReader::new(random_file);
        let mut random_lines = random_reader.lines().map(|l| l.unwrap());
        let random_commitment_vec: Vec<Point<Secp256k1>> = serde_json::from_str(&random_lines.next().unwrap()).unwrap();
        let mut flag = true;

        for msg in msg_vec{
            let s_new_g = random_commitment_vec[msg.sender as usize - 1].clone() - FE::from_bigint(&msg.h_new) * key_commitment_vec[msg.sender as usize -1].clone();
            let s_new_g_ = FE::from_bigint(&msg.s_new) * Point::generator();
            if s_new_g != s_new_g_ {flag = flag && false;} 
        }

        if flag {
            let mut lagrange_vec = Vec::new();
            for i in 0 ..= self.threashold_param.threshold as usize
            {
                lagrange_vec.push(BigInt::from(msg_vec.get(i).unwrap().sender));
            }
             
            let mut s_new_total = FE::zero();
            for i in 0 ..= self.threashold_param.threshold as usize
            {
                let msg = msg_vec.get(i).unwrap();
                let li = Self::map_share_to_new_params(BigInt::from(msg.sender), &lagrange_vec);
                s_new_total = s_new_total + FE::from_bigint(&msg.s_new) * li.clone();
    
            }
            info!("s_new_tatal is {:?}", s_new_total);
    

            info!("CH adapt is finished");
            Ok
            (
                ProxyToNodeGetSTotal
                {
                    s_total: s_new_total.clone().to_bigint()
                }
            ) 
        }else {
            Err(Error::InvalidCom)
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

