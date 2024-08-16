use curv::elliptic::curves::{Point, Scalar, Secp256k1};
use serde::{Deserialize, Serialize};


use message::params::ThreasholdParam;
use message::proxy::setup_msg::NodeInfo;

 
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Node
{
    pub id: Option<u16>,
    pub role:String,
    pub address: String, 
    pub listen_addr:String,
    pub proxy_address: String,
    pub threashold_param: ThreasholdParam,
    pub pk:Point<Secp256k1>,
    pub r_new:Point<Secp256k1>,
    pub h_new:Scalar<Secp256k1>,
    pub node_info_vec: Option<Vec<NodeInfo>>,
}