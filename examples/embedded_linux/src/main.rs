// the library for the embedded linux device
extern crate linux_embedded_hal as lin_hal;

// the eink library
extern crate eink_waveshare_rs;


use eink_waveshare_rs::{epd4in2::EPD4in2, drawing::{Graphics, Color}};

use lin_hal::spidev::{self, SpidevOptions};
use lin_hal::{Pin, Spidev};
use lin_hal::sysfs_gpio::Direction;
use lin_hal::Delay;

// activate spi, gpio in raspi-config
// needs to be run with sudo because of some sysfs_gpio permission problems and follow-up timing problems
// see https://github.com/rust-embedded/rust-sysfs-gpio/issues/5 and follow-up issues


// DigitalIn Hack as long as it's not in the linux_embedded_hal
// from https://github.com/rudihorn/max31865/blob/extra_examples/examples/rpi.rs
// (slightly changed now as OutputPin doesn't provide is_high and is_low anymore)
extern crate embedded_hal;
use embedded_hal::digital::{InputPin};

struct HackInputPin<'a> {
    pin: &'a Pin
}

impl<'a> HackInputPin<'a> {
    fn new(p : &'a Pin) -> HackInputPin {
        HackInputPin {
            pin: p
        }
    }
}

//TODO: make it safer?? or handle the errors better?
// now it defaults to is_low if an error appears
impl<'a> InputPin for HackInputPin<'a> {
    fn is_low(&self) -> bool {
        self.pin.get_value().unwrap_or(0) == 0
    }

    fn is_high(&self) -> bool {
        self.pin.get_value().unwrap_or(0) == 1
    }
}


/*
*
* BE CAREFUL: this wasn't tested yet, and the pins are also not choosen correctly (just some random ones atm)
*
*/

fn main() {

    // Configure SPI
    let mut spi = Spidev::open("/dev/spidev0.0").expect("spidev directory");
    let options = SpidevOptions::new()
        .bits_per_word(8)
        .max_speed_hz(1_000_000)
        .mode(spidev::SPI_MODE_0)
        .build();
    spi.configure(&options).expect("spi configuration");

    // Configure Digital I/O Pin to be used as Chip Select for SPI
    let cs = Pin::new(26);//BCM7 CE0
    cs.export().expect("cs export");
    while !cs.is_exported() {}
    cs.set_direction(Direction::Out).expect("CS Direction");
    cs.set_value(1).expect("CS Value set to 1");

    let busy = Pin::new(5);//pin 29
    busy.export().expect("busy export");
    while !busy.is_exported() {}
    busy.set_direction(Direction::In).expect("busy Direction");
    //busy.set_value(1).expect("busy Value set to 1");
    let busy_in = HackInputPin::new(&busy);

    let dc = Pin::new(6); //pin 31 //bcm6
    dc.export().expect("dc export");
    while !dc.is_exported() {}
    dc.set_direction(Direction::Out).expect("dc Direction");
    dc.set_value(1).expect("dc Value set to 1");

    let rst = Pin::new(16); //pin 36 //bcm16
    rst.export().expect("rst export");
    while !rst.is_exported() {}
    rst.set_direction(Direction::Out).expect("rst Direction");
    rst.set_value(1).expect("rst Value set to 1");   

    let delay = Delay {};

    let white = Color::White;
    let black = Color::Black;
 
    

    //TODO: wait for Digital::InputPin
    //fixed currently with the HackInputPin, see further above
    let mut epd4in2 = EPD4in2::new(spi, cs, busy_in, dc, rst, delay).expect("eink inialize error");

    //let mut buffer =  [0u8, epd4in2.get_width() / 8 * epd4in2.get_height()];
    let mut buffer = [0u8; 15000];

    // draw something
    let graphics = Graphics::new(400, 300);
    graphics.clear(&mut buffer, &white);
    graphics.draw_line(&mut buffer, 0,0,400,300, &Color::Black); 

    graphics.draw_filled_rectangle(&mut buffer, 200,200, 230, 230, &Color::Black); 
    graphics.draw_line(&mut buffer, 202,202,218,228, &Color::White);

    graphics.draw_circle(&mut buffer, 200, 150, 130, &Color::Black);

    graphics.draw_pixel(&mut buffer, 390, 290, &Color::Black);

    graphics.draw_horizontal_line(&mut buffer, 0, 150, 400, &Color::Black);

    graphics.draw_vertical_line(&mut buffer, 200, 50, 200, &Color::Black);

    epd4in2.display_and_transfer_frame(&buffer, None).expect("display and transfer error");
 
    epd4in2.delay_ms(3000);

    epd4in2.clear_frame(None).expect("clear frame error");

    //Test fast updating a bit more
    let mut small_buffer = [0x00; 128];
    let circle_graphics = Graphics::new(32,32);
    graphics.draw_circle(&mut small_buffer, 16,16, 10, &Color::Black);

    epd4in2.set_partial_window(&small_buffer, 16,16, 32, 32, false);
    epd4in2.display_frame();

    epd4in2.set_partial_window(&small_buffer, 128,64, 32, 32, false);
    epd4in2.display_frame();

    epd4in2.set_partial_window(&small_buffer, 320,24, 32, 32, false);
    epd4in2.display_frame();

    epd4in2.set_partial_window(&small_buffer, 160,240, 32, 32, false);



    epd4in2.display_frame();

    epd4in2.delay_ms(3000);
    //pub fn draw_string(&self, buffer: &mut[u8], x0: u16, y0: u16, input: String, color: &Color) {
    graphics.draw_string(&mut buffer, 16, 16, &['h', 'e', 'l', 'l', 'o'], &Color::Black);
    epd4in2.display_and_transfer_frame(&buffer, None).expect("display and transfer error");

    epd4in2.delay_ms(3000);

    

    epd4in2.clear_frame(None).expect("clear frame error");

    epd4in2.sleep().expect("sleeping error");
}