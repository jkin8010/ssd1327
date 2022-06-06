//! Bounce a DVD player logo around the screen
//!
//! Like this, but with no color changing: https://bouncingdvdlogo.com/
//!
//! For best results, run with the `--release` flag.

#![no_std]
#![no_main]

use display_interface_spi::SPIInterfaceNoCS;
use embedded_graphics::{
    geometry::Point,
    image::Image,
    pixelcolor::{BinaryColor, Rgb565},
    prelude::*,
    primitives::{PrimitiveStyle, Rectangle},
};
use panic_halt as _;
use rtic::app;
use ssd1306::{mode::BufferedGraphicsMode, prelude::*, Ssd1306};
use stm32f1xx_hal::{
    delay::Delay,
    gpio,
    pac::{self, SPI1},
    prelude::*,
    spi::{self, Mode, Phase, Polarity, Spi},
    timer::{CountDownTimer, Event, Timer},
};
use tinybmp::Bmp;

type Display = Ssd1306<
    SPIInterfaceNoCS<
        spi::Spi<
            SPI1,
            spi::Spi1NoRemap,
            (
                gpio::gpioa::PA5<gpio::Alternate<gpio::PushPull>>,
                gpio::gpioa::PA6<gpio::Input<gpio::Floating>>,
                gpio::gpioa::PA7<gpio::Alternate<gpio::PushPull>>,
            ),
            u8,
        >,
        gpio::gpiob::PB1<gpio::Output<gpio::PushPull>>,
    >,
    DisplaySize128x64,
    BufferedGraphicsMode<DisplaySize128x64>,
>;

#[app(device = stm32f1xx_hal::pac, peripherals = true)]
const APP: () = {
    struct Resources {
        display: Display,
        timer: CountDownTimer<pac::TIM1>,
        top_left: Point,
        velocity: Point,
        bmp: Bmp<Rgb565, 'static>,
    }

    #[init]
    fn init(cx: init::Context) -> init::LateResources {
        let dp = cx.device;
        let core = cx.core;

        let mut flash = dp.FLASH.constrain();
        let mut rcc = dp.RCC.constrain();

        let clocks = rcc
            .cfgr
            .use_hse(8.mhz())
            .sysclk(72.mhz())
            .pclk1(36.mhz())
            .freeze(&mut flash.acr);

        let mut afio = dp.AFIO.constrain(&mut rcc.apb2);

        let mut gpiob = dp.GPIOB.split(&mut rcc.apb2);
        let mut gpioa = dp.GPIOA.split(&mut rcc.apb2);

        // SPI1
        let sck = gpioa.pa5.into_alternate_push_pull(&mut gpioa.crl);
        let miso = gpioa.pa6;
        let mosi = gpioa.pa7.into_alternate_push_pull(&mut gpioa.crl);

        let mut delay = Delay::new(core.SYST, clocks);

        let mut rst = gpiob.pb0.into_push_pull_output(&mut gpiob.crl);
        let dc = gpiob.pb1.into_push_pull_output(&mut gpiob.crl);

        let spi = Spi::spi1(
            dp.SPI1,
            (sck, miso, mosi),
            &mut afio.mapr,
            Mode {
                polarity: Polarity::IdleLow,
                phase: Phase::CaptureOnFirstTransition,
            },
            8.mhz(),
            clocks,
            &mut rcc.apb2,
        );

        let interface = display_interface_spi::SPIInterfaceNoCS::new(spi, dc);
        let mut display = Ssd1306::new(interface, DisplaySize128x64, DisplayRotation::Rotate180)
            .into_buffered_graphics_mode();

        display.reset(&mut rst, &mut delay).unwrap();
        display.init().unwrap();

        // Update framerate
        let fps = 20;

        let mut timer = Timer::tim1(dp.TIM1, &clocks, &mut rcc.apb2).start_count_down(fps.hz());

        timer.listen(Event::Update);

        let bmp = Bmp::from_slice(include_bytes!("dvd.bmp")).unwrap();

        // Init the static resources to use them later through RTIC
        init::LateResources {
            timer,
            display,
            top_left: Point::new(5, 3),
            velocity: Point::new(1, 1),
            bmp,
        }
    }

    #[task(binds = TIM1_UP, resources = [display, top_left, velocity, timer, bmp])]
    fn update(cx: update::Context) {
        let update::Resources {
            display,
            top_left,
            velocity,
            timer,
            bmp,
            ..
        } = cx.resources;

        let bottom_right = *top_left + bmp.bounding_box().size;

        // Erase previous image position with a filled black rectangle
        Rectangle::with_corners(*top_left, bottom_right)
            .into_styled(PrimitiveStyle::with_fill(BinaryColor::Off))
            .draw(display)
            .unwrap();

        // Check if the image collided with a screen edge
        {
            if bottom_right.x > display.size().width as i32 || top_left.x < 0 {
                velocity.x = -velocity.x;
            }

            if bottom_right.y > display.size().height as i32 || top_left.y < 0 {
                velocity.y = -velocity.y;
            }
        }

        // Move the image
        *top_left += *velocity;

        // Draw image at new position
        Image::new(bmp, *top_left)
            .draw(&mut display.color_converted())
            .unwrap();

        // Write changes to the display
        display.flush().unwrap();

        // Clears the update flag
        timer.clear_update_interrupt_flag();
    }

    extern "C" {
        fn EXTI0();
    }
};
