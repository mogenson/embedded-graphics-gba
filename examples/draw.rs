#![no_std]
#![feature(start)]
#![forbid(unsafe_code)]
#![feature(exclusive_range_pattern)]
#![feature(bindings_after_at)]

use embedded_graphics_gba::{Mode3Display, PaletteColor, Tile8bppDisplay};

use core::convert::{Infallible, TryInto};

use embedded_graphics::{
    fonts::{Font6x8, Text},
    image::Image,
    pixelcolor::Bgr555,
    prelude::*,
    primitives::{Rectangle, Triangle},
    style::{PrimitiveStyle, TextStyle},
};

use gba::{
    fatal,
    io::{
        display::{DisplayControlSetting, DisplayMode, DisplayStatusSetting, DISPCNT, DISPSTAT},
        irq::{set_irq_handler, IrqEnableSetting, IrqFlags, BIOS_IF, IE, IF, IME},
        keypad::read_key_input,
    },
    oam::{write_obj_attributes, OBJAttr0, OBJAttr1, OBJAttr2, ObjectAttributes},
    palram::index_palram_obj_8bpp,
    vram::get_8bpp_character_block,
    Color,
};

use tinytga::Tga;

// color palette table
const COLORS: [Bgr555; 8] = [
    Bgr555::WHITE,
    Bgr555::BLACK,
    Bgr555::RED,
    Bgr555::GREEN,
    Bgr555::BLUE,
    Bgr555::YELLOW,
    Bgr555::MAGENTA,
    Bgr555::CYAN,
];

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    fatal!("{}", info);
    loop {}
}

#[start]
fn main(_argc: isize, _argv: *const *const u8) -> isize {
    // setup display hardware
    DISPCNT.write(
        DisplayControlSetting::new()
            .with_mode(DisplayMode::Mode3) // bitmap
            .with_bg2(true) // use background
            .with_obj(true) // use sprites
            .with_oam_memory_1d(true) // 1 dimensional vram mapping
            .with_force_vblank(true), // disable display
    );

    // setup embedded graphics display
    let mut display = Mode3Display;
    draw_canvas(&mut display).ok();
    draw_text(&mut display).ok();
    register_palette();
    draw_cursor().ok();

    // setup interrupts
    set_irq_handler(irq_handler);
    DISPSTAT.write(DisplayStatusSetting::new().with_vblank_irq_enable(true));
    IE.write(IrqFlags::new().with_vblank(true));
    IME.write(IrqEnableSetting::IRQ_YES);
    DISPCNT.write(DISPCNT.read().with_force_vblank(false)); // enable display

    // state variables
    let mut point = Point::new(120, 80); // start in middle of display
    let mut index = 0; // index into color palette

    loop {
        // sleep until vblank interrupt
        gba::bios::vblank_interrupt_wait();

        // read buttons input
        let input = read_key_input();

        // clear display
        if input.start() {
            draw_canvas(&mut display).ok();
            draw_text(&mut display).ok();
            continue;
        }

        // cycle cursor
        if input.b() {
            index += 1;
            if index >= COLORS.len() {
                index = 0;
            }
            while read_key_input().b() {
                // wait for button to be released
                gba::bios::vblank_interrupt_wait();
            }
        }

        // update point
        let offset = Point::new(input.x_tribool() as i32, input.y_tribool() as i32);
        point += offset;

        // draw cursor and pixel
        if let Ok((x @ 0..240, y @ 0..160)) = point.try_into() {
            move_cursor(index as u16, x as u16, y as u16);
            if input.a() {
                Pixel(Point::new(x as i32, y as i32), COLORS[index])
                    .draw(&mut display)
                    .ok();
            }
        } else {
            point -= offset; // undo
        }
    }
}

extern "C" fn irq_handler(flags: IrqFlags) {
    if flags.vblank() {
        // need to clear vblank flag in bios and hardware
        BIOS_IF.write(BIOS_IF.read().with_vblank(true));
        IF.write(IF.read().with_vblank(true));
    }
}

fn draw_canvas<D>(display: &mut D) -> Result<(), D::Error>
where
    D: DrawTarget<Color = Bgr555>,
{
    let tga = Tga::from_slice(include_bytes!("../assets/background.tga")).unwrap();
    let image: Image<_, Bgr555> = Image::new(&tga, Point::zero());
    image.draw(display)?;
    Ok(())
}

fn draw_text<D>(display: &mut D) -> Result<(), D::Error>
where
    D: DrawTarget<Color = Bgr555>,
{
    Rectangle::new(Point::new(0, 0), Size::new(49, 25))
        .into_styled(PrimitiveStyle::with_fill(Bgr555::WHITE))
        .draw(display)?;
    Text::new("A: Draw", Point::new(1, 1))
        .into_styled(TextStyle::new(Font6x8, Bgr555::RED))
        .draw(display)?;
    Text::new("B: Color", Point::new(1, 9))
        .into_styled(TextStyle::new(Font6x8, Bgr555::GREEN))
        .draw(display)?;
    Text::new("S: Clear", Point::new(1, 17))
        .into_styled(TextStyle::new(Font6x8, Bgr555::BLUE))
        .draw(display)?;
    Ok(())
}

fn register_palette() {
    // slot 0 is for transparency
    for (i, color) in COLORS.iter().enumerate() {
        index_palram_obj_8bpp(i as u8 + 1).write(Color(color.into_storage()));
    }
}

fn draw_cursor() -> Result<(), Infallible> {
    let mut tile = Tile8bppDisplay::new(PaletteColor::TANSPARENT);

    for i in 0..COLORS.len() {
        Triangle::new(Point::new(0, 0), Point::new(7, 4), Point::new(4, 7))
            .into_styled(PrimitiveStyle::with_fill(PaletteColor::new(i as u8 + 1)))
            .draw(&mut tile)?;
        get_8bpp_character_block(5).index(i).write(tile.tile);
    }

    Ok(())
}

fn move_cursor(index: u16, x: u16, y: u16) {
    write_obj_attributes(
        0, // overwritting object 0
        ObjectAttributes {
            attr0: OBJAttr0::new().with_row_coordinate(y).with_is_8bpp(true),
            attr1: OBJAttr1::new().with_col_coordinate(x),
            // Mode3 tiles start at 512 and 8bpp tiles are even with 2x width
            attr2: OBJAttr2::new().with_tile_id(512 + (index * 2)),
        },
    );
}
