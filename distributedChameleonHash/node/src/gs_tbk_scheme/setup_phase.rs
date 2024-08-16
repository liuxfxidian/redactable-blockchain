use log::info;
use curv::elliptic::curves::{Point, Scalar, Secp256k1};
use crate::config::config::Config;
use crate::node::Node;
use message::proxy::setup_msg::ProxySetupPhaseBroadcastMsg;
use message::node::setup_msg::{NodeToProxySetupPhaseP2PMsg,NodeSetupPhaseFinishFlag};

pub type FE = Scalar<Secp256k1>;

impl Node{
    /// 初始化自身信息，加载配置，生成cl密钥对等
    pub fn init(gs_tbk_config:Config) -> Self
    {
        Self
        { 
            id:None,
            role:"Group Manager Node".to_string(),
            address:gs_tbk_config.listen_addr.clone(),
            listen_addr:gs_tbk_config.listen_addr,
            proxy_address:gs_tbk_config.proxy_addr,
            threashold_param:gs_tbk_config.threshold_params,
            pk:Point::zero(),
            r_new:Point::zero(),
            h_new:Scalar::zero(),
            node_info_vec:None
        }
        
    }

    /// 发送自己的公钥和地址给代理
    pub fn setup_phase_one(&self)->NodeToProxySetupPhaseP2PMsg
    {
        info!("Setup phase is starting!");
        NodeToProxySetupPhaseP2PMsg
        {
            role:self.role.clone(),
            address:self.address.clone(),
           
        }

    }

    /// 存储所有管理员的基本信息，公钥，id，地址等等
    pub fn setup_phase_two(&mut self, msg:ProxySetupPhaseBroadcastMsg)-> NodeSetupPhaseFinishFlag
    {
        for node in msg.node_info_vec.iter()
        {
            if node.address == self.address
            {
                self.id = Some(node.id);
            }
        }
        self.node_info_vec = Some(msg.node_info_vec);
        NodeSetupPhaseFinishFlag 
        { 
            sender: self.id.unwrap(), 
            role:self.role.clone(),
        }
    }
}