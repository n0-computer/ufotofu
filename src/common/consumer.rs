#[cfg(feature = "alloc")]
mod into_vec;
#[cfg(feature = "alloc")]
pub use into_vec::IntoVec_ as IntoVec;

#[cfg(feature = "alloc")]
mod into_vec_fallible;
#[cfg(feature = "alloc")]
pub use into_vec_fallible::{IntoVecError, IntoVecFallible_ as IntoVecFallible};

#[cfg(test)]
mod invariant;
#[cfg(not(test))]
mod invariant_noop;
#[cfg(test)]
pub use invariant::Invariant;
#[cfg(not(test))]
pub use invariant_noop::Invariant;

#[cfg(feature = "dev")]
mod scramble;
#[cfg(feature = "dev")]
pub use scramble::{ConsumeOperations, Scramble};
