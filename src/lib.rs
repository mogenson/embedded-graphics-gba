#![no_std]
#![feature(exclusive_range_pattern)]

use core::convert::{Infallible, TryInto};
use embedded_graphics::{
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

pub struct Mode3Display;

impl DrawTarget for Mode3Display {
    type Color = Bgr555;
    type Error = Infallible;

    fn draw_iter<I>(&mut self, pixels: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = Pixel<Self::Color>>,
    {
        for Pixel(coord, color) in pixels.into_iter() {
            Mode3::write(
                coord.x as usize,
                coord.y as usize,
                Color(color.into_storage()),
            );
        }

        Ok(())
    }

    fn size(&self) -> Size {
        Size::new(Mode3::WIDTH as u32, Mode3::HEIGHT as u32)
    }

    fn clear(&mut self, color: Self::Color) -> Result<(), Self::Error> {
        Mode3::dma_clear_to(Color(color.into_storage()));
        Ok(())
    }
}

pub struct Mode4Display {
    pub page: Page,
}

impl DrawTarget for Mode4Display {
    type Color = PaletteColor;
    type Error = Infallible;

    fn draw_iter<I>(&mut self, pixels: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = Pixel<Self::Color>>,
    {
        for Pixel(coord, color) in pixels.into_iter() {
            Mode4::write(
                self.page,
                coord.x as usize,
                coord.y as usize,
                color.into_storage(),
            );
        }

        Ok(())
    }

    fn size(&self) -> Size {
        Size::new(Mode4::WIDTH as u32, Mode4::HEIGHT as u32)
    }

    fn clear(&mut self, color: Self::Color) -> Result<(), Self::Error> {
        Mode4::dma_clear_to(self.page, color.into_storage());
        Ok(())
    }
}

pub struct Mode5Display {
    pub page: Page,
}

impl DrawTarget for Mode5Display {
    type Color = Bgr555;
    type Error = Infallible;

    fn draw_iter<I>(&mut self, pixels: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = Pixel<Self::Color>>,
    {
        for Pixel(coord, color) in pixels.into_iter() {
            Mode5::write(
                self.page,
                coord.x as usize,
                coord.y as usize,
                Color(color.into_storage()),
            );
        }
        Ok(())
    }

    fn size(&self) -> Size {
        Size::new(Mode5::WIDTH as u32, Mode5::HEIGHT as u32)
    }

    fn clear(&mut self, color: Self::Color) -> Result<(), Self::Error> {
        Mode5::dma_clear_to(self.page, Color(color.into_storage()));
        Ok(())
    }
}

pub struct Tile4bppDisplay {
    pub tile: Tile4bpp,
}

impl Tile4bppDisplay {
    pub fn new(color: PaletteColor) -> Self {
        Tile4bppDisplay {
            tile: Tile4bpp([color.into_storage().into(); 8]),
        }
    }
}

impl DrawTarget for Tile4bppDisplay {
    type Color = PaletteColor;
    type Error = Infallible;

    fn draw_iter<I>(&mut self, pixels: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = Pixel<Self::Color>>,
    {
        for Pixel(coord, color) in pixels.into_iter() {
            if let Ok((x @ 0..8, y @ 0..8)) = coord.try_into() {
                let index: u32 = x + (y * 8); // index into [u4; 64] array
                let word: &mut u32 = &mut self.tile.0[index as usize / 8];
                *word &= !(0xF << ((index % 8) * 4)); // clear nibble
                *word |= (color.into_storage() as u32) << ((index % 8) * 4); // set nibble
            }
        }
        Ok(())
    }

    fn size(&self) -> Size {
        Size::new(8, 8)
    }
}

pub struct Tile8bppDisplay {
    pub tile: Tile8bpp,
}

impl Tile8bppDisplay {
    pub fn new(color: PaletteColor) -> Self {
        Tile8bppDisplay {
            tile: Tile8bpp([color.into_storage().into(); 16]),
        }
    }
}

impl DrawTarget for Tile8bppDisplay {
    type Color = PaletteColor;
    type Error = Infallible;

    fn draw_iter<I>(&mut self, pixels: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = Pixel<Self::Color>>,
    {
        for Pixel(coord, color) in pixels.into_iter() {
            if let Ok((x @ 0..8, y @ 0..8)) = coord.try_into() {
                let index: u32 = x + (y * 8); // index into [u8; 64] array
                let word: &mut u32 = &mut self.tile.0[index as usize / 4];
                *word &= !(0xFF << ((index % 4) * 8)); // clear byte
                *word |= (color.into_storage() as u32) << ((index % 4) * 8); // set byte
            }
        }
        Ok(())
    }

    fn size(&self) -> Size {
        Size::new(8, 8)
    }
}
