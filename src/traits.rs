pub trait ToAstring {
    fn to_astring(self) -> String;
}

impl ToAstring for i64 {
    fn to_astring(self) -> String {
        let mut buffer = itoa::Buffer::new();
        buffer.format(self).to_owned()
    }
}

impl ToAstring for f64 {
    fn to_astring(self) -> String {
        let mut buffer = ryu::Buffer::new();
        buffer.format(self).to_owned()
    }
}
