use core::cell::RefCell;

use embassy_time::Timer;
use embassy_usb::{class::hid::HidReaderWriter, driver::Driver};

use crate::descriptor::RpcReport;
use crate::hid::{HidError, HidReaderTrait, HidWriterTrait};
use crate::keymap::KeyMap;
use crate::state::{CONNECTION_STATE, ConnectionState};

use ssmarshal::serialize;

#[derive(PartialEq, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum RpcError {
    HidError(HidError),
}

impl From<HidError> for RpcError {
    fn from(value: HidError) -> Self {
        Self::HidError(value)
    }
}

pub(crate) struct RpcService<
    'a,
    RW: HidWriterTrait<ReportType = RpcReport> + HidReaderTrait<ReportType = RpcReport>,
    const ROW: usize,
    const COL: usize,
    const NUM_LAYER: usize,
    const NUM_ENCODER: usize,
> {
    keymap: &'a RefCell<KeyMap<'a, ROW, COL, NUM_LAYER, NUM_ENCODER>>,
    pub(crate) reader_writer: RW,
}

impl<
    'a,
    RW: HidWriterTrait<ReportType = RpcReport> + HidReaderTrait<ReportType = RpcReport>,
    const ROW: usize,
    const COL: usize,
    const NUM_LAYER: usize,
    const NUM_ENCODER: usize,
> RpcService<'a, RW, ROW, COL, NUM_LAYER, NUM_ENCODER>
{
    pub(crate) fn new(
        keymap: &'a RefCell<KeyMap<'a, ROW, COL, NUM_LAYER, NUM_ENCODER>>,
        reader_writer: RW,
    ) -> Self {
        Self { keymap, reader_writer }
    }

    pub(crate) async fn run(&mut self) {
        loop {
            match self.process().await {
                Ok(_) => continue,
                Err(e) => {
                    if ConnectionState::Disconnected == ConnectionState::from(&CONNECTION_STATE) {
                        Timer::after_millis(1000).await;
                    } else {
                        error!("Process rpc error: {:?}", e);
                        Timer::after_millis(10000).await;
                    }
                }
            }
        }
    }

    pub(crate) async fn process(&mut self) -> Result<(), RpcError> {
        let mut report = self.reader_writer.read_report().await?;
        // TODO: 实际的 RPC 处理逻辑将在后续添加
        report.input_data = [0; 32];
        self.reader_writer.write_report(report).await?;
        Ok(())
    }
}

pub struct UsbRpcReaderWriter<'a, 'd, D: Driver<'d>> {
    pub(crate) reader_writer: &'a mut HidReaderWriter<'d, D, 32, 32>,
}

impl<'a, 'd, D: Driver<'d>> UsbRpcReaderWriter<'a, 'd, D> {
    pub(crate) fn new(reader_writer: &'a mut HidReaderWriter<'d, D, 32, 32>) -> Self {
        Self { reader_writer }
    }
}

impl<'d, D: Driver<'d>> HidWriterTrait for UsbRpcReaderWriter<'_, 'd, D> {
    type ReportType = RpcReport;

    async fn write_report(&mut self, report: Self::ReportType) -> Result<usize, HidError> {
        let mut buffer = [0u8; 32];
        let n = serialize(&mut buffer, &report).map_err(|_| HidError::ReportSerializeError)?;
        self.reader_writer
            .write(&buffer[0..n])
            .await
            .map_err(HidError::UsbEndpointError)?;
        Ok(n)
    }
}

impl<'d, D: Driver<'d>> HidReaderTrait for UsbRpcReaderWriter<'_, 'd, D> {
    type ReportType = RpcReport;

    async fn read_report(&mut self) -> Result<Self::ReportType, HidError> {
        let mut read_report = RpcReport {
            input_data: [0; 32],
            output_data: [0; 32],
        };
        self.reader_writer
            .read(&mut read_report.output_data)
            .await
            .map_err(HidError::UsbReadError)?;
        Ok(read_report)
    }
}
