use core::mem::MaybeUninit;

#[cfg(all(feature = "alloc", not(feature = "std")))]
use alloc::{
    alloc::{Allocator, Global},
    vec::Vec,
};
#[cfg(feature = "std")]
use std::{
    alloc::{Allocator, Global},
    vec::Vec,
};

use wrapper::Wrapper;

use crate::local_nb::consumer::SyncToLocalNb;
use crate::local_nb::{LocalBufferedConsumer, LocalBulkConsumer, LocalConsumer};
use crate::sync::consumer::IntoVec as SyncIntoVec;

/// Collects data and can at any point be converted into a `Vec<T>`.
#[derive(Debug)]
pub struct IntoVec<T, A: Allocator = Global>(SyncToLocalNb<SyncIntoVec<T, A>>);

impl<T> Default for IntoVec<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> IntoVec<T> {
    pub fn new() -> IntoVec<T> {
        let into_vec = SyncIntoVec::new();

        IntoVec(SyncToLocalNb(into_vec))
    }

    pub fn into_vec(self) -> Vec<T> {
        let inner = self.0.into_inner();
        inner.into_inner()
    }
}

impl<T, A: Allocator> IntoVec<T, A> {
    pub fn new_in(alloc: A) -> IntoVec<T, A> {
        let into_vec = SyncIntoVec::new_in(alloc);

        IntoVec(SyncToLocalNb(into_vec))
    }
}

impl<T> AsRef<Vec<T>> for IntoVec<T> {
    fn as_ref(&self) -> &Vec<T> {
        let inner = self.0.as_ref();
        inner.as_ref()
    }
}

impl<T> AsMut<Vec<T>> for IntoVec<T> {
    fn as_mut(&mut self) -> &mut Vec<T> {
        let inner = self.0.as_mut();
        inner.as_mut()
    }
}

impl<T> Wrapper<Vec<T>> for IntoVec<T> {
    fn into_inner(self) -> Vec<T> {
        let inner = self.0.into_inner();
        inner.into_inner()
    }
}

impl<T> LocalConsumer for IntoVec<T> {
    type Item = T;
    type Final = ();
    type Error = !;

    async fn consume(&mut self, item: Self::Item) -> Result<(), Self::Error> {
        self.0.consume(item).await
    }

    async fn close(&mut self, final_val: Self::Final) -> Result<(), Self::Error> {
        self.0.close(final_val).await
    }
}

impl<T> LocalBufferedConsumer for IntoVec<T> {
    async fn flush(&mut self) -> Result<(), Self::Error> {
        self.0.flush().await
    }
}

impl<T: Copy> LocalBulkConsumer for IntoVec<T> {
    async fn consumer_slots<'a>(
        &'a mut self,
    ) -> Result<&'a mut [MaybeUninit<Self::Item>], Self::Error>
    where
        T: 'a,
    {
        self.0.consumer_slots().await
    }

    async unsafe fn did_consume(&mut self, amount: usize) -> Result<(), Self::Error> {
        self.0.did_consume(amount).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn converts_into_vec() {
        smol::block_on(async {
            let mut into_vec = IntoVec::new();
            let _ = into_vec.bulk_consume(b"ufotofu").await;
            let _ = into_vec.close(()).await;

            let vec = into_vec.into_vec();
            assert_eq!(vec.len(), 7);
        })
    }

    // Panic conditions:
    //
    // - `consume()` must not be called after `close()` or error
    // - `close()` must not be called after `close()` or error
    // - `flush()` must not be called after `close()` or error
    // - `consumer_slots()` must not be called after `close()` or error
    // - `did_consume()` must not be called after `close()` or error
    // - `bulk_consume()` must not be called after `close()` or error
    // - `did_consume(amount)` must not be called with `amount` greater than available slots

    // In each of the following tests, the final function call should panic.

    #[test]
    #[should_panic(expected = "may not call `Consumer` methods after the sequence has ended")]
    fn panics_on_consume_after_close() {
        smol::block_on(async {
            let mut into_vec = IntoVec::new();
            let _ = into_vec.close(()).await;
            let _ = into_vec.consume(7).await;
        })
    }

    #[test]
    #[should_panic(expected = "may not call `Consumer` methods after the sequence has ended")]
    fn panics_on_close_after_close() {
        smol::block_on(async {
            // Type annotations are required because we never provide a `T`.
            let mut into_vec: IntoVec<u8> = IntoVec::new();
            let _ = into_vec.close(()).await;
            let _ = into_vec.close(()).await;
        })
    }

    #[test]
    #[should_panic(expected = "may not call `Consumer` methods after the sequence has ended")]
    fn panics_on_flush_after_close() {
        smol::block_on(async {
            let mut into_vec: IntoVec<u8> = IntoVec::new();
            let _ = into_vec.close(()).await;
            let _ = into_vec.flush().await;
        })
    }

    #[test]
    #[should_panic(expected = "may not call `Consumer` methods after the sequence has ended")]
    fn panics_on_consumer_slots_after_close() {
        smol::block_on(async {
            let mut into_vec: IntoVec<u8> = IntoVec::new();
            let _ = into_vec.close(()).await;
            let _ = into_vec.consumer_slots().await;
        })
    }

    #[test]
    #[should_panic(expected = "may not call `Consumer` methods after the sequence has ended")]
    fn panics_on_did_consume_after_close() {
        smol::block_on(async {
            let mut into_vec: IntoVec<u8> = IntoVec::new();
            let _ = into_vec.close(()).await;

            unsafe {
                let _ = into_vec.did_consume(7).await;
            }
        })
    }

    #[test]
    #[should_panic(expected = "may not call `Consumer` methods after the sequence has ended")]
    fn panics_on_bulk_consume_after_close() {
        smol::block_on(async {
            let mut into_vec = IntoVec::new();
            let _ = into_vec.close(()).await;
            let _ = into_vec.bulk_consume(b"ufo").await;
        })
    }

    #[test]
    #[should_panic(
        expected = "may not call `did_consume` with an amount exceeding the total number of exposed slots"
    )]
    fn panics_on_did_consume_with_amount_greater_than_available_slots() {
        smol::block_on(async {
            let mut into_vec: IntoVec<u8> = IntoVec::new();

            unsafe {
                let _ = into_vec.did_consume(21).await;
            }
        })
    }
}
