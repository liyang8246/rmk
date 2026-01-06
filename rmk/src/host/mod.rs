#[cfg(feature = "storage")]
pub(crate) mod storage;
#[cfg(feature = "rpc")]
pub mod rpc;
pub mod via;

use core::cell::RefCell;

// TODO: Remove those aliases
pub use via::UsbVialReaderWriter as UsbHostReaderWriter;
#[cfg(feature = "vial")]
pub(crate) use via::VialService as HostService;

#[cfg(feature = "vial")]
use crate::config::VialConfig;
use crate::descriptor::ViaReport;
#[cfg(feature = "rpc")]
use crate::descriptor::RpcReport;
use crate::hid::{HidReaderTrait, HidWriterTrait};
use crate::keymap::KeyMap;

#[cfg(feature = "vial")]
pub(crate) async fn run_host_communicate_task<
    'a,
    Rw: HidReaderTrait<ReportType = ViaReport> + HidWriterTrait<ReportType = ViaReport>,
    const ROW: usize,
    const COL: usize,
    const NUM_LAYER: usize,
    const NUM_ENCODER: usize,
>(
    keymap: &'a RefCell<KeyMap<'a, ROW, COL, NUM_LAYER, NUM_ENCODER>>,
    reader_writer: Rw,
    vial_config: VialConfig<'static>,
) {
    let mut service = HostService::new(keymap, vial_config, reader_writer);
    service.run().await
}

#[cfg(not(feature = "vial"))]
pub(crate) async fn run_host_communicate_task<
    'a,
    Rw: HidReaderTrait<ReportType = ViaReport> + HidWriterTrait<ReportType = ViaReport>,
    const ROW: usize,
    const COL: usize,
    const NUM_LAYER: usize,
    const NUM_ENCODER: usize,
>(
    _keymap: &'a RefCell<KeyMap<'a, ROW, COL, NUM_LAYER, NUM_ENCODER>>,
    _reader_writer: Rw,
) {
    todo!()
}

#[cfg(feature = "rpc")]
pub use rpc::UsbRpcReaderWriter;

#[cfg(feature = "rpc")]
pub(crate) async fn run_rpc_communicate_task<
    'a,
    Rw: HidReaderTrait<ReportType = RpcReport> + HidWriterTrait<ReportType = RpcReport>,
    const ROW: usize,
    const COL: usize,
    const NUM_LAYER: usize,
    const NUM_ENCODER: usize,
>(
    keymap: &'a RefCell<KeyMap<'a, ROW, COL, NUM_LAYER, NUM_ENCODER>>,
    reader_writer: Rw,
) {
    let mut service = rpc::RpcService::new(keymap, reader_writer);
    service.run().await
}
