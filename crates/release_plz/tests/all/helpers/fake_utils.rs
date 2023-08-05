use fake::{Fake, StringFaker};

pub fn fake_id() -> String {
    const LETTERS: &str = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ";
    let f = StringFaker::with(Vec::from(LETTERS), 8);
    f.fake()
}
