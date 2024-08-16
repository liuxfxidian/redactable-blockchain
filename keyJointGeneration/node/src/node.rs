use std::collections::HashMap;


use curv::elliptic::curves::{Secp256k1, Point, Scalar};
use serde::{Deserialize, Serialize};


use message::params::{ThreasholdParam, CLKeypair};
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
    pub cl_keypair:CLKeypair,
    pub secret:Scalar<Secp256k1>,
    pub random:Scalar<Secp256k1>,
    pub commit:Point<Secp256k1>,
    pub commit_map: HashMap<u16, Point<Secp256k1>>,
    pub commitments: Vec<Point<Secp256k1>>,
    pub secret_coefficients: Vec<Scalar<Secp256k1>>,
    pub random_coefficients: Vec<Scalar<Secp256k1>>,
    pub sk:Scalar<Secp256k1>,
    pub node_info_vec: Option<Vec<NodeInfo>>,
}