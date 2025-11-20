#[cfg(test)]
mod tests {
    use super::*;
    #[cfg(feature = "decoder")]
    use crate::decoder::Decoder;

    #[test]
    #[cfg(feature = "decoder")]
    fn test_decoder_init() {
        let (mut dec, cfg) = Decoder::init_default().expect("Failed to init decoder");

        dec.init().expect("Failed to init decoder instance");
    }
}
