#![no_std]
#![feature(exclusive_range_pattern)]

use core::convert::{Infallible, TryInto};
use embedded_graphics::{
    drawable::Pixel,
    geometry::Size,
    pixelcolor::{raw::RawU8, Bgr555, PixelColor},
    prelude::*,
};
use gba::{
    vram::{
        bitmap::{Mode3, Mode4, Mode5, Page},
        Tile4bpp, Tile8bpp,
    },
    Color,
};

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct PaletteColor(pub RawU8); // holds an index into a color palette

impl PaletteColor {
    pub const fn new(index: u8) -> Self {
        Self(RawU8::new(index))
    }

    // palette index 0 is transparent
    pub const TANSPARENT: Self = Self(RawU8::new(0));
}

impl PixelColor for PaletteColor {
    type Raw = RawU8;
}

impl From<RawU8> for PaletteColor {
    fn from(data: RawU8) -> Self {
        Self(data)
    }
}

impl From<PaletteColor> for RawU8 {
    fn from(value: PaletteColor) -> Self {
        value.0
    }
}

// contains one type of bitmap display
pub enum Display {
    Mode3,
    Mode4(Page),
    Mode5(Page),
}

impl DrawTarget<Bgr555> for Display {
    type Error = Infallible;

    fn draw_pixel(&mut self, pixel: Pixel<Bgr555>) -> Result<(), Self::Error> {
        let x = pixel.0.x as usize;
        let y = pixel.0.y as usize;
        let color = Color(pixel.1.into_storage());

        match self {
            Display::Mode3 => Mode3::write(x, y, color),
            Display::Mode4(_) => return Ok(()), // Mode4 uses PaletteColor(RawU8)
            Display::Mode5(page) => Mode5::write(*page, x, y, color),
        };

        Ok(())
    }

    fn size(&self) -> Size {
        let (w, h) = match self {
            Display::Mode3 => (Mode3::WIDTH, Mode3::HEIGHT),
            Display::Mode4(_) => (Mode4::WIDTH, Mode4::HEIGHT),
            Display::Mode5(_) => (Mode5::WIDTH, Mode5::HEIGHT),
        };

        Size::new(w as u32, h as u32)
    }

    fn clear(&mut self, color: Bgr555) -> Result<(), Self::Error> {
        let color = Color(color.into_storage());

        match self {
            Display::Mode3 => Mode3::dma_clear_to(color),
            Display::Mode4(_) => return Ok(()), // Mode4 uses PaletteColor(RawU8)
            Display::Mode5(page) => Mode5::dma_clear_to(*page, color),
        }

        Ok(())
    }
}

impl DrawTarget<PaletteColor> for Display {
    type Error = Infallible;

    fn draw_pixel(&mut self, pixel: Pixel<PaletteColor>) -> Result<(), Self::Error> {
        if let Display::Mode4(page) = self {
            Mode4::write(
                *page,
                pixel.0.x as usize,
                pixel.0.y as usize,
                pixel.1.into_storage(),
            );
        } else {
            return Ok(()); // Mode3 and Mode5 use Bgr555 color
        }

        Ok(())
    }

    fn size(&self) -> Size {
        let (w, h) = match self {
            Display::Mode3 => (Mode3::WIDTH, Mode3::HEIGHT),
            Display::Mode4(_) => (Mode4::WIDTH, Mode4::HEIGHT),
            Display::Mode5(_) => (Mode5::WIDTH, Mode5::HEIGHT),
        };

        Size::new(w as u32, h as u32)
    }

    fn clear(&mut self, color: PaletteColor) -> Result<(), Self::Error> {
        if let Display::Mode4(page) = self {
            Mode4::dma_clear_to(*page, color.into_storage());
        }

        Ok(())
    }
}

impl DrawTarget<PaletteColor> for Tile4bpp {
    type Error = Infallible;

    fn draw_pixel(&mut self, pixel: Pixel<PaletteColor>) -> Result<(), Self::Error> {
        if let Ok((x @ 0..8, y @ 0..8)) = pixel.0.try_into() {
            let index: u32 = x + (y * 8); // index into [u4; 64] array
            let word: &mut u32 = &mut self.0[index as usize / 8];
            *word &= !(0xF << ((index % 8) * 4)); // clear nibble
            *word |= (pixel.1.into_storage() as u32) << ((index % 8) * 4); // set nibble
        }
        Ok(())
    }

    fn size(&self) -> Size {
        Size::new(8, 8)
    }
}

impl DrawTarget<PaletteColor> for Tile8bpp {
    type Error = Infallible;

    fn draw_pixel(&mut self, pixel: Pixel<PaletteColor>) -> Result<(), Self::Error> {
        if let Ok((x @ 0..8, y @ 0..8)) = pixel.0.try_into() {
            let index: u32 = x + (y * 8); // index into [u8; 64] array
            let word: &mut u32 = &mut self.0[index as usize / 4];
            *word &= !(0xFF << ((index % 4) * 8)); // clear byte
            *word |= (pixel.1.into_storage() as u32) << ((index % 4) * 8); // set byte
        }
        Ok(())
    }

    fn size(&self) -> Size {
        Size::new(8, 8)
    }
}
