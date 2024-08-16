use std::collections::HashMap;
use curv::elliptic::curves::{Point, Secp256k1};
use serde::{Deserialize, Serialize};
use utilities::{vss::vss::Vss, zkp::zkp::DlPcProof};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NodeKeyGenPhaseOneBroadcastMsg
{
    pub sender:u16,
    pub role:String,
    pub commit:Point<Secp256k1>,
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
    pub share_enc_map:HashMap<u16, String>,
    pub random_enc_map:HashMap<u16, String>,
    pub enc_proof:String,
    pub vss:Vss
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NodeToProxyKeyGenPhaseFourP2PMsg
{//p to p
    pub sender:u16,
    pub role:String,
    pub s_coefficients_log: Vec<Point<Secp256k1>>,
    pub dlog_proof:DlPcProof
}