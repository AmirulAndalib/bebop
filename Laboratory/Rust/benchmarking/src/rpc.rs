use std::str::FromStr;
use std::sync::Arc;

use bebop::prelude::*;
use bebop::rpc::{handlers, LocalRpcError};
use criterion::black_box;

use crate::bebops::rpc::owned::SHandlersDef;

pub struct Service;

#[handlers(crate::bebops::rpc)]
impl SHandlersDef for Arc<Service> {
    #[handler]
    async fn ping(self, _details: &dyn CallDetails) -> LocalRpcResponse<()> {
        Ok(())
    }

    #[handler]
    async fn submit_a(
        self,
        _details: &dyn CallDetails,
        a: crate::bebops::rpc::owned::ObjA,
    ) -> LocalRpcResponse<()> {
        if (a.a == -34234 && a.b == 695) {
            Ok(())
        } else {
            Err(LocalRpcError::CustomErrorStatic(
                1,
                "Expected different data",
            ))
        }
    }

    #[handler]
    async fn submit_b(
        self,
        _details: &dyn CallDetails,
        b: crate::bebops::rpc::owned::ObjB,
    ) -> LocalRpcResponse<()> {
        if matches!(b.a, Some(3))
            && matches!(b.b, Some(5743))
            && matches!(b.e.as_deref(), Some("s890df7g"))
        {
            Ok(())
        } else {
            Err(LocalRpcError::CustomErrorStatic(
                2,
                "Expected different data",
            ))
        }
    }

    #[handler]
    async fn submit_a_b(
        self,
        _details: &dyn CallDetails,
        a: crate::bebops::rpc::owned::ObjA,
        b: crate::bebops::rpc::owned::ObjB,
        c: bool,
        d: String,
    ) -> LocalRpcResponse<()> {
        if c && d == "08fdg" && a.b == -579 && matches!(b.b, Some(9543)) {
            Ok(())
        } else {
            Err(LocalRpcError::CustomErrorStatic(
                3,
                "Expected different data",
            ))
        }
    }

    #[handler]
    async fn submit_c(
        self,
        _details: &dyn CallDetails,
        a: i32,
        b: crate::bebops::rpc::owned::ObjC,
    ) -> LocalRpcResponse<()> {
        if a == 1234 && b.b == "iogy" && b.a.b == 6 && b.c[1] == 23.34 {
            Ok(())
        } else {
            Err(LocalRpcError::CustomErrorStatic(
                4,
                "Expected different data",
            ))
        }
    }

    #[handler]
    async fn respond_a(
        self,
        _details: &dyn CallDetails,
    ) -> LocalRpcResponse<crate::bebops::rpc::ObjA> {
        black_box(Ok(crate::bebops::rpc::ObjA {
            a: -34234,
            b: 695,
            c: Date::from_secs(8497589634),
            d: Guid::from_str("f7df0b17-6fc3-4a09-b8c2-308d06e3e558").unwrap(),
        }))
    }

    #[handler]
    async fn respond_b<'sup>(
        self,
        _details: &dyn CallDetails,
    ) -> LocalRpcResponse<crate::bebops::rpc::ObjB<'sup>> {
        black_box(Ok(crate::bebops::rpc::ObjB {
            a: Some(3),
            b: Some(5743),
            c: Some(Date::from_secs(9754234)),
            d: Some(Guid::from_str("7b5de104-5a40-47e6-b5d3-ab33f8c82fd2").unwrap()),
            e: Some("s890df7g"),
        }))
    }

    #[handler]
    async fn respond_c<'sup>(
        self,
        _details: &dyn CallDetails,
    ) -> LocalRpcResponse<crate::bebops::rpc::ObjC<'sup>> {
        black_box(Ok(crate::bebops::rpc::ObjC {
            a: crate::bebops::rpc::ObjA {
                a: 324,
                b: 6,
                c: Date::from_secs(9864234),
                d: Guid::from_str("2265afc4-7e12-4174-a17c-ee75c4dff074").unwrap(),
            },
            b: "iogy",
            c: SliceWrapper::Cooked(&[234.4, 23.34, 54., 543.1]),
        }))
    }
}

// /// A clone of the call table in Bebop but for our handrolled types; some simplifications have been
// /// made.
// pub struct HandrolledCallTable {
//     default_timeout: Option<Duration>,
//
//     /// Table of calls which have yet to be resolved.
//     call_table: HashMap<NonZeroU16, Box<dyn ResponseHandle>>,
//
//     /// The next ID value which should be used.
//     next_id: u16,
// }
//
// impl Default for HandrolledCallTable {
//     fn default() -> Self {
//         Self {
//             default_timeout: Some(Duration::from_secs(2)),
//             next_id: 1,
//             call_table: Default::default(),
//         }
//     }
// }
//
// impl HandrolledCallTable {
//     /// Get the ID which should be used for the next call that gets made.
//     pub fn next_call_id(&mut self) -> NonZeroU16 {
//         // prevent an infinite loop; if this is a problem in production, we can either increase the
//         // id size OR we can stall on overflow and try again creating back pressure.
//         assert!(
//             self.call_table.len() < (u16::MAX as usize),
//             "Call table overflow"
//         );
//
//         // zero is a "null" id
//         while self.next_id == 0
//             || self
//                 .call_table
//                 .contains_key(unsafe { &NonZeroU16::new_unchecked(self.next_id) })
//         {
//             // the value is not valid because it is 0 or is in use already
//             self.next_id = self.next_id.wrapping_add(1);
//         }
//
//         // found our id, guaranteed to not be zero
//         let id = unsafe { NonZeroU16::new_unchecked(self.next_id) };
//         self.next_id = self.next_id.wrapping_add(1);
//         id
//     }
//
//     /// Register a datagram before we send it. This will set the call_id.
//     pub fn register<R>(&mut self, datagram: &HandrolledDatagram) -> PendingResponse<R>
//     where
//         R: 'static + OwnedRecord,
//     {
//         debug_assert!(datagram.is_request(), "Only requests should be registered.");
//         let call_id = datagram
//             .call_id()
//             .expect("Datagram call ids must be set by the router.");
//
//         let timeout = datagram.timeout().or(self.default_timeout);
//         let (handle, pending) = new_pending_response(call_id, timeout);
//
//         self.call_table.insert(call_id, handle);
//         pending
//     }
//
//     /// Receive a datagram and routes it. This is used by the handler for the TransportProtocol.
//     pub fn resolve(&mut self, datagram: &HandrolledDatagram) {
//         debug_assert!(datagram.is_response(), "Only responses should be resolved.");
//         let v = match datagram {
//             HandrolledDatagram::Unknown => panic!("Unknown datagram received!"),
//             HandrolledDatagram::PingRequest { .. } => unreachable!("Only responses are allowed"),
//             HandrolledDatagram::PingResponse { header } => (header.id, Ok(())),
//             HandrolledDatagram::HandrolledResponseCallNotSupported { header } => {
//                 (header.id, Err(RemoteRpcError::CallNotSupported))
//             }
//             HandrolledDatagram::HandrolledResponseUnknownCall { header } => {
//                 (header.id, Err(RemoteRpcError::UnknownCall))
//             }
//             HandrolledDatagram::HandrolledResponseInvalidSignature { header, signature } => {
//                 (header.id, Err(RemoteRpcError::InvalidSignature(*signature)))
//             }
//             HandrolledDatagram::HandrolledDecodeError { header, info } => (
//                 header.id,
//                 Err(RemoteRpcError::RemoteDecodeError(Some(info.to_string()))),
//             ),
//         };
//         let v = (NonZeroU16::new(v.0).unwrap(), v.1);
//
//         if let (Some(id), res) = v {
//             if let Some(mut call) = self.call_table.remove(&id) {
//                 call.resolve(res);
//             }
//         }
//     }
//
//     pub fn drop_expired(&mut self, id: NonZeroU16) {
//         if let Entry::Occupied(e) = self.call_table.entry(id) {
//             if e.get().is_expired() {
//                 e.remove();
//             }
//         }
//     }
// }