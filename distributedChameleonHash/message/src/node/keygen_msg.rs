use curv::{elliptic::curves::{Scalar, Secp256k1}, BigInt};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NodeKeyGenPhaseOneBroadcastMsg
{
    pub sender:u16,
    pub role:String,
    pub s_new:BigInt,
    pub h_new:BigInt
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EncAndProof
{
    pub share_enc:String,
    pub share_proof:String
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NodeToProxyKeyGenPhaseTwoP2PMsg
{//p to p
    pub sender:u16,
    pub role:String,
}