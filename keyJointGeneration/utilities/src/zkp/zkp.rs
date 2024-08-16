use curv::{arithmetic::Converter, BigInt};
use serde::{Deserialize, Serialize};

use crate::{code::code::{sha256, to_hex}, FE, GE};
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DlPcProof {
    pub z: BigInt,
    pub e: BigInt
}

impl DlPcProof {
    pub fn dl_pc_proof(pc: GE, dl: GE, bt: FE) -> Self{
        let s = FE::random();
        let s_h = s.clone() * GE::base_point2();
        let e_bn = BigInt::from_hex(&sha256(&(to_hex(pc.to_bytes(true).as_ref()) + &to_hex(dl.to_bytes(true).as_ref()).clone() + &to_hex(s_h.to_bytes(true).as_ref()).clone()))).unwrap();
        let e = FE::from_bigint(&e_bn);
        let z = s.clone() + e.clone() * bt;

        DlPcProof{
            z: z.to_bigint(),
            e: e.to_bigint()
        }
    }
    pub fn dl_pc_verify(&self, pc: GE, dl: GE) -> bool{
        let r = pc.clone() - dl.clone();
        let s_h = FE::from_bigint(&self.z) * GE::base_point2() - r * FE::from_bigint(&self.e);
        let e_bn = BigInt::from_hex(&sha256(&(to_hex(pc.to_bytes(true).as_ref()) + &to_hex(dl.to_bytes(true).as_ref()).clone() + &to_hex(s_h.to_bytes(true).as_ref()).clone()))).unwrap();
        return self.e == e_bn;
    }
}
