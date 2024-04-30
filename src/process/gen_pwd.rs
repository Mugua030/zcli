use rand::seq::SliceRandom;

const UPPER: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ";
const LOWER: &[u8] = b"abcdefghijklmnopqrstuvwxyz";
const NUMBER: &[u8] = b"1234567890";
const SYMBOL: &[u8] = b"!@#$%^&*_";

pub fn process_genpwd(
    length: u8,
    no_upper: bool,
    lower: bool,
    number: bool,
    symbol: bool,
) -> anyhow::Result<String> {
    let mut rng = rand::thread_rng();
    let mut password = Vec::new();
    let mut chars = Vec::new();

    if !no_upper {
        chars.extend_from_slice(UPPER);
        password.push(*UPPER.choose(&mut rng).expect("won't empty"));
    }

    if lower {
        chars.extend_from_slice(LOWER);
        password.push(*LOWER.choose(&mut rng).expect("won't empty"));
    }

    if number {
        chars.extend_from_slice(NUMBER);
        password.push(*NUMBER.choose(&mut rng).expect("won't empty"));
    }

    if symbol {
        chars.extend_from_slice(SYMBOL);
        password.push(*SYMBOL.choose(&mut rng).expect("won't empty"));
    }

    for _ in 0..(length - password.len() as u8) {
        //let idx = rng.gen_range(0..chars.len());
        let c = chars.choose(&mut rng).expect("won not empty");
        password.push(*c);
    }
    password.shuffle(&mut rng);

    Ok(String::from_utf8(password)?)
}
