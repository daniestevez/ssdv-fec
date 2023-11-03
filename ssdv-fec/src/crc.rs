use core::borrow::Borrow;

const CRC32_DSLWP_MAGIC_VALUE: u32 = 0x4EE4FDE1;

pub fn crc32<I, T>(data: I) -> u32
where
    I: Iterator<Item = T>,
    T: Borrow<u8>,
{
    let mut crc = CRC32_DSLWP_MAGIC_VALUE;
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
        ssdv::{SSDVPacket, SSDV_PACKET_LEN},
        test_data::IMG_230_SSDV,
    };

    #[test]
    fn check_img_230_crcs() {
        for packet in IMG_230_SSDV.chunks_exact(SSDV_PACKET_LEN) {
            let packet = SSDVPacket(packet.try_into().unwrap());
            let crc_calc = crc32(packet.crc32_data().iter());
            let crc_packet = packet.crc32();
            assert_eq!(crc_calc, crc_packet);
        }
    }
}
