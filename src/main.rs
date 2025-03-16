#![no_std]
#![no_main]

use core::num::{NonZeroU16, NonZeroU8};

use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::{bind_interrupts, Config};
use embassy_stm32::time::mhz;
use embassy_stm32::can::filter::Mask32;
use embassy_time::Timer;
use embassy_stm32::can::util::NominalBitTiming;
use embassy_stm32::can::{
    Can, Fifo, Frame, Rx0InterruptHandler, Rx1InterruptHandler, SceInterruptHandler, StandardId, TxInterruptHandler,
};
use embassy_stm32::peripherals::CAN;
use {defmt_rtt as _, panic_probe as _};

bind_interrupts!(struct Irqs {
    USB_LP_CAN_RX0 => Rx0InterruptHandler<CAN>;
    CAN_RX1 => Rx1InterruptHandler<CAN>;
    CAN_SCE => SceInterruptHandler<CAN>;
    USB_HP_CAN_TX => TxInterruptHandler<CAN>;
});

#[embassy_executor::main]
async fn main(_spawner: Spawner) -> ! {
    info!("Hello World!");
    let mut config = Config::default();
    {
        use embassy_stm32::rcc::*;
        config.rcc.hse = Some(Hse {
            freq: mhz(12),
            mode: HseMode::Oscillator,
        });
        config.rcc.pll = Some(Pll {
            src: PllSource::HSE,
            prediv: PllPreDiv::DIV1,
            mul: PllMul::MUL6,
        });
        config.rcc.sys = Sysclk::PLL1_P;
        config.rcc.ahb_pre = AHBPrescaler::DIV1;
        config.rcc.apb1_pre = APBPrescaler::DIV2;
        config.rcc.apb2_pre = APBPrescaler::DIV1;
    }

    let p = embassy_stm32::init(config);

    let mut can = Can::new(p.CAN, p.PA11, p.PA12, Irqs);

    can.modify_filters().enable_bank(0, Fifo::Fifo0, Mask32::accept_all());

    can.modify_config()
        .set_bit_timing(NominalBitTiming {
            prescaler: NonZeroU16::new(2).unwrap(),
            seg1: NonZeroU8::new(13).unwrap(),
            seg2: NonZeroU8::new(4).unwrap(),
            sync_jump_width: NonZeroU8::new(1).unwrap(),
        }) // http://www.bittiming.can-wiki.info/
        .set_loopback(false)
        .set_silent(false);

    can.enable().await;
    Timer::after_millis(10).await;

    let mut i: u8 = 0;

    loop {
        let tx_frame = Frame::new_data(unwrap!(StandardId::new(10 as _)), &[42, i]).unwrap();
        can.write(&tx_frame).await;

        let envelope = can.read().await.unwrap();
        i = envelope.frame.data()[0] + 1;
    }
}
