use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use curv::arithmetic::Converter;
use curv::elliptic::curves::{Secp256k1, Point, Scalar};
use curv::cryptographic_primitives::secret_sharing::feldman_vss::ShamirSecretSharing;
use cl_encrypt::cl::clwarpper::*;
use curv::BigInt;
use log::info;
use message::proxy::keygen_msg::{ProxyToNodeKeyGenPhaseThreeP2PMsg, ProxyToNodeKeyGenPhaseFiveP2PMsg};
use message::node::keygen_msg::{NodeToProxyKeyGenPhaseTwoP2PMsg, NodeToProxyKeyGenPhaseFourP2PMsg};
use num::Zero;
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use utilities::code::code::sha256;
use utilities::vss::vss::{self, map_share_to_new_params, Vss};
use utilities::{FE, GE};
use crate::proxy::Proxy;
use crate::Error::{self};

impl Proxy
{   /// 验证 CLDLProof 然后合并系数承诺和share碎片
    pub fn keygen_phase_three(&mut self,msg_vec:Vec<NodeToProxyKeyGenPhaseTwoP2PMsg>) -> Result<HashMap<u16, ProxyToNodeKeyGenPhaseThreeP2PMsg>,Error>
    {
        info!("proxy keygen phase three start");
        assert_eq!(msg_vec.len(), self.threashold_param.share_counts as usize);
        // Verify CLDLProof
        let all_verify_flag = Arc::new(Mutex::new(true));
        let all_verify_flag_clone = Arc::clone(&all_verify_flag);
        let commitments_map: Arc<Mutex<HashMap<u16, Vec<Point<Secp256k1>>>>> = Arc::new(Mutex::new(HashMap::new()));

        (msg_vec.clone()).into_par_iter().for_each(|msg| {
            let share_enc_map = msg.share_enc_map;
            let random_enc_map = msg.random_enc_map;
            let vss = msg.vss;
            let mut map = commitments_map.lock().unwrap();
            map.insert(msg.sender, vss.commitments.clone());
            let enc_proof = msg.enc_proof;

            let mut pc = GE::zero();
            let mut t_vec = String::new();
            let mut pk_vec = String::new();
            let mut t_total = BigInt::zero();

            let mut s_enc_t_vec = Vec::new();
            let mut r_enc_t_vec = Vec::new();
            for node in self.node_info_vec.as_ref().unwrap(){
                let share_enc = share_enc_map.get(&node.id).unwrap();
                let random_enc = random_enc_map.get(&node.id).unwrap();
                
                let t_bn = BigInt::from_hex(&sha256(&(share_enc.clone() + &random_enc.clone()))).unwrap();
                t_vec = t_vec + &t_bn.to_string() + ":";
                t_total = t_total + t_bn.clone();

                let commit_t = vss.pedersen_get_point_commitment(node.id) * FE::from_bigint(&t_bn);
                pc = pc + commit_t;
                pk_vec = pk_vec + &node.cl_pk + ";";

                s_enc_t_vec.push(scal_ciphertexts(share_enc.clone(), t_bn.to_string()));
                r_enc_t_vec.push(scal_ciphertexts(random_enc.clone(), t_bn.to_string()));
            }
            t_vec.pop();
            pk_vec.pop();
            let s_sum = s_enc_t_vec.iter().skip(1).fold(s_enc_t_vec.get(0).unwrap().to_string(), |acc,v|{add_ciphertexts(acc, v.to_string())});
            let r_sum = r_enc_t_vec.iter().skip(1).fold(r_enc_t_vec.get(0).unwrap().to_string(), |acc,v|{add_ciphertexts(acc, v.to_string())});
            let h = GE::base_point2();
            let res_flag;
            
            let res =  batch_cl_pc_verify(enc_proof.clone(), to_hex(pc.to_bytes(true).as_ref()), s_sum.clone(), r_sum.clone(), to_hex(h.to_bytes(true).as_ref()), t_total.to_string(), t_vec.clone(), pk_vec.clone());
            info!("res: {}", res);

            if res == "true" {res_flag = true;}
            else{res_flag = false;} 
            let mut flag = all_verify_flag_clone.lock().unwrap();
            *flag = *flag && res_flag;
        });
        let mutex_map = commitments_map.lock().unwrap();
        let hashmap_ref = &*mutex_map;
        self.commitments_map = Some(hashmap_ref.clone());
        let mutex_flag = all_verify_flag.lock().unwrap();
        let all_flag_ref = *mutex_flag;
        if all_flag_ref
        { 
            // Merge commitment
            let vss_commitments_vec:Vec<Vec<Point<Secp256k1>>> = msg_vec.iter().map(|msg|msg.vss.commitments.clone()).collect();
           
            let total_vss_commitments = vss_commitments_vec
            .iter()
            .fold(vec![Point::<Secp256k1>::zero();vss_commitments_vec.len()], |acc,v| 
                { 
                    acc.iter()
                    .zip(v.iter())
                    .map(|(a,b)| a+b)
                    .collect()
                }
            );
            let share_proof_map_vec:Vec<HashMap<u16, String>> = msg_vec.iter().map(|msg| msg.share_enc_map.clone()).collect();
            let random_proof_map_vec:Vec<HashMap<u16, String>> = msg_vec.iter().map(|msg| msg.random_enc_map.clone()).collect();
            // Merge CL share
           
            let mut msg_map:HashMap<u16, ProxyToNodeKeyGenPhaseThreeP2PMsg> = HashMap::new();
            for node in self.node_info_vec.as_ref().unwrap()
            {
                let random = Scalar::<Secp256k1>::random().to_bigint().to_string();
                let c_zero = encrypt(node.cl_pk.clone(), "0".to_string(), random.clone());
                
                let share_enc_sum:String = share_proof_map_vec.iter().fold(c_zero.clone(), |acc,v|{add_ciphertexts(acc, v.get(&node.id).unwrap().to_string())});
                let random_enc_sum:String = random_proof_map_vec.iter().fold(c_zero, |acc,v|{add_ciphertexts(acc, v.get(&node.id).unwrap().to_string())});
                msg_map.insert
                (node.id.clone(), ProxyToNodeKeyGenPhaseThreeP2PMsg
                    {
                        sender:self.id.clone(),
                        role:self.role.clone(),
                        share_enc_sum,
                        random_enc_sum,
                        vss_scheme_sum:Vss
                        { 
                            parameters: 
                            ShamirSecretSharing 
                            { 
                                threshold: self.threashold_param.threshold, 
                                share_count: self.threashold_param.share_counts 
                            }, 
                            commitments: total_vss_commitments.clone() 
                        }
                    }
                );
            }
            Ok(msg_map)
        }
        else  
        { 
            Err(Error::InvalidZkp)
        }
    }
    pub fn keygen_phase_five(&self,msg_vec:Vec<NodeToProxyKeyGenPhaseFourP2PMsg>) -> Result<ProxyToNodeKeyGenPhaseFiveP2PMsg, Error>
    {
        let mut all_verify_flag = true;
        let commitments_map = self.commitments_map.as_ref().unwrap().clone();
        for msg in msg_vec.clone() {
            let s_coefficients_log = msg.s_coefficients_log;
            let commitments = commitments_map.get(&msg.sender).unwrap();
            let mut pc = Point::zero();
            let mut dl = Point::zero();
            for i in 0..commitments.len(){
                let t_bn = BigInt::from_hex(&sha256(&(to_hex(commitments[i].to_bytes(true).as_ref()) + &to_hex(s_coefficients_log[i].to_bytes(true).as_ref()).clone()))).unwrap();
                let t_fe = FE::from_bigint(&t_bn);
                pc = pc + commitments[i].clone() * t_fe.clone();
                dl = dl + s_coefficients_log[i].clone() * t_fe.clone();
            }
            let res = msg.dlog_proof.dl_pc_verify(pc, dl);
            all_verify_flag = all_verify_flag && res;
        }
        if all_verify_flag{
            let s_coefficients_log_vec:Vec<Vec<Point<Secp256k1>>> = msg_vec.iter().map(|msg|msg.s_coefficients_log.clone()).collect();
            let total_s_coefficients_log = s_coefficients_log_vec
            .iter()
            .fold(vec![Point::<Secp256k1>::zero();s_coefficients_log_vec.len()], |acc,v| 
                { 
                    acc.iter()
                    .zip(v.iter())
                    .map(|(a,b)| a+b)
                    .collect()
                }
            );
            Ok
            (
                ProxyToNodeKeyGenPhaseFiveP2PMsg
                {
                    total_s_coefficients_log
                }
            )
        }
        else  
        { 
            Err(Error::InvalidSlog)
        }

    }
}

