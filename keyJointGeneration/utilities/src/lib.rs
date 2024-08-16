use curv::elliptic::curves::{Point, Scalar, Secp256k1};
pub type CU = Secp256k1;
pub type FE = Scalar<Secp256k1>;
pub type GE = Point<Secp256k1>;

pub mod vss;
pub mod code;
pub mod zkp;