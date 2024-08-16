use tokio::net::TcpListener;
use tokio::sync::Mutex as TokioMutex;
use tokio_util::codec::{Framed, LinesCodec};
use std::net::SocketAddr;
use std::sync::Arc;
use std::env;
use log::{error, info};


use node::communication::communication::*;
use node::node::Node;
use node::config::config::Config;
use message::common_msg::GSTBKMsg;
use message::node::keygen_msg::NodeKeyGenPhaseOneBroadcastMsg;
use message::node::common_msg::{SetupMsg, KeyGenMsg};

#[tokio::main]
pub async fn main() -> Result<(), anyhow::Error> 
{
    let log_path = String::from(env::current_dir().unwrap().as_path().to_str().unwrap())+"/src/node/node1/config/config_file/log4rs.yaml";
    log4rs::init_file(log_path, Default::default()).unwrap();
    
    let gs_tbk_config_path  = String::from(std::env::current_dir().unwrap().as_path().to_str().unwrap())+"/src/node/node1/config/config_file/node_config.json";
    let gs_tbk_config:Config = serde_json::from_str(&Config::load_config(&gs_tbk_config_path)).unwrap();

    let node = Node::init(gs_tbk_config);
    let shared_node = Arc::new(TokioMutex::new(node.clone()));

    let shared_keygen_phase_one_msg_vec = Arc::new(TokioMutex::new(Vec::<NodeKeyGenPhaseOneBroadcastMsg>::new()));

    let node_addr:SocketAddr = node.listen_addr.parse()?;
    let listener = TcpListener::bind(node_addr).await?;
    info!("node1 is listening on {}",node.address);

    let node_setup_msg_str = serde_json::to_string(&message::common_msg::GSTBKMsg::GSTBKMsgN(message::node::common_msg::GSTBKMsg::SetupMsg(SetupMsg::NodeToProxySetupPhaseP2PMsg(node.setup_phase_one())))).unwrap();
    match p2p(node_setup_msg_str, node.proxy_address).await
    {
        Ok(_) => {}
        Err(e) => 
        {
            error!("node setup msg can not sent Err:{}",e);
        }
    };

    while let Result::Ok(( tcp_stream,_)) = listener.accept().await
    {
        let node_clone = shared_node.clone();
        
        let keygen_phase_one_msg_vec_clone = shared_keygen_phase_one_msg_vec.clone();

        tokio::spawn(async move
            {
            let node = node_clone.clone();

            let keygen_phase_one_msg_vec = keygen_phase_one_msg_vec_clone.clone();

            let framed = Framed::new( tcp_stream,LinesCodec::new());
            let message = match get_message(framed).await
            {
                Ok(v) => v,
                Err(e) => 
                {
                    error!("Failed to get nodemessage: {:?}",e);
                    return ;
                }
            }; 
            match message 
            {
                GSTBKMsg::GSTBKMsgP(gstbk_proxy_msg) => 
                {
                    match gstbk_proxy_msg
                    {
                        message::proxy::common_msg::GSTBKMsg::SetupMsg(setup_msg) => 
                        {
                            match setup_msg 
                            {
                                message::proxy::common_msg::SetupMsg::ProxySetupPhaseBroadcastMsg(msg) => 
                                {
                                    info!("From id : 0 ,Role : Proxy  Get ProxySetupPhaseBroadcastMsg");
                                    let mut locked_node = node.lock().await;
                                    let setup_phase_two_msg_str = setup_to_gstbk(SetupMsg::NodeSetupPhaseFinishFlag(locked_node.setup_phase_two(msg)));
                                    match p2p(setup_phase_two_msg_str, (*locked_node.proxy_address).to_string()).await 
                                    {
                                        Ok(_) => {}
                                        Err(e) => 
                                        {
                                            error!("Error: {}, NodeToProxySetupFinishMsg can not sent ",e);
                                            return ;
                                        }
                                    };
                                }
                                message::proxy::common_msg::SetupMsg::ProxySetupPhaseFinishFlag(_msg) => 
                                {
                                    info!("From id : 0 ,Role : Proxy  Get ProxySetupPhaseFinishFlag");
                                    info!("Keygen phase is staring!");

                                    let mut locked_node = node.lock().await;
                                    let mut locked_vec = keygen_phase_one_msg_vec.lock().await;

                                    let keygen_phase_one_msg = locked_node.keygen_phase_one();
                                    locked_vec.push(keygen_phase_one_msg.clone());

                                    let keygen_phase_one_msg_str = keygen_to_gstbk(KeyGenMsg::NodeKeyGenPhaseOneBroadcastMsg(keygen_phase_one_msg));

                                    let mut msg_vec:Vec<String> = Vec::new();
                                    msg_vec.push(keygen_phase_one_msg_str);
                                    let node_list = locked_node.node_info_vec.clone().unwrap();

                                    let node_id = locked_node.id.clone().unwrap();

                                    for msg in msg_vec
                                    {
                                        match broadcast(msg, node_list.clone(),node_id.clone()).await
                                        {
                                            Ok(_) => {}
                                            Err(e) => 
                                            {
                                                error!("Error: {}, NodeKeyGenPhaseOneBroadcastMsg can not sent ",e);
                                                return ;
                                            }
                                        };
                                    }
                                }
                            }
                        }
                        message::proxy::common_msg::GSTBKMsg::KeyGenMsg(keygen_msg) => 
                        {
                            match keygen_msg  
                            {
                                message::proxy::common_msg::KeyGenMsg::ProxyToNodeKeyGenPhaseThreeP2PMsg(msg) => 
                                {
                                    info!("From Proxy Node Get ProxyToNodeKeyGenPhaseThreeP2PMsg");
                                    let mut locked_node = node.lock().await;

                                    let keygen_phase_four_msg = match locked_node.keygen_phase_four(msg) 
                                    {
                                        Ok(v) => v,
                                        Err(e) => 
                                        {
                                            error!("Error:{}, can not get NodeToProxyKeyGenPhaseFourP2PMsg ",e);
                                            return ;
                                        }
                                    };
                                    let keygen_phase_four_msg_str = serde_json::to_string(&message::common_msg::GSTBKMsg::GSTBKMsgN(message::node::common_msg::GSTBKMsg::KeyGenMsg(message::node::common_msg::KeyGenMsg::NodeToProxyKeyGenPhaseFourP2PMsg(keygen_phase_four_msg)))).unwrap();
                                    match p2p(keygen_phase_four_msg_str, (*locked_node.proxy_address).to_string()).await 
                                    {
                                        Ok(_) => {}
                                        Err(e) => 
                                        {
                                            error!("Error:{}, NodeToProxyKeyGenPhaseTwoP2PMsg can not sent",e);
                                            return ;
                                        }
                                    };
                                }
                                message::proxy::common_msg::KeyGenMsg::ProxyToNodeKeyGenPhaseFiveP2PMsg(msg) => 
                                {
                                    info!("From Proxy Node Get ProxyToNodeKeyGenPhaseFiveP2PMsg");
                                    let mut locked_node = node.lock().await;

                                    match locked_node.keygen_phase_six(msg) 
                                    {
                                        Ok(v) => v,
                                        Err(e) => 
                                        {
                                            error!("Error:{}, can not get NodeToProxyKeyGenPhaseSixP2PMsg ",e);
                                            return ;
                                        }
                                    };
                                }
                            }
                        }
                    }
                }
                GSTBKMsg::GSTBKMsgN(gstbk_node_msg) => 
                {
                    match gstbk_node_msg
                    {
                        message::node::common_msg::GSTBKMsg::KeyGenMsg(keygen_msg) => 
                        {
                            match keygen_msg 
                            {
                                message::node::common_msg::KeyGenMsg::NodeKeyGenPhaseOneBroadcastMsg(msg) => 
                                {
                                    info!("From id : {} ,Role : {} Get NodeKeyGenPhaseOneBroadcastMsg ",msg.sender,msg.role);
                                    let mut locked_node = node.lock().await;
                                    let mut locked_vec = keygen_phase_one_msg_vec.lock().await;
                                    locked_vec.push(msg);
                                    if locked_vec.len() == locked_node.threashold_param.share_counts as usize  
                                    {
                                        let vec = (*locked_vec).clone();
                                        let keygen_phase_two_msg = match locked_node.keygen_phase_two(&vec) 
                                        {
                                            Ok(v) => v,
                                            Err(e) => 
                                            {
                                                error!("Error:{}, can not get NodeToProxyKeyGenPhaseTwoP2PMsg ",e);
                                                return ;
                                            }
                                        };
                                        let keygen_phase_two_msg_str = serde_json::to_string(&message::common_msg::GSTBKMsg::GSTBKMsgN(message::node::common_msg::GSTBKMsg::KeyGenMsg(message::node::common_msg::KeyGenMsg::NodeToProxyKeyGenPhaseTwoP2PMsg(keygen_phase_two_msg)))).unwrap();
                                        match p2p(keygen_phase_two_msg_str, (*locked_node.proxy_address).to_string()).await 
                                        {
                                            Ok(_) => {}
                                            Err(e) => 
                                            {
                                                error!("Error:{}, NodeToProxyKeyGenPhaseTwoP2PMsg can not sent",e);
                                                return ;
                                            }
                                        };
                                    }
                                }
                                _ => 
                                {}   
                            }
                        }
                        _ => {}
                    }
                }
            }
        });
    }
    Ok(())
}

#[test]
fn test() 
{
    match main() 
    {
        Ok(_) => 
        {
            println!("Ok");
        }
        Err(_) => 
        {
            println!("No");
        } 
    };
}