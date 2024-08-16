use curv::elliptic::curves::{Secp256k1, Point};
use serde::{Deserialize, Serialize};
use utilities::vss::vss::Vss;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ProxyKeyGenPhaseStartFlag
{
    pub sender:u16,
    pub role:String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ProxyKeyGenPhaseOneBroadcastMsg{
    pub g:Point<Secp256k1>,
    pub participants:Vec<u16>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ProxyToNodeKeyGenPhaseThreeP2PMsg
{
    pub sender:u16,
    pub role:String,
    pub share_enc_sum:String,
    pub random_enc_sum:String,
    pub vss_scheme_sum:Vss,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ProxyToNodeKeyGenPhaseFiveP2PMsg
{
    pub total_s_coefficients_log:Vec<Point<Secp256k1>>
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ProxyToNodesKeyGenPhasefiveBroadcastMsg
{
    pub sender:u16,
    pub role:String,
}