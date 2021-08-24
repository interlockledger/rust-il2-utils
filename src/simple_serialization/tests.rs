/*
 * BSD 3-Clause License
 *
 * Copyright (c) 2019-2020, InterlockLedger Network
 * All rights reserved.
 *
 * Redistribution and use in source and binary forms, with or without
 * modification, are permitted provided that the following conditions are met:
 *
 * * Redistributions of source code must retain the above copyright notice, this
 *   list of conditions and the following disclaimer.
 *
 * * Redistributions in binary form must reproduce the above copyright notice,
 *   this list of conditions and the following disclaimer in the documentation
 *   and/or other materials provided with the distribution.
 *
 * * Neither the name of the copyright holder nor the names of its
 *   contributors may be used to endorse or promote products derived from
 *   this software without specific prior written permission.
 *
 * THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS "AS IS"
 * AND ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT LIMITED TO, THE
 * IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR A PARTICULAR PURPOSE ARE
 * DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT HOLDER OR CONTRIBUTORS BE LIABLE
 * FOR ANY DIRECT, INDIRECT, INCIDENTAL, SPECIAL, EXEMPLARY, OR CONSEQUENTIAL
 * DAMAGES (INCLUDING, BUT NOT LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR
 * SERVICES; LOSS OF USE, DATA, OR PROFITS; OR BUSINESS INTERRUPTION) HOWEVER
 * CAUSED AND ON ANY THEORY OF LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY,
 * OR TORT (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE
 * OF THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.
 */
use super::*;

//=============================================================================
// Test error remaping
//-----------------------------------------------------------------------------
#[derive(Debug)]
enum MyErrorkind {
    MyError,
}

type Result<T> = std::result::Result<T, MyErrorkind>;

impl From<super::ErrorKind> for MyErrorkind {
    fn from(_error: super::ErrorKind) -> Self {
        MyErrorkind::MyError
    }
}

fn translation() -> Result<()> {
    let mut buff: [u8; 0] = [];
    let mut serializer = SimpleSliceSerializer::new(&mut buff);
    serializer.write_u8(0)?;
    Ok(())
}

#[test]
fn test_error_conversion() {
    match translation() {
        Err(MyErrorkind::MyError) => (),
        Ok(()) => panic!("Error expected!"),
    }
}

//=============================================================================
// Samples
//-----------------------------------------------------------------------------
const SAMPLE: &'static [u8; 50] = &[
    0x36, 0x57, 0x2c, 0x7d, 0xe4, 0x48, 0xb2, 0x70, 0xc4, 0x87, 0x47, 0x46, 0xab, 0x46, 0x67, 0x5f,
    0x43, 0xea, 0xee, 0xf6, 0xe5, 0x5a, 0xf7, 0x0e, 0x57, 0xb5, 0x60, 0xa6, 0x8f, 0x81, 0x66, 0x42,
    0x11, 0x40, 0x49, 0x0f, 0xdb, 0x40, 0x09, 0x21, 0xFB, 0x54, 0x44, 0x2D, 0x18, 0x00, 0x03, 0x19,
    0x26, 0x39,
];

const SAMPLE00: &'static [u8; 3] = &[0x36, 0x57, 0x2c];
const SAMPLE01: u8 = 0x7d;
const SAMPLE02: u16 = 0xe448;
const SAMPLE03: u32 = 0xb270c487;
const SAMPLE04: u64 = 0x4746ab46675f43ea;
const SAMPLE05: i8 = -18;
const SAMPLE06: i16 = -2331;
const SAMPLE07: i32 = 0x5af70e57 as i32;
const SAMPLE08: i64 = -5377114819798875631i64;
const SAMPLE09: f32 = 3.14159274101257324;
const SAMPLE10: f64 = 3.141592653589793;
const SAMPLE11: &'static [u8; 3] = &[0x19, 0x26, 0x39];

//=============================================================================
// Samples
//-----------------------------------------------------------------------------
#[test]
fn test_simpledataserializer_vec() {
    let mut v = Vec::<u8>::new();

    v.write(SAMPLE00).unwrap();
    v.write_u8(SAMPLE01).unwrap();
    v.write_u16(SAMPLE02).unwrap();
    v.write_u32(SAMPLE03).unwrap();
    v.write_u64(SAMPLE04).unwrap();
    v.write_i8(SAMPLE05).unwrap();
    v.write_i16(SAMPLE06).unwrap();
    v.write_i32(SAMPLE07).unwrap();
    v.write_i64(SAMPLE08).unwrap();
    v.write_f32(SAMPLE09).unwrap();
    v.write_f64(SAMPLE10).unwrap();
    v.write_byte_array(SAMPLE11).unwrap();
    assert_eq!(v.as_slice(), SAMPLE);
}

//=============================================================================
// SimpleSliceSerializer
//-----------------------------------------------------------------------------
#[test]
fn test_simplesliceserializer_impl() {
    let mut data: [u8; 4] = [0; 4];
    let mut v = SimpleSliceSerializer::new(&mut data);

    for i in 0..4 {
        v.offset = i;
        assert_eq!(v.offset(), i);
        assert_eq!(v.available(), 4 - i);
        assert_eq!(v.offset(), i);
        for r in 0..v.available() {
            v.can_write(r).unwrap();
        }
        assert!(matches!(
            v.can_write(v.available() + 1),
            Err(ErrorKind::UnableToWrite)
        ));
    }
}

#[test]
fn test_simplesliceserializer_simpledataserializer_write() {
    let mut data: [u8; 50] = [0; 50];
    let mut v = SimpleSliceSerializer::new(&mut data);

    v.write(SAMPLE00).unwrap();
    v.write_u8(SAMPLE01).unwrap();
    v.write_u16(SAMPLE02).unwrap();
    v.write_u32(SAMPLE03).unwrap();
    v.write_u64(SAMPLE04).unwrap();
    v.write_i8(SAMPLE05).unwrap();
    v.write_i16(SAMPLE06).unwrap();
    v.write_i32(SAMPLE07).unwrap();
    v.write_i64(SAMPLE08).unwrap();
    v.write_f32(SAMPLE09).unwrap();
    v.write_f64(SAMPLE10).unwrap();
    v.write_byte_array(SAMPLE11).unwrap();
    assert_eq!(&data, SAMPLE);
}

#[test]
fn test_simplesliceserializer_simpledataserializer_write_fail() {
    let mut data: [u8; 2] = [0; 2];
    let mut v = SimpleSliceSerializer::new(&mut data);
    assert!(matches!(v.write(SAMPLE00), Err(ErrorKind::UnableToWrite)));

    let mut data: [u8; 0] = [];
    let mut v = SimpleSliceSerializer::new(&mut data);
    assert!(matches!(
        v.write_u8(SAMPLE01),
        Err(ErrorKind::UnableToWrite)
    ));
    assert!(matches!(
        v.write_i8(SAMPLE05),
        Err(ErrorKind::UnableToWrite)
    ));

    let mut data: [u8; 1] = [0];
    let mut v = SimpleSliceSerializer::new(&mut data);
    assert!(matches!(
        v.write_u16(SAMPLE02),
        Err(ErrorKind::UnableToWrite)
    ));
    assert!(matches!(
        v.write_i16(SAMPLE06),
        Err(ErrorKind::UnableToWrite)
    ));

    let mut data: [u8; 3] = [0; 3];
    let mut v = SimpleSliceSerializer::new(&mut data);
    assert!(matches!(
        v.write_u32(SAMPLE03),
        Err(ErrorKind::UnableToWrite)
    ));
    assert!(matches!(
        v.write_i32(SAMPLE07),
        Err(ErrorKind::UnableToWrite)
    ));
    assert!(matches!(
        v.write_f32(SAMPLE09),
        Err(ErrorKind::UnableToWrite)
    ));

    let mut data: [u8; 7] = [0; 7];
    let mut v = SimpleSliceSerializer::new(&mut data);
    assert!(matches!(
        v.write_u64(SAMPLE04),
        Err(ErrorKind::UnableToWrite)
    ));
    assert!(matches!(
        v.write_i64(SAMPLE08),
        Err(ErrorKind::UnableToWrite)
    ));
    assert!(matches!(
        v.write_f64(SAMPLE10),
        Err(ErrorKind::UnableToWrite)
    ));

    let mut data: [u8; 4] = [0; 4];
    let mut v = SimpleSliceSerializer::new(&mut data);
    assert!(matches!(
        v.write_byte_array(SAMPLE11),
        Err(ErrorKind::UnableToWrite)
    ));
}

//=============================================================================
// SimpleSliceDeserializer
//-----------------------------------------------------------------------------
#[test]
fn test_simpleslicedeserializer_impl() {
    let data: [u8; 4] = [0; 4];
    let mut v = SimpleSliceDeserializer::new(&data);

    for i in 0..4 {
        v.offset = i;
        assert_eq!(v.offset(), i);
        assert_eq!(v.avaliable(), 4 - i);
        for r in 0..v.avaliable() {
            v.can_read(r).unwrap();
        }
        assert!(matches!(
            v.can_read(v.avaliable() + 1),
            Err(ErrorKind::UnableToRead)
        ));
    }
}

#[test]
fn test_simpleslicedeserializer_simpledeserializer_read() {
    let mut v = SimpleSliceDeserializer::new(&*SAMPLE);

    let mut offs = 0;
    let size = 3;
    v.read(SAMPLE00.len()).unwrap();
    assert_eq!(SAMPLE00, v.data());
    offs += size;

    let size = 1;
    assert_eq!(v.read_u8().unwrap(), SAMPLE01);
    assert_eq!(&SAMPLE[offs..offs + size], v.data());
    offs += size;

    let size = 2;
    assert_eq!(v.read_u16().unwrap(), SAMPLE02);
    assert_eq!(&SAMPLE[offs..offs + size], v.data());
    offs += size;

    let size = 4;
    assert_eq!(v.read_u32().unwrap(), SAMPLE03);
    assert_eq!(&SAMPLE[offs..offs + size], v.data());
    offs += size;

    let size = 8;
    assert_eq!(v.read_u64().unwrap(), SAMPLE04);
    assert_eq!(&SAMPLE[offs..offs + size], v.data());
    offs += size;

    let size = 1;
    assert_eq!(v.read_i8().unwrap(), SAMPLE05);
    assert_eq!(&SAMPLE[offs..offs + size], v.data());
    offs += size;

    let size = 2;
    assert_eq!(v.read_i16().unwrap(), SAMPLE06);
    assert_eq!(&SAMPLE[offs..offs + size], v.data());
    offs += size;

    let size = 4;
    assert_eq!(v.read_i32().unwrap(), SAMPLE07);
    assert_eq!(&SAMPLE[offs..offs + size], v.data());
    offs += size;

    let size = 8;
    assert_eq!(v.read_i64().unwrap(), SAMPLE08);
    assert_eq!(&SAMPLE[offs..offs + size], v.data());
    offs += size;

    let size = 4;
    assert_eq!(v.read_f32().unwrap(), SAMPLE09);
    assert_eq!(&SAMPLE[offs..offs + size], v.data());
    offs += size;

    let size = 8;
    assert_eq!(v.read_f64().unwrap(), SAMPLE10);
    assert_eq!(&SAMPLE[offs..offs + size], v.data());
    offs += size;

    let size = 3;
    offs += 2;
    v.read_byte_array().unwrap();
    assert_eq!(&SAMPLE[offs..offs + size], v.data());
    offs += size;

    assert_eq!(v.offset(), offs);
}

#[test]
fn test_simpleslicedeserializer_simpledeserializer_read_fail() {
    let s: [u8; 8] = [0, 3, 0, 0, 0, 0, 0, 0];

    let mut v = SimpleSliceDeserializer::new(&s[..2]);
    assert!(matches!(v.read(3), Err(ErrorKind::UnableToRead)));
    assert_eq!(v.offset, 0);
    assert_eq!(v.data_offset, 0);

    let mut v = SimpleSliceDeserializer::new(&s[..0]);
    assert!(matches!(v.read_u8(), Err(ErrorKind::UnableToRead)));
    assert_eq!(v.offset, 0);
    assert_eq!(v.data_offset, 0);
    assert!(matches!(v.read_i8(), Err(ErrorKind::UnableToRead)));
    assert_eq!(v.offset, 0);
    assert_eq!(v.data_offset, 0);

    let mut v = SimpleSliceDeserializer::new(&s[..1]);
    assert!(matches!(v.read_u16(), Err(ErrorKind::UnableToRead)));
    assert_eq!(v.offset, 0);
    assert_eq!(v.data_offset, 0);
    assert!(matches!(v.read_i16(), Err(ErrorKind::UnableToRead)));
    assert_eq!(v.offset, 0);
    assert_eq!(v.data_offset, 0);

    let mut v = SimpleSliceDeserializer::new(&s[..3]);
    assert!(matches!(v.read_u32(), Err(ErrorKind::UnableToRead)));
    assert_eq!(v.offset, 0);
    assert_eq!(v.data_offset, 0);
    assert!(matches!(v.read_i32(), Err(ErrorKind::UnableToRead)));
    assert_eq!(v.offset, 0);
    assert_eq!(v.data_offset, 0);
    assert!(matches!(v.read_f32(), Err(ErrorKind::UnableToRead)));
    assert_eq!(v.offset, 0);
    assert_eq!(v.data_offset, 0);

    let mut v = SimpleSliceDeserializer::new(&s[..7]);
    assert!(matches!(v.read_u64(), Err(ErrorKind::UnableToRead)));
    assert_eq!(v.offset, 0);
    assert_eq!(v.data_offset, 0);
    assert!(matches!(v.read_i64(), Err(ErrorKind::UnableToRead)));
    assert_eq!(v.offset, 0);
    assert_eq!(v.data_offset, 0);
    assert!(matches!(v.read_f64(), Err(ErrorKind::UnableToRead)));
    assert_eq!(v.offset, 0);
    assert_eq!(v.data_offset, 0);

    let mut v = SimpleSliceDeserializer::new(&s[..4]);
    assert!(matches!(v.read_byte_array(), Err(ErrorKind::UnableToRead)));
}
