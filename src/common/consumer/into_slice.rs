use core::convert::{AsMut, AsRef};
use core::mem::MaybeUninit;

use wrapper::Wrapper;

use crate::common::consumer::Invariant;
use crate::local_nb::{
    BufferedConsumer as BufferedConsumerLocalNb, BulkConsumer as BulkConsumerLocalNb,
    Consumer as ConsumerLocalNb,
};
use crate::maybe_uninit_slice_mut;
use crate::sync::{BufferedConsumer, BulkConsumer, Consumer};

// Macro syntax for handling generic parameters: https://stackoverflow.com/a/61189128

macro_rules! invarianted_consumer_outer_type {
    ($(#[$doc:meta])* $outer:ident $inner:ident $(< $( $lt:tt $( : $clt:tt $(+ $dlt:tt )* )? ),+ >)? ) => {
        $(#[$doc])*
        pub struct $outer $(< $( $lt $( : $clt $(+ $dlt )* )? ),+ >)?(crate::common::consumer::Invariant<$inner $(< $( $lt ),+ >)?>);
    }
}

macro_rules! invarianted_consumer_impl_debug {
    ($outer:ident $(< $( $lt:tt $( : $clt:tt $(+ $dlt:tt )* )? ),+ >)? ) => {
        impl $(< $( $lt $( : $clt $(+ $dlt )* )? ),+ >)?
            core::fmt::Debug
        for $outer
            $(< $( $lt ),+ >)?
        {
            fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                self.0.fmt(f)
            }
        }
    }
}

macro_rules! invarianted_consumer_impl_as_ref {
    ($outer:ident $(< $( $lt:tt $( : $clt:tt $(+ $dlt:tt )* )? ),+ >)?; $t:ty) => {
        impl $(< $( $lt $( : $clt $(+ $dlt )* )? ),+ >)?
            core::convert::AsRef<$t>
        for $outer
            $(< $( $lt ),+ >)?
        {
            fn as_ref(&self) -> &$t {
                self.0.as_ref().as_ref()
            }
        }
    }
}

macro_rules! invarianted_consumer_impl_as_mut {
    ($outer:ident $(< $( $lt:tt $( : $clt:tt $(+ $dlt:tt )* )? ),+ >)?; $t:ty) => {
        impl $(< $( $lt $( : $clt $(+ $dlt )* )? ),+ >)?
            core::convert::AsMut<$t>
        for $outer
            $(< $( $lt ),+ >)?
        {
            fn as_mut(&mut self) -> &mut $t {
                self.0.as_mut().as_mut()
            }
        }
    }
}

macro_rules! invarianted_consumer_impl_wrapper {
    ($outer:ident $(< $( $lt:tt $( : $clt:tt $(+ $dlt:tt )* )? ),+ >)?; $t:ty) => {
        impl $(< $( $lt $( : $clt $(+ $dlt )* )? ),+ >)?
            wrapper::Wrapper<$t>
        for $outer
            $(< $( $lt ),+ >)?
        {
            fn into_inner(self) -> $t {
                self.0.into_inner().into_inner()
            }
        }
    }
}

macro_rules! invarianted_consumer_impl_consumer {
    ($outer:ident $(< $( $lt:tt $( : $clt:tt $(+ $dlt:tt )* )? ),+ >)? $(#[$doc_item:meta])? Item $t_item:ty; $(#[$doc_final:meta])? Final $t_final:ty; $(#[$doc_error:meta])? Error $t_error:ty) => {
        impl $(< $( $lt $( : $clt $(+ $dlt )* )? ),+ >)?
            crate::sync::Consumer
        for $outer
            $(< $( $lt ),+ >)?
        {
            $(#[$doc_item])*
            type Item = $t_item;
            $(#[$doc_final])*
            type Final = $t_final;
            $(#[$doc_error])*
            type Error = $t_error;

            fn consume(&mut self, item: T) -> Result<(), Self::Error> {
                Consumer::consume(&mut self.0, item)
            }

            fn close(&mut self, fin: Self::Final) -> Result<(), Self::Error> {
                Consumer::close(&mut self.0, fin)
            }
        }
    }
}

macro_rules! invarianted_consumer_impl_buffered_consumer {
    ($outer:ident $(< $( $lt:tt $( : $clt:tt $(+ $dlt:tt )* )? ),+ >)?) => {
        impl $(< $( $lt $( : $clt $(+ $dlt )* )? ),+ >)?
            crate::sync::BufferedConsumer
        for $outer
            $(< $( $lt ),+ >)?
        {
            fn flush(&mut self) -> Result<(), Self::Error> {
                BufferedConsumer::flush(&mut self.0)
            }
        }
    }
}

macro_rules! invarianted_consumer_impl_bulk_consumer {
    ($outer:ident $(< $( $lt:tt $( : $clt:tt $(+ $dlt:tt )* )? ),+ >)?) => {
        impl $(< $( $lt $( : $clt $(+ $dlt )* )? ),+ >)?
            crate::sync::BulkConsumer
        for $outer
            $(< $( $lt ),+ >)?
        {
            fn expose_slots(&mut self) -> Result<&mut [MaybeUninit<Self::Item>], Self::Error> {
                BulkConsumer::expose_slots(&mut self.0)
            }
        
            unsafe fn consume_slots(&mut self, amount: usize) -> Result<(), Self::Error> {
                BulkConsumer::consume_slots(&mut self.0, amount)
            }
        }
    }
}

macro_rules! invarianted_consumer_impl_consumer_local_nb {
    ($outer:ident $(< $( $lt:tt $( : $clt:tt $(+ $dlt:tt )* )? ),+ >)? $(#[$doc_item:meta])? Item $t_item:ty; $(#[$doc_final:meta])? Final $t_final:ty; $(#[$doc_error:meta])? Error $t_error:ty) => {
        impl $(< $( $lt $( : $clt $(+ $dlt )* )? ),+ >)?
            crate::local_nb::Consumer
        for $outer
            $(< $( $lt ),+ >)?
        {
            $(#[$doc_item])*
            type Item = $t_item;
            $(#[$doc_final])*
            type Final = $t_final;
            $(#[$doc_error])*
            type Error = $t_error;

            async fn consume(&mut self, item: Self::Item) -> Result<(), Self::Error> {
                ConsumerLocalNb::consume(&mut self.0, item).await
            }

            async fn close(&mut self, f: Self::Final) -> Result<(), Self::Error> {
                ConsumerLocalNb::close(&mut self.0, f).await
            }
        }
    }
}

macro_rules! invarianted_consumer_impl_buffered_consumer_local_nb {
    ($outer:ident $(< $( $lt:tt $( : $clt:tt $(+ $dlt:tt )* )? ),+ >)?) => {
        impl $(< $( $lt $( : $clt $(+ $dlt )* )? ),+ >)?
            crate::local_nb::BufferedConsumer
        for $outer
            $(< $( $lt ),+ >)?
        {
            async fn flush(&mut self) -> Result<(), Self::Error> {
                BufferedConsumerLocalNb::flush(&mut self.0).await
            }
        }
    }
}

macro_rules! invarianted_consumer_impl_bulk_consumer_local_nb {
    ($outer:ident $(< $( $lt:tt $( : $clt:tt $(+ $dlt:tt )* )? ),+ >)?) => {
        impl $(< $( $lt $( : $clt $(+ $dlt )* )? ),+ >)?
            crate::local_nb::BulkConsumer
        for $outer
            $(< $( $lt ),+ >)?
        {
            async fn expose_slots<'b>(
                &'b mut self,
            ) -> Result<&'b mut [MaybeUninit<Self::Item>], Self::Error>
            where
                Self::Item: 'b,
            {
                BulkConsumerLocalNb::expose_slots(&mut self.0).await
            }

            async unsafe fn consume_slots(&mut self, amount: usize) -> Result<(), Self::Error> {
                BulkConsumerLocalNb::consume_slots(&mut self.0, amount).await
            }
        }
    }
}

macro_rules! invarianted_consumer_impl_consumer_sync_and_local_nb {
    ($outer:ident $(< $( $lt:tt $( : $clt:tt $(+ $dlt:tt )* )? ),+ >)? $(#[$doc_item:meta])? Item $t_item:ty; $(#[$doc_final:meta])? Final $t_final:ty; $(#[$doc_error:meta])? Error $t_error:ty) => {
        invarianted_consumer_impl_consumer!($outer $(< $( $lt $( : $clt $(+ $dlt )* )? ),+ >)? $(#[$doc_item])? Item $t_item; $(#[$doc_final])? Final $t_final; $(#[$doc_error])? Error $t_error);
        invarianted_consumer_impl_consumer_local_nb!($outer $(< $( $lt $( : $clt $(+ $dlt )* )? ),+ >)? $(#[$doc_item])? Item $t_item; $(#[$doc_final])? Final $t_final; $(#[$doc_error])? Error $t_error);
    }
}

macro_rules! invarianted_consumer_impl_buffered_consumer_sync_and_local_nb {
    ($outer:ident $(< $( $lt:tt $( : $clt:tt $(+ $dlt:tt )* )? ),+ >)?) => {
        invarianted_consumer_impl_buffered_consumer!($outer $(< $( $lt $( : $clt $(+ $dlt )* )? ),+ >)?);
        invarianted_consumer_impl_buffered_consumer_local_nb!($outer $(< $( $lt $( : $clt $(+ $dlt )* )? ),+ >)?);
    }
}

macro_rules! invarianted_consumer_impl_bulk_consumer_sync_and_local_nb {
    ($outer:ident $(< $( $lt:tt $( : $clt:tt $(+ $dlt:tt )* )? ),+ >)?) => {
        invarianted_consumer_impl_bulk_consumer!($outer $(< $( $lt $( : $clt $(+ $dlt )* )? ),+ >)?);
        invarianted_consumer_impl_bulk_consumer_local_nb!($outer $(< $( $lt $( : $clt $(+ $dlt )* )? ),+ >)?);
    }
}


invarianted_consumer_outer_type!(
    /// Consumes data into a mutable slice.
    IntoSlice_ IntoSlice <'a, T>
);

use core::fmt::Debug;
invarianted_consumer_impl_debug!(IntoSlice_<'a, T: Debug>);

invarianted_consumer_impl_as_ref!(IntoSlice_<'a, T>; [T]);
invarianted_consumer_impl_as_mut!(IntoSlice_<'a, T>; [T]);
invarianted_consumer_impl_wrapper!(IntoSlice_<'a, T>; &'a [T]);

invarianted_consumer_impl_consumer_sync_and_local_nb!(IntoSlice_<'a, T> Item T; Final ();
    /// Emitted when the slice has been fully overwritten and an attempt to consume more items is made.
    Error ()
);
invarianted_consumer_impl_buffered_consumer_sync_and_local_nb!(IntoSlice_<'a, T>);
invarianted_consumer_impl_bulk_consumer_sync_and_local_nb!(IntoSlice_<'a, T: Copy>);

/// Create a consumer which places consumed data into the given slice.
impl<'a, T> IntoSlice_<'a, T> {
    pub fn new(slice: &mut [T]) -> IntoSlice_<'_, T> {
        IntoSlice_(Invariant::new(IntoSlice(slice, 0)))
    }

    /// Return the offset into the slice at which the next item consumed item will be written.
    pub fn get_offset(&self) -> usize {
        (self.0).as_ref().1
    }

    /// Return the subslice that has been overwritten with consumed items.
    pub fn get_overwritten_so_far(&self) -> &[T] {
        &(self.0).as_ref().0[..self.get_offset()]
    }

    /// Return the subslice of items that have not yet been overwritten with consumed items.
    pub fn get_not_yet_overwritten(&self) -> &[T] {
        &(self.0).as_ref().0[self.get_offset()..]
    }

    /// Return a mutable reference to the subslice that has been overwritten with consumed items.
    pub fn get_overwritten_so_far_mut(&mut self) -> &mut [T] {
        let offset = self.get_offset();
        &mut (self.0).as_mut().0[..offset]
    }

    /// Return a mutable reference to the subslice of items that have not yet been overwritten with consumed items.
    pub fn get_not_yet_overwritten_mut(&mut self) -> &mut [T] {
        let offset = self.get_offset();
        &mut (self.0).as_mut().0[offset..]
    }
}

// The usize is  the offset into the slice where to place the next item.
#[derive(Debug)]
struct IntoSlice<'a, T>(&'a mut [T], usize);

impl<'a, T> AsRef<[T]> for IntoSlice<'a, T> {
    fn as_ref(&self) -> &[T] {
        self.0
    }
}

impl<'a, T> AsMut<[T]> for IntoSlice<'a, T> {
    fn as_mut(&mut self) -> &mut [T] {
        self.0
    }
}

impl<'a, T> Wrapper<&'a mut [T]> for IntoSlice<'a, T> {
    fn into_inner(self) -> &'a mut [T] {
        self.0
    }
}

impl<'a, T> Consumer for IntoSlice<'a, T> {
    type Item = T;
    type Final = ();
    type Error = ();

    fn consume(&mut self, item: T) -> Result<Self::Final, Self::Error> {
        if self.0.len() == self.1 {
            // We already overwrote the full slice, notify the caller via an error return.
            Err(())
        } else {
            self.0[self.1] = item;
            self.1 += 1;

            Ok(())
        }
    }

    fn close(&mut self, _: Self::Final) -> Result<(), Self::Error> {
        Ok(())
    }
}

impl<'a, T> BufferedConsumer for IntoSlice<'a, T> {
    fn flush(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }
}

impl<'a, T: Copy> BulkConsumer for IntoSlice<'a, T> {
    fn expose_slots(&mut self) -> Result<&mut [MaybeUninit<Self::Item>], Self::Error> {
        if self.0.len() == self.1 {
            // We already overwrote the full slice, notify the caller via an error return.
            Err(())
        } else {
            Ok(maybe_uninit_slice_mut(&mut self.0[self.1..]))
        }
    }

    unsafe fn consume_slots(&mut self, amount: usize) -> Result<(), Self::Error> {
        self.1 += amount;

        Ok(())
    }
}

impl<'a, T> ConsumerLocalNb for IntoSlice<'a, T> {
    type Item = T;
    type Final = ();
    type Error = ();

    async fn consume(&mut self, item: Self::Item) -> Result<(), Self::Error> {
        Consumer::consume(self, item)
    }

    async fn close(&mut self, f: Self::Final) -> Result<(), Self::Error> {
        Consumer::close(self, f)
    }
}

impl<'a, T> BufferedConsumerLocalNb for IntoSlice<'a, T> {
    async fn flush(&mut self) -> Result<(), Self::Error> {
        BufferedConsumer::flush(self)
    }
}

impl<'a, T: Copy> BulkConsumerLocalNb for IntoSlice<'a, T> {
    async fn expose_slots<'b>(
        &'b mut self,
    ) -> Result<&'b mut [MaybeUninit<Self::Item>], Self::Error>
    where
        Self::Item: 'b,
    {
        BulkConsumer::expose_slots(self)
    }

    async unsafe fn consume_slots(&mut self, amount: usize) -> Result<(), Self::Error> {
        BulkConsumer::consume_slots(self, amount)
    }
}

// #[cfg(test)]
// mod tests {
//     use super::super::*;
//     use crate::sync::*;

//     // Panic conditions:
//     //
//     // - `consume()` must not be called after `close()` or error
//     // - `close()` must not be called after `close()` or error
//     // - `flush()` must not be called after `close()` or error
//     // - `consumer_slots()` must not be called after `close()` or error
//     // - `did_consume()` must not be called after `close()` or error
//     // - `bulk_consume()` must not be called after `close()` or error
//     // - `did_consume(amount)` must not be called with `amount` greater than available slots

//     // In each of the following tests, the final function call should panic.

//     #[test]
//     #[should_panic(expected = "may not call `Consumer` methods after the sequence has ended")]
//     fn panics_on_consume_after_close() {
//         let mut buf = [0; 1];

//         let mut slice_consumer = IntoSlice::new(&mut buf);
//         let _ = slice_consumer.close(());
//         let _ = slice_consumer.consume(7);
//     }

//     #[test]
//     #[should_panic(expected = "may not call `Consumer` methods after the sequence has ended")]
//     fn panics_on_close_after_close() {
//         let mut buf = [0; 1];

//         let mut slice_consumer = IntoSlice::new(&mut buf);
//         let _ = slice_consumer.close(());
//         let _ = slice_consumer.close(());
//     }

//     #[test]
//     #[should_panic(expected = "may not call `Consumer` methods after the sequence has ended")]
//     fn panics_on_flush_after_close() {
//         let mut buf = [0; 1];

//         let mut slice_consumer = IntoSlice::new(&mut buf);
//         let _ = slice_consumer.close(());
//         let _ = slice_consumer.flush();
//     }

//     #[test]
//     #[should_panic(expected = "may not call `Consumer` methods after the sequence has ended")]
//     fn panics_on_consumer_slots_after_close() {
//         let mut buf = [0; 1];

//         let mut slice_consumer = IntoSlice::new(&mut buf);
//         let _ = slice_consumer.close(());
//         let _ = slice_consumer.expose_slots();
//     }

//     #[test]
//     #[should_panic(expected = "may not call `Consumer` methods after the sequence has ended")]
//     fn panics_on_did_consume_after_close() {
//         let mut buf = [0; 8];

//         let mut slice_consumer = IntoSlice::new(&mut buf);
//         let _ = slice_consumer.close(());

//         unsafe {
//             let _ = slice_consumer.consume_slots(7);
//         }
//     }

//     #[test]
//     #[should_panic(expected = "may not call `Consumer` methods after the sequence has ended")]
//     fn panics_on_bulk_consume_after_close() {
//         let mut buf = [0; 8];

//         let mut slice_consumer = IntoSlice::new(&mut buf);
//         let _ = slice_consumer.close(());
//         let _ = slice_consumer.bulk_consume(b"ufo");
//     }

//     #[test]
//     #[should_panic(
//         expected = "may not call `consume_slots` with an amount exceeding the total number of exposed slots"
//     )]
//     fn panics_on_did_consume_with_amount_greater_than_available_slots() {
//         let mut buf = [0; 8];

//         let mut slice_consumer = IntoSlice::new(&mut buf);

//         unsafe {
//             let _ = slice_consumer.consume_slots(21);
//         }
//     }
// }
