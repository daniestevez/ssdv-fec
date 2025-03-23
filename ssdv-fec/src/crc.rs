use core::borrow::Borrow;

pub fn crc32<I, T>(data: I) -> u32
where
    I: Iterator<Item = T>,
    T: Borrow<u8>,
{
    crc32_with_init_value(data, 0xFFFFFFFF)
}

const CRC32_DSLWP_MAGIC_VALUE: u32 = 0x4EE4FDE1;

pub fn crc32_dslwp<I, T>(data: I) -> u32
where
    I: Iterator<Item = T>,
    T: Borrow<u8>,
{
    crc32_with_init_value(data, CRC32_DSLWP_MAGIC_VALUE)
}

fn crc32_with_init_value<I, T>(data: I, init_value: u32) -> u32
where
    I: Iterator<Item = T>,
    T: Borrow<u8>,
{
    //let mut crc = CRC32_DSLWP_MAGIC_VALUE;
    let mut crc = init_value;
    for d in data {
        let mut x = (crc ^ *d.borrow() as u32) & 0xff;
        for _ in 0..8 {
            if x & 1 != 0 {
                x = (x >> 1) ^ 0xEDB88320;
            } else {
                x >>= 1;
            }
        }
        crc = (crc >> 8) ^ x;
    }
    crc ^ 0xFFFFFFFF
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::{
        SSDVPacket as _, SSDVParameters,
        packet_formats::longjiang2::{Parameters, SSDVPacket},
        test_data::IMG_230_SSDV,
    };
    use generic_array::typenum::Unsigned;

    #[test]
    fn check_img_230_crcs() {
        const PACKET_LEN: usize = <Parameters as SSDVParameters>::PacketLen::USIZE;

        for packet in IMG_230_SSDV.chunks_exact(PACKET_LEN) {
            let packet = SSDVPacket::new_from_slice(packet).unwrap();
            let crc_calc = crc32_dslwp(packet.crc32_data().iter());
            let crc_packet = packet.crc32();
            assert_eq!(crc_calc, crc_packet);
        }
    }
}
