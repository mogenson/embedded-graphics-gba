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

pub struct Mode3Display();

impl DrawTarget<Bgr555> for Mode3Display {
    type Error = Infallible;

    fn draw_pixel(&mut self, pixel: Pixel<Bgr555>) -> Result<(), Self::Error> {
        Mode3::write(
            pixel.0.x as usize,
            pixel.0.y as usize,
            Color(pixel.1.into_storage()),
        );
        Ok(())
    }

    fn size(&self) -> Size {
        Size::new(Mode3::WIDTH as u32, Mode3::HEIGHT as u32)
    }

    fn clear(&mut self, color: Bgr555) -> Result<(), Self::Error> {
        Mode3::dma_clear_to(Color(color.into_storage()));
        Ok(())
    }
}

pub struct Mode4Display(Page);

impl DrawTarget<PaletteColor> for Mode4Display {
    type Error = Infallible;

    fn draw_pixel(&mut self, pixel: Pixel<PaletteColor>) -> Result<(), Self::Error> {
        Mode4::write(
            self.0,
            pixel.0.x as usize,
            pixel.0.y as usize,
            pixel.1.into_storage(),
        );

        Ok(())
    }

    fn size(&self) -> Size {
        Size::new(Mode4::WIDTH as u32, Mode4::HEIGHT as u32)
    }

    fn clear(&mut self, color: PaletteColor) -> Result<(), Self::Error> {
        Mode4::dma_clear_to(self.0, color.into_storage());
        Ok(())
    }
}

pub struct Mode5Display(Page);

impl DrawTarget<Bgr555> for Mode5Display {
    type Error = Infallible;

    fn draw_pixel(&mut self, pixel: Pixel<Bgr555>) -> Result<(), Self::Error> {
        Mode5::write(
            self.0,
            pixel.0.x as usize,
            pixel.0.y as usize,
            Color(pixel.1.into_storage()),
        );
        Ok(())
    }

    fn size(&self) -> Size {
        Size::new(Mode5::WIDTH as u32, Mode5::HEIGHT as u32)
    }

    fn clear(&mut self, color: Bgr555) -> Result<(), Self::Error> {
        Mode5::dma_clear_to(self.0, Color(color.into_storage()));
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
