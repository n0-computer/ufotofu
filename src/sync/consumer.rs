mod into_vec;
mod into_vec_fallible;
mod invariant;
mod invariant_noop;
mod slice_consumer;

#[cfg(feature = "dev")]
mod scramble;

pub use into_vec::IntoVec;
pub use into_vec_fallible::{IntoVecError, IntoVecFallible};
pub use slice_consumer::{SliceConsumer, SliceConsumerFullError};

#[cfg(feature = "dev")]
pub use scramble::{ConsumeOperations, Scramble};

// During testing we use a wrapper which panics on invariant transgressions.
// The no-op version of the wrapper is used for production code compilation.
#[cfg(test)]
pub use invariant::Invariant;
#[cfg(not(test))]
pub use invariant_noop::Invariant;
