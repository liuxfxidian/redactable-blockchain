use crate::{CU, FE, GE};
use curv::arithmetic::Converter;
use curv::arithmetic::One;
use curv::cryptographic_primitives::secret_sharing::feldman_vss::ShamirSecretSharing;
use curv::cryptographic_primitives::secret_sharing::Polynomial;
use curv::BigInt;
use curv::ErrorSS;
use curv::ErrorSS::VerifyShareError;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Vss {
    pub parameters: ShamirSecretSharing,
    pub commitments: Vec<GE>,
}

pub struct PedersenSecretShares{
    pub secret_shares: Vec<FE>,
    pub random_shares: Vec<FE>
}

pub struct Coefficients {
    pub secret_coefficients: Vec<FE>,
    pub random_coefficients: Vec<FE>
}

impl Vss {
    pub fn validate_share(&self, secret_share: &FE, index: u16) -> Result<(), ErrorSS> {
        let ss_point = GE::generator() * secret_share;
        self.validate_share_public(&ss_point, index)
    }

    pub fn validate_share_public(&self, ss_point: &GE, index: u16) -> Result<(), ErrorSS> {
        let comm_to_point = self.get_point_commitment(index);
        if *ss_point == comm_to_point {
            Ok(())
        } else {
            Err(VerifyShareError)
        }
    }

    pub fn get_point_commitment(&self, index: u16) -> GE {
        let index_fe = FE::from(&BigInt::from(index));
        let mut comm_iterator = self.commitments.iter().rev();
        let head = comm_iterator.next().unwrap();
        let tail = comm_iterator;
        tail.fold(head.clone(), |acc, x: &GE| x + acc * &index_fe)
    }

    pub fn pedersen_validate_share(&self, secret_share: &FE, random: &FE, index: u16) -> Result<(), ErrorSS> {
        let ss_point = GE::generator() * secret_share + GE::base_point2() * random;
        self.pedersen_validate_share_public(&ss_point, index)
    }

    pub fn pedersen_validate_share_public(&self, ss_point: &GE, index: u16) -> Result<(), ErrorSS> {
        let comm_to_point = self.pedersen_get_point_commitment(index);
        if *ss_point == comm_to_point {
            Ok(())
        } else {
            Err(VerifyShareError)
        }
    }
    
    pub fn pedersen_get_point_commitment(&self, index: u16) -> GE {
        let index_fe: FE = FE::from(&BigInt::from(index));
        let mut comm_iterator = self.commitments.iter().rev();
        let head = comm_iterator.next().unwrap();
        let tail = comm_iterator;
        tail.fold(head.clone(), |acc, x: &GE| x + acc * &index_fe)
    }
}

pub fn share_at_indices(
    t: usize,
    n: usize,
    secret: &FE,
    index_vec: &Vec<String>,
) -> (Vss, HashMap<String, FE>) {
    assert_eq!(n, index_vec.len());
    let poly = Polynomial::<CU>::sample_exact_with_fixed_const_term(t as u16, secret.clone());
    let secret_shares = evaluate_polynomial(&poly, &index_vec);
    let g = GE::generator();
    let poly = poly.coefficients();
    let commitments = (0..poly.len()).map(|i| g * &poly[i]).collect::<Vec<GE>>();
    (
        Vss {
            parameters: ShamirSecretSharing {
                threshold: t as u16,
                share_count: n as u16,
            },
            commitments,
        },
        secret_shares,
    )
}

pub fn pedersen_share(
    t: u16,
    n: u16,
    secret: &FE,
    random: &FE,
) -> (Vss, PedersenSecretShares, Coefficients) {
    let poly_fx = Polynomial::<CU>::sample_exact_with_fixed_const_term(t, secret.clone());
    let poly_gx = Polynomial::<CU>::sample_exact_with_fixed_const_term(t, random.clone());
    let secret_shares = poly_fx.evaluate_many_bigint(1..=n).collect();
    let random_shares = poly_gx.evaluate_many_bigint(1..=n).collect();
    let g = GE::generator();
    let h = GE::base_point2();

    let poly_fx_co = poly_fx.coefficients();
    let poly_gx_co = poly_gx.coefficients();
    let commitments = (0..poly_fx_co.len()).map(|i| g * &poly_fx_co[i] + h * &poly_gx_co[i]).collect::<Vec<GE>>();
    (
        Vss {
            parameters: ShamirSecretSharing {
                threshold: t as u16,
                share_count: n as u16,
            },
            commitments,
        },

        PedersenSecretShares{
            secret_shares,
            random_shares
        },

        Coefficients{
            secret_coefficients: poly_fx_co.to_vec(),
            random_coefficients: poly_gx_co.to_vec()
        }
    )
}

fn evaluate_polynomial(poly: &Polynomial<CU>, index_vec_string: &[String]) -> HashMap<String, FE> {
    let mut share_map: HashMap<String, FE> = HashMap::new();
    for i in index_vec_string {
        let value = poly.evaluate(&FE::from(&BigInt::from_str_radix(&i, 16).unwrap()));
        share_map.insert((*i).clone(), value);
    }
    return share_map;
}

pub fn map_share_to_new_params(index: BigInt, s: &[BigInt]) -> FE {
    let s_len = s.len();
    // add one to indices to get points
    let points: Vec<FE> = s.iter().map(|i| FE::from(i)).collect();

    let xi: FE = FE::from(&index);
    let num: FE = FE::from(&BigInt::one());
    let denum: FE = FE::from(&BigInt::one());
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